#!/usr/bin/env node

const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { assertBinaryArchitecture } = require('../binary-architecture.cjs');
const { nativePlatformArch, packagedCalculixPath, sidecarExecutableName } = require('../package-boundary.cjs');
const { releaseContract } = require('../release-contract.cjs');

function option(argv, name, fallback = null) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : fallback;
}

function run(command, args, { cwd, env = process.env, input, capture = false, binary = false, allowFailure = false } = {}) {
  const result = spawnSync(command, args, {
    cwd, env, input, encoding: binary || input ? undefined : 'utf8',
    stdio: capture ? ['pipe', 'pipe', 'pipe'] : input ? ['pipe', 'inherit', 'inherit'] : 'inherit',
    maxBuffer: 512 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0 && !allowFailure) throw new Error(`${command} failed with status ${result.status}.`);
  return result;
}

function findExactlyOne(root, predicate, label) {
  const matches = [];
  function walk(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const candidate = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(candidate);
      else if (entry.isFile() && predicate(candidate)) matches.push(candidate);
    }
  }
  walk(root);
  if (matches.length !== 1) throw new Error(`Expected exactly one ${label}; found ${matches.length}.`);
  return matches[0];
}

function validateLayout(executable, resources, arch) {
  const sidecar = path.join(resources, 'sidecar', nativePlatformArch(process.platform, arch), sidecarExecutableName(process.platform));
  const calculix = packagedCalculixPath(resources, process.platform, arch);
  for (const target of [executable, sidecar, calculix]) {
    if (!fs.existsSync(target)) throw new Error(`Packaged native executable is missing: ${target}`);
    assertBinaryArchitecture(target, arch);
  }
  const calculixNotices = path.join(path.dirname(calculix), 'THIRD_PARTY_NOTICES.txt');
  if (!fs.existsSync(calculixNotices) || fs.statSync(calculixNotices).size === 0) {
    throw new Error(`Packaged CalculiX notices are missing or empty: ${calculixNotices}`);
  }
  const calculixManifest = path.join(path.dirname(calculix), 'runtime-manifest.json');
  if (!fs.existsSync(calculixManifest) || fs.statSync(calculixManifest).size === 0) {
    throw new Error(`Packaged CalculiX runtime manifest is missing or empty: ${calculixManifest}`);
  }
  return { executable, sidecar };
}

function smoke(executable) {
  const environment = {
    ...process.env,
    FRAIA_DISABLE_UPDATES: '1',
    FRAIA_PACKAGED_EXECUTABLE: executable,
    FRAIA_REQUIRE_PACKAGED_CALCULIX: '1',
  };
  const command = process.platform === 'win32' ? 'node.exe' : process.execPath;
  run(command, ['scripts/run-packaged-e2e.cjs'], { cwd: path.resolve(__dirname, '..'), env: environment });
}

function verifyWindows(contract) {
  const installer = path.join(contract.outputDir, `${contract.artifactPrefix}-Windows-${contract.arch}-Setup.exe`);
  const blockmap = `${installer}.blockmap`;
  for (const target of [installer, blockmap]) if (!fs.existsSync(target)) throw new Error(`Windows release artifact is missing: ${target}`);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-nsis-'));
  const install = path.join(root, 'installed');
  try {
    run(installer, ['/S', `/D=${install}`]);
    const executable = path.join(install, `${contract.productName}.exe`);
    validateLayout(executable, path.join(install, 'resources'), contract.arch);
    smoke(executable);
    const uninstaller = findExactlyOne(install, (candidate) => /^uninstall.*\.exe$/i.test(path.basename(candidate)), 'NSIS uninstaller');
    run(uninstaller, ['/S']);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
}

function extractRpm(packagePath, destination) {
  const archive = run('rpm2cpio', [packagePath], { capture: true, binary: true });
  run('cpio', ['-idm', '--quiet'], { cwd: destination, input: archive.stdout });
}

function verifyExtractedLinux(root, contract, label) {
  const executable = findExactlyOne(
    root,
    (candidate) => path.basename(candidate) === contract.packageName,
    `${label} app executable`,
  );
  const resources = path.join(path.dirname(executable), 'resources');
  validateLayout(executable, resources, contract.arch);
  smoke(executable);
}

function verifyLinux(contract) {
  const stem = path.join(contract.outputDir, `${contract.artifactPrefix}-Linux-${contract.arch}`);
  const appImage = `${stem}.AppImage`;
  const blockmap = `${appImage}.blockmap`;
  const deb = `${stem}.deb`;
  const rpm = `${stem}.rpm`;
  for (const target of [appImage, blockmap, deb, rpm]) if (!fs.existsSync(target)) throw new Error(`Linux release artifact is missing: ${target}`);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-linux-packages-'));
  try {
    fs.chmodSync(appImage, 0o755);
    const appImageRoot = path.join(root, 'appimage');
    fs.mkdirSync(appImageRoot);
    run(appImage, ['--appimage-extract'], { cwd: appImageRoot });
    const appImageExecutable = path.join(appImageRoot, 'squashfs-root', contract.packageName);
    validateLayout(appImageExecutable, path.join(appImageRoot, 'squashfs-root', 'resources'), contract.arch);
    smoke(appImageExecutable);

    const debRoot = path.join(root, 'deb');
    fs.mkdirSync(debRoot);
    run('dpkg-deb', ['--extract', deb, debRoot]);
    verifyExtractedLinux(debRoot, contract, 'deb');

    const rpmRoot = path.join(root, 'rpm');
    fs.mkdirSync(rpmRoot);
    extractRpm(rpm, rpmRoot);
    verifyExtractedLinux(rpmRoot, contract, 'rpm');
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
}

function verifyNativePackage({ channel, arch, outputDir }) {
  if (!['win32', 'linux'].includes(process.platform)) throw new Error('Use verify-macos-package.cjs for macOS.');
  if (process.arch !== arch) throw new Error(`Native ${arch} verification requires a ${arch} runner; current Node is ${process.arch}.`);
  const contract = releaseContract({ channel, platform: process.platform, arch, outputDir });
  if (process.platform === 'win32') verifyWindows(contract);
  else verifyLinux(contract);
  return contract;
}

function main(argv = process.argv.slice(2)) {
  verifyNativePackage({
    channel: option(argv, '--channel', process.env.FRAIA_RELEASE_CHANNEL || 'stable'),
    arch: option(argv, '--arch', process.env.FRAIA_RELEASE_ARCH || process.arch),
    outputDir: path.resolve(option(argv, '--output-dir', process.env.FRAIA_RELEASE_OUTPUT_DIR || 'release')),
  });
}

if (require.main === module) main();

module.exports = { verifyNativePackage };
