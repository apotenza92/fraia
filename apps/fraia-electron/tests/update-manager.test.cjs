const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const test = require('node:test');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {
  UPDATE_FREQUENCY_MS,
  UPDATE_RETRY_MS,
  configureAutoUpdates,
  normalizeDownloadProgress,
  normalizeReleaseNotes,
  readCheckHistory,
  safeWriteEvent,
  userFacingUpdateError,
  validateTestFeedUrl,
} = require('../update-manager.cjs');

function updaterDouble() {
  const updater = new EventEmitter();
  updater.setFeedURL = (value) => { updater.feed = value; };
  updater.checkForUpdates = async () => {};
  updater.quitAndInstall = (...args) => { updater.installArgs = args; };
  return updater;
}

test('updater tests accept only loopback feed overrides', () => {
  assert.match(validateTestFeedUrl('http://127.0.0.1:1234/feed'), /^http:\/\/127\.0\.0\.1/);
  assert.throws(() => validateTestFeedUrl('https://example.com/feed'), /loopback-only/);
  assert.throws(() => validateTestFeedUrl('https://localhost/feed'), /loopback-only HTTP/);
  assert.throws(() => validateTestFeedUrl('ftp://127.0.0.1/feed'), /loopback-only HTTP/);
});

test('macOS stable updater is automatic and configurable', async () => {
  const updater = updaterDouble();
  const scheduled = [];
  const timeoutCallbacks = [];
  const schedule = {
    clearInterval: () => {},
    clearTimeout: () => {},
    setTimeout: (fn, ms) => { timeoutCallbacks.push(fn); scheduled.push(['timeout', ms]); return 1; },
    setInterval: (fn, ms) => { scheduled.push(['interval', ms]); return 2; },
  };
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-settings-'));
  const result = configureAutoUpdates({
    app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.1.0' },
    autoUpdater: updater,
    packageMetadata: {
      fraiaReleaseChannel: 'stable',
      fraiaUpdateFeedUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/darwin/arm64',
    },
    env: {},
    platform: 'darwin',
    schedule,
  });
  assert.equal(result.enabled, true);
  assert.equal(updater.autoDownload, true);
  assert.equal(updater.autoInstallOnAppQuit, true);
  assert.equal(updater.allowPrerelease, false);
  assert.equal(result.frequency, 'daily');
  assert.deepEqual(scheduled, [['timeout', 30_000]]);
  await timeoutCallbacks[0]();
  assert.deepEqual(scheduled.at(-1), ['interval', UPDATE_FREQUENCY_MS.daily]);
  updater.emit('update-not-available', { version: '0.1.0' });
  result.setFrequency('daily');
  assert.equal(result.frequency, 'daily');
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(userData, 'update-frequency.json'))), { frequency: 'daily' });
  assert.equal(scheduled.at(-1)[0], 'timeout');
  assert.ok(scheduled.at(-1)[1] <= UPDATE_FREQUENCY_MS.daily && scheduled.at(-1)[1] > UPDATE_FREQUENCY_MS.daily - 1_000);
  fs.rmSync(userData, { recursive: true, force: true });
});

test('beta updater stays on the isolated beta feed and accepts prereleases', () => {
  const updater = updaterDouble();
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-beta-updater-'));
  const result = configureAutoUpdates({
    app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.1.0-beta.1' },
    autoUpdater: updater,
    packageMetadata: {
      fraiaReleaseChannel: 'beta',
      fraiaUpdateFeedUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/beta/darwin/arm64',
    },
    env: {},
    platform: 'darwin',
    schedule: {
      clearInterval() {},
      clearTimeout() {},
      setInterval() { return 1; },
      setTimeout() { return 2; },
    },
  });
  assert.equal(result.channel, 'beta');
  assert.equal(updater.allowPrerelease, true);
  assert.deepEqual(updater.feed, {
    provider: 'generic',
    url: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/beta/darwin/arm64',
    channel: 'latest',
  });
  fs.rmSync(userData, { recursive: true, force: true });
});

