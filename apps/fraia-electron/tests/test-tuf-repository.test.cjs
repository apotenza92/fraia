const assert = require('node:assert/strict');
const { verify } = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { canonicalize } = require('@tufjs/canonical-json');
const {
  createTestRepositoryMetadata,
  createTestTrust,
} = require('../scripts/test-tuf-repository.cjs');

test('ephemeral native updater trust signs an internally consistent TUF repository', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-test-tuf-'));
  try {
    const privateKeyPath = path.join(directory, 'private', 'key.pem');
    const rootPath = path.join(directory, 'public', 'root.json');
    const { keyID, root } = createTestTrust({ privateKeyPath, rootPath });
    assert.equal(root.signed.roles.root.threshold, 1);
    assert.deepEqual(root.signed.roles.targets.keyids, [keyID]);
    assert.equal(fs.statSync(privateKeyPath).mode & 0o777, 0o600);

    const targetBytes = Buffer.from('version: 0.0.2\nfiles: []\n');
    const metadata = createTestRepositoryMetadata({
      privateKeyPath,
      rootPath,
      targetBytes,
      targetName: 'latest.yml',
    });
    const targets = JSON.parse(metadata['targets.json']);
    assert.equal(targets.signed.targets['latest.yml'].length, targetBytes.length);
    assert.equal(targets.signatures[0].keyid, keyID);
    const privateKey = fs.readFileSync(privateKeyPath, 'utf8');
    assert.equal(
      verify(
        null,
        Buffer.from(canonicalize(targets.signed)),
        require('node:crypto').createPublicKey(privateKey),
        Buffer.from(targets.signatures[0].sig, 'hex'),
      ),
      true,
    );
    assert.equal(JSON.parse(metadata['snapshot.json']).signed.meta['targets.json'].version, 1);
    assert.equal(JSON.parse(metadata['timestamp.json']).signed.meta['snapshot.json'].version, 1);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('ephemeral trust refuses overwrite and mismatched keys', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-test-tuf-refusal-'));
  try {
    const firstKey = path.join(directory, 'first-key.pem');
    const firstRoot = path.join(directory, 'first-root.json');
    createTestTrust({ privateKeyPath: firstKey, rootPath: firstRoot });
    assert.throws(
      () => createTestTrust({ privateKeyPath: firstKey, rootPath: path.join(directory, 'other-root.json') }),
      /must not already exist/,
    );
    const secondKey = path.join(directory, 'second-key.pem');
    const secondRoot = path.join(directory, 'second-root.json');
    createTestTrust({ privateKeyPath: secondKey, rootPath: secondRoot });
    assert.throws(
      () => createTestRepositoryMetadata({
        privateKeyPath: secondKey,
        rootPath: firstRoot,
        targetBytes: Buffer.from('target'),
        targetName: 'latest.yml',
      }),
      /does not match/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
