const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const { assertBinaryArchitecture } = require('../binary-architecture.cjs');
const { nativePlatformArch, sidecarExecutableName } = require('../package-boundary.cjs');

const appRoot = path.resolve(__dirname, '..');
const releaseRoot = path.join(appRoot, 'release');

function packagedLayout() {
  const explicitExecutable = process.env.FRAIA_PACKAGED_EXECUTABLE?.trim();
  if (explicitExecutable) {
    const executable = path.resolve(explicitExecutable);
    const resources = process.platform === 'darwin'
      ? path.join(path.dirname(path.dirname(executable)), 'Resources')
      : path.join(path.dirname(executable), 'resources');
    return { executable, resources };
  }
  if (process.platform === 'darwin') {
    const directory = process.arch === 'x64' ? 'mac' : `mac-${process.arch}`;
    const appRoot = path.join(releaseRoot, directory, 'Fraia.app');
    return {
      executable: path.join(appRoot, 'Contents', 'MacOS', 'Fraia'),
      resources: path.join(appRoot, 'Contents', 'Resources'),
    };
  }
  if (process.platform === 'win32') {
    const directory = process.arch === 'x64' ? 'win-unpacked' : `win-${process.arch}-unpacked`;
    const appRoot = path.join(releaseRoot, directory);
    return { executable: path.join(appRoot, 'Fraia.exe'), resources: path.join(appRoot, 'resources') };
  }
  const directory = process.arch === 'x64' ? 'linux-unpacked' : `linux-${process.arch}-unpacked`;
  const appRoot = path.join(releaseRoot, directory);
  return { executable: path.join(appRoot, 'fraia-electron'), resources: path.join(appRoot, 'resources') };
}

const layout = packagedLayout();
if (!fs.existsSync(layout.executable)) {
  throw new Error(`Exact packaged Fraia executable is missing: ${layout.executable}.`);
}
const sidecar = path.join(layout.resources, 'sidecar', nativePlatformArch(), sidecarExecutableName());
if (!fs.existsSync(sidecar)) throw new Error(`Exact packaged Fraia sidecar is missing: ${sidecar}.`);
assertBinaryArchitecture(layout.executable, process.arch);
assertBinaryArchitecture(sidecar, process.arch);

const npx = process.platform === 'win32' ? 'npx.cmd' : 'npx';
const result = spawnSync(npx, [
  '--no-install',
  'playwright',
  'test',
  '--config',
  'playwright.electron.config.ts',
  'tests/electron/packaged-app.spec.ts',
], {
  cwd: appRoot,
  env: { ...process.env, FRAIA_PACKAGED_EXECUTABLE: layout.executable, FRAIA_DISABLE_UPDATES: '1' },
  stdio: 'inherit',
});
if (result.error) throw result.error;
if (result.status !== 0) process.exitCode = result.status ?? 1;