test('downloaded updates show release notes and respect restart or later', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-ready-'));
  const schedule = { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } };
  const metadata = { fraiaReleaseChannel: 'stable', fraiaUpdateFeedUrl: 'https://example.invalid/feed' };
  const prompts = [];
  const laterUpdater = updaterDouble();
  configureAutoUpdates({
    app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
    autoUpdater: laterUpdater,
    packageMetadata: metadata,
    platform: 'darwin',
    schedule,
    showUpdateReady: async (details) => { prompts.push(details); return { response: 1 }; },
  });
  laterUpdater.emit('update-downloaded', {
    version: '0.0.2',
    releaseNotes: [{ version: '0.0.2', note: '### Added\n\n- Native solver.' }],
  });
  laterUpdater.emit('update-downloaded', {
    version: '0.0.2',
    releaseNotes: 'Duplicate event must not open a second prompt.',
  });
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(prompts, [{
    version: '0.0.2',
    releaseNotes: '0.0.2\nAdded\n\n• Native solver.',
  }]);
  assert.equal(laterUpdater.installArgs, undefined);
  assert.equal(laterUpdater.autoInstallOnAppQuit, true);

  const restartUpdater = updaterDouble();
  configureAutoUpdates({
    app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
    autoUpdater: restartUpdater,
    packageMetadata: metadata,
    platform: 'darwin',
    schedule,
    showUpdateReady: async () => ({ response: 0 }),
  });
  restartUpdater.emit('update-downloaded', { version: '0.0.2', releaseNotes: 'Fixed analysis.' });
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(restartUpdater.installArgs, [false, true]);
  fs.rmSync(userData, { recursive: true, force: true });
});

