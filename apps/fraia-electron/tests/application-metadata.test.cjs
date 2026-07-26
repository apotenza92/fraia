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

test('resolves the separate beta identity', () => {
  assert.deepEqual(resolveApplicationMetadata({
    fraiaReleaseChannel: 'beta',
    productName: 'Fraia Beta',
  }), {
    channel: 'beta',
    productName: 'Fraia Beta',
    userDataDirectoryName: 'Fraia Beta',
  });
});

test('stable and beta resolve to separate persistent data directories', () => {
  const stable = resolveApplicationMetadata();
  const beta = resolveApplicationMetadata({
    fraiaReleaseChannel: 'beta',
    productName: 'Fraia Beta',
  });
  const appDataPath = path.resolve('fixture-app-data');
  assert.equal(
    resolveUserDataDirectory({ appDataPath, metadata: stable }),
    path.join(appDataPath, 'Fraia'),
  );
  assert.equal(
    resolveUserDataDirectory({ appDataPath, metadata: beta }),
    path.join(appDataPath, 'Fraia Beta'),
  );
  assert.notEqual(stable.userDataDirectoryName, beta.userDataDirectoryName);
  const configuredPath = path.resolve('fixture-isolated-test-data');
  assert.equal(
    resolveUserDataDirectory({
      appDataPath,
      configuredPath,
      metadata: beta,
    }),
    configuredPath,
  );
});

test('rejects an unknown release channel', () => {
  assert.throws(
    () => resolveApplicationMetadata({ fraiaReleaseChannel: 'nightly' }),
    /release channel must be stable or beta/,
  );
});

test('rejects product metadata that disagrees with its channel', () => {
  assert.throws(
    () => resolveApplicationMetadata({
      fraiaReleaseChannel: 'beta',
      productName: 'Fraia',
    }),
    /beta metadata requires productName Fraia Beta/,
  );
});
