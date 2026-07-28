const assert = require('node:assert/strict');
const test = require('node:test');
const { releaseContract, metadataFileName } = require('../release-contract.cjs');
const packageMetadata = require('../package.json');

test('native package metadata points to Fraia public source', () => {
  assert.equal(packageMetadata.homepage, 'https://github.com/apotenza92/fraia');
});

test('Fraia has one stable durable application identity', () => {
  const stable = releaseContract({ channel: 'stable', platform: 'darwin', arch: 'arm64' });
  assert.equal(stable.appId, 'app.fraia.desktop');
  assert.equal(stable.productName, 'Fraia');
  assert.equal(stable.packageName, 'fraia-electron');
  assert.match(stable.feedUrl, /\/stable\/darwin\/arm64$/);
  assert.throws(
    () => releaseContract({ channel: 'beta', platform: 'darwin', arch: 'arm64' }),
    /channel must be one of stable/,
  );
});

test('the exact five solver-backed native targets resolve without cross-compilation aliases', () => {
  for (const [platform, arch] of [
    ['darwin', 'arm64'],
    ['darwin', 'x64'],
    ['linux', 'arm64'],
    ['linux', 'x64'],
    ['win32', 'x64'],
  ]) {
    const contract = releaseContract({ channel: 'stable', platform, arch });
    assert.equal(contract.platform, platform);
    assert.equal(contract.arch, arch);
    assert.equal(contract.outputDir.endsWith(`/stable/${platform}/${arch}`), true);
  }
  assert.throws(() => releaseContract({ platform: 'win32', arch: 'arm64' }), /does not support win32-arm64/);
  assert.throws(() => releaseContract({ platform: 'darwin', arch: 'universal' }), /architecture/);
});

test('updater metadata names match electron-builder platform conventions', () => {
  assert.equal(metadataFileName('darwin', 'arm64'), 'latest-mac.yml');
  assert.equal(metadataFileName('win32', 'x64'), 'latest.yml');
  assert.equal(metadataFileName('linux', 'x64'), 'latest-linux.yml');
  assert.equal(metadataFileName('linux', 'arm64'), 'latest-linux-arm64.yml');
  assert.throws(() => metadataFileName('win32', 'arm64'), /does not support win32-arm64/);
});
