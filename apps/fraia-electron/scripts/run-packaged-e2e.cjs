const { spawnSync } = require('node:child_process');
const asar = require('@electron/asar');
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

function isMachO(filePath) {
  if (!fs.statSync(filePath).isFile()) return false;
  const descriptor = fs.openSync(filePath, 'r');
  try {
    const header = Buffer.alloc(4);
    if (fs.readSync(descriptor, header, 0, header.length, 0) !== header.length) return false;
    return new Set([
      'feedface', 'feedfacf', 'cefaedfe', 'cffaedfe',
      'cafebabe', 'bebafeca', 'cafebabf', 'bfbafeca',
    ]).has(header.toString('hex'));
  } finally {
    fs.closeSync(descriptor);
  }
}

function prepareUnsignedMacosRuntime(resources) {
  if (process.platform !== 'darwin') return;
  const runtimeDirectory = path.join(resources, 'runtimes', 'calculix', nativePlatformArch());
  if (!fs.existsSync(runtimeDirectory)) return;
  const calculix = path.join(runtimeDirectory, 'ccx');
  const signature = spawnSync('codesign', ['--verify', '--strict', calculix], {
    encoding: 'utf8',
    stdio: 'pipe',
  });
  if (signature.status === 0) return;

  const nativeFiles = fs.readdirSync(runtimeDirectory)
    .map((name) => path.join(runtimeDirectory, name))
    .filter((candidate) => isMachO(candidate))
    .sort((left, right) => (left === calculix ? 1 : 0) - (right === calculix ? 1 : 0));
  for (const target of nativeFiles) {
    const result = spawnSync('codesign', ['--force', '--sign', '-', target], { stdio: 'inherit' });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`Failed to ad-hoc sign local packaged runtime: ${target}`);
  }
  console.log('[package] Ad-hoc signed the unsigned local CalculiX runtime for macOS execution testing.');
}

function verifyProductionDependencyBoundary(resources) {
  const archive = path.join(resources, 'app.asar');
  if (!fs.existsSync(archive)) throw new Error(`Packaged Fraia ASAR is missing: ${archive}.`);
  const entries = new Set(
    asar.listPackage(archive).map((entry) => entry.replaceAll('\\', '/')),
  );
  const packageLock = JSON.parse(fs.readFileSync(path.join(appRoot, 'package-lock.json'), 'utf8'));
  const productionDependencies = [
    '@earendil-works/pi-agent-core',
    '@earendil-works/pi-ai',
    'electron-updater',
    'typebox',
  ];
  for (const dependency of productionDependencies) {
    const packagePath = `node_modules/${dependency}/package.json`;
    if (!entries.has(`/${packagePath}`)) {
      throw new Error(`Packaged Fraia is missing production dependency ${dependency}.`);
    }
    const packaged = JSON.parse(asar.extractFile(archive, packagePath).toString('utf8'));
    const locked = packageLock.packages[`node_modules/${dependency}`];
    if (packaged.version !== locked?.version || packaged.license !== 'MIT') {
      throw new Error(`Packaged Fraia has unreviewed ${dependency} version or licence metadata.`);
    }
  }

  const excludedPackages = [
    '@base-ui/react',
    '@earendil-works/pi-coding-agent',
    '@playwright/test',
    '@tailwindcss/vite',
    '@vitejs/plugin-react',
    'electron',
    'electron-builder',
    'jsdom',
    'lucide-react',
    'react',
    'react-dom',
    'shadcn',
    'tailwindcss',
    'three',
    'typescript',
    'vite',
    'vitest',
  ];
  for (const dependency of excludedPackages) {
    if (entries.has(`/node_modules/${dependency}/package.json`)) {
      throw new Error(`Packaged Fraia unexpectedly contains development dependency ${dependency}.`);
    }
  }
}

const layout = packagedLayout();
if (!fs.existsSync(layout.executable)) {
  throw new Error(`Exact packaged Fraia executable is missing: ${layout.executable}.`);
}
const sidecar = path.join(layout.resources, 'sidecar', nativePlatformArch(), sidecarExecutableName());
if (!fs.existsSync(sidecar)) throw new Error(`Exact packaged Fraia sidecar is missing: ${sidecar}.`);
assertBinaryArchitecture(layout.executable, process.arch);
assertBinaryArchitecture(sidecar, process.arch);
verifyProductionDependencyBoundary(layout.resources);
prepareUnsignedMacosRuntime(layout.resources);

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
