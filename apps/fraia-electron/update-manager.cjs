const fs = require('node:fs');
const path = require('node:path');
const { createTufVerifiedUpdateFeed } = require('./tuf-update-feed.cjs');

const UPDATE_FREQUENCY_MS = Object.freeze({
  hourly: 60 * 60 * 1000,
  sixHours: 6 * 60 * 60 * 1000,
  twelveHours: 12 * 60 * 60 * 1000,
  daily: 24 * 60 * 60 * 1000,
  weekly: 7 * 24 * 60 * 60 * 1000,
});
const UPDATE_FREQUENCIES = Object.freeze(['never', 'startup', ...Object.keys(UPDATE_FREQUENCY_MS)]);

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
  if (
    parsed.protocol !== 'http:'
    || !['127.0.0.1', '::1', 'localhost'].includes(parsed.hostname)
  ) {
    throw new Error('Fraia updater test feeds must use loopback-only HTTP.');
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

function normalizeReleaseNotes(value) {
  const notes = Array.isArray(value)
    ? value.map((entry) => {
      if (typeof entry === 'string') return entry;
      if (!entry || typeof entry !== 'object') return '';
      const note = typeof entry.note === 'string' ? entry.note.trim() : '';
      const version = typeof entry.version === 'string' ? entry.version.trim() : '';
      return note && version ? `${version}\n${note}` : note;
    }).filter(Boolean).join('\n\n')
    : typeof value === 'string' ? value : '';
  return (notes.trim() || 'This update includes reliability and compatibility improvements.')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^[ \t]*[-*][ \t]+/gm, '• ')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
}

function configureAutoUpdates({
  app,
  autoUpdater,
  createVerifiedFeed = createTufVerifiedUpdateFeed,
  packageMetadata,
  env = process.env,
  log = console,
  platform = process.platform,
  prepareForInstall = async () => {},
  resourcesPath = process.resourcesPath,
  schedule = { clearInterval, clearTimeout, setInterval, setTimeout },
  showUpdateReady = async () => ({ response: 1 }),
} = {}) {
  if (
    !app?.isPackaged
    || !['darwin', 'win32', 'linux'].includes(platform)
    || env.FRAIA_DISABLE_UPDATES === '1'
  ) {
    return { enabled: false };
  }
  if (platform === 'linux' && !env.APPIMAGE && env.FRAIA_E2E_UPDATER !== '1') {
    return { enabled: false, reason: 'linux-package-manager' };
  }

  const testMode = env.FRAIA_E2E_UPDATER === '1';
  const testTufRepositoryUrl = testMode && env.FRAIA_E2E_TUF_REPOSITORY_URL
    ? validateTestFeedUrl(env.FRAIA_E2E_TUF_REPOSITORY_URL)
    : null;
  const configuredFeedUrl = testMode && env.FRAIA_UPDATE_FEED_URL
    ? validateTestFeedUrl(env.FRAIA_UPDATE_FEED_URL)
    : packageMetadata.fraiaUpdateFeedUrl;
  const channel = packageMetadata.fraiaReleaseChannel;
  if (channel !== 'stable' || typeof configuredFeedUrl !== 'string' || !configuredFeedUrl) {
    throw new Error('Packaged Fraia updater metadata is invalid.');
  }

  if (platform !== 'darwin' && (!testMode || testTufRepositoryUrl)) {
    const targetName = packageMetadata.fraiaUpdateTargetName;
    const repositoryUrl = testTufRepositoryUrl || packageMetadata.fraiaTufRepositoryUrl;
    if (typeof targetName !== 'string' || !targetName || typeof repositoryUrl !== 'string' || !repositoryUrl) {
      throw new Error('Packaged Fraia TUF updater metadata is invalid.');
    }
    const userData = app.getPath('userData');
    return createVerifiedFeed({
      embeddedRootPath: path.join(resourcesPath, 'update-trust', 'root.json'),
      repositoryUrl,
      targetName,
      trustDir: path.join(userData, 'update-trust'),
      allowLoopbackHttp: Boolean(testTufRepositoryUrl),
    }).then((verifiedFeed) => activateUpdater({
      app,
      autoUpdater,
      channel,
      env,
      feedUrl: verifiedFeed.feedUrl,
      log,
      platform,
      prepareForInstall,
      schedule,
      showUpdateReady,
      testMode,
      verifiedFeed,
    }));
  }

  return activateUpdater({
    app,
    autoUpdater,
    channel,
    env,
    feedUrl: configuredFeedUrl,
    log,
    platform,
    prepareForInstall,
    schedule,
    showUpdateReady,
    testMode,
    verifiedFeed: null,
  });
}

