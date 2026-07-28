#!/usr/bin/env node

const { spawnSync } = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const asar = require('@electron/asar');
const { assertBinaryArchitecture } = require('../binary-architecture.cjs');
const { assertMacosMinimumVersion } = require('../macos-version-contract.cjs');
const { nativePlatformArch, packagedCalculixPath, sidecarExecutableName } = require('../package-boundary.cjs');
const { releaseContract } = require('../release-contract.cjs');

function option(argv, name, fallback = null) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : fallback;
}

function normalizeFingerprint(value) {
  const normalized = String(value || '').replace(/[^A-Fa-f0-9]/g, '').toUpperCase();
  if (!/^[A-F0-9]{64}$/.test(normalized)) throw new Error('Expected a 64-digit SHA-256 certificate fingerprint.');
  return normalized;
}

function run(command, args, { capture = true, input, allowFailure = false, env = process.env } = {}) {
  const result = spawnSync(command, args, {
    encoding: 'utf8', input, env,
    stdio: capture ? ['pipe', 'pipe', 'pipe'] : 'inherit',
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0 && !allowFailure) {
    throw new Error(`${command} ${args.join(' ')} failed (${result.status}):\n${result.stdout || ''}${result.stderr || ''}`);
  }
  return result;
}

function hashFile(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex').toUpperCase();
}

function appManifest(appPath) {
  const entries = [];
  function walk(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
      const candidate = path.join(directory, entry.name);
      const relative = path.relative(appPath, candidate);
      const details = fs.lstatSync(candidate);
      if (details.isSymbolicLink()) {
        const target = fs.readlinkSync(candidate);
        const resolved = path.resolve(path.dirname(candidate), target);
        if (resolved !== appPath && !resolved.startsWith(`${appPath}${path.sep}`)) throw new Error(`App symlink escapes its bundle: ${relative}`);
        entries.push({ path: relative, symlink: target });
      } else if (details.isDirectory()) {
        entries.push({ path: relative, type: 'directory' });
        walk(candidate);
      } else if (details.isFile()) {
        entries.push({ path: relative, mode: details.mode & 0o777, sha256: hashFile(candidate), size: details.size });
      } else {
        throw new Error(`Unsupported app filesystem entry: ${relative}`);
      }
    }
  }
  walk(appPath);
  return entries;
}

function isMachO(filePath) {
  if (!fs.statSync(filePath).isFile()) return false;
  const descriptor = fs.openSync(filePath, 'r');
  try {
    const buffer = Buffer.alloc(4);
    if (fs.readSync(descriptor, buffer, 0, 4, 0) !== 4) return false;
    return new Set(['feedface', 'feedfacf', 'cefaedfe', 'cffaedfe', 'cafebabe', 'bebafeca', 'cafebabf', 'bfbafeca']).has(buffer.toString('hex'));
  } finally { fs.closeSync(descriptor); }
}

function signedTargets(appPath) {
  const result = [appPath];
  function walk(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const candidate = path.join(directory, entry.name);
      if (entry.isSymbolicLink()) continue;
      if (entry.isDirectory()) {
        if (/\.(app|framework|xpc|appex|bundle)$/.test(entry.name)) result.push(candidate);
        walk(candidate);
      } else if (entry.isFile() && isMachO(candidate)) result.push(candidate);
    }
  }
  walk(appPath);
  return [...new Set(result)].sort((left, right) => right.length - left.length);
}

function parseEntitlements(target) {
  const result = run('codesign', ['-d', '--xml', '--entitlements', '-', target]);
  const output = result.stdout || '';
  const xmlStart = output.indexOf('<?xml');
  if (xmlStart < 0) return {};
  const plist = run('plutil', ['-convert', 'json', '-o', '-', '--', '-'], { input: output.slice(xmlStart) });
  return JSON.parse(plist.stdout);
}

function validateSignature(target, expectations, certificateRoot, index, validateChain = false) {
  run('codesign', ['--verify', '--strict', '--verbose=2', target]);
  const details = run('codesign', ['-d', '--verbose=4', target], { allowFailure: false });
  const combined = `${details.stdout || ''}${details.stderr || ''}`;
  if (!combined.includes(`TeamIdentifier=${expectations.teamId}`)) throw new Error(`${target} has the wrong signing team.`);
  if (!combined.includes(`Authority=${expectations.identity}`)) throw new Error(`${target} has the wrong Developer ID identity.`);
  if (isMachO(target) && (!/Timestamp=/.test(combined) || !/(?:flags=.*runtime|Runtime Version=)/i.test(combined))) {
    throw new Error(`${target} lacks hardened runtime or secure timestamp evidence.`);
  }
  const allowedEntitlements = new Set([
    'com.apple.security.cs.allow-jit',
    'com.apple.security.cs.allow-unsigned-executable-memory',
  ]);
  const entitlements = parseEntitlements(target);
  for (const [name, value] of Object.entries(entitlements)) {
    if (name === 'com.apple.security.get-task-allow' && value === true) throw new Error(`${target} enables get-task-allow.`);
    if (!allowedEntitlements.has(name)) throw new Error(`${target} has unexpected entitlement ${name}.`);
  }

  const prefix = path.join(certificateRoot, `cert-${index}-`);
  run('codesign', ['-d', `--extract-certificates=${prefix}`, target]);
  const leaf = `${prefix}0`;
  if (!fs.existsSync(leaf)) throw new Error(`${target} did not expose its leaf certificate.`);
  if (hashFile(leaf) !== expectations.fingerprint) throw new Error(`${target} uses an unexpected signing certificate.`);
  if (validateChain && (!fs.existsSync(`${prefix}1`) || !fs.existsSync(`${prefix}2`))) {
    throw new Error(`${target} does not embed the complete Developer ID certificate chain.`);
  }
}

