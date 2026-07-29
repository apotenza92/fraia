const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const {
  createProductionTrust,
} = require('../scripts/create-tuf-production-trust.cjs');
const {
  signUpdateRepository,
  verifyEnvelope,
} = require('../scripts/sign-tuf-update-repository.cjs');

function fixture(context) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-tuf-signing-'));
  context.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const privateKeyBundlePath = path.join(temporary, 'private-keys.json');
  const rootPath = path.join(temporary, 'root.json');
  createProductionTrust({ privateKeyBundlePath, rootPath });
  const privateBundle = JSON.parse(fs.readFileSync(privateKeyBundlePath, 'utf8'));
  return {
    privateKeys: Object.fromEntries(
      ['targets', 'snapshot', 'timestamp'].map((role) => [
        role,
        privateBundle.roles[role].private_key_pem,
      ]),
    ),
    rootPath,
    temporary,
  };
}

test('production signer creates distinct, verifiable, expiring metadata', (context) => {
  const { privateKeys, rootPath, temporary } = fixture(context);
  const targetPath = path.join(temporary, 'latest.yml');
  const outputDirectory = path.join(temporary, 'repository-v1');
  fs.writeFileSync(targetPath, 'version: 0.0.2\n');
  const result = signUpdateRepository({
    now: new Date('2026-07-29T00:00:00Z'),
    outputDirectory,
    previousMetadataDirectory: null,
    privateKeys,
    rootPath,
    targetName: 'latest.yml',
    targetPath,
  });
  const root = JSON.parse(fs.readFileSync(rootPath, 'utf8'));
  const targets = JSON.parse(fs.readFileSync(path.join(outputDirectory, 'metadata', 'targets.json')));
  const snapshot = JSON.parse(fs.readFileSync(path.join(outputDirectory, 'metadata', 'snapshot.json')));
  const timestamp = JSON.parse(fs.readFileSync(path.join(outputDirectory, 'metadata', 'timestamp.json')));

  assert.deepEqual(result.versions, { targets: 1, snapshot: 1, timestamp: 1 });
  verifyEnvelope(targets, root, 'targets');
  verifyEnvelope(snapshot, root, 'snapshot');
  verifyEnvelope(timestamp, root, 'timestamp');
  assert.equal(
    fs.readFileSync(path.join(outputDirectory, 'targets', 'latest.yml'), 'utf8'),
    'version: 0.0.2\n',
  );
  assert.match(
    fs.readFileSync(path.join(outputDirectory, 'EVIDENCE.txt'), 'utf8'),
    /Target SHA-256:/,
  );
  assert.doesNotMatch(
    fs.readFileSync(path.join(outputDirectory, 'metadata', 'root.json'), 'utf8'),
    /PRIVATE KEY/,
  );
});

test('production signer increments verified previous metadata and rejects tampering', (context) => {
  const { privateKeys, rootPath, temporary } = fixture(context);
  const targetPath = path.join(temporary, 'latest.yml');
  const first = path.join(temporary, 'repository-v1');
  const second = path.join(temporary, 'repository-v2');
  fs.writeFileSync(targetPath, 'version: 0.0.2\n');
  signUpdateRepository({
    now: new Date('2026-07-29T00:00:00Z'),
    outputDirectory: first,
    previousMetadataDirectory: null,
    privateKeys,
    rootPath,
    targetName: 'latest.yml',
    targetPath,
  });
  fs.writeFileSync(targetPath, 'version: 0.0.3\n');
  const result = signUpdateRepository({
    now: new Date('2026-08-01T00:00:00Z'),
    outputDirectory: second,
    previousMetadataDirectory: path.join(first, 'metadata'),
    privateKeys,
    rootPath,
    targetName: 'latest.yml',
    targetPath,
  });
  assert.deepEqual(result.versions, { targets: 2, snapshot: 2, timestamp: 2 });

  const targetsPath = path.join(first, 'metadata', 'targets.json');
  const tampered = JSON.parse(fs.readFileSync(targetsPath, 'utf8'));
  tampered.signed.version = 99;
  fs.writeFileSync(targetsPath, JSON.stringify(tampered));
  assert.throws(
    () => signUpdateRepository({
      now: new Date('2026-08-02T00:00:00Z'),
      outputDirectory: path.join(temporary, 'repository-tampered'),
      previousMetadataDirectory: path.join(first, 'metadata'),
      privateKeys,
      rootPath,
      targetName: 'latest.yml',
      targetPath,
    }),
    /signature threshold/,
  );
});

test('production signer rejects a key that does not match its reviewed role', (context) => {
  const { privateKeys, rootPath, temporary } = fixture(context);
  const targetPath = path.join(temporary, 'latest.yml');
  fs.writeFileSync(targetPath, 'version: 0.0.2\n');
  assert.throws(
    () => signUpdateRepository({
      outputDirectory: path.join(temporary, 'repository'),
      previousMetadataDirectory: null,
      privateKeys: {
        ...privateKeys,
        targets: privateKeys.timestamp,
      },
      rootPath,
      targetName: 'latest.yml',
      targetPath,
    }),
    /targets key does not match/,
  );
});
