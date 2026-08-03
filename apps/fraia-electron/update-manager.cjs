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
const DEFAULT_UPDATE_FREQUENCY = Object.freeze({
  stable: 'weekly',
  beta: 'daily',
});
const UPDATE_FREQUENCIES = Object.freeze(['never', 'startup', ...Object.keys(UPDATE_FREQUENCY_MS)]);
const UPDATE_RETRY_MS = Object.freeze([5 * 60 * 1000, 15 * 60 * 1000, 60 * 60 * 1000, 6 * 60 * 60 * 1000]);

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

function readCheckHistory(filePath) {
  try {
    const parsed = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    const legacyTimestamp = Number.isFinite(parsed.timestamp) && parsed.timestamp >= 0
      ? parsed.timestamp
      : 0;
    return {
      lastAttemptAt: Number.isFinite(parsed.lastAttemptAt) && parsed.lastAttemptAt >= 0
        ? parsed.lastAttemptAt
        : legacyTimestamp,
      lastSuccessfulCheckAt: Number.isFinite(parsed.lastSuccessfulCheckAt) && parsed.lastSuccessfulCheckAt >= 0
        ? parsed.lastSuccessfulCheckAt
        : legacyTimestamp,
    };
  } catch {
    return { lastAttemptAt: 0, lastSuccessfulCheckAt: 0 };
  }
}

function userFacingUpdateError(error) {
  const message = String(error?.message || error || '').toLowerCase();
  if (/signature|metadata|tuf|trust|verification|checksum/.test(message)) {
    return 'Fraia could not securely verify the update information. No update was installed.';
  }
  if (/network|internet|fetch|http|socket|dns|timed? ?out|enotfound|econn/.test(message)) {
    return 'Fraia could not reach the update service. Check your internet connection and try again.';
  }
  return 'Fraia could not check for updates. Try again in a few minutes.';
}

function normalizeDownloadProgress(info = {}) {
  const total = Number.isFinite(info.total) && info.total > 0 ? Math.round(info.total) : 0;
  const transferred = Number.isFinite(info.transferred) && info.transferred >= 0
    ? Math.min(Math.round(info.transferred), total || Number.MAX_SAFE_INTEGER)
    : 0;
  const bytesPerSecond = Number.isFinite(info.bytesPerSecond) && info.bytesPerSecond > 0
    ? Math.round(info.bytesPerSecond)
    : 0;
  const reportedPercent = Number.isFinite(info.percent) ? info.percent : null;
  const percent = Math.max(0, Math.min(100, Math.round(
    reportedPercent ?? (total ? (transferred / total) * 100 : 0),
  )));
  const remainingBytes = total ? Math.max(0, total - transferred) : 0;
  const etaSeconds = bytesPerSecond && remainingBytes
    ? Math.max(1, Math.round(remainingBytes / bytesPerSecond))
    : null;
  return { bytesPerSecond, etaSeconds, percent, total, transferred };
}

function disabledUpdaterStatus({ app, reason = 'unavailable' } = {}) {
  return {
    channel: null,
    currentVersion: app?.getVersion?.() ?? null,
    enabled: false,
    frequency: 'never',
    phase: reason === 'linux-package-manager' ? 'managed' : 'disabled',
    reason,
  };
}

