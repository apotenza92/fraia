const { spawn } = require('node:child_process');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');

const appDir = path.resolve(__dirname, '..');
const host = '127.0.0.1';
const defaultPort = 5173;

function processIsRunning(pid) {
  if (!Number.isInteger(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code === 'EPERM';
  }
}

function defaultLockPath(env = process.env) {
  if (env.FRAIA_DEV_LOCK_PATH?.trim()) return path.resolve(env.FRAIA_DEV_LOCK_PATH.trim());
  const userToken = typeof process.getuid === 'function' ? process.getuid() : 'user';
  return path.join(os.tmpdir(), `fraia-electron-dev-${userToken}.lock`);
}

function releaseLaunchLock(lockPath, pid = process.pid) {
  try {
    const lock = JSON.parse(fs.readFileSync(lockPath, 'utf8'));
    if (lock.pid === pid) fs.unlinkSync(lockPath);
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error;
  }
}

function acquireLaunchLock(lockPath, pid = process.pid) {
  fs.mkdirSync(path.dirname(lockPath), { recursive: true });
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      const descriptor = fs.openSync(lockPath, 'wx');
      fs.writeFileSync(descriptor, `${JSON.stringify({ pid, appDir, startedAt: new Date().toISOString() })}\n`);
      fs.closeSync(descriptor);
      return;
    } catch (error) {
      if (error?.code !== 'EEXIST') throw error;
      let existingPid = null;
      try {
        existingPid = JSON.parse(fs.readFileSync(lockPath, 'utf8')).pid;
      } catch {
        // An unreadable lock is stale and can be replaced.
      }
      if (processIsRunning(existingPid)) {
        throw new Error(`Fraia Dev is already running under launcher PID ${existingPid}. Use that window instead of starting another copy.`);
      }
      fs.unlinkSync(lockPath);
    }
  }
  throw new Error('Could not acquire the Fraia Dev launch lock.');
}

function resolveLaunchConfig(args = process.argv.slice(2), env = process.env) {
  const rawPort = Number.parseInt(env.FRAIA_DEV_SERVER_PORT || String(defaultPort), 10);
  if (!Number.isInteger(rawPort) || rawPort < 1024 || rawPort > 65535) {
    throw new Error(`FRAIA_DEV_SERVER_PORT must be between 1024 and 65535; received ${env.FRAIA_DEV_SERVER_PORT}.`);
  }
  return {
    clean: args.includes('--clean'),
    freshGuide: args.includes('--fresh-guide'),
    host,
    port: rawPort,
    serverUrl: `http://${host}:${rawPort}`,
    lockPath: defaultLockPath(env),
  };
}

function viteArguments(config) {
  return ['--host', config.host, '--port', String(config.port), '--strictPort', '--force'];
}

function sourceProvenance(directory = appDir) {
  const files = ['main.js', 'preload.js'];
  return Object.fromEntries(files.map((file) => [file, fs.statSync(path.join(directory, file)).mtimeMs]));
}

function serverIsReady(url) {
  return new Promise((resolve) => {
    const request = http.get(url, (response) => {
      response.resume();
      resolve(Boolean(response.statusCode && response.statusCode < 500));
    });
    request.setTimeout(500, () => {
      request.destroy();
      resolve(false);
    });
    request.on('error', () => resolve(false));
  });
}

async function waitForServer(url, viteProcess, timeoutMs = 20_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (viteProcess.exitCode !== null) {
      throw new Error(`Vite exited before Fraia Dev was ready (code ${viteProcess.exitCode}).`);
    }
    if (await serverIsReady(url)) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`Vite did not become ready at ${url} within ${timeoutMs} ms.`);
}

function stopChild(child, signal = 'SIGTERM') {
  if (child && child.exitCode === null && !child.killed) child.kill(signal);
}

