const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const test = require('node:test');
const { releaseContract, metadataFileName } = require('../release-contract.cjs');
const packageMetadata = require('../package.json');

test('native package metadata points to Fraia public source', () => {
  assert.equal(packageMetadata.homepage, 'https://github.com/apotenza92/fraia');
});

test('Linux packages retain Fraia canonical architecture tokens across formats', () => {
  const configPath = path.resolve(__dirname, '..', 'electron-builder.config.cjs');
  for (const arch of ['arm64', 'x64']) {
    const artifactName = execFileSync(
      process.execPath,
      ['-e', `process.stdout.write(require(${JSON.stringify(configPath)}).linux.artifactName)`],
      {
        encoding: 'utf8',
        env: {
          ...process.env,
          FRAIA_RELEASE_ARCH: arch,
          FRAIA_RELEASE_CHANNEL: 'stable',
          FRAIA_RELEASE_PLATFORM: 'linux',
        },
      },
    );
    assert.equal(artifactName, `Fraia-Linux-${arch}.\${ext}`);
  }
});

test('Windows uses one-click NSIS while retaining an isolated assisted-installer migration fixture', () => {
  const configPath = path.resolve(__dirname, '..', 'electron-builder.config.cjs');
  const readNsisConfig = (extraEnvironment = {}) => JSON.parse(execFileSync(
    process.execPath,
    ['-e', `process.stdout.write(JSON.stringify(require(${JSON.stringify(configPath)}).nsis))`],
    {
      encoding: 'utf8',
      env: {
        ...process.env,
        FRAIA_RELEASE_ARCH: 'x64',
        FRAIA_RELEASE_CHANNEL: 'stable',
        FRAIA_RELEASE_PLATFORM: 'win32',
        ...extraEnvironment,
      },
    },
  ));

  assert.deepEqual(readNsisConfig(), {
    oneClick: true,
    perMachine: false,
    allowElevation: false,
    allowToChangeInstallationDirectory: false,
    deleteAppDataOnUninstall: false,
  });
  assert.deepEqual(readNsisConfig({
    FRAIA_E2E_UPDATER: '1',
    FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE: '1',
  }), {
    oneClick: false,
    perMachine: false,
    allowElevation: true,
    allowToChangeInstallationDirectory: true,
    deleteAppDataOnUninstall: false,
  });
  assert.equal(readNsisConfig({
    FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE: '1',
  }).oneClick, true);
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
