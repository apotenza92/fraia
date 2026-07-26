const fs = require('node:fs');
const path = require('node:path');

const UPDATE_FREQUENCY_MS = Object.freeze({
  hourly: 60 * 60 * 1000,
  sixHours: 6 * 60 * 60 * 1000,
  twelveHours: 12 * 60 * 60 * 1000,
  daily: 24 * 60 * 60 * 1000,
  weekly: 7 * 24 * 60 * 60 * 1000,
});
const UPDATE_FREQUENCIES = Object.freeze(['never', 'startup', ...Object.keys(UPDATE_FREQUENCY_MS)]);
const SIX_HOURS_MS = 6 * 60 * 60 * 1000;

function safeWriteEvent(eventPath, event) {
  if (!eventPath) return;
  const target = path.resolve(eventPath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.appendFileSync(`${target}.jsonl`, `${JSON.stringify(event)}\n`, { mode: 0o600 });
  const temporary = `${target}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify(event)}\n`, { mode: 0o600 });
  fs.renameSync(temporary, target);
}

function validateTestFeedUrl(value) {
  const parsed = new URL(value);
  if (!['127.0.0.1', '::1', 'localhost'].includes(parsed.hostname)) {
    throw new Error('Fraia updater test feeds must be loopback-only.');
  }
  return parsed.toString().replace(/\/$/, '');
}

function readFrequency(filePath, fallback) {
  try {
    const parsed = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    if (UPDATE_FREQUENCIES.includes(parsed.frequency)) return parsed.frequency;
  } catch { /* missing or invalid settings use the channel default */ }
  return fallback;
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  const temporary = `${filePath}.${process.pid}.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify(value)}\n`, { mode: 0o600 });
  fs.renameSync(temporary, filePath);
}

function readLastCheck(filePath) {
  try {
    const timestamp = JSON.parse(fs.readFileSync(filePath, 'utf8')).timestamp;
    return Number.isFinite(timestamp) && timestamp >= 0 ? timestamp : 0;
  } catch { return 0; }
}

function configureAutoUpdates({
  app,
  autoUpdater,
  packageMetadata,
  env = process.env,
  log = console,
  platform = process.platform,
  schedule = { clearInterval, clearTimeout, setInterval, setTimeout },
} = {}) {
  if (!app?.isPackaged || platform !== 'darwin' || env.FRAIA_DISABLE_UPDATES === '1') {
    return { enabled: false };
  }

  const testMode = env.FRAIA_E2E_UPDATER === '1';
  const feedUrl = testMode && env.FRAIA_UPDATE_FEED_URL
    ? validateTestFeedUrl(env.FRAIA_UPDATE_FEED_URL)
    : packageMetadata.fraiaUpdateFeedUrl;
  const channel = packageMetadata.fraiaReleaseChannel;
  if (!['stable', 'beta'].includes(channel) || typeof feedUrl !== 'string' || !feedUrl) {
    throw new Error('Packaged Fraia updater metadata is invalid.');
  }

  autoUpdater.autoDownload = true;
  autoUpdater.autoInstallOnAppQuit = true;
  autoUpdater.allowPrerelease = channel === 'beta';
  autoUpdater.setFeedURL({ provider: 'generic', url: feedUrl, channel: 'latest' });

  const eventPath = testMode ? env.FRAIA_UPDATER_EVENT_PATH : null;
  const event = (name, details = {}) => safeWriteEvent(eventPath, {
    name,
    channel,
    currentVersion: app.getVersion(),
    executablePath: process.execPath,
    pid: process.pid,
    ...details,
  });

  if (testMode && env.FRAIA_E2E_EXPECT_VERSION === app.getVersion()) {
    event('updated-runtime-launched');
    schedule.setTimeout(() => app.quit(), 100);
    return { channel, enabled: true, feedUrl, installed: true };
  }

  autoUpdater.on('error', (error) => {
    event('error', { message: String(error?.message || error) });
    log.error('[updater] update check failed', error);
  });
  autoUpdater.on('update-available', (info) => event('update-available', { version: info.version }));
  autoUpdater.on('update-not-available', (info) => event('update-not-available', { version: info.version }));
  autoUpdater.on('update-downloaded', (info) => {
    event('update-downloaded', { version: info.version });
    if (testMode && env.FRAIA_E2E_INSTALL_UPDATE === '1') {
      schedule.setTimeout(() => autoUpdater.quitAndInstall(false, true), 100);
    }
  });

  const userData = app.getPath('userData');
  const frequencyPath = path.join(userData, 'update-frequency.json');
  const lastCheckPath = path.join(userData, 'last-update-check.json');
  const defaultFrequency = channel === 'beta' ? 'sixHours' : 'daily';
  let frequency = testMode ? 'sixHours' : readFrequency(frequencyPath, defaultFrequency);
  let initialTimer = null;
  let intervalTimer = null;
  let checkPromise = null;

  const check = () => {
    if (checkPromise) return checkPromise;
    checkPromise = autoUpdater.checkForUpdates()
      .catch((error) => {
        event('error', { message: String(error?.message || error) });
        log.error('[updater] update check failed', error);
        throw error;
      })
      .finally(() => {
        if (!testMode) writeJson(lastCheckPath, { timestamp: Date.now() });
        checkPromise = null;
      });
    return checkPromise;
  };

  const stopSchedule = () => {
    if (initialTimer !== null) schedule.clearTimeout(initialTimer);
    if (intervalTimer !== null) schedule.clearInterval(intervalTimer);
    initialTimer = null;
    intervalTimer = null;
  };
  const startSchedule = () => {
    stopSchedule();
    if (frequency === 'never') return;
    const scheduledFrequency = frequency;
    const interval = UPDATE_FREQUENCY_MS[frequency];
    const elapsed = interval ? Date.now() - readLastCheck(lastCheckPath) : 0;
    const initialDelay = testMode
      ? 0
      : interval && elapsed < interval
        ? Math.max(30_000, interval - elapsed)
        : 30_000;
    initialTimer = schedule.setTimeout(async () => {
      await check().catch(() => {});
      if (scheduledFrequency === frequency && interval) {
        intervalTimer = schedule.setInterval(() => { void check().catch(() => {}); }, interval);
      }
    }, initialDelay);
  };
  const setFrequency = (next) => {
    if (!UPDATE_FREQUENCIES.includes(next)) throw new Error(`Invalid updater frequency: ${next}`);
    frequency = next;
    if (!testMode) writeJson(frequencyPath, { frequency });
    startSchedule();
    return frequency;
  };

  startSchedule();
  return {
    channel,
    checkNow: check,
    enabled: true,
    feedUrl,
    get frequency() { return frequency; },
    installed: false,
    setFrequency,
    stop: stopSchedule,
  };
}

module.exports = {
  SIX_HOURS_MS,
  UPDATE_FREQUENCIES,
  UPDATE_FREQUENCY_MS,
  configureAutoUpdates,
  readFrequency,
  readLastCheck,
  safeWriteEvent,
  validateTestFeedUrl,
};
