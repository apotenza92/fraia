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
  waitForPathRemoval,
} = require('../scripts/test-nonmac-update.cjs');
const workflow = fs.readFileSync(
  path.resolve(__dirname, '..', '..', '..', '.github', 'workflows', 'nonmac-updater-audit.yml'),
  'utf8',
);
const continuousIntegration = fs.readFileSync(
  path.resolve(__dirname, '..', '..', '..', '.github', 'workflows', 'ci.yml'),
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

test('native updater workflow performs real TUF-backed Windows and AppImage replacements', () => {
  assert.match(workflow, /runs-on:.*windows-2025.*ubuntu-24\.04/);
  assert.match(workflow, /ubuntu-24\.04-arm/);
  assert.match(workflow, /workflow_call:/);
  assert.match(workflow, /Require the matching native runner/);
  assert.match(workflow, /verify-calculix-runtimes\.cjs --target/);
  assert.match(workflow, /FRAIA_REQUIRE_TUF_ROOT: '1'/);
  assert.match(workflow, /test-tuf-repository\.cjs/);
  assert.match(workflow, /test-nonmac-update\.cjs/);
  assert.match(workflow, /Fraia-Windows-x64-Setup\.exe/);
  assert.match(workflow, /Fraia-Linux-\$\{arch\}\.AppImage/);
  assert.match(workflow, /latest-linux-arm64\.yml/);
  assert.match(workflow, /--linux AppImage --arm64/);
  assert.match(workflow, /Synthetic prior package used only for the native updater migration audit/);
  assert.match(workflow, /FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE=1/);
  assert.match(workflow, /unset FRAIA_E2E_UPDATER FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE/);
  assert.match(nsisInclude, /customCheckAppRunning/);
  assert.match(nsisInclude, /!include "getProcessInfo\.nsh"/);
  assert.match(nsisInclude, /ExecutablePath/);
  assert.match(nsisInclude, /OrdinalIgnoreCase/);
  assert.match(nsisInclude, /Var \/GLOBAL FraiaInstallerPid/);
  assert.match(nsisInclude, /ProcessId -ne \$FraiaInstallerPid/);
  assert.match(nsisInclude, /FRAIA_NSIS_INSTALL_DIR/);
  assert.match(nsisInclude, /GetFullPath/);
  assert.doesNotMatch(nsisInclude, /StartsWith\('\$INSTDIR/);
  assert.match(nsisInclude, /AddSeconds\(15\)/);
  assert.match(nsisInclude, /SetErrorLevel 2/);
  assert.doesNotMatch(nsisInclude, /\$\$_\.Path/);
  assert.match(workflow, /latest\.yml/);
  assert.match(workflow, /latest-linux\.yml/);
  assert.match(workflow, /Remove disposable private key and package outputs/);
  assert.doesNotMatch(workflow, /secrets\./);
  assert.match(auditScript, /updated-runtime-launched/);
  assert.match(auditScript, /update-downloaded/);
  assert.match(auditScript, /installedPackageVersion/);
  assert.match(auditScript, /waitForInstalledWindowsPackage/);
  assert.match(auditScript, /installedPackageDigest/);
  assert.match(auditScript, /candidateDirectory.*win-unpacked.*resources.*app\.asar/s);
  assert.match(auditScript, /Get-CimInstance Win32_Process/);
  assert.match(auditScript, /StringComparison.*OrdinalIgnoreCase/);
  assert.match(auditScript, /taskkill\.exe/);
  assert.match(auditScript, /normal user launch/);
  assert.match(auditScript, /waitForPathRemoval\(installDirectory\)/);
  assert.match(auditScript, /Installed candidate app\.asar SHA-256/);
  assert.match(auditScript, /LOCALAPPDATA.*Programs.*contract\.productName/);
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