async function run() {
  const config = resolveLaunchConfig();
  acquireLaunchLock(config.lockPath);
  let launchRoot = null;
  let viteProcess = null;
  let electronProcess = null;
  let stopping = false;

  const cleanup = () => {
    releaseLaunchLock(config.lockPath);
    if (launchRoot) fs.rmSync(launchRoot, { recursive: true, force: true });
  };
  process.once('exit', cleanup);

  try {
    const launchEnv = {
      ...process.env,
      FRAIA_DEV_RUNTIME: '1',
      FRAIA_DEV_APP_DIR: appDir,
      VITE_DEV_SERVER_URL: config.serverUrl,
      FRAIA_DEV_SOURCE_PROVENANCE: JSON.stringify(sourceProvenance()),
    };
    if (config.freshGuide) {
      launchEnv.FRAIA_DEV_FRESH_GUIDE = '1';
      launchEnv.VITE_FRAIA_DEV_FRESH_GUIDE = '1';
    }
    if (config.clean) {
      launchRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-clean-'));
      const userDataDir = path.join(launchRoot, 'user-data');
      const projectDir = path.join(launchRoot, 'project');
      launchEnv.FRAIA_USER_DATA_DIR = userDataDir;
      launchEnv.FRAIA_DEFAULT_PROJECT_DIR = projectDir;
      launchEnv.VITE_FRAIA_DEFAULT_PROJECT_DIR = projectDir;
      console.log(`[dev-launch] Disposable data: ${launchRoot}`);
    }

    const viteCommand = path.join(appDir, 'node_modules', '.bin', process.platform === 'win32' ? 'vite.cmd' : 'vite');
    viteProcess = spawn(viteCommand, viteArguments(config), {
      cwd: appDir,
      env: launchEnv,
      stdio: 'inherit',
    });
    viteProcess.once('error', (error) => {
      console.error(`[dev-launch] Could not start Vite: ${error.message}`);
    });
    await waitForServer(config.serverUrl, viteProcess);

    const electronBinary = require('electron');
    console.log('[dev-launch] App identity: Fraia Dev');
    console.log(`[dev-launch] Source: ${appDir}`);
    console.log(`[dev-launch] Renderer: ${config.serverUrl}`);
    if (launchEnv.FRAIA_USER_DATA_DIR) console.log(`[dev-launch] User data: ${launchEnv.FRAIA_USER_DATA_DIR}`);
    electronProcess = spawn(electronBinary, ['.'], {
      cwd: appDir,
      env: launchEnv,
      stdio: 'inherit',
    });

    const forwardSignal = (signal) => {
      if (stopping) return;
      stopping = true;
      stopChild(electronProcess, signal);
      stopChild(viteProcess, signal);
    };
    process.once('SIGINT', () => forwardSignal('SIGINT'));
    process.once('SIGTERM', () => forwardSignal('SIGTERM'));

    viteProcess.once('exit', (code) => {
      if (stopping) return;
      stopping = true;
      stopChild(electronProcess);
      console.error(`[dev-launch] Vite stopped unexpectedly (code ${code}).`);
      process.exitCode = code || 1;
    });
    electronProcess.once('error', (error) => {
      if (stopping) return;
      stopping = true;
      stopChild(viteProcess);
      console.error(`[dev-launch] Could not start Fraia Dev: ${error.message}`);
      process.exitCode = 1;
    });
    electronProcess.once('exit', (code, signal) => {
      if (!stopping) stopping = true;
      stopChild(viteProcess);
      if (signal) console.log(`[dev-launch] Fraia Dev stopped by ${signal}.`);
      process.exitCode = code ?? (signal ? 0 : 1);
    });
  } catch (error) {
    stopping = true;
    stopChild(electronProcess);
    stopChild(viteProcess);
    cleanup();
    throw error;
  }
}

if (require.main === module) {
  run().catch((error) => {
    console.error(`[dev-launch] ${error.message}`);
    process.exitCode = 1;
  });
}

module.exports = {
  acquireLaunchLock,
  defaultLockPath,
  processIsRunning,
  releaseLaunchLock,
  resolveLaunchConfig,
  serverIsReady,
  sourceProvenance,
  viteArguments,
  waitForServer,
};
