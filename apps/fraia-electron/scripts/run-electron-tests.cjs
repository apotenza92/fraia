const { spawnSync } = require('node:child_process');
const path = require('node:path');

const appRoot = path.resolve(__dirname, '..');
const repoRoot = path.resolve(appRoot, '..', '..');
const cargo = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
const sidecarName = process.platform === 'win32' ? 'fraia-appd.exe' : 'fraia-appd';

function resolveTargetDirectory(root, configuredTargetDirectory) {
  const configured = configuredTargetDirectory?.trim();
  return configured ? path.resolve(root, configured) : path.join(root, 'target');
}

function resolvePlaywrightArguments(requestedArguments) {
  return [
    'test',
    '--config',
    'playwright.electron.config.ts',
    ...requestedArguments,
  ];
}

function run() {
  const build = spawnSync(cargo, ['build', '--locked', '-p', 'fraia-appd'], {
    cwd: repoRoot,
    env: process.env,
    stdio: 'inherit',
  });
  if (build.error) throw build.error;
  if (build.status !== 0) return build.status ?? 1;

  const targetDir = resolveTargetDirectory(repoRoot, process.env.CARGO_TARGET_DIR);
  const sidecarPath = path.join(targetDir, 'debug', sidecarName);
  const playwrightCli = require.resolve('@playwright/test/cli');
  const playwrightArgs = resolvePlaywrightArguments(process.argv.slice(2));

  const testRun = spawnSync(process.execPath, [playwrightCli, ...playwrightArgs], {
    cwd: appRoot,
    env: {
      ...process.env,
      FRAIA_APPD_PATH: sidecarPath,
      FRAIA_ELECTRON_TEST_RUNTIME: '1',
    },
    stdio: 'inherit',
  });
  if (testRun.error) throw testRun.error;
  return testRun.status ?? 1;
}

if (require.main === module) process.exit(run());

module.exports = {
  resolvePlaywrightArguments,
  resolveTargetDirectory,
};
