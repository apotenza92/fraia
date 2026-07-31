const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const { releaseContract, metadataFileName } = require('../release-contract.cjs');
const packageMetadata = require('../package.json');

test('native package metadata points to Fraia public source', () => {
  assert.equal(packageMetadata.homepage, 'https://github.com/apotenza92/fraia');
});

test('Homebrew remains an optional update path and cannot take ownership of the in-app updater', () => {
  const repositoryRoot = path.resolve(__dirname, '..', '..', '..');
  const agentContract = fs.readFileSync(path.join(repositoryRoot, 'AGENTS.md'), 'utf8');
  const desktopReadme = fs.readFileSync(path.join(repositoryRoot, 'apps', 'fraia-electron', 'README.md'), 'utf8');
  const updateManager = fs.readFileSync(path.join(repositoryRoot, 'apps', 'fraia-electron', 'update-manager.cjs'), 'utf8');

  assert.match(agentContract, /`auto_updates true`/);
  assert.match(agentContract, /must not disable, redirect, wrap, or become a prerequisite/);
  assert.match(desktopReadme, /Homebrew remains an optional installation and update route/);
  assert.doesNotMatch(updateManager, /HOMEBREW|spawnSync|execFile|brew upgrade/);
});

test('the renderer receives only the public updater state and narrow update actions', () => {
  const repositoryRoot = path.resolve(__dirname, '..', '..', '..');
  const main = fs.readFileSync(path.join(repositoryRoot, 'apps', 'fraia-electron', 'main.js'), 'utf8');
  const preload = fs.readFileSync(path.join(repositoryRoot, 'apps', 'fraia-electron', 'preload.js'), 'utf8');

  for (const channel of [
    'fraia:updateStatus',
    'fraia:checkForUpdates',
    'fraia:setUpdateFrequency',
    'fraia:installUpdate',
  ]) {
    assert.match(main, new RegExp(`ipcMain\\.handle\\(['"]${channel}`));
    assert.match(preload, new RegExp(`ipcRenderer\\.invoke\\(['"]${channel}`));
  }
  assert.match(preload, /ipcRenderer\.on\(['"]fraia:updateStatus/);
  assert.doesNotMatch(preload, /autoUpdater|feedUrl|TUF|root\.json/);
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

test('electron-builder emits isolated beta package identity and artifact names', () => {
  const configPath = path.resolve(__dirname, '..', 'electron-builder.config.cjs');
  const config = JSON.parse(execFileSync(
    process.execPath,
    ['-e', `const value=require(${JSON.stringify(configPath)}); process.stdout.write(JSON.stringify({appId:value.appId,productName:value.productName,name:value.extraMetadata.name,artifactName:value.linux.artifactName,executableName:value.linux.executableName,icon:value.linux.icon,feed:value.extraMetadata.fraiaUpdateFeedUrl}))`],
    {
      encoding: 'utf8',
      env: {
        ...process.env,
        FRAIA_RELEASE_ARCH: 'arm64',
        FRAIA_RELEASE_CHANNEL: 'beta',
        FRAIA_RELEASE_PLATFORM: 'linux',
      },
    },
  ));
  assert.equal(config.appId, 'app.fraia.desktop.beta');
  assert.equal(config.productName, 'Fraia Beta');
  assert.equal(config.name, 'fraia-electron-beta');
  assert.equal(config.executableName, 'fraia-electron-beta');
  assert.equal(config.artifactName, 'Fraia-Beta-Linux-arm64.${ext}');
  assert.match(config.icon, /build\/beta\/icons$/);
  assert.match(config.feed, /\/beta\/linux\/arm64$/);
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
    include: 'build/installer.nsh',
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
    include: 'build/installer.nsh',
  });
  assert.equal(readNsisConfig({
    FRAIA_NSIS_ASSISTED_MIGRATION_FIXTURE: '1',
  }).oneClick, true);
});

test('Fraia stable and beta releases have separate durable application identities', () => {
  const stable = releaseContract({ channel: 'stable', platform: 'darwin', arch: 'arm64' });
  const beta = releaseContract({ channel: 'beta', platform: 'darwin', arch: 'arm64' });
  assert.equal(stable.appId, 'app.fraia.desktop');
  assert.equal(stable.productName, 'Fraia');
  assert.equal(stable.packageName, 'fraia-electron');
  assert.equal(stable.userDataDirectoryName, 'Fraia');
  assert.equal(stable.artifactPrefix, 'Fraia');
  assert.equal(stable.iconVariant, 'stable');
  assert.match(stable.feedUrl, /\/stable\/darwin\/arm64$/);
  assert.equal(beta.appId, 'app.fraia.desktop.beta');
  assert.equal(beta.productName, 'Fraia Beta');
  assert.equal(beta.packageName, 'fraia-electron-beta');
  assert.equal(beta.userDataDirectoryName, 'Fraia Beta');
  assert.equal(beta.artifactPrefix, 'Fraia-Beta');
  assert.equal(beta.iconVariant, 'beta');
  assert.match(beta.feedUrl, /\/beta\/darwin\/arm64$/);
  assert.notEqual(beta.appId, stable.appId);
  assert.notEqual(beta.userDataDirectoryName, stable.userDataDirectoryName);
});

test('the exact six solver-backed native targets resolve without cross-compilation aliases', () => {
  for (const [platform, arch] of [
    ['darwin', 'arm64'],
    ['darwin', 'x64'],
    ['linux', 'arm64'],
    ['linux', 'x64'],
    ['win32', 'arm64'],
    ['win32', 'x64'],
  ]) {
    for (const channel of ['stable', 'beta']) {
      const contract = releaseContract({ channel, platform, arch });
      assert.equal(contract.platform, platform);
      assert.equal(contract.arch, arch);
      assert.equal(contract.outputDir.endsWith(`/${channel}/${platform}/${arch}`), true);
    }
  }
  assert.throws(() => releaseContract({ platform: 'darwin', arch: 'universal' }), /architecture/);
});

test('updater metadata names match electron-builder platform conventions', () => {
  assert.equal(metadataFileName('darwin', 'arm64'), 'latest-mac.yml');
  assert.equal(metadataFileName('win32', 'arm64'), 'latest.yml');
  assert.equal(metadataFileName('win32', 'x64'), 'latest.yml');
  assert.equal(metadataFileName('linux', 'x64'), 'latest-linux.yml');
  assert.equal(metadataFileName('linux', 'arm64'), 'latest-linux-arm64.yml');
});