function extractPackageMetadata(appPath) {
  const archive = path.join(appPath, 'Contents', 'Resources', 'app.asar');
  if (!fs.existsSync(archive)) throw new Error(`Packaged asar is missing: ${archive}`);
  return JSON.parse(asar.extractFile(archive, 'package.json').toString('utf8'));
}

function verifyApp(appPath, contract, expectations) {
  const info = path.join(appPath, 'Contents', 'Info.plist');
  const readPlist = (key) => run('plutil', ['-extract', key, 'raw', '-o', '-', info]).stdout.trim();
  if (readPlist('CFBundleIdentifier') !== contract.appId) throw new Error(`${appPath} has the wrong bundle identifier.`);
  if (readPlist('CFBundleName') !== contract.productName) throw new Error(`${appPath} has the wrong product name.`);
  if (readPlist('LSMinimumSystemVersion') !== '15.0') {
    throw new Error(`${appPath} does not declare the reviewed macOS 15.0 minimum.`);
  }
  const iconName = readPlist('CFBundleIconFile');
  if (!iconName || /electron/i.test(iconName)) throw new Error(`${appPath} uses Electron's default icon.`);
  const version = readPlist('CFBundleShortVersionString');
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error(`${appPath} has an invalid stable version: ${version}`);

  const executable = path.join(appPath, 'Contents', 'MacOS', contract.productName);
  const sidecar = path.join(appPath, 'Contents', 'Resources', 'sidecar', nativePlatformArch('darwin', contract.arch), sidecarExecutableName('darwin'));
  const calculix = packagedCalculixPath(path.join(appPath, 'Contents', 'Resources'), 'darwin', contract.arch);
  for (const target of [executable, sidecar, calculix]) {
    if (!fs.existsSync(target)) throw new Error(`Required native executable is missing: ${target}`);
    assertBinaryArchitecture(target, contract.arch);
    assertMacosMinimumVersion(target);
  }
  const calculixNotices = path.join(path.dirname(calculix), 'THIRD_PARTY_NOTICES.txt');
  if (!fs.existsSync(calculixNotices) || fs.statSync(calculixNotices).size === 0) {
    throw new Error(`Packaged CalculiX notices are missing or empty: ${calculixNotices}`);
  }
  const calculixManifest = path.join(path.dirname(calculix), 'runtime-manifest.json');
  if (!fs.existsSync(calculixManifest) || fs.statSync(calculixManifest).size === 0) {
    throw new Error(`Packaged CalculiX runtime manifest is missing or empty: ${calculixManifest}`);
  }

  const metadata = extractPackageMetadata(appPath);
  if (metadata.fraiaReleaseChannel !== contract.channel || metadata.fraiaUpdateFeedUrl !== contract.feedUrl) {
    throw new Error(`${appPath} has incorrect updater channel metadata.`);
  }
  if (metadata.name !== contract.packageName || metadata.productName !== contract.productName) {
    throw new Error(`${appPath} has incorrect package identity metadata.`);
  }

  const certificateRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-certificates-'));
  try {
    const targets = signedTargets(appPath);
    targets.forEach((target, index) => validateSignature(target, expectations, certificateRoot, index, target === appPath));
  } finally { fs.rmSync(certificateRoot, { recursive: true, force: true }); }

  const mainDetails = `${run('codesign', ['-d', '--verbose=4', appPath]).stderr}`;
  if (!/Runtime Version=/.test(mainDetails) || !/Timestamp=/.test(mainDetails)) {
    throw new Error('Main app signature lacks hardened runtime or secure timestamp evidence.');
  }
  run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', appPath]);
  run('xcrun', ['stapler', 'validate', appPath]);
  run('spctl', ['--assess', '--type', 'execute', '--verbose=4', appPath]);
  return { executable, manifest: appManifest(appPath), version };
}