test('updater publishes check, progress, ready, and up-to-date states for the in-app experience', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-status-'));
  try {
    const statuses = [];
    const updater = updaterDouble();
    updater.checkForUpdates = async () => {
      updater.emit('checking-for-update');
      updater.emit('update-not-available', { version: '0.0.1' });
    };
    const result = configureAutoUpdates({
      app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
      autoUpdater: updater,
      packageMetadata: { fraiaReleaseChannel: 'stable', fraiaUpdateFeedUrl: 'https://updates.example/stable' },
      platform: 'darwin',
      schedule: { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } },
      onStatusChange: (status) => statuses.push(status),
    });

    await result.checkNow();
    assert.equal(result.getStatus().phase, 'up-to-date');
    assert.ok(result.getStatus().lastSuccessfulCheckAt);
    assert.equal(readCheckHistory(path.join(userData, 'last-update-check.json')).lastSuccessfulCheckAt > 0, true);

    updater.emit('update-available', { version: '0.0.2' });
    updater.emit('download-progress', {
      bytesPerSecond: 2_000_000,
      percent: 50.4,
      total: 20_000_000,
      transferred: 10_000_000,
    });
    assert.deepEqual(result.getStatus().progress, {
      bytesPerSecond: 2_000_000,
      etaSeconds: 5,
      percent: 50,
      total: 20_000_000,
      transferred: 10_000_000,
    });
    updater.emit('update-downloaded', { version: '0.0.2', releaseNotes: '- Safer updates' });
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(result.getStatus().phase, 'ready');
    assert.equal(result.getStatus().releaseNotes, '• Safer updates');
    assert.ok(statuses.some((status) => status.phase === 'checking'));
    assert.ok(statuses.some((status) => status.phase === 'downloading'));
    assert.ok(statuses.some((status) => status.phase === 'ready'));
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('download progress and user-facing errors are bounded and safe to display', () => {
  assert.deepEqual(normalizeDownloadProgress({
    bytesPerSecond: 100,
    percent: 140,
    total: 1_000,
    transferred: 2_000,
  }), {
    bytesPerSecond: 100,
    etaSeconds: null,
    percent: 100,
    total: 1_000,
    transferred: 1_000,
  });
  assert.match(userFacingUpdateError(new Error('TUF metadata signature verification failed')), /securely verify/);
  assert.match(userFacingUpdateError(new Error('network request timed out')), /internet connection/);
  assert.doesNotMatch(userFacingUpdateError(new Error('secret internal path /tmp/private')), /private|\/tmp/);
});

test('legacy last-check timestamps migrate silently to successful-check history', () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-history-'));
  try {
    const historyPath = path.join(userData, 'last-update-check.json');
    fs.writeFileSync(historyPath, '{"timestamp":123456}\n');
    assert.deepEqual(readCheckHistory(historyPath), {
      lastAttemptAt: 123456,
      lastSuccessfulCheckAt: 123456,
    });
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('failed automatic checks do not become successful checks and retry with backoff', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-retry-'));
  try {
    const timeouts = [];
    const updater = updaterDouble();
    updater.checkForUpdates = async () => {
      throw new Error('network request timed out');
    };
    const result = configureAutoUpdates({
      app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
      autoUpdater: updater,
      packageMetadata: { fraiaReleaseChannel: 'stable', fraiaUpdateFeedUrl: 'https://updates.example/stable' },
      platform: 'darwin',
      log: { error() {} },
      schedule: {
        clearInterval() {},
        clearTimeout() {},
        setInterval() { return 1; },
        setTimeout(fn, ms) { timeouts.push({ fn, ms }); return timeouts.length; },
      },
    });

    await assert.rejects(() => result.checkNow({ automatic: true }), /timed out/);
    assert.equal(result.getStatus().phase, 'error');
    assert.equal(result.getStatus().lastSuccessfulCheckAt, null);
    assert.equal(timeouts.some(({ ms }) => ms === UPDATE_RETRY_MS[0]), true);
    assert.equal(readCheckHistory(path.join(userData, 'last-update-check.json')).lastSuccessfulCheckAt, 0);
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('macOS in-app updating is independent of a Homebrew installation origin', () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-homebrew-'));
  try {
    const updater = updaterDouble();
    const result = configureAutoUpdates({
      app: {
        isPackaged: true,
        getAppPath: () => '/opt/homebrew/Caskroom/fraia/0.0.5/Fraia.app/Contents/Resources/app.asar',
        getPath: () => userData,
        getVersion: () => '0.0.5',
      },
      autoUpdater: updater,
      env: {
        HOMEBREW_CASK_OPTS: '--appdir=/Applications',
        HOMEBREW_CELLAR: '/opt/homebrew/Cellar',
        HOMEBREW_PREFIX: '/opt/homebrew',
      },
      packageMetadata: {
        fraiaReleaseChannel: 'stable',
        fraiaUpdateFeedUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/darwin/arm64',
      },
      platform: 'darwin',
      schedule: { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } },
    });

    assert.equal(result.enabled, true);
    assert.equal(result.getStatus().phase, 'idle');
    assert.equal(updater.autoDownload, true);
    assert.equal(updater.feed.url, 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/darwin/arm64');
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('release-note normalization handles updater string, array, and empty forms', () => {
  assert.equal(normalizeReleaseNotes('  Added native solving.  '), 'Added native solving.');
  assert.equal(
    normalizeReleaseNotes([{ version: '0.0.2', note: 'Added native solving.' }]),
    '0.0.2\nAdded native solving.',
  );
  assert.equal(
    normalizeReleaseNotes('### Fixed\n\n- [Updater](https://example.invalid) reliability.'),
    'Fixed\n\n• Updater reliability.',
  );
  assert.match(normalizeReleaseNotes(undefined), /reliability and compatibility/);
});

test('stable defaults daily and persisted frequency survives restart', () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-settings-'));
  const schedule = { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } };
  const app = { isPackaged: true, getPath: () => userData, getVersion: () => '0.1.0' };
  const metadata = { fraiaReleaseChannel: 'stable', fraiaUpdateFeedUrl: 'https://example.invalid/stable/darwin/arm64' };
  const first = configureAutoUpdates({ app, autoUpdater: updaterDouble(), packageMetadata: metadata, platform: 'darwin', schedule });
  assert.equal(first.frequency, 'daily');
  first.setFrequency('weekly');
  const second = configureAutoUpdates({ app, autoUpdater: updaterDouble(), packageMetadata: metadata, platform: 'darwin', schedule });
  assert.equal(second.frequency, 'weekly');
  fs.rmSync(userData, { recursive: true, force: true });
});

test('unpackaged and unsupported runtimes cannot activate production updating', () => {
  const updater = updaterDouble();
  const result = configureAutoUpdates({
    app: { isPackaged: false },
    autoUpdater: updater,
    packageMetadata: {},
  });
  assert.equal(result.enabled, false);
  assert.equal(result.getStatus().phase, 'disabled');
  assert.equal(updater.feed, undefined);
  const packaged = configureAutoUpdates({
    app: { isPackaged: true },
    autoUpdater: updater,
    packageMetadata: {},
    platform: 'freebsd',
  });
  assert.equal(packaged.enabled, false);
  assert.equal(packaged.getStatus().phase, 'disabled');
});

test('Linux distro packages defer to their package manager while AppImage can self-update', () => {
  const updater = updaterDouble();
  const packaged = configureAutoUpdates({
    app: { isPackaged: true },
    autoUpdater: updater,
    env: {},
    packageMetadata: {},
    platform: 'linux',
  });
  assert.equal(packaged.enabled, false);
  assert.equal(packaged.reason, 'linux-package-manager');
  assert.equal(packaged.getStatus().phase, 'managed');
  assert.equal(updater.feed, undefined);
});

test('Linux AppImage refreshes TUF trust before checking and preserves user data', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-appimage-'));
  try {
    const projectState = '{"schemaVersion":1,"name":"Existing model"}\n';
    fs.mkdirSync(path.join(userData, 'projects', 'default'), { recursive: true });
    fs.writeFileSync(path.join(userData, 'projects', 'default', 'fraia.project.json'), projectState);
    const updater = updaterDouble();
    let refreshes = 0;
    let closes = 0;
    const result = await configureAutoUpdates({
      app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
      autoUpdater: updater,
      createVerifiedFeed: async (options) => ({
        close: async () => { closes += 1; },
        feedUrl: 'http://127.0.0.1:43124',
        refresh: async () => { refreshes += 1; },
        options,
      }),
      env: { APPIMAGE: '/opt/Fraia-Linux-x64.AppImage' },
      packageMetadata: {
        fraiaReleaseChannel: 'stable',
        fraiaTufRepositoryUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/linux/x64/tuf',
        fraiaUpdateFeedUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/linux/x64',
        fraiaUpdateTargetName: 'latest-linux.yml',
      },
      platform: 'linux',
      resourcesPath: path.join(userData, 'resources'),
      schedule: { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } },
      showUpdateReady: async () => ({ response: 0 }),
    });
    assert.equal(result.enabled, true);
    assert.equal(result.trustedMetadata, true);
    assert.equal(updater.feed.url, 'http://127.0.0.1:43124');
    await result.checkNow();
    assert.equal(refreshes, 1);
    updater.emit('update-downloaded', { version: '0.0.2', releaseNotes: 'Secure AppImage update.' });
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(updater.installArgs, [true, true]);
    assert.equal(
      fs.readFileSync(path.join(userData, 'projects', 'default', 'fraia.project.json'), 'utf8'),
      projectState,
    );
    result.stop();
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(closes, 1);
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('packaged non-macOS E2E runs can exercise TUF only through loopback HTTP', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-tuf-e2e-'));
  try {
    let received;
    const updater = updaterDouble();
    const result = await configureAutoUpdates({
      app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
      autoUpdater: updater,
      createVerifiedFeed: async (options) => {
        received = options;
        return {
          close: async () => {},
          feedUrl: 'http://127.0.0.1:43125',
          refresh: async () => {},
        };
      },
      env: {
        FRAIA_E2E_TUF_REPOSITORY_URL: 'http://127.0.0.1:43126/tuf',
        FRAIA_E2E_UPDATER: '1',
      },
      packageMetadata: {
        fraiaReleaseChannel: 'stable',
        fraiaTufRepositoryUrl: 'https://production.invalid/tuf',
        fraiaUpdateFeedUrl: 'https://production.invalid',
        fraiaUpdateTargetName: 'latest.yml',
      },
      platform: 'win32',
      resourcesPath: path.join(userData, 'resources'),
      schedule: { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout(fn) { void fn(); return 2; } },
    });
    assert.equal(result.enabled, true);
    assert.equal(result.trustedMetadata, true);
    assert.equal(received.repositoryUrl, 'http://127.0.0.1:43126/tuf');
    assert.equal(received.allowLoopbackHttp, true);
    result.stop();
    await new Promise((resolve) => setImmediate(resolve));

    assert.throws(
      () => configureAutoUpdates({
        app: { isPackaged: true, getPath: () => userData },
        autoUpdater: updaterDouble(),
        env: {
          FRAIA_E2E_TUF_REPOSITORY_URL: 'https://attacker.example/tuf',
          FRAIA_E2E_UPDATER: '1',
        },
        packageMetadata: {
          fraiaReleaseChannel: 'stable',
          fraiaUpdateFeedUrl: 'https://production.invalid',
        },
        platform: 'win32',
      }),
      /loopback-only HTTP/,
    );
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('Windows uses TUF-authenticated metadata, keeps settings, and requests a silent install', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-windows-'));
  try {
    fs.writeFileSync(path.join(userData, 'update-frequency.json'), '{"frequency":"weekly"}\n');
    fs.writeFileSync(path.join(userData, 'main-window-state.json'), '{"width":1180,"height":760}\n');
    fs.mkdirSync(path.join(userData, 'ai'), { recursive: true });
    const encryptedCredentialBytes = Buffer.from([0x01, 0x03, 0x03, 0x07]);
    fs.writeFileSync(path.join(userData, 'ai', 'credentials.bin'), encryptedCredentialBytes);
    const updater = updaterDouble();
    let received;
    let closed = false;
    let refreshes = 0;
    const result = await configureAutoUpdates({
      app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.0.1' },
      autoUpdater: updater,
      createVerifiedFeed: async (options) => {
        received = options;
        return {
          close: async () => { closed = true; },
          feedUrl: 'http://127.0.0.1:43123',
          refresh: async () => { refreshes += 1; },
        };
      },
      packageMetadata: {
        fraiaReleaseChannel: 'stable',
        fraiaTufRepositoryUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/win32/x64/tuf',
        fraiaUpdateFeedUrl: 'https://raw.githubusercontent.com/apotenza92/fraia/updates/stable/win32/x64',
        fraiaUpdateTargetName: 'latest.yml',
      },
      env: {},
      platform: 'win32',
      resourcesPath: path.join(userData, 'resources'),
      schedule: { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } },
      showUpdateReady: async () => ({ response: 0 }),
    });
  assert.equal(result.enabled, true);
  assert.equal(updater.disableWebInstaller, true);
    assert.equal(result.trustedMetadata, true);
    assert.equal(result.frequency, 'weekly');
    assert.equal(updater.feed.url, 'http://127.0.0.1:43123');
    assert.equal(received.targetName, 'latest.yml');
    assert.equal(received.trustDir, path.join(userData, 'update-trust'));
    assert.equal(
      received.embeddedRootPath,
      path.join(userData, 'resources', 'update-trust', 'root.json'),
    );
    await result.checkNow();
    assert.equal(refreshes, 1);
    updater.emit('update-downloaded', { version: '0.0.2', releaseNotes: 'Secure update.' });
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(updater.installArgs, [true, true]);
    result.stop();
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(closed, true);
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(userData, 'update-frequency.json'))), {
      frequency: 'weekly',
    });
    assert.equal(
      fs.readFileSync(path.join(userData, 'main-window-state.json'), 'utf8'),
      '{"width":1180,"height":760}\n',
    );
    assert.deepEqual(
      fs.readFileSync(path.join(userData, 'ai', 'credentials.bin')),
      encryptedCredentialBytes,
    );
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('Windows closes its verified local feed before handing off to the installer', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-windows-handoff-'));
  try {
    const order = [];
    let releaseClose;
    let forceExitCallback;
    const updater = updaterDouble();
    updater.quitAndInstall = (...args) => {
      order.push(['install', args]);
    };
    await configureAutoUpdates({
      app: {
        exit: (code) => { order.push(['exit', code]); },
        isPackaged: true,
        getPath: () => userData,
        getVersion: () => '0.0.1',
      },
      autoUpdater: updater,
      createVerifiedFeed: async () => ({
        close: () => new Promise((resolve) => {
          order.push(['close-start']);
          releaseClose = () => {
            order.push(['close-finished']);
            resolve();
          };
        }),
        feedUrl: 'http://127.0.0.1:43127',
        refresh: async () => {},
      }),
      packageMetadata: {
        fraiaReleaseChannel: 'stable',
        fraiaTufRepositoryUrl: 'https://updates.example/fraia/tuf',
        fraiaUpdateFeedUrl: 'https://updates.example/fraia',
        fraiaUpdateTargetName: 'latest.yml',
      },
      env: {},
      platform: 'win32',
      prepareForInstall: async () => {
        order.push(['prepare-start']);
        await new Promise((resolve) => setImmediate(resolve));
        order.push(['prepare-finished']);
      },
      resourcesPath: path.join(userData, 'resources'),
      schedule: {
        clearInterval() {},
        clearTimeout() {},
        setInterval() { return 1; },
        setTimeout(callback, delay) {
          if (delay === 3_000) forceExitCallback = callback;
          return 2;
        },
      },
      showUpdateReady: async () => ({ response: 0 }),
    });
    updater.emit('update-downloaded', { version: '0.0.2', releaseNotes: 'Secure update.' });
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(order, [['prepare-start']]);
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(order, [
      ['prepare-start'],
      ['prepare-finished'],
      ['close-start'],
    ]);
    releaseClose();
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(order, [
      ['prepare-start'],
      ['prepare-finished'],
      ['close-start'],
      ['close-finished'],
      ['install', [true, true]],
    ]);
    assert.equal(typeof forceExitCallback, 'function');
    forceExitCallback();
    assert.deepEqual(order.at(-1), ['exit', 0]);
  } finally {
    fs.rmSync(userData, { recursive: true, force: true });
  }
});

test('never and startup frequencies do not create repeating schedules', async () => {
  const timeouts = [];
  const intervals = [];
  const schedule = {
    clearInterval() {},
    clearTimeout() {},
    setInterval(fn, ms) { intervals.push([fn, ms]); return intervals.length; },
    setTimeout(fn, ms) { timeouts.push([fn, ms]); return timeouts.length; },
  };
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-frequency-'));
  const result = configureAutoUpdates({
    app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.1.0' },
    autoUpdater: updaterDouble(),
    packageMetadata: { fraiaReleaseChannel: 'stable', fraiaUpdateFeedUrl: 'https://example.invalid/feed' },
    platform: 'darwin',
    schedule,
  });
  result.setFrequency('never');
  const timeoutCount = timeouts.length;
  result.setFrequency('startup');
  assert.equal(timeouts.length, timeoutCount + 1);
  assert.equal(timeouts.at(-1)[1], 30_000);
  await timeouts.at(-1)[0]();
  assert.equal(intervals.length, 0);
  result.setFrequency('never');
  assert.equal(timeouts.length, timeoutCount + 1);
  fs.rmSync(userData, { recursive: true, force: true });
});

test('concurrent manual checks share one in-flight update request', async () => {
  const userData = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-concurrency-'));
  const updater = updaterDouble();
  let resolveCheck;
  let checks = 0;
  updater.checkForUpdates = () => {
    checks += 1;
    return new Promise((resolve) => { resolveCheck = resolve; });
  };
  const result = configureAutoUpdates({
    app: { isPackaged: true, getPath: () => userData, getVersion: () => '0.1.0' },
    autoUpdater: updater,
    packageMetadata: { fraiaReleaseChannel: 'stable', fraiaUpdateFeedUrl: 'https://example.invalid/feed' },
    platform: 'darwin',
    schedule: { clearInterval() {}, clearTimeout() {}, setInterval() { return 1; }, setTimeout() { return 2; } },
  });
  const first = result.checkNow();
  const second = result.checkNow();
  assert.equal(first, second);
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(checks, 1);
  resolveCheck();
  await first;
  fs.rmSync(userData, { recursive: true, force: true });
});

test('test event evidence is atomic, append-only, and process-identifiable', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-events-'));
  const eventPath = path.join(root, 'event.json');
  safeWriteEvent(eventPath, { name: 'first', pid: process.pid, executablePath: process.execPath });
  safeWriteEvent(eventPath, { name: 'second', pid: process.pid, executablePath: process.execPath });
  assert.equal(JSON.parse(fs.readFileSync(eventPath, 'utf8')).name, 'second');
  const history = fs.readFileSync(`${eventPath}.jsonl`, 'utf8').trim().split('\n').map(JSON.parse);
  assert.deepEqual(history.map((event) => event.name), ['first', 'second']);
  assert.ok(history.every((event) => event.pid === process.pid && event.executablePath === process.execPath));
  fs.rmSync(root, { recursive: true, force: true });
});
