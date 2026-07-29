const assert = require('node:assert/strict');
const {
  createHash,
  createPublicKey,
  verify,
} = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { canonicalize } = require('@tufjs/canonical-json');
const {
  ROLE_NAMES,
  createProductionTrust,
} = require('../scripts/create-tuf-production-trust.cjs');
const { verifyEnvelope } = require('../scripts/sign-tuf-update-repository.cjs');

const ED25519_SPKI_PREFIX = Buffer.from('302a300506032b6570032100', 'hex');
const REVIEWED_ROOT_SHA256 = 'db88d4445135c02065824de9d035803bfc0b2b7a6eb0e5bb2fc57556e39d478e';

test('committed production root is the reviewed public trust anchor', () => {
  const rootPath = path.join(__dirname, '..', 'build', 'update-trust', 'root.json');
  const rootBytes = fs.readFileSync(rootPath);
  const root = JSON.parse(rootBytes);

  assert.equal(createHash('sha256').update(rootBytes).digest('hex'), REVIEWED_ROOT_SHA256);
  assert.doesNotMatch(rootBytes.toString('utf8'), /PRIVATE KEY/);
  assert.deepEqual(Object.keys(root.signed.roles).sort(), [...ROLE_NAMES].sort());
  assert.equal(
    new Set(ROLE_NAMES.map((role) => root.signed.roles[role].keyids[0])).size,
    ROLE_NAMES.length,
  );
  verifyEnvelope(root, root, 'root');
});

test('production trust uses distinct role keys and a valid offline-root signature', (context) => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-tuf-production-'));
  context.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const privateKeyBundlePath = path.join(temporary, 'private', 'keys.json');
  const rootPath = path.join(temporary, 'public', 'root.json');
  const result = createProductionTrust({ privateKeyBundlePath, rootPath });
  const root = JSON.parse(fs.readFileSync(rootPath, 'utf8'));
  const privateBundle = JSON.parse(fs.readFileSync(privateKeyBundlePath, 'utf8'));

  assert.equal(root.signed._type, 'root');
  assert.equal(root.signed.version, 1);
  assert.equal(root.signed.spec_version, '1.0.31');
  assert.equal(root.signed.consistent_snapshot, false);
  assert.ok(Date.parse(root.signed.expires) > Date.now() + (5 * 365 * 24 * 60 * 60 * 1000));
  assert.deepEqual(Object.keys(root.signed.roles).sort(), [...ROLE_NAMES].sort());
  assert.equal(new Set(Object.values(result.keyIDs)).size, ROLE_NAMES.length);

  for (const role of ROLE_NAMES) {
    assert.deepEqual(root.signed.roles[role], {
      keyids: [result.keyIDs[role]],
      threshold: 1,
    });
    assert.equal(privateBundle.roles[role].keyid, result.keyIDs[role]);
    assert.match(privateBundle.roles[role].private_key_pem, /^-----BEGIN PRIVATE KEY-----/);
  }
  assert.doesNotMatch(fs.readFileSync(rootPath, 'utf8'), /PRIVATE KEY/);
  assert.equal(fs.statSync(privateKeyBundlePath).mode & 0o777, 0o600);

  const rootKey = root.signed.keys[result.keyIDs.root];
  const publicKey = createPublicKey({
    key: Buffer.concat([
      ED25519_SPKI_PREFIX,
      Buffer.from(rootKey.keyval.public, 'hex'),
    ]),
    format: 'der',
    type: 'spki',
  });
  assert.equal(
    verify(
      null,
      Buffer.from(canonicalize(root.signed)),
      publicKey,
      Buffer.from(root.signatures[0].sig, 'hex'),
    ),
    true,
  );
});

test('production trust refuses to overwrite public or private ceremony outputs', (context) => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-tuf-production-'));
  context.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const privateKeyBundlePath = path.join(temporary, 'keys.json');
  const rootPath = path.join(temporary, 'root.json');
  fs.writeFileSync(rootPath, 'reviewed');
  assert.throws(
    () => createProductionTrust({ privateKeyBundlePath, rootPath }),
    /must not already exist/,
  );
  assert.equal(fs.readFileSync(rootPath, 'utf8'), 'reviewed');
});
