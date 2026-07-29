const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const YAML = require('yaml');
const {
  artifactName,
  prepareSignedTarget,
} = require('../scripts/test-nonmac-update.cjs');
const workflow = fs.readFileSync(
  path.resolve(__dirname, '..', '..', '..', '.github', 'workflows', 'nonmac-updater-audit.yml'),
  'utf8',
);
const auditScript = fs.readFileSync(
  path.resolve(__dirname, '..', 'scripts', 'test-nonmac-update.cjs'),
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

test('native updater workflow performs real TUF-backed Windows and AppImage replacements', () => {
  assert.match(workflow, /runs-on:.*windows-2025.*ubuntu-24\.04/);
  assert.match(workflow, /Require the matching native runner/);
  assert.match(workflow, /verify-calculix-runtimes\.cjs --target/);
  assert.match(workflow, /FRAIA_REQUIRE_TUF_ROOT: '1'/);
  assert.match(workflow, /test-tuf-repository\.cjs/);
  assert.match(workflow, /test-nonmac-update\.cjs/);
  assert.match(workflow, /Fraia-Windows-x64-Setup\.exe/);
  assert.match(workflow, /Fraia-Linux-x64\.AppImage/);
  assert.match(workflow, /latest\.yml/);
  assert.match(workflow, /latest-linux\.yml/);
  assert.match(workflow, /Remove disposable private key and package outputs/);
  assert.doesNotMatch(workflow, /secrets\./);
  assert.match(auditScript, /updated-runtime-launched/);
  assert.match(auditScript, /Updater changed existing project data/);
  assert.match(auditScript, /Updater changed encrypted AI credentials/);
  assert.match(auditScript, /update-trust.*metadata.*root\.json/);
  assert.match(auditScript, /AppImage updater did not replace the installed bytes/);
  assert.match(auditScript, /PACKAGE_SHA256SUMS/);
  assert.doesNotMatch(auditScript, /copyFileSync\(privateKeyPath/);
});
