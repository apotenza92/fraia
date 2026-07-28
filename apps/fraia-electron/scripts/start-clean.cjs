const { spawn } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const appDir = path.resolve(__dirname, '..');
const launchRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-clean-'));
const userDataDir = path.join(launchRoot, 'user-data');
const projectDir = path.join(launchRoot, 'project');
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';

const child = spawn(npmCommand, ['run', 'start:fresh-guide'], {
  cwd: appDir,
  env: {
    ...process.env,
    FRAIA_USER_DATA_DIR: userDataDir,
    FRAIA_DEFAULT_PROJECT_DIR: projectDir,
    VITE_FRAIA_DEFAULT_PROJECT_DIR: projectDir,
  },
  stdio: 'inherit',
});

console.log(`[clean-launch] Disposable Fraia data: ${launchRoot}`);

function forwardSignal(signal) {
  if (!child.killed) child.kill(signal);
}

process.once('SIGINT', () => forwardSignal('SIGINT'));
process.once('SIGTERM', () => forwardSignal('SIGTERM'));

child.once('exit', (code, signal) => {
  fs.rmSync(launchRoot, { recursive: true, force: true });
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exitCode = code ?? 1;
});

child.once('error', (error) => {
  fs.rmSync(launchRoot, { recursive: true, force: true });
  console.error(`[clean-launch] Could not start Fraia: ${error.message}`);
  process.exitCode = 1;
});
