const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const asar = require('@electron/asar');
const YAML = require('yaml');
const {
  artifactName,
  installedPackageDigest,
  installedPackageVersion,
  prepareSignedTarget,
  seedTestTrustedRoot,
  waitForPath,
  waitForPathRemoval,
  windowsInstallDirectory,
  windowsUnpackedDirectoryName,
} = require('../scripts/test-nonmac-update.cjs');
const workflow = fs.readFileSync(
  path.resolve(__dirname, '..', '..', '..', '.github', 'workflows', 'nonmac-updater-audit.yml'),
  'utf8',
);
const continuousIntegration = fs.readFileSync(
  path.resolve(__dirname, '..', '..', '..', '.github', 'workflows', 'ci.yml'),
  'utf8',
);
const releaseWorkflow = fs.readFileSync(
  path.resolve(__dirname, '..', '..', '..', '.github', 'workflows', 'release.yml'),
  'utf8',
);
const auditScript = fs.readFileSync(
  path.resolve(__dirname, '..', 'scripts', 'test-nonmac-update.cjs'),
  'utf8',
);
const nsisInclude = fs.readFileSync(
  path.resolve(__dirname, '..', 'build', 'installer.nsh'),
  'utf8',
);

test('native updater audit rewrites only checksum-verified package URLs', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-native-update-target-'));
  try {
    const artifact = Buffer.from('candidate package');
    const artifactPath = path.join(directory, 'Fraia-Windows-x64-Setup.exe');
    fs.writeFileSync(artifactPath, artifact);
    const sha512 = require('node:crypto').createHash('sha512').update(artifact).digest('base64');
    const metadataPath = path.join(directory, 'latest.yml');
    fs.writeFileSync(metadataPath, YAML.stringify({
      version: '0.0.2',
      files: [{ url: path.basename(artifactPath), sha512, size: artifact.length }],
      path: path.basename(artifactPath),
      sha512,
    }));
    const prepared = prepareSignedTarget({
      baseUrl: 'http://127.0.0.1:43127',
      candidateDirectory: directory,
      candidateMetadata: metadataPath,
    });
    const rewritten = YAML.parse(prepared.bytes.toString('utf8'));
    assert.equal(
      rewritten.files[0].url,
      'http://127.0.0.1:43127/assets/Fraia-Windows-x64-Setup.exe',
    );
    assert.equal(rewritten.path, rewritten.files[0].url);
    assert.equal(prepared.version, '0.0.2');

    fs.appendFileSync(artifactPath, 'tampered');
    assert.throws(
      () => prepareSignedTarget({
        baseUrl: 'http://127.0.0.1:43127',
        candidateDirectory: directory,
        candidateMetadata: metadataPath,
      }),
      /SHA-512 does not match/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('native updater audit rejects escaping artifact names', () => {
  assert.equal(artifactName('https://example.invalid/Fraia.AppImage'), 'Fraia.AppImage');
  assert.throws(() => artifactName(''), /invalid artifact URL/);
  assert.throws(() => artifactName('%2F'), /Unsafe/);
});

test('native updater audit reads the version from an installed ASAR', async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-native-installed-version-'));
  try {
    const source = path.join(directory, 'source');
    const executable = path.join(directory, 'Fraia.exe');
    fs.mkdirSync(path.join(directory, 'resources'), { recursive: true });
    fs.mkdirSync(source);
    fs.writeFileSync(path.join(source, 'package.json'), '{"version":"0.0.2"}\n');
    fs.writeFileSync(executable, '');
    await asar.createPackage(source, path.join(directory, 'resources', 'app.asar'));
    assert.equal(installedPackageVersion(executable), '0.0.2');
    const firstDigest = installedPackageDigest(executable);
    fs.writeFileSync(path.join(source, 'package.json'), '{"version":"0.0.3"}\n');
    fs.rmSync(path.join(directory, 'resources', 'app.asar'));
    await asar.createPackage(source, path.join(directory, 'resources', 'app.asar'));
    assert.equal(installedPackageVersion(executable), '0.0.3');
    assert.notEqual(installedPackageDigest(executable), firstDigest);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('native updater audit resolves electron-builder unpacked directories for both Windows architectures', () => {
  assert.equal(windowsUnpackedDirectoryName('x64'), 'win-unpacked');
  assert.equal(windowsUnpackedDirectoryName('arm64'), 'win-arm64-unpacked');
  assert.throws(() => windowsUnpackedDirectoryName('ia32'), /Unsupported Windows package architecture/);
});

test('native updater audit follows Electron Builder package-name install directories', () => {
  assert.equal(
    windowsInstallDirectory('C:\\Users\\runneradmin\\AppData\\Local', 'fraia-electron'),
    'C:\\Users\\runneradmin\\AppData\\Local\\Programs\\fraia-electron',
  );
  assert.equal(
    windowsInstallDirectory('C:\\Users\\runneradmin\\AppData\\Local', 'fraia-electron-beta'),
    'C:\\Users\\runneradmin\\AppData\\Local\\Programs\\fraia-electron-beta',
  );
});

test('native updater cleanup waits for confirmed uninstaller removal', () => {
  const removedPath = path.join(os.tmpdir(), `fraia-removed-install-${process.pid}`);
  fs.rmSync(removedPath, { recursive: true, force: true });
  assert.doesNotThrow(() => waitForPathRemoval(removedPath, { timeoutMs: 10 }));

  const retainedPath = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-retained-install-'));
  try {
    assert.throws(
      () => waitForPathRemoval(retainedPath, { timeoutMs: 10, intervalMs: 1 }),
      /Timed out waiting for the native uninstaller/,
    );
  } finally {
    fs.rmSync(retainedPath, { recursive: true, force: true });
  }
});

test('native updater audit waits for an asynchronous predecessor install', () => {
  const installedPath = path.join(os.tmpdir(), `fraia-installed-${process.pid}`);
  fs.writeFileSync(installedPath, 'installed');
  try {
    assert.doesNotThrow(() => waitForPath(installedPath, { timeoutMs: 10 }));
  } finally {
    fs.rmSync(installedPath, { force: true });
  }
  assert.throws(
    () => waitForPath(installedPath, { timeoutMs: 10, intervalMs: 1 }),
    /Timed out waiting for the native installer/,
  );
});

test('native updater audit seeds its disposable root before the public predecessor launches', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-native-updater-root-'));
  try {
    const rootPath = path.join(directory, 'ephemeral-root.json');
    const userData = path.join(directory, 'user-data');
    const rootBytes = Buffer.from('{"signed":{"version":1}}\n');
    fs.writeFileSync(rootPath, rootBytes);
    const trustedRootPath = seedTestTrustedRoot({ rootPath, userData });
    assert.equal(
      trustedRootPath,
      path.join(userData, 'update-trust', 'metadata', 'root.json'),
    );
    assert.deepEqual(fs.readFileSync(trustedRootPath), rootBytes);
    assert.throws(
      () => seedTestTrustedRoot({ rootPath, userData }),
      /EEXIST/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('native updater workflow performs real TUF-backed Windows and AppImage replacements', () => {
  assert.match(workflow, /runs-on:.*windows-11-arm.*windows-2025.*ubuntu-24\.04/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
  assert.match(workflow, /workflow_call:/);
  assert.match(workflow, /channel:[\s\S]*?options:\n\s+- stable\n\s+- beta/);
  assert.match(workflow, /Require the matching native runner/);
  assert.match(workflow, /verify-calculix-runtimes\.cjs --target/);
  assert.match(workflow, /FRAIA_REQUIRE_TUF_ROOT: '1'/);
  assert.match(workflow, /test-tuf-repository\.cjs/);
  assert.match(workflow, /test-nonmac-update\.cjs/);
  assert.match(workflow, /releaseContract\(\{channel: '\$channel', platform: '\$platform', arch: '\$arch'\}\)\.artifactPrefix/);
  assert.match(workflow, /\$prefix-Windows-\$\{arch\}-Setup\.exe/);
  assert.match(workflow, /\$prefix-Linux-\$\{arch\}\.AppImage/);
  assert.match(workflow, /FRAIA_RELEASE_CHANNEL: \$\{\{ inputs\.channel \}\}/);
  assert.match(workflow, /release-version-policy\.cjs/);
  assert.match(workflow, /compareVersions\(candidateVersion, previousVersion\) <= 0/);
  assert.match(workflow, /latest-linux-arm64\.yml/);
  assert.match(workflow, /--linux AppImage --arm64/);
  assert.match(workflow, /Synthetic prior package used only for the native updater migration audit/);
  assert.match(workflow, /FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE=1/);
  assert.match(workflow, /unset FRAIA_E2E_UPDATER FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE/);
  assert.match(nsisInclude, /customCheckAppRunning/);
  assert.doesNotMatch(nsisInclude, /getProcessInfo|GetProcessInfo/);
  assert.match(nsisInclude, /!include "nsProcess\.nsh"/);
  assert.match(nsisInclude, /nsProcess::FindProcess/);
  assert.match(nsisInclude, /APP_EXECUTABLE_FILENAME/);
  assert.match(nsisInclude, /nsProcess::Unload/);
  assert.match(nsisInclude, /\$R1 >= 240/);
  assert.match(nsisInclude, /Sleep 250/);
  assert.doesNotMatch(nsisInclude, /powershell|Get-CimInstance|SetEnvironmentVariable/i);
  assert.doesNotMatch(nsisInclude, /KillProcess|CloseProcess/);
  assert.match(nsisInclude, /SetErrorLevel 2/);
  assert.match(workflow, /latest\.yml/);
  assert.match(workflow, /latest-linux\.yml/);
  assert.match(workflow, /Remove disposable private key and package outputs/);
  assert.doesNotMatch(workflow, /secrets\./);
  assert.match(auditScript, /updated-runtime-launched/);
  assert.match(auditScript, /timeoutMs = 180_000/);
  assert.match(auditScript, /seedTestTrustedRoot\(\{ rootPath, userData \}\)/);
  assert.match(auditScript, /update-downloaded/);
  assert.match(auditScript, /installedPackageVersion/);
  assert.match(auditScript, /waitForInstalledWindowsPackage/);
  assert.match(auditScript, /installedPackageDigest/);
  assert.match(auditScript, /--candidate-asar-checksum/);
  assert.match(auditScript, /Candidate Windows app\.asar SHA-256 is invalid/);
  assert.match(releaseWorkflow, /ci-output\/audit\/app\.asar\.sha256/);
  assert.match(auditScript, /Get-CimInstance Win32_Process/);
  assert.match(auditScript, /StringComparison.*OrdinalIgnoreCase/);
  assert.match(auditScript, /taskkill\.exe/);
  assert.match(auditScript, /normal user launch/);
  assert.match(auditScript, /waitForPathRemoval\(installDirectory\)/);
  assert.match(auditScript, /Installed candidate app\.asar SHA-256/);
  assert.match(auditScript, /windowsInstallDirectory\(process\.env\.LOCALAPPDATA, contract\.packageName\)/);
  assert.doesNotMatch(auditScript, /`\/D=\$\{installDirectory\}`/);
  assert.match(auditScript, /Updater changed existing project data/);
  assert.match(auditScript, /Updater changed existing AI data/);
  assert.match(auditScript, /update-trust.*metadata.*root\.json/);
  assert.match(auditScript, /AppImage updater did not replace the installed bytes/);
  assert.match(auditScript, /PACKAGE_SHA256SUMS/);
  assert.match(auditScript, /failure: failure \|\| cleanupFailure/);
  assert.doesNotMatch(auditScript, /copyFileSync\(privateKeyPath/);
  assert.match(continuousIntegration, /nonmac_updater_target:/);
  assert.match(continuousIntegration, /- linux-arm64/);
  assert.match(continuousIntegration, /group:.*inputs\.nonmac_updater_target/);
  assert.match(continuousIntegration, /uses: \.\/\.github\/workflows\/nonmac-updater-audit\.yml/);
});
