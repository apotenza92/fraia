const assert = require('node:assert/strict');
const { createHash, generateKeyPairSync, sign } = require('node:crypto');
const { EventEmitter, once } = require('node:events');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { canonicalize } = require('@tufjs/canonical-json');
const {
  createTufVerifiedUpdateFeed,
  initializeTrustedRoot,
  validateRepositoryUrl,
  validateTargetName,
} = require('../tuf-update-feed.cjs');

function signedMetadata(signed, keyID, privateKey) {
  const signature = sign(null, Buffer.from(canonicalize(signed)), privateKey).toString('hex');
  return { signatures: [{ keyid: keyID, sig: signature }], signed };
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

function tufFixture(targetName = 'latest.yml') {
  const { privateKey, publicKey } = generateKeyPairSync('ed25519');
  const publicDer = publicKey.export({ format: 'der', type: 'spki' });
  const key = {
    keytype: 'ed25519',
    scheme: 'ed25519',
    keyval: { public: publicDer.subarray(-32).toString('hex') },
  };
  const keyID = sha256(Buffer.from(canonicalize(key)));
  const expires = '2035-01-01T00:00:00Z';
  const role = { keyids: [keyID], threshold: 1 };
  const targetBytes = Buffer.from('version: 0.0.2\nfiles: []\n');
  const targets = signedMetadata({
    _type: 'targets',
    spec_version: '1.0.31',
    version: 1,
    expires,
    targets: {
      [targetName]: {
        length: targetBytes.length,
        hashes: { sha256: sha256(targetBytes) },
      },
    },
  }, keyID, privateKey);
  const targetsBytes = Buffer.from(JSON.stringify(targets));
  const snapshot = signedMetadata({
    _type: 'snapshot',
    spec_version: '1.0.31',
    version: 1,
    expires,
    meta: {
      'targets.json': {
        version: 1,
        length: targetsBytes.length,
        hashes: { sha256: sha256(targetsBytes) },
      },
    },
  }, keyID, privateKey);
  const snapshotBytes = Buffer.from(JSON.stringify(snapshot));
  const timestamp = signedMetadata({
    _type: 'timestamp',
    spec_version: '1.0.31',
    version: 1,
    expires,
    meta: {
      'snapshot.json': {
        version: 1,
        length: snapshotBytes.length,
        hashes: { sha256: sha256(snapshotBytes) },
      },
    },
  }, keyID, privateKey);
  const root = signedMetadata({
    _type: 'root',
    spec_version: '1.0.31',
    version: 1,
    expires,
    consistent_snapshot: false,
    keys: { [keyID]: key },
    roles: {
      root: role,
      snapshot: role,
      targets: role,
      timestamp: role,
    },
  }, keyID, privateKey);
  return {
    metadata: {
      'root.json': Buffer.from(JSON.stringify(root)),
      'snapshot.json': snapshotBytes,
      'targets.json': targetsBytes,
      'timestamp.json': Buffer.from(JSON.stringify(timestamp)),
    },
    targetBytes,
    targetName,
  };
}

async function fixtureServer(fixture) {
  const server = http.createServer((request, response) => {
    const match = request.url.match(/^\/(metadata|targets)\/([^/?]+)$/);
    if (!match) {
      response.writeHead(404).end();
      return;
    }
    if (match[1] === 'metadata' && match[2] === '2.root.json') {
      response.writeHead(404).end();
      return;
    }
    const bytes = match[1] === 'metadata'
      ? fixture.metadata[match[2]]
      : match[2] === fixture.targetName ? fixture.targetBytes : null;
    if (!bytes) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { 'Content-Length': bytes.length });
    response.end(bytes);
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  return {
    close: () => new Promise((resolve) => server.close(resolve)),
    url: `http://127.0.0.1:${server.address().port}`,
  };
}

test('TUF repository and target validation fail closed', () => {
  assert.equal(validateRepositoryUrl('https://updates.example/fraia/'), 'https://updates.example/fraia');
  assert.throws(() => validateRepositoryUrl('http://updates.example/fraia'), /must use HTTPS/);
  assert.equal(
    validateRepositoryUrl('http://127.0.0.1:1234/repository', { allowLoopbackHttp: true }),
    'http://127.0.0.1:1234/repository',
  );
  assert.equal(validateTargetName('latest.yml'), 'latest.yml');
  assert.throws(() => validateTargetName('../latest.yml'), /Unsafe/);
  assert.throws(() => validateTargetName('nested/latest.yml'), /Unsafe/);
});

test('embedded root initializes once and never overwrites advanced client trust', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-tuf-bootstrap-'));
  try {
    const embedded = path.join(root, 'embedded-root.json');
    const metadata = path.join(root, 'metadata');
    fs.writeFileSync(embedded, 'root version one');
    const first = initializeTrustedRoot({ embeddedRootPath: embedded, metadataDir: metadata });
    assert.equal(first.initialized, true);
    fs.writeFileSync(first.trustedRootPath, 'advanced root version two');
    fs.writeFileSync(embedded, 'replacement app root');
    const second = initializeTrustedRoot({ embeddedRootPath: embedded, metadataDir: metadata });
    assert.equal(second.initialized, false);
    assert.equal(fs.readFileSync(second.trustedRootPath, 'utf8'), 'advanced root version two');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test('verified local feed serves only target bytes authenticated by TUF', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-tuf-feed-'));
  const fixture = tufFixture();
  const repository = await fixtureServer(fixture);
  let feed;
  try {
    const embeddedRootPath = path.join(root, 'embedded-root.json');
    fs.writeFileSync(embeddedRootPath, fixture.metadata['root.json']);
    feed = await createTufVerifiedUpdateFeed({
      embeddedRootPath,
      repositoryUrl: repository.url,
      targetName: fixture.targetName,
      trustDir: path.join(root, 'trust'),
      allowLoopbackHttp: true,
    });
    assert.equal(feed.trustInitialized, true);
    const response = await fetch(`${feed.feedUrl}/${fixture.targetName}`);
    assert.equal(response.status, 200);
    assert.deepEqual(Buffer.from(await response.arrayBuffer()), fixture.targetBytes);
    assert.equal((await fetch(`${feed.feedUrl}/unexpected.yml`)).status, 404);
    assert.deepEqual(fs.readFileSync(feed.targetPath), fixture.targetBytes);
    await feed.refresh();
    assert.deepEqual(fs.readFileSync(feed.targetPath), fixture.targetBytes);
  } finally {
    if (feed) await feed.close();
    await repository.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test('tampered target bytes are rejected and never exposed locally', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-tuf-tamper-'));
  const fixture = tufFixture();
  fixture.targetBytes = Buffer.from('malicious update metadata');
  const repository = await fixtureServer(fixture);
  try {
    const embeddedRootPath = path.join(root, 'embedded-root.json');
    fs.writeFileSync(embeddedRootPath, fixture.metadata['root.json']);
    await assert.rejects(
      createTufVerifiedUpdateFeed({
        embeddedRootPath,
        repositoryUrl: repository.url,
        targetName: fixture.targetName,
        trustDir: path.join(root, 'trust'),
        allowLoopbackHttp: true,
      }),
      /hash|length/i,
    );
  } finally {
    await repository.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});