function disabledUpdaterController(options) {
  const status = disabledUpdaterStatus(options);
  return {
    enabled: false,
    reason: status.reason,
    getStatus: () => ({ ...status }),
  };
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
  showUpdateReady = null,
  onStatusChange = () => {},
} = {}) {
  if (
    !app?.isPackaged
    || !['darwin', 'win32', 'linux'].includes(platform)
    || env.FRAIA_DISABLE_UPDATES === '1'
  ) {
    return disabledUpdaterController({ app });
  }
  if (platform === 'linux' && !env.APPIMAGE && env.FRAIA_E2E_UPDATER !== '1') {
    return disabledUpdaterController({ app, reason: 'linux-package-manager' });
  }

  const testMode = env.FRAIA_E2E_UPDATER === '1';
  const testTufRepositoryUrl = testMode && env.FRAIA_E2E_TUF_REPOSITORY_URL
    ? validateTestFeedUrl(env.FRAIA_E2E_TUF_REPOSITORY_URL)
    : null;
  const configuredFeedUrl = testMode && env.FRAIA_UPDATE_FEED_URL
    ? validateTestFeedUrl(env.FRAIA_UPDATE_FEED_URL)
    : packageMetadata.fraiaUpdateFeedUrl;
  const channel = packageMetadata.fraiaReleaseChannel;
  if (!['stable', 'beta'].includes(channel) || typeof configuredFeedUrl !== 'string' || !configuredFeedUrl) {
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
      onStatusChange,
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
    onStatusChange,
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
  onStatusChange,
  testMode,
  verifiedFeed,
}) {
  autoUpdater.autoDownload = true;
  autoUpdater.autoInstallOnAppQuit = true;
  autoUpdater.allowPrerelease = channel === 'beta';
  if (platform === 'win32') autoUpdater.disableWebInstaller = true;
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
  const userData = app.getPath('userData');
  const frequencyPath = path.join(userData, 'update-frequency.json');
  const lastCheckPath = path.join(userData, 'last-update-check.json');
  const defaultFrequency = DEFAULT_UPDATE_FREQUENCY[channel];
  let frequency = testMode ? 'sixHours' : readFrequency(frequencyPath, defaultFrequency);
  let checkHistory = readCheckHistory(lastCheckPath);
  let status = {
    channel,
    currentVersion: app.getVersion(),
    enabled: true,
    frequency,
    lastAttemptAt: checkHistory.lastAttemptAt || null,
    lastSuccessfulCheckAt: checkHistory.lastSuccessfulCheckAt || null,
    phase: 'idle',
    trustedMetadata: Boolean(verifiedFeed),
  };
  const publishStatus = (patch) => {
    status = { ...status, ...patch, frequency };
    onStatusChange({ ...status });
    return { ...status };
  };
  const recordCheckHistory = (patch) => {
    checkHistory = { ...checkHistory, ...patch };
    if (!testMode) writeJson(lastCheckPath, checkHistory);
    publishStatus({
      lastAttemptAt: checkHistory.lastAttemptAt || null,
      lastSuccessfulCheckAt: checkHistory.lastSuccessfulCheckAt || null,
    });
  };
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
    if (status.phase !== 'ready') {
      return Promise.reject(new Error('No downloaded Fraia update is ready to install.'));
    }
    stopSchedule();
    publishStatus({ phase: 'installing' });
    installPromise = Promise.resolve()
      .then(() => prepareForInstall())
      .then(() => closeVerifiedFeed())
      .then(() => {
        autoUpdater.quitAndInstall(platform !== 'darwin', true);
        if (platform === 'win32') {
          schedule.setTimeout(() => {
            event('update-install-force-exit');
            app.exit(0);
          }, 3_000);
        }
      })
      .catch((error) => {
        event('error', { message: String(error?.message || error) });
        log.error('[updater] could not prepare the downloaded update for installation', error);
        publishStatus({
          errorMessage: 'Fraia could not prepare the update for installation. Your current version was not changed.',
          phase: 'error',
        });
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
    publishStatus({ errorMessage: userFacingUpdateError(error), phase: 'error' });
  });
  autoUpdater.on('checking-for-update', () => {
    publishStatus({ errorMessage: null, phase: 'checking' });
  });
  const markSuccessfulCheck = () => {
    const timestamp = Date.now();
    recordCheckHistory({ lastAttemptAt: timestamp, lastSuccessfulCheckAt: timestamp });
  };
  autoUpdater.on('update-available', (info) => {
    event('update-available', { version: info.version });
    markSuccessfulCheck();
    publishStatus({ errorMessage: null, phase: 'available', version: info.version });
  });
  autoUpdater.on('update-not-available', (info) => {
    event('update-not-available', { version: info.version });
    markSuccessfulCheck();
    publishStatus({ errorMessage: null, phase: 'up-to-date', version: null });
  });
  autoUpdater.on('download-progress', (info) => {
    publishStatus({
      errorMessage: null,
      phase: 'downloading',
      progress: normalizeDownloadProgress(info),
    });
  });
  let promptedVersion = null;
  autoUpdater.on('update-downloaded', async (info) => {
    const releaseNotes = normalizeReleaseNotes(info.releaseNotes);
    event('update-downloaded', { version: info.version, releaseNotes });
    publishStatus({
      errorMessage: null,
      phase: 'ready',
      progress: normalizeDownloadProgress({
        bytesPerSecond: 0,
        percent: 100,
        total: status.progress?.total ?? 0,
        transferred: status.progress?.total ?? 0,
      }),
      releaseNotes,
      version: info.version,
    });
    if (testMode && env.FRAIA_E2E_INSTALL_UPDATE === '1') {
      schedule.setTimeout(() => {
        void installDownloadedUpdate().catch(() => {});
      }, 100);
      return;
    }
    if (!showUpdateReady || promptedVersion === info.version) return;
    promptedVersion = info.version;
    try {
      const result = await showUpdateReady({ releaseNotes, version: info.version });
      if (result?.response === 0) await installDownloadedUpdate();
    } catch (error) {
      log.error('[updater] could not present downloaded update', error);
    }
  });

  let initialTimer = null;
  let intervalTimer = null;
  let retryTimer = null;
  let consecutiveFailures = 0;
  let checkPromise = null;

  const scheduleRetry = () => {
    if (frequency === 'never' || retryTimer !== null) return;
    const delay = UPDATE_RETRY_MS[Math.min(consecutiveFailures - 1, UPDATE_RETRY_MS.length - 1)];
    retryTimer = schedule.setTimeout(() => {
      retryTimer = null;
      void check({ automatic: true }).catch(() => {});
    }, delay);
  };
  const check = ({ automatic = false } = {}) => {
    if (checkPromise) return checkPromise;
    const attemptAt = Date.now();
    recordCheckHistory({ lastAttemptAt: attemptAt });
    publishStatus({ errorMessage: null, phase: 'checking' });
    checkPromise = Promise.resolve(verifiedFeed?.refresh?.())
      .then(() => autoUpdater.checkForUpdates())
      .then((result) => {
        consecutiveFailures = 0;
        if (retryTimer !== null) schedule.clearTimeout(retryTimer);
        retryTimer = null;
        return result;
      })
      .catch((error) => {
        consecutiveFailures += 1;
        event('error', { message: String(error?.message || error) });
        log.error('[updater] update check failed', error);
        publishStatus({ errorMessage: userFacingUpdateError(error), phase: 'error' });
        if (automatic) scheduleRetry();
        throw error;
      })
      .finally(() => {
        checkPromise = null;
      });
    return checkPromise;
  };

  stopSchedule = () => {
    if (initialTimer !== null) schedule.clearTimeout(initialTimer);
    if (intervalTimer !== null) schedule.clearInterval(intervalTimer);
    if (retryTimer !== null) schedule.clearTimeout(retryTimer);
    initialTimer = null;
    intervalTimer = null;
    retryTimer = null;
  };
  const startSchedule = () => {
    stopSchedule();
    if (frequency === 'never') return;
    const scheduledFrequency = frequency;
    const interval = UPDATE_FREQUENCY_MS[frequency];
    const elapsed = interval ? Date.now() - checkHistory.lastSuccessfulCheckAt : 0;
    const initialDelay = testMode
      ? 0
      : interval && elapsed < interval
        ? Math.max(30_000, interval - elapsed)
        : 30_000;
    initialTimer = schedule.setTimeout(async () => {
      await check({ automatic: true }).catch(() => {});
      if (scheduledFrequency === frequency && interval) {
        intervalTimer = schedule.setInterval(() => { void check({ automatic: true }).catch(() => {}); }, interval);
      }
    }, initialDelay);
  };
  const setFrequency = (next) => {
    if (!UPDATE_FREQUENCIES.includes(next)) throw new Error(`Invalid updater frequency: ${next}`);
    frequency = next;
    if (!testMode) writeJson(frequencyPath, { frequency });
    publishStatus({ frequency });
    startSchedule();
    return frequency;
  };

  startSchedule();
  return {
    channel,
    checkNow: check,
    enabled: true,
    feedUrl,
    getStatus: () => ({ ...status }),
    get frequency() { return frequency; },
    installUpdate: installDownloadedUpdate,
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
  DEFAULT_UPDATE_FREQUENCY,
  UPDATE_FREQUENCIES,
  UPDATE_FREQUENCY_MS,
  UPDATE_RETRY_MS,
  configureAutoUpdates,
  disabledUpdaterStatus,
  normalizeDownloadProgress,
  normalizeReleaseNotes,
  readCheckHistory,
  readFrequency,
  safeWriteEvent,
  validateTestFeedUrl,
  userFacingUpdateError,
};
