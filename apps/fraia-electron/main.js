const { app, BrowserWindow, Menu, dialog, ipcMain, nativeTheme, safeStorage, screen, session, shell } = require('electron');
const { spawn, spawnSync } = require('node:child_process');
const { randomBytes } = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { round, selectedBudget } = require('./scripts/perf-budgets.cjs');
const { FakeFraiaAiRuntime, FraiaAiRuntime, publicFraiaCatalogue } = require('./ai-runtime.cjs');
const { resolveApplicationMetadata, resolveUserDataDirectory } = require('./application-metadata.cjs');
const { nativePlatformArch, resolveCalculixRuntime, resolveSidecarLaunch } = require('./package-boundary.cjs');
const { configureAutoUpdates } = require('./update-manager.cjs');
const packageMetadata = require('./package.json');
const developmentChannel = process.env.FRAIA_RELEASE_CHANNEL
  || (packageMetadata.version.includes('-beta.') ? 'beta' : 'stable');
const applicationMetadata = resolveApplicationMetadata(app.isPackaged
  ? packageMetadata
  : {
      ...packageMetadata,
      fraiaReleaseChannel: developmentChannel,
      productName: developmentChannel === 'beta' ? 'Fraia Beta' : 'Fraia',
    });

app.setName(applicationMetadata.productName);

const configuredUserDataDir = process.env.FRAIA_USER_DATA_DIR?.trim();
const resolvedUserDataDir = resolveUserDataDirectory({
  appDataPath: app.getPath('appData'),
  configuredPath: configuredUserDataDir,
  metadata: applicationMetadata,
});
fs.mkdirSync(resolvedUserDataDir, { recursive: true });
app.setPath('userData', resolvedUserDataDir);

const repoRoot = path.resolve(__dirname, '..', '..');
const sidecarPort = 43000 + Math.floor(Math.random() * 1000);
const sidecarToken = randomBytes(32).toString('base64url');
let sidecarProcess = null;
let sidecarReadyPromise = null;
let mainWindow = null;
let aiRuntime = null;
let aiRuntimeReadyPromise = null;
let updateController = null;
let quitCleanupComplete = false;
let quitCleanupPromise = null;
const FRAIA_AI_PROVIDER_ID = 'openai-codex';

function isPipeClosedError(error) {
  return error?.code === 'EPIPE' || error?.code === 'ERR_STREAM_DESTROYED';
}

function ignoreClosedPipe(error) {
  if (!isPipeClosedError(error)) {
    throw error;
  }
}

process.stdout.on('error', ignoreClosedPipe);
process.stderr.on('error', ignoreClosedPipe);

function safeLog(message, stream = process.stdout) {
  try {
    stream.write(`${message}\n`);
  } catch (error) {
    if (!isPipeClosedError(error)) {
      throw error;
    }
  }
}

function safeError(message) {
  safeLog(message, process.stderr);
}

const legacyTmpProjectDir = '/tmp/fraia-electron-raw-cad-geometry-test';

function mainWindowStatePath() {
  return path.join(app.getPath('userData'), 'main-window-state.json');
}

function defaultDevProjectDir() {
  return process.env.FRAIA_DEFAULT_PROJECT_DIR || path.join(app.getPath('userData'), 'projects', 'raw-cad-geometry-test');
}

function projectFilePath(projectDir) {
  return path.join(projectDir, 'fraia.project.json');
}

function backupProjectBeforeReset(projectDir) {
  if (!fs.existsSync(projectFilePath(projectDir))) {
    return;
  }
  const backupRoot = path.join(app.getPath('userData'), 'project-backups');
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const backupDir = path.join(backupRoot, `${path.basename(projectDir)}-${timestamp}`);
  fs.mkdirSync(backupRoot, { recursive: true });
  fs.cpSync(projectDir, backupDir, { recursive: true, force: false });
  safeLog(`[dev-reset] Backed up project data to ${backupDir}`);
}

function ensureDefaultProjectDir() {
  const targetDir = defaultDevProjectDir();
  if (process.env.FRAIA_DEFAULT_PROJECT_DIR) {
    return targetDir;
  }
  if (fs.existsSync(projectFilePath(targetDir))) {
    return targetDir;
  }
  if (fs.existsSync(projectFilePath(legacyTmpProjectDir))) {
    fs.mkdirSync(path.dirname(targetDir), { recursive: true });
    fs.cpSync(legacyTmpProjectDir, targetDir, { recursive: true, force: false });
    safeLog(`[project-migration] Copied legacy dev project from ${legacyTmpProjectDir} to ${targetDir}`);
  }
  return targetDir;
}

function shouldFreshStartBaseGuide() {
  return process.env.FRAIA_DEV_FRESH_GUIDE === '1';
}

function electronCapturePath() {
  return process.env.FRAIA_ELECTRON_CAPTURE_PATH || '';
}

function electronMetricsPath() {
  return process.env.FRAIA_ELECTRON_METRICS_PATH || '';
}

function isElectronCaptureMode() {
  return Boolean(electronCapturePath() || electronMetricsPath());
}