function activateUpdater({
  app,
  autoUpdater,
  channel,
  env,
  feedUrl,
  log,
  platform,
  prepareForInstall,
  schedule,
  showUpdateReady,
  testMode,
  verifiedFeed,
}) {
  autoUpdater.autoDownload = true;
  autoUpdater.autoInstallOnAppQuit = true;
  autoUpdater.allowPrerelease = false;
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
  let stopSchedule = () => {};
  let verifiedFeedClosePromise = null;
  const closeVerifiedFeed = () => {
    if (!verifiedFeed) return Promise.resolve();
    if (!verifiedFeedClosePromise) {
      verifiedFeedClosePromise = Promise.resolve().then(() => verifiedFeed.close());
    }
    return verifiedFeedClosePromise;
  };
  let installPromise = null;
  const installDownloadedUpdate = () => {
    if (installPromise) return installPromise;
    stopSchedule();
    installPromise = Promise.resolve()
      .then(() => prepareForInstall())
      .then(() => closeVerifiedFeed())
      .then(() => autoUpdater.quitAndInstall(platform !== 'darwin', true))
      .catch((error) => {
        event('error', { message: String(error?.message || error) });
        log.error('[updater] could not prepare the downloaded update for installation', error);
        throw error;
      });
    return installPromise;
  };

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
  let promptedVersion = null;
  autoUpdater.on('update-downloaded', async (info) => {
    const releaseNotes = normalizeReleaseNotes(info.releaseNotes);
    event('update-downloaded', { version: info.version, releaseNotes });
    if (testMode && env.FRAIA_E2E_INSTALL_UPDATE === '1') {
      schedule.setTimeout(() => {
        void installDownloadedUpdate().catch(() => {});
      }, 100);
      return;
    }
    if (promptedVersion === info.version) return;
    promptedVersion = info.version;
    try {
      const result = await showUpdateReady({ releaseNotes, version: info.version });
      if (result?.response === 0) await installDownloadedUpdate();
    } catch (error) {
      log.error('[updater] could not present downloaded update', error);
    }
  });

  const userData = app.getPath('userData');
  const frequencyPath = path.join(userData, 'update-frequency.json');
  const lastCheckPath = path.join(userData, 'last-update-check.json');
  const defaultFrequency = 'daily';
  let frequency = testMode ? 'sixHours' : readFrequency(frequencyPath, defaultFrequency);
  let initialTimer = null;
  let intervalTimer = null;
  let checkPromise = null;

  const check = () => {
    if (checkPromise) return checkPromise;
    checkPromise = Promise.resolve(verifiedFeed?.refresh?.())
      .then(() => autoUpdater.checkForUpdates())
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

  stopSchedule = () => {
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
    stop: () => {
      stopSchedule();
      return closeVerifiedFeed().catch((error) => {
        log.error('[updater] could not close verified local feed', error);
      });
    },
    trustedMetadata: Boolean(verifiedFeed),
  };
}

module.exports = {
  UPDATE_FREQUENCIES,
  UPDATE_FREQUENCY_MS,
  configureAutoUpdates,
  normalizeReleaseNotes,
  readFrequency,
  readLastCheck,
  safeWriteEvent,
  validateTestFeedUrl,
};
