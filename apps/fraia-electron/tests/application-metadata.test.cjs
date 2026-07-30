const assert = require('node:assert/strict');
const path = require('node:path');
const test = require('node:test');

const {
  resolveApplicationMetadata,
  resolveUserDataDirectory,
} = require('../application-metadata.cjs');

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