function electronCaptureDelayMs() {
  const parsed = Number.parseInt(process.env.FRAIA_ELECTRON_CAPTURE_DELAY_MS || '5000', 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : 5000;
}

function electronMetricsSamples() {
  const parsed = Number.parseInt(process.env.FRAIA_ELECTRON_METRICS_SAMPLES || '1', 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 1;
}

function electronMetricsSampleIntervalMs() {
  const parsed = Number.parseInt(process.env.FRAIA_ELECTRON_METRICS_SAMPLE_INTERVAL_MS || '1000', 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : 1000;
}

function electronCaptureBounds() {
  const width = Number.parseInt(process.env.FRAIA_ELECTRON_CAPTURE_WIDTH || '1440', 10);
  const height = Number.parseInt(process.env.FRAIA_ELECTRON_CAPTURE_HEIGHT || '900', 10);
  return {
    width: Number.isFinite(width) && width >= 980 ? width : 1440,
    height: Number.isFinite(height) && height >= 640 ? height : 900,
  };
}

function electronCaptureClicks() {
  const raw = process.env.FRAIA_ELECTRON_CAPTURE_CLICKS || '';
  return raw
    .split(';')
    .map((item) => item.trim())
    .filter(Boolean)
    .map((item) => {
      const [x, y] = item.split(',').map((part) => Number.parseInt(part.trim(), 10));
      return Number.isFinite(x) && Number.isFinite(y) ? { x, y } : null;
    })
    .filter(Boolean);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function appMetricSummary(metric) {
  return {
    type: metric.type,
    pid: metric.pid,
    cpuPercent: metric.cpu?.percentCPUUsage ?? null,
    workingSetMb: metric.memory?.workingSetSize != null ? metric.memory.workingSetSize / 1024 : null,
    peakWorkingSetMb: metric.memory?.peakWorkingSetSize != null ? metric.memory.peakWorkingSetSize / 1024 : null,
  };
}

async function appMetricsSamples(sampleCount, sampleIntervalMs) {
  const count = Math.max(1, Math.floor(sampleCount));
  const samples = [];
  for (let index = 0; index < count; index += 1) {
    samples.push({
      capturedAt: new Date().toISOString(),
      appMetrics: app.getAppMetrics().map(appMetricSummary),
    });
    if (index < count - 1 && sampleIntervalMs > 0) {
      await sleep(sampleIntervalMs);
    }
  }
  return samples;
}

function idleSummaryForSamples(samples) {
  const totalCpuSamples = samples
    .map((sample) => sample.appMetrics
      .map((metric) => metric.cpuPercent)
      .filter(Number.isFinite)
      .reduce((sum, value) => sum + value, 0))
    .filter(Number.isFinite);
  const totalWorkingSetSamples = samples
    .map((sample) => sample.appMetrics
      .map((metric) => metric.workingSetMb)
      .filter(Number.isFinite)
      .reduce((sum, value) => sum + value, 0))
    .filter(Number.isFinite);
  return {
    sampleCount: samples.length,
    avgTotalCpuPercent: totalCpuSamples.length
      ? round(totalCpuSamples.reduce((sum, value) => sum + value, 0) / totalCpuSamples.length)
      : null,
    maxTotalCpuPercent: totalCpuSamples.length ? round(Math.max(...totalCpuSamples)) : null,
    avgTotalWorkingSetMb: totalWorkingSetSamples.length
      ? round(totalWorkingSetSamples.reduce((sum, value) => sum + value, 0) / totalWorkingSetSamples.length)
      : null,
    maxTotalWorkingSetMb: totalWorkingSetSamples.length ? round(Math.max(...totalWorkingSetSamples)) : null,
  };
}

async function electronMetricsSnapshot(window, sampleCount = 1, sampleIntervalMs = 1000) {
  const mainProcessMemory = typeof process.getProcessMemoryInfo === 'function'
    ? await process.getProcessMemoryInfo()
    : null;
  const metricSamples = await appMetricsSamples(sampleCount, sampleIntervalMs);
  const latestSample = metricSamples[metricSamples.length - 1] ?? { appMetrics: app.getAppMetrics().map(appMetricSummary) };
  const rendererSnapshot = await window.webContents.executeJavaScript(`(() => ({
    url: location.href,
    domNodeCount: document.getElementsByTagName('*').length,
    canvasCount: document.getElementsByTagName('canvas').length,
    canvasRoles: Array.from(document.getElementsByTagName('canvas')).reduce((counts, canvas) => {
      const role = canvas.dataset.fraiaCanvasRole || 'unclassified';
      counts[role] = (counts[role] || 0) + 1;
      return counts;
    }, {}),
    jsHeap: performance.memory ? {
      usedJsHeapSizeMb: performance.memory.usedJSHeapSize / 1024 / 1024,
      totalJsHeapSizeMb: performance.memory.totalJSHeapSize / 1024 / 1024,
      jsHeapSizeLimitMb: performance.memory.jsHeapSizeLimit / 1024 / 1024
    } : null,
    viewport: window.__FRAIA_VIEWPORT_STATS__ || null
  }))()`, true);
  return {
    capturedAt: new Date().toISOString(),
    electronVersion: process.versions.electron,
    chromeVersion: process.versions.chrome,
    nodeVersion: process.versions.node,
    performanceBudget: selectedBudget(),
    rendererPid: window.webContents.getOSProcessId(),
    mainProcessMemoryMb: mainProcessMemory
      ? {
          workingSetMb: mainProcessMemory.workingSetSize / 1024,
          peakWorkingSetMb: mainProcessMemory.peakWorkingSetSize / 1024,
          privateBytesMb: mainProcessMemory.privateBytes / 1024,
          sharedBytesMb: mainProcessMemory.sharedBytes / 1024,
        }
      : null,
    appMetrics: latestSample.appMetrics,
    appMetricSamples: metricSamples,
    idleSummary: idleSummaryForSamples(metricSamples),
    renderer: rendererSnapshot,
  };
}

function safeRemovePath(targetPath) {
  if (!targetPath || typeof targetPath !== 'string') {
    return;
  }
  const resolved = path.resolve(targetPath);
  const allowedRoots = [
    path.resolve(os.tmpdir()),
    path.resolve('/tmp'),
    path.resolve('/private/tmp'),
    path.resolve(app.getPath('userData')),
  ];
  const isAllowed = allowedRoots.some((root) => resolved === root || resolved.startsWith(`${root}${path.sep}`));
  if (!isAllowed) {
    throw new Error(`Refusing to remove non-dev path: ${resolved}`);
  }
  fs.rmSync(resolved, { recursive: true, force: true });
}

async function wipeDevLaunchState() {
  if (!shouldFreshStartBaseGuide()) {
    return;
  }
  backupProjectBeforeReset(defaultDevProjectDir());
  safeLog(`[dev-reset] Removing project data at ${defaultDevProjectDir()}`);
  safeRemovePath(defaultDevProjectDir());
  safeLog(`[dev-reset] Clearing Electron renderer storage at ${app.getPath('userData')}`);
  safeRemovePath(mainWindowStatePath());
  await session.defaultSession.clearStorageData({
    storages: ['localstorage', 'indexdb', 'cachestorage', 'serviceworkers', 'websql'],
  });
}

function isUsableBounds(bounds) {
  if (!bounds || typeof bounds !== 'object') {
    return false;
  }
  const { x, y, width, height } = bounds;
  if (![x, y, width, height].every((value) => Number.isFinite(value))) {
    return false;
  }
  if (width < 980 || height < 640) {
    return false;
  }

  const centerX = x + width / 2;
  const centerY = y + height / 2;
  return screen.getAllDisplays().some(({ workArea }) =>
    centerX >= workArea.x &&
    centerX <= workArea.x + workArea.width &&
    centerY >= workArea.y &&
    centerY <= workArea.y + workArea.height
  );
}

function readMainWindowState() {
  try {
    const state = JSON.parse(fs.readFileSync(mainWindowStatePath(), 'utf8'));
    const bounds = state?.bounds;
    if (!isUsableBounds(bounds)) {
      return { bounds: null, maximized: false };
    }
    return { bounds, maximized: Boolean(state.maximized) };
  } catch (_error) {
    return { bounds: null, maximized: false };
  }
}

function writeMainWindowState(window) {
  if (!window || window.isDestroyed()) {
    return;
  }
  const bounds = window.isMaximized() ? window.getNormalBounds() : window.getBounds();
  if (!isUsableBounds(bounds)) {
    return;
  }
  const state = {
    bounds,
    maximized: window.isMaximized(),
  };
  try {
    fs.mkdirSync(path.dirname(mainWindowStatePath()), { recursive: true });
    fs.writeFileSync(mainWindowStatePath(), `${JSON.stringify(state, null, 2)}\n`);
  } catch (error) {
    safeError(`[window-state] Could not persist main window bounds: ${error?.message ?? error}`);
  }
}

function watchMainWindowState(window) {
  window.on('close', () => writeMainWindowState(window));
}

function windowBackgroundColor() {
  return nativeTheme.shouldUseDarkColors ? '#141517' : '#f1f3f5';
}

function syncWindowTheme(window) {
  if (!window || window.isDestroyed()) {
    return;
  }
  window.setBackgroundColor(windowBackgroundColor());
}

function syncAllWindowThemes() {
  syncWindowTheme(mainWindow);
}

function sidecarBaseUrl() {
  return `http://127.0.0.1:${sidecarPort}`;
}

function broadcastAiRuntimeStatus(status) {
  for (const window of BrowserWindow.getAllWindows()) {
    if (!window.isDestroyed()) window.webContents.send('fraia:aiRuntimeStatus', status);
  }
}

async function ensureAiRuntime() {
  if (!aiRuntimeReadyPromise) {
    aiRuntimeReadyPromise = (async () => {
      const Runtime = process.env.FRAIA_FAKE_AI_RUNTIME === '1' ? FakeFraiaAiRuntime : FraiaAiRuntime;
      aiRuntime = new Runtime({
        safeStorage,
        shell,
        userDataDir: app.getPath('userData'),
        emitStatus: broadcastAiRuntimeStatus,
      });
      await aiRuntime.initialize();
      await aiRuntime.startLoopback();
      return aiRuntime;
    })().catch((error) => {
      aiRuntime = null;
      aiRuntimeReadyPromise = null;
      throw error;
    });
  }
  return aiRuntimeReadyPromise;
}

function platformArch() {
  return nativePlatformArch();
}

function ccxInDirectory(dir) {
  if (!dir || !fs.existsSync(dir)) {
    return null;
  }
  const direct = path.join(dir, process.platform === 'win32' ? 'ccx.exe' : 'ccx');
  if (fs.existsSync(direct) && fs.statSync(direct).isFile()) {
    return direct;
  }
  const matches = fs.readdirSync(dir)
    .filter((name) => name.startsWith('ccx_') || name.startsWith('ccx-'))
    .map((name) => path.join(dir, name))
    .filter((candidate) => fs.existsSync(candidate) && fs.statSync(candidate).isFile())
    .sort();
  return matches.length ? matches[matches.length - 1] : null;
}

function pushRuntimeCandidates(candidates, base) {
  if (!base) {
    return;
  }
  candidates.push(base);
  candidates.push(path.join(base, 'bin'));
  candidates.push(path.join(base, platformArch()));
  candidates.push(path.join(base, platformArch(), 'bin'));
}

function findManagedCalculixRuntime() {
  if (process.env.FRAIA_DISABLE_CALCULIX_RUNTIME === '1') {
    return null;
  }
  if (process.env.FRAIA_CCX_PATH && fs.existsSync(process.env.FRAIA_CCX_PATH)) {
    return process.env.FRAIA_CCX_PATH;
  }
  const candidates = [];
  pushRuntimeCandidates(candidates, process.env.FRAIA_CALCULIX_DIR);
  pushRuntimeCandidates(candidates, path.join(app.getPath('userData'), 'runtimes', 'calculix'));
  pushRuntimeCandidates(candidates, path.join(process.resourcesPath, 'runtimes', 'calculix'));
  pushRuntimeCandidates(candidates, path.join(__dirname, 'resources', 'runtimes', 'calculix'));
  pushRuntimeCandidates(candidates, path.join(repoRoot, 'vendor', 'calculix'));
  candidates.push('/opt/homebrew/bin', '/usr/local/bin', '/opt/local/bin');
  for (const segment of (process.env.PATH || '').split(path.delimiter)) {
    if (segment) {
      candidates.push(segment);
    }
  }
  const seen = new Set();
  for (const candidate of candidates) {
    const resolved = path.resolve(candidate);
    if (seen.has(resolved)) {
      continue;
    }
    seen.add(resolved);
    const ccx = ccxInDirectory(resolved);
    if (ccx) {
      return ccx;
    }
  }
  return null;
}

function commandPath(name) {
  const probe = spawnSync('/usr/bin/env', ['which', name], { encoding: 'utf8' });
  if (probe.status !== 0) {
    return null;
  }
  return probe.stdout.trim() || null;
}

function runBrewCalculixBootstrap() {
  if (app.isPackaged
    || process.env.FRAIA_DISABLE_CALCULIX_RUNTIME === '1'
    || process.env.FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP === '1') {
    return null;
  }
  const brew = commandPath('brew') || ['/opt/homebrew/bin/brew', '/usr/local/bin/brew'].find((candidate) => fs.existsSync(candidate));
  if (!brew) {
    return null;
  }
  safeLog('[calculix-runtime] CalculiX was not found; attempting managed Homebrew bootstrap for costerwi/calculix/calculix-ccx.');
  const tap = spawnSync(brew, ['tap', 'costerwi/calculix'], {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 10 * 60 * 1000,
  });
  if (tap.status !== 0) {
    safeError(`[calculix-runtime] Homebrew tap failed: ${tap.stderr || tap.stdout}`);
    return null;
  }
  const install = spawnSync(brew, ['install', 'costerwi/calculix/calculix-ccx'], {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 30 * 60 * 1000,
  });
  if (install.status !== 0) {
    safeError(`[calculix-runtime] Homebrew install failed: ${install.stderr || install.stdout}`);
    return null;
  }
  return findManagedCalculixRuntime();
}

function ensureCalculixRuntimeEnv() {
  process.env.FRAIA_APP_RESOURCE_DIR = process.resourcesPath;
  const runtime = resolveCalculixRuntime({
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    explicitPath: process.env.FRAIA_CCX_PATH,
    developmentResolver: () => findManagedCalculixRuntime() || runBrewCalculixBootstrap(),
  });
  const ccx = runtime.executable;
  process.env.FRAIA_RUNTIME_DIR = app.isPackaged
    ? path.dirname(ccx)
    : path.join(app.getPath('userData'), 'runtimes');
  if (!app.isPackaged) {
    const pathSegments = [
      ccx ? path.dirname(ccx) : null,
      '/opt/homebrew/bin',
      '/usr/local/bin',
      process.env.PATH,
    ].filter(Boolean);
    process.env.PATH = pathSegments.join(path.delimiter);
  }
  if (ccx) {
    process.env.FRAIA_CCX_PATH = ccx;
    safeLog(`[calculix-runtime] Using ${runtime.source} CalculiX solver at ${ccx}`);
  } else {
    safeError('[calculix-runtime] CalculiX solver was not found. Analysis requests will report a missing managed runtime.');
  }
}

async function callApi(endpoint, options = {}) {
  await ensureSidecar();
  const headers = new Headers(options.headers ?? {});
  headers.set('content-type', 'application/json');
  headers.set('authorization', `Bearer ${sidecarToken}`);
  const response = await fetch(`${sidecarBaseUrl()}${endpoint}`, {
    ...options,
    headers,
  });
  const body = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(body.error || `${response.status} ${response.statusText}`);
  }
  return body;
}

async function waitForHealth(timeoutMs = 30000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      const response = await fetch(`${sidecarBaseUrl()}/health`, {
        headers: {
          'content-type': 'application/json',
          authorization: `Bearer ${sidecarToken}`,
        },
      });
      if (response.ok) {
        return;
      }
    } catch (_error) {
    }
    await new Promise((resolve) => setTimeout(resolve, 400));
  }
  throw new Error('Timed out waiting for Fraia app service to start.');
}

function startSidecar() {
  if (sidecarProcess && !sidecarProcess.killed) {
    return;
  }

  ensureCalculixRuntimeEnv();

  if (!aiRuntime?.serverUrl || !aiRuntime?.serverToken) {
    throw new Error('Fraia AI runtime must start before the application service.');
  }
  const sidecarEnv = {};
  for (const name of [
    'FRAIA_APP_RESOURCE_DIR',
    'FRAIA_CCX_PATH',
    'FRAIA_DISABLE_CALCULIX_RUNTIME',
    'FRAIA_RUNTIME_DIR',
    'HOME',
    'LANG',
    'LC_ALL',
    'PATH',
    'SystemRoot',
    'TEMP',
    'TMP',
    'TMPDIR',
    'USERPROFILE',
    'WINDIR',
  ]) {
    if (typeof process.env[name] === 'string' && process.env[name] !== '') {
      sidecarEnv[name] = process.env[name];
    }
  }
  Object.assign(sidecarEnv, {
    FRAIA_AI_URL: aiRuntime.serverUrl,
    FRAIA_AI_TOKEN: aiRuntime.serverToken,
    FRAIA_APPD_TOKEN: sidecarToken,
  });
  const launch = resolveSidecarLaunch({
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    repoRoot,
    explicitPath: process.env.FRAIA_APPD_PATH,
  });
  safeLog(`[sidecar] Launching Fraia app service from ${launch.source}.`);
  sidecarProcess = spawn(launch.command, [...launch.args, '--port', String(sidecarPort)], {
    cwd: launch.cwd,
    env: sidecarEnv,
    stdio: 'inherit',
  });

  const processRef = sidecarProcess;
  processRef.once('exit', () => {
    if (sidecarProcess === processRef) {
      sidecarProcess = null;
      sidecarReadyPromise = null;
    }
  });
}

function ensureSidecar() {
  if (!sidecarReadyPromise) {
    startSidecar();
    sidecarReadyPromise = waitForHealth().catch((error) => {
      sidecarReadyPromise = null;
      throw error;
    });
  }
  return sidecarReadyPromise;
}

function stopSidecar() {
  const processRef = sidecarProcess;
  sidecarProcess = null;
  sidecarReadyPromise = null;
  if (processRef && !processRef.killed) {
    processRef.kill();
  }
  return processRef;
}

function waitForChildExit(processRef, timeoutMs = 5_000) {
  if (!processRef || processRef.exitCode !== null || processRef.signalCode !== null) {
    return Promise.resolve();
  }
  return new Promise((resolve, reject) => {
    const onExit = () => {
      clearTimeout(timer);
      resolve();
    };
    const timer = setTimeout(() => {
      processRef.off('exit', onExit);
      reject(new Error(`Fraia sidecar process ${processRef.pid} did not stop within ${timeoutMs} ms.`));
    }, timeoutMs);
    processRef.once('exit', onExit);
  });
}

async function stopSidecarAndWait() {
  const processRef = stopSidecar();
  try {
    await waitForChildExit(processRef);
  } catch (error) {
    safeError(`[sidecar] ${error} Forcing termination.`);
    if (processRef && processRef.exitCode === null && processRef.signalCode === null) {
      processRef.kill('SIGKILL');
      await waitForChildExit(processRef);
    }
  }
}

async function stopRuntimeServices() {
  const runtime = aiRuntime;
  aiRuntime = null;
  aiRuntimeReadyPromise = null;
  await Promise.all([
    stopSidecarAndWait(),
    runtime?.stop?.(),
  ]);
}

async function prepareForUpdateInstall() {
  await stopRuntimeServices();
  quitCleanupComplete = true;
}

function reloadWindow(window, ignoreCache = false) {
  const target = window ?? mainWindow;
  if (!target || target.isDestroyed()) return;
  if (ignoreCache) {
    target.webContents.reloadIgnoringCache();
  } else {
    target.webContents.reload();
  }
}

function currentUpdateStatus() {
  return updateController?.getStatus?.() ?? {
    channel: applicationMetadata.channel,
    currentVersion: app.getVersion(),
    enabled: false,
    frequency: 'never',
    phase: app.isPackaged ? 'initializing' : 'disabled',
    reason: app.isPackaged ? 'initializing' : 'development',
  };
}

function broadcastUpdateStatus(status) {
  for (const window of BrowserWindow.getAllWindows()) {
    if (!window.isDestroyed()) window.webContents.send('fraia:updateStatus', status);
  }
}

function openUpdateDialog() {
  if (!mainWindow || mainWindow.isDestroyed()) return;
  mainWindow.show();
  mainWindow.focus();
  mainWindow.webContents.send('fraia:openUpdateDialog');
}

function installApplicationMenu() {
  const frequencyLabels = {
    never: 'Never',
    startup: 'On Startup',
    hourly: 'Hourly',
    sixHours: 'Every 6 Hours',
    twelveHours: 'Every 12 Hours',
    daily: 'Daily',
    weekly: 'Weekly',
  };
  const updateSubmenu = updateController?.enabled ? [
    {
      label: 'Check for Updates…',
      click: () => {
        openUpdateDialog();
        void updateController.checkNow().catch(() => {});
      },
    },
    { type: 'separator' },
    ...Object.entries(frequencyLabels).map(([frequency, label]) => ({
      label,
      type: 'radio',
      checked: updateController.frequency === frequency,
      click: () => {
        updateController.setFrequency(frequency);
        installApplicationMenu();
      },
    })),
  ] : [];
  const template = [
    {
      label: applicationMetadata.productName,
      submenu: [
        ...(updateSubmenu.length ? [{ label: 'Updates', submenu: updateSubmenu }, { type: 'separator' }] : []),
        { label: `Quit ${applicationMetadata.productName}`, role: 'quit' },
      ],
    },
    {
      label: 'Developer',
      submenu: [
        {
          label: 'Reload Window',
          accelerator: 'CmdOrCtrl+R',
          click: (_menuItem, browserWindow) => {
            reloadWindow(browserWindow);
          },
        },
        {
          label: 'Force Reload Window',
          accelerator: 'CmdOrCtrl+Shift+R',
          click: (_menuItem, browserWindow) => {
            reloadWindow(browserWindow, true);
          },
        },
      ],
    },
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}

function createWindow() {
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.show();
    mainWindow.focus();
    return mainWindow;
  }

  const windowState = isElectronCaptureMode()
    ? { bounds: electronCaptureBounds(), maximized: false }
    : readMainWindowState();
  mainWindow = new BrowserWindow({
    ...(windowState.bounds ?? {}),
    minWidth: 980,
    minHeight: 640,
    show: !isElectronCaptureMode(),
    title: applicationMetadata.productName,
    backgroundColor: windowBackgroundColor(),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });
  if (windowState.maximized) {
    mainWindow.maximize();
  }
  watchMainWindowState(mainWindow);

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
  mainWindow.webContents.on('console-message', (_event, level, message, line, sourceId) => {
    safeLog(`[renderer:${level}] ${message} (${sourceId}:${line})`);
  });
  mainWindow.webContents.on('did-fail-load', (_event, errorCode, errorDescription, validatedUrl) => {
    safeError(`[renderer:load-failed] ${errorCode} ${errorDescription} ${validatedUrl}`);
  });

  const devServerUrl = process.env.VITE_DEV_SERVER_URL || process.env.ELECTRON_RENDERER_URL;
  if (devServerUrl) {
    mainWindow.loadURL(devServerUrl);
  } else {
    mainWindow.loadFile(path.join(__dirname, 'dist', 'index.html'));
  }

  if (isElectronCaptureMode()) {
    scheduleElectronCapture(mainWindow);
  }
  return mainWindow;
}

function scheduleElectronCapture(window) {
  window.webContents.once('did-finish-load', () => {
    setTimeout(async () => {
      try {
        await window.webContents.executeJavaScript('document.fonts?.ready ?? Promise.resolve()', true);
        for (const click of electronCaptureClicks()) {
          window.webContents.sendInputEvent({ type: 'mouseMove', x: click.x, y: click.y });
          window.webContents.sendInputEvent({ type: 'mouseDown', x: click.x, y: click.y, button: 'left', clickCount: 1 });
          window.webContents.sendInputEvent({ type: 'mouseUp', x: click.x, y: click.y, button: 'left', clickCount: 1 });
          await window.webContents.executeJavaScript(`(() => {
            const x = ${JSON.stringify(click.x)};
            const y = ${JSON.stringify(click.y)};
            const target = document.elementFromPoint(x, y);
            if (!target) return false;
            const pointerOptions = { bubbles: true, cancelable: true, clientX: x, clientY: y, button: 0, buttons: 1, pointerId: 1, pointerType: 'mouse' };
            target.dispatchEvent(new PointerEvent('pointerdown', pointerOptions));
            target.dispatchEvent(new PointerEvent('pointerup', { ...pointerOptions, buttons: 0 }));
            target.dispatchEvent(new MouseEvent('mousedown', pointerOptions));
            target.dispatchEvent(new MouseEvent('mouseup', { ...pointerOptions, buttons: 0 }));
            return true;
          })()`, true);
          await sleep(120);
        }
        const metricsTarget = electronMetricsPath();
        if (metricsTarget) {
          fs.mkdirSync(path.dirname(metricsTarget), { recursive: true });
          fs.writeFileSync(metricsTarget, `${JSON.stringify(await electronMetricsSnapshot(window, electronMetricsSamples(), electronMetricsSampleIntervalMs()), null, 2)}\n`);
          safeLog(`[metrics] Wrote Electron metrics to ${metricsTarget}`);
        }
        const captureTarget = electronCapturePath();
        if (captureTarget) {
          fs.mkdirSync(path.dirname(captureTarget), { recursive: true });
          const image = await window.webContents.capturePage();
          fs.writeFileSync(captureTarget, image.toPNG());
          safeLog(`[capture] Wrote Electron UI snapshot to ${captureTarget}`);
        }
        stopSidecar();
        app.quit();
      } catch (error) {
        safeError(`[capture] Could not capture Electron UI: ${error?.message ?? error}`);
        stopSidecar();
        app.exit(1);
      }
    }, electronCaptureDelayMs());
  });
}

ipcMain.handle('fraia:health', () => callApi('/health'));
ipcMain.handle('fraia:applicationMetadata', () => applicationMetadata);
ipcMain.handle('fraia:defaultProjectDir', () => ensureDefaultProjectDir());
ipcMain.handle('fraia:pickDirectory', async () => {
  const result = await dialog.showOpenDialog({
    properties: ['openDirectory', 'createDirectory'],
  });
  return result.canceled ? null : result.filePaths[0];
});
ipcMain.handle('fraia:pickProjectFile', async () => {
  const result = await dialog.showOpenDialog({
    title: 'Open Fraia Model',
    properties: ['openFile'],
    filters: [{ name: 'Fraia project', extensions: ['json'] }],
  });
  if (result.canceled) return null;
  const projectFile = result.filePaths[0];
  if (path.basename(projectFile) !== 'fraia.project.json') {
    throw new Error('Select the fraia.project.json file inside a Fraia model folder.');
  }
  return path.dirname(projectFile);
});
ipcMain.handle('fraia:createProject', (_event, payload) =>
  callApi('/projects/create', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:openProject', (_event, payload) =>
  callApi('/projects/open', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:savePlanningDraft', (_event, payload) =>
  callApi('/projects/planning-draft', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:materializePlanning', (_event, payload) =>
  callApi('/projects/materialize-planning', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:analysePlanning', (_event, payload) =>
  callApi('/projects/analyse', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:analyseDesignOptions', (_event, payload) =>
  callApi('/projects/design-option-analysis', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:updateDesignOptionDecision', (_event, payload) =>
  callApi('/projects/design-options/decision', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:rawDesignOptionAnalysis', (_event, payload) => {
  const params = new URLSearchParams({ projectDir: payload.projectDir });
  if (payload.runId) params.set('runId', payload.runId);
  return callApi(`/projects/design-option-analysis/raw?${params.toString()}`);
});
ipcMain.handle('fraia:prepareSchemaHandoff', (_event, payload) =>
  callApi('/schemas/base-model-handoff', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:reviewReply', (_event, payload) =>
  callApi('/agent/review-reply', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:preSolveCoordinator', (_event, payload) =>
  callApi('/agent/pre-solve-coordinator', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:generateDesignOptions', (_event, payload) =>
  callApi('/agent/design-options/generate', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:agentProviderStatus', (_event, payload) =>
  callApi('/agent/provider-status', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:aiProviders', async () =>
  publicFraiaCatalogue(await (await ensureAiRuntime()).catalog())
);
ipcMain.handle('fraia:aiRefreshCatalog', async () =>
  publicFraiaCatalogue(await (await ensureAiRuntime()).refreshCatalog('manual'))
);
ipcMain.handle('fraia:aiStartOAuth', async (_event, payload) => {
  if (payload?.providerId !== FRAIA_AI_PROVIDER_ID) {
    throw new Error(`Fraia ${packageMetadata.version} supports only Sign in with ChatGPT.`);
  }
  return (await ensureAiRuntime()).startOAuth(FRAIA_AI_PROVIDER_ID);
});
ipcMain.handle('fraia:aiAnswerAuthPrompt', async (_event, payload) =>
  (await ensureAiRuntime()).answerAuthPrompt(payload?.flowId, payload?.value)
);
ipcMain.handle('fraia:aiCancelAuth', async (_event, payload) =>
  (await ensureAiRuntime()).cancelAuth(payload?.flowId)
);
ipcMain.handle('fraia:aiDisconnect', async (_event, payload) => {
  if (payload?.providerId !== FRAIA_AI_PROVIDER_ID) {
    throw new Error(`Fraia ${packageMetadata.version} supports only the ChatGPT connection.`);
  }
  return publicFraiaCatalogue(await (await ensureAiRuntime()).disconnect(FRAIA_AI_PROVIDER_ID));
});
ipcMain.handle('fraia:resetBaseModelGuide', (_event, payload) =>
  callApi('/agent/base-model-guide/reset', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:agentStartSession', (_event, payload) =>
  callApi('/agent/sessions/start', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:agentRespondSession', (_event, payload) =>
  callApi('/agent/sessions/respond', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:agentCancelSession', (_event, payload) =>
  callApi('/agent/sessions/cancel', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:applyReview', (_event, payload) =>
  callApi('/agent/apply-review', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:editBaseModel', (_event, payload) =>
  callApi('/projects/base-model/edit', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:refreshProject', (_event, projectDir) =>
  callApi(`/projects/state?projectDir=${encodeURIComponent(projectDir)}`)
);
ipcMain.handle('fraia:refreshProjectIfExists', async (_event, projectDir) => {
  const projectFile = projectFilePath(projectDir);
  if (!fs.existsSync(projectFile)) {
    return null;
  }
  return callApi(`/projects/state?projectDir=${encodeURIComponent(projectDir)}`);
});
ipcMain.handle('fraia:seedFrameDemo', (_event, payload) =>
  callApi('/projects/seed-frame-demo', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:seedFrameReviewDemo', (_event, payload) =>
  callApi('/projects/seed-frame-review-demo', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:seedBeamDemo', (_event, payload) =>
  callApi('/projects/seed-beam-demo', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:sizeBeam', (_event, payload) =>
  callApi('/projects/beam-size', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:validateProject', (_event, payload) =>
  callApi('/projects/validate', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:runFrameCalculix', (_event, payload) =>
  callApi('/projects/frame-run-calculix', { method: 'POST', body: JSON.stringify(payload) })
);
ipcMain.handle('fraia:setThemeSource', (_event, themeSource) => {
  if (!['light', 'dark', 'system'].includes(themeSource)) {
    return { ok: false };
  }
  nativeTheme.themeSource = themeSource;
  syncAllWindowThemes();
  return { ok: true, themeSource };
});
ipcMain.handle('fraia:updateStatus', () => currentUpdateStatus());
ipcMain.handle('fraia:checkForUpdates', async () => {
  if (!updateController?.enabled) return currentUpdateStatus();
  try {
    await updateController.checkNow();
  } catch { /* the controller publishes a user-safe error state */ }
  return currentUpdateStatus();
});
ipcMain.handle('fraia:setUpdateFrequency', (_event, frequency) => {
  if (!updateController?.enabled) return currentUpdateStatus();
  updateController.setFrequency(frequency);
  installApplicationMenu();
  return currentUpdateStatus();
});
ipcMain.handle('fraia:installUpdate', async () => {
  if (!updateController?.enabled) return currentUpdateStatus();
  await updateController.installUpdate();
  return currentUpdateStatus();
});
ipcMain.handle('fraia:reloadWindow', (event) => {
  reloadWindow(BrowserWindow.fromWebContents(event.sender));
  return { ok: true };
});
ipcMain.handle('fraia:forceReloadWindow', (event) => {
  reloadWindow(BrowserWindow.fromWebContents(event.sender), true);
  return { ok: true };
});
ipcMain.handle('fraia:quitApp', () => {
  app.quit();
  return { ok: true };
});
app.whenReady().then(async () => {
  try {
    nativeTheme.themeSource = 'system';
    nativeTheme.on('updated', syncAllWindowThemes);
    await wipeDevLaunchState();
    await ensureAiRuntime();
    await ensureSidecar();
    createWindow();
    if (app.isPackaged) {
      try {
        const { autoUpdater } = require('electron-updater');
        updateController = await configureAutoUpdates({
          app,
          autoUpdater,
          packageMetadata,
          onStatusChange: broadcastUpdateStatus,
          prepareForInstall: prepareForUpdateInstall,
        });
        broadcastUpdateStatus(currentUpdateStatus());
      } catch (error) {
        safeError(`[updater] secure updater initialization failed: ${error}`);
        updateController = {
          enabled: false,
          getStatus: () => ({
            channel: applicationMetadata.channel,
            currentVersion: app.getVersion(),
            enabled: false,
            errorMessage: 'Fraia could not securely start the updater. No update was installed.',
            frequency: 'never',
            phase: 'error',
            reason: 'initialization-failed',
          }),
        };
        broadcastUpdateStatus(currentUpdateStatus());
      }
    }
    installApplicationMenu();
  } catch (error) {
    dialog.showErrorBox('Fraia startup failed', String(error));
    stopSidecar();
    app.quit();
  }
});

app.on('activate', async () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    try {
      await ensureAiRuntime();
      await ensureSidecar();
      createWindow();
    } catch (error) {
      dialog.showErrorBox('Fraia reopen failed', String(error));
    }
  }
});

app.on('browser-window-focus', () => {
  if (aiRuntime) void aiRuntime.refreshAfterFocus();
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  } else {
    void stopSidecarAndWait();
  }
});

app.on('before-quit', (event) => {
  updateController?.stop?.();
  if (quitCleanupComplete) return;
  event.preventDefault();
  if (quitCleanupPromise) return;
  quitCleanupPromise = stopRuntimeServices()
    .catch((error) => safeError(`[shutdown] Runtime cleanup failed: ${error}`))
    .finally(() => {
      quitCleanupComplete = true;
      app.quit();
    });
});
