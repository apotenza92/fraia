const { spawnSync } = require('node:child_process');
const asar = require('@electron/asar');
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { assertBinaryArchitecture } = require('../binary-architecture.cjs');
const { assertMacosMinimumVersion } = require('../macos-version-contract.cjs');
const {
  nativePlatformArch,
  packagedCalculixPath,
  sidecarExecutableName,
} = require('../package-boundary.cjs');
const packageMetadata = require('../package.json');
const { releaseContract } = require('../release-contract.cjs');

const appRoot = path.resolve(__dirname, '..');
const releaseRoot = path.join(appRoot, 'release');
const defaultChannel = packageMetadata.version.includes('-beta.') ? 'beta' : 'stable';
const contract = releaseContract({
  channel: process.env.FRAIA_RELEASE_CHANNEL || defaultChannel,
  platform: process.platform,
  arch: process.arch,
});

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
    const packagedAppRoot = path.join(releaseRoot, directory, contract.appName);
    return {
      executable: path.join(packagedAppRoot, 'Contents', 'MacOS', contract.productName),
      resources: path.join(packagedAppRoot, 'Contents', 'Resources'),
    };
  }
  if (process.platform === 'win32') {
    const directory = process.arch === 'x64' ? 'win-unpacked' : `win-${process.arch}-unpacked`;
    const packagedAppRoot = path.join(releaseRoot, directory);
    return {
      executable: path.join(packagedAppRoot, `${contract.productName}.exe`),
      resources: path.join(packagedAppRoot, 'resources'),
    };
  }
  const directory = process.arch === 'x64' ? 'linux-unpacked' : `linux-${process.arch}-unpacked`;
  const packagedAppRoot = path.join(releaseRoot, directory);
  return {
    executable: path.join(packagedAppRoot, contract.packageName),
    resources: path.join(packagedAppRoot, 'resources'),
  };
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
  for (const packagedContractFile of [
    '/IMPORT_RUNTIME_NOTICES.txt',
    '/import-runtime-licenses/LOPDF-MIT.txt',
    '/import-runtime-licenses/PDFJS-APACHE-2.0.txt',
    '/import-runtime-licenses/TESSERACTJS-APACHE-2.0.txt',
    '/import-runtime-licenses/TESSDATA-FAST-APACHE-2.0.txt',
    '/import-runtime-contract.cjs',
    '/ocr-runtime.cjs',
    '/ocr-runtime/eng.traineddata',
    '/node_modules/tesseract.js/package.json',
    '/node_modules/tesseract.js-core/package.json',
  ]) {
    if (!entries.has(packagedContractFile)) {
      throw new Error(`Packaged Fraia is missing ${packagedContractFile}.`);
    }
  }
  const packageLock = JSON.parse(fs.readFileSync(path.join(appRoot, 'package-lock.json'), 'utf8'));
  const productionDependencies = [
    '@earendil-works/pi-agent-core',
    '@earendil-works/pi-ai',
    'electron-updater',
    'typebox',
    'tesseract.js',
    'tesseract.js-core',
  ];
  for (const dependency of productionDependencies) {
    const packagePath = `node_modules/${dependency}/package.json`;
    if (!entries.has(`/${packagePath}`)) {
      throw new Error(`Packaged Fraia is missing production dependency ${dependency}.`);
    }
    const extractionPath = packagePath.split('/').join(path.sep);
    const packaged = JSON.parse(asar.extractFile(archive, extractionPath).toString('utf8'));
    const locked = packageLock.packages[`node_modules/${dependency}`];
    const expectedLicense = dependency.startsWith('tesseract.js') ? 'Apache-2.0' : 'MIT';
    if (packaged.version !== locked?.version || packaged.license !== expectedLicense) {
      throw new Error(`Packaged Fraia has unreviewed ${dependency} version or licence metadata.`);
    }
  }
  const importContract = require(path.join(appRoot, 'import-runtime-contract.cjs')).importRuntimeContract;
  const ocr = importContract.importers.ocr;
  for (const [asset, expectedSha256] of Object.entries(ocr.coreAssetSha256)) {
    const packagePath = `node_modules/${ocr.corePackage}/${asset}`;
    if (!entries.has(`/${packagePath}`)) {
      throw new Error(`Packaged Fraia is missing reviewed OCR core asset ${asset}.`);
    }
    const packagedBytes = asar.extractFile(archive, packagePath.split('/').join(path.sep));
    if (createHash('sha256').update(packagedBytes).digest('hex') !== expectedSha256) {
      throw new Error(`Packaged OCR core asset ${asset} differs from the reviewed SHA-256.`);
    }
  }
  const modelBytes = asar.extractFile(archive, ocr.modelFile.split('/').join(path.sep));
  if (modelBytes.byteLength !== ocr.modelByteSize
    || createHash('sha256').update(modelBytes).digest('hex') !== ocr.modelSha256) {
    throw new Error('Packaged English OCR model differs from the reviewed contract.');
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
if (process.platform === 'darwin') {
  for (const target of [
    layout.executable,
    sidecar,
    packagedCalculixPath(layout.resources, 'darwin', process.arch),
  ]) {
    assertMacosMinimumVersion(target);
  }
}
verifyProductionDependencyBoundary(layout.resources);
prepareUnsignedMacosRuntime(layout.resources);

const playwrightCli = require.resolve('@playwright/test/cli');
const result = spawnSync(process.execPath, [
  playwrightCli,
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
