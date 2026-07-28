const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const test = require('node:test');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { UPDATE_FREQUENCY_MS, configureAutoUpdates, safeWriteEvent, validateTestFeedUrl } = require('../update-manager.cjs');

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
  result.setFrequency('daily');
  assert.equal(result.frequency, 'daily');
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(userData, 'update-frequency.json'))), { frequency: 'daily' });
  assert.equal(scheduled.at(-1)[0], 'timeout');
  assert.ok(scheduled.at(-1)[1] <= UPDATE_FREQUENCY_MS.daily && scheduled.at(-1)[1] > UPDATE_FREQUENCY_MS.daily - 1_000);
  fs.rmSync(userData, { recursive: true, force: true });
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

test('unpackaged and non-macOS runtimes cannot activate production updating', () => {
  const updater = updaterDouble();
  const result = configureAutoUpdates({
    app: { isPackaged: false },
    autoUpdater: updater,
    packageMetadata: {},
  });
  assert.deepEqual(result, { enabled: false });
  assert.equal(updater.feed, undefined);
  const packaged = configureAutoUpdates({
    app: { isPackaged: true },
    autoUpdater: updater,
    packageMetadata: {},
    platform: 'linux',
  });
  assert.deepEqual(packaged, { enabled: false });
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
