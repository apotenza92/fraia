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

test('rejects an unknown release channel', () => {
  assert.throws(
    () => resolveApplicationMetadata({ fraiaReleaseChannel: 'nightly' }),
    /release channel must be stable/,
  );
});

test('rejects a separate beta application identity', () => {
  assert.throws(
    () => resolveApplicationMetadata({
      fraiaReleaseChannel: 'beta',
      productName: 'Fraia Beta',
    }),
    /release channel must be stable/,
  );
});
