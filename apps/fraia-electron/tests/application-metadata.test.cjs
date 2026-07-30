const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const asar = require('@electron/asar');

const {
  resolveApplicationMetadata,
  resolveUserDataDirectory,
} = require('../application-metadata.cjs');
const { extractPackageMetadata } = require('../scripts/verify-macos-package.cjs');

test('defaults development metadata to the stable identity', () => {
  assert.deepEqual(resolveApplicationMetadata(), {
    channel: 'stable',
    productName: 'Fraia',
    userDataDirectoryName: 'Fraia',
  });
});

test('stable releases retain the canonical persistent data directory', () => {
  const stable = resolveApplicationMetadata();
  const appDataPath = path.resolve('fixture-app-data');
  assert.equal(
    resolveUserDataDirectory({ appDataPath, metadata: stable }),
    path.join(appDataPath, 'Fraia'),
  );
  const configuredPath = path.resolve('fixture-isolated-test-data');
  assert.equal(
    resolveUserDataDirectory({
      appDataPath,
      configuredPath,
      metadata: stable,
    }),
    configuredPath,
  );
});

test('beta releases use an isolated product and persistent data directory', () => {
  const beta = resolveApplicationMetadata({
    fraiaReleaseChannel: 'beta',
    productName: 'Fraia Beta',
  });
  assert.deepEqual(beta, {
    channel: 'beta',
    productName: 'Fraia Beta',
    userDataDirectoryName: 'Fraia Beta',
  });
  const appDataPath = path.resolve('fixture-app-data');
  assert.equal(
    resolveUserDataDirectory({ appDataPath, metadata: beta }),
    path.join(appDataPath, 'Fraia Beta'),
  );
});

test('rejects an unknown release channel', () => {
  assert.throws(
    () => resolveApplicationMetadata({ fraiaReleaseChannel: 'nightly' }),
    /release channel must be stable or beta/,
  );
});

test('rejects a product name that does not match its channel identity', () => {
  assert.throws(
    () => resolveApplicationMetadata({
      fraiaReleaseChannel: 'beta',
      productName: 'Fraia',
    }),
    /requires productName Fraia Beta/,
  );
});

test('package verification discards stale ASAR headers after an in-place update', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-asar-replacement-'));
  try {
    const source = path.join(root, 'source');
    const app = path.join(root, 'Fraia.app');
    const resources = path.join(app, 'Contents', 'Resources');
    const archive = path.join(resources, 'app.asar');
    fs.mkdirSync(source);
    fs.mkdirSync(resources, { recursive: true });

    fs.writeFileSync(path.join(source, 'package.json'), '{"name":"old"}\n');
    await asar.createPackage(source, archive);
    assert.equal(extractPackageMetadata(app).name, 'old');

    fs.writeFileSync(
      path.join(source, 'package.json'),
      `${JSON.stringify({ name: 'new', marker: 'x'.repeat(1024) })}\n`,
    );
    fs.rmSync(archive);
    await asar.createPackage(source, archive);
    const updated = extractPackageMetadata(app);
    assert.equal(updated.name, 'new');
    assert.equal(updated.marker.length, 1024);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