function mountedDmg(dmgPath, callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-dmg-'));
  try {
    run('hdiutil', ['attach', '-nobrowse', '-readonly', '-mountpoint', root, dmgPath]);
    return callback(root);
  } finally {
    run('hdiutil', ['detach', root], { allowFailure: true });
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function extractedZip(zipPath, callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-zip-'));
  try {
    run('ditto', ['-x', '-k', zipPath, root]);
    return callback(root);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
}

function verifyChecksum(filePath) {
  const checksumPath = `${filePath}.sha256`;
  if (!fs.existsSync(checksumPath)) throw new Error(`Checksum is missing: ${checksumPath}`);
  const [expected, name] = fs.readFileSync(checksumPath, 'utf8').trim().split(/\s+/);
  if (name !== path.basename(filePath) || expected.toUpperCase() !== hashFile(filePath)) {
    throw new Error(`Checksum mismatch for ${filePath}.`);
  }
}

function verifyMacPackage({ channel, arch, outputDir, skipLaunch = false }) {
  if (process.platform !== 'darwin') throw new Error('macOS package verification must run on macOS.');
  const contract = releaseContract({ channel, platform: 'darwin', arch, outputDir });
  const expectations = {
    fingerprint: normalizeFingerprint(process.env.APPLE_SIGNING_CERTIFICATE_SHA256),
    identity: process.env.APPLE_SIGNING_IDENTITY?.trim(),
    teamId: process.env.APPLE_TEAM_ID?.trim(),
  };
  if (!expectations.identity || !expectations.teamId) throw new Error('Signing identity and team variables are required.');
  const stem = `${contract.artifactPrefix}-macOS-${arch}`;
  const dmg = path.join(contract.outputDir, `${stem}.dmg`);
  const zip = path.join(contract.outputDir, `${stem}.zip`);
  const notarizationPath = path.join(contract.outputDir, `notarization-${channel}-${arch}.json`);
  for (const target of [dmg, zip]) if (!fs.existsSync(target)) throw new Error(`Release artifact is missing: ${target}`);
  if (!fs.existsSync(notarizationPath)) throw new Error(`Notarization evidence is missing: ${notarizationPath}`);
  const notarization = JSON.parse(fs.readFileSync(notarizationPath, 'utf8'));
  for (const evidence of [notarization.app, notarization.dmg]) {
    if (evidence?.response?.status !== 'Accepted') throw new Error('Notarization evidence is not Accepted.');
    if ((evidence?.log?.issues || []).some((issue) => String(issue.severity).toLowerCase() === 'error')) {
      throw new Error('Notarization evidence contains an error issue.');
    }
  }
  verifyChecksum(dmg);
  verifyChecksum(zip);
  const dmgCertificateRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-dmg-certificates-'));
  try {
    validateSignature(dmg, expectations, dmgCertificateRoot, 0, true);
    const details = run('codesign', ['-d', '--verbose=4', dmg]);
    if (!/Timestamp=/.test(`${details.stdout || ''}${details.stderr || ''}`)) {
      throw new Error('DMG signature lacks a secure timestamp.');
    }
  } finally { fs.rmSync(dmgCertificateRoot, { recursive: true, force: true }); }
  run('xcrun', ['stapler', 'validate', dmg]);
  run('spctl', ['--assess', '--type', 'open', '--context', 'context:primary-signature', '--verbose=4', dmg]);
  const dmgResult = mountedDmg(dmg, (root) => verifyApp(path.join(root, contract.appName), contract, expectations));
  const zipResult = extractedZip(zip, (root) => verifyApp(path.join(root, contract.appName), contract, expectations));
  if (dmgResult.version !== zipResult.version) throw new Error('DMG and ZIP versions differ.');
  if (JSON.stringify(dmgResult.manifest) !== JSON.stringify(zipResult.manifest)) throw new Error('DMG and ZIP app contents differ.');
  if (!skipLaunch) {
    extractedZip(zip, (root) => {
      const executable = path.join(root, contract.appName, 'Contents', 'MacOS', contract.productName);
      const result = run(process.execPath, ['scripts/run-packaged-e2e.cjs'], {
        capture: false,
        allowFailure: true,
        env: { ...process.env, FRAIA_PACKAGED_EXECUTABLE: executable, FRAIA_DISABLE_UPDATES: '1', FRAIA_REQUIRE_PACKAGED_CALCULIX: '1' },
      });
      if (result.status !== 0) throw new Error(`Credential-free packaged launch failed with ${result.status}.`);
    });
  }
  return { contract, version: dmgResult.version };
}

function main(argv = process.argv.slice(2)) {
  verifyMacPackage({
    channel: option(argv, '--channel', process.env.FRAIA_RELEASE_CHANNEL || 'stable'),
    arch: option(argv, '--arch', process.env.FRAIA_RELEASE_ARCH || process.arch),
    outputDir: path.resolve(option(argv, '--output-dir', process.env.FRAIA_RELEASE_OUTPUT_DIR || 'release')),
    skipLaunch: argv.includes('--skip-launch'),
  });
}

if (require.main === module) main();

module.exports = {
  extractedZip,
  normalizeFingerprint,
  verifyApp,
  verifyMacPackage,
};
