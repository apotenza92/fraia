const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const {
  nativePlatformArch,
  sidecarExecutableName,
} = require('../package-boundary.cjs');
const { assertBinaryArchitecture } = require('../binary-architecture.cjs');

const appRoot = path.resolve(__dirname, '..');
const repositoryRoot = path.resolve(appRoot, '..', '..');
const platformArch = nativePlatformArch();
const executableName = sidecarExecutableName();
const cargo = process.platform === 'win32' ? 'cargo.exe' : 'cargo';
const source = path.join(repositoryRoot, 'target', 'release', executableName);
const destination = path.join(appRoot, '.package', 'sidecar', platformArch, executableName);

const build = spawnSync(cargo, ['build', '--locked', '--release', '-p', 'fraia-appd'], {
  cwd: repositoryRoot,
  stdio: 'inherit',
});
if (build.error) throw build.error;
if (build.status !== 0) {
  throw new Error(`Fraia app service release build failed with status ${build.status}.`);
}
if (!fs.existsSync(source)) {
  throw new Error(`Fraia app service build did not produce ${source}.`);
}
assertBinaryArchitecture(source, process.arch);

fs.mkdirSync(path.dirname(destination), { recursive: true });
fs.copyFileSync(source, destination);
if (process.platform !== 'win32') fs.chmodSync(destination, 0o755);
assertBinaryArchitecture(destination, process.arch);

console.log(`[package] Bundled native sidecar ${source} -> ${destination}`);
