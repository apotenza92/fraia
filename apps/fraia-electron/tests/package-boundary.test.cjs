const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const { createHash } = require('node:crypto');
const { spawnSync } = require('node:child_process');
const {
  SUPPORTED_TARGETS,
  nativePlatformArch,
  packagedCalculixPath,
  packagedSidecarPath,
  resolveCalculixRuntime,
  resolveSidecarLaunch,
} = require('../package-boundary.cjs');
const {
  importRuntimeContract,
  validateImportRuntimeSources,
} = require('../import-runtime-contract.cjs');

test('all importers have one reviewed offline package contract', () => {
  const appRoot = path.join(__dirname, '..');
  const builderSource = fs.readFileSync(path.join(appRoot, 'electron-builder.config.cjs'), 'utf8');
  const notices = fs.readFileSync(path.join(appRoot, 'IMPORT_RUNTIME_NOTICES.txt'), 'utf8');

  assert.equal(validateImportRuntimeSources(appRoot), importRuntimeContract);
  assert.equal(importRuntimeContract.networkPolicy, 'offline-only-no-runtime-downloads');
  assert.deepEqual(importRuntimeContract.targets, [
    'darwin-arm64',
    'darwin-x64',
    'linux-arm64',
    'linux-x64',
    'win32-arm64',
    'win32-x64',
  ]);
  assert.equal(importRuntimeContract.importers.ocr.implementation, 'bundled-node-worker-wasm');
  assert.equal(importRuntimeContract.importers.ocr.coreVariants.length, 6);
  assert.match(builderSource, /'IMPORT_RUNTIME_NOTICES\.txt'/);
  assert.match(builderSource, /'import-runtime-licenses\/\*\*\/\*'/);
  assert.match(builderSource, /'import-runtime-contract\.cjs'/);
  assert.match(notices, /PDF\.js 6\.2\.108/);
  assert.match(notices, /lopdf 0\.44\.0/);
  assert.match(notices, /import-runtime-licenses\/PDFJS-APACHE-2\.0\.txt/);
  assert.match(notices, /import-runtime-licenses\/LOPDF-MIT\.txt/);
  assert.match(notices, /Tesseract\.js 7\.0\.0 and tesseract\.js-core 7\.0\.0/);
  assert.match(notices, /English tessdata_fast 4\.1\.0/);
  assert.match(notices, /unconfirmed inferred candidate/);
});

test('scanned OCR PDF fixture is raster-only and byte-deterministic', () => {
  const appRoot = path.join(__dirname, '..');
  const fixture = path.join(appRoot, 'tests', 'fixtures', 'scanned-architectural-drawing.pdf');
  const generator = path.join(appRoot, 'tests', 'fixtures', 'generate-scanned-ocr-fixture.cjs');
  const expected = 'b6180dec6b6f33a323c36d6dc5215ebc6382d04e418a4ae7e87546d2786cd8ae';
  const before = fs.readFileSync(fixture);
  assert.equal(createHash('sha256').update(before).digest('hex'), expected);
  assert.doesNotMatch(before.toString('latin1'), /CreationDate|Quartz|\/Font\b/);
  assert.match(before.toString('latin1'), /\/Subtype \/Image/);
  const generated = spawnSync(process.execPath, [generator], { encoding: 'utf8' });
  assert.equal(generated.status, 0, generated.stderr);
  assert.equal(createHash('sha256').update(fs.readFileSync(fixture)).digest('hex'), expected);
});

test('production uses the reviewed low-level Pi runtime without coding-agent shrinkwrap', () => {
  const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'));
  const packageLock = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package-lock.json'), 'utf8'));

  assert.equal(packageJson.dependencies['@earendil-works/pi-ai'], '0.82.1');
  assert.equal(packageJson.dependencies['@earendil-works/pi-agent-core'], '0.82.1');
  assert.equal(packageJson.dependencies['@earendil-works/pi-coding-agent'], undefined);
  assert.deepEqual(Object.keys(packageJson.dependencies).sort(), [
    '@earendil-works/pi-agent-core',
    '@earendil-works/pi-ai',
    '@fontsource-variable/geist',
    '@shadcn/react',
    'electron-updater',
    'tesseract.js',
    'tesseract.js-core',
    'tuf-js',
    'typebox',
  ]);
  for (const dependency of Object.keys(packageJson.dependencies)) {
    const expectedLicense = dependency === '@fontsource-variable/geist'
      ? 'OFL-1.1'
      : (dependency.startsWith('tesseract.js') ? 'Apache-2.0' : 'MIT');
    assert.equal(
      packageLock.packages[`node_modules/${dependency}`]?.license,
      expectedLicense,
      `${dependency} must retain reviewed licence metadata`,
    );
  }
  assert.equal(
    Object.keys(packageLock.packages).some((packagePath) => packagePath.endsWith('/@earendil-works/pi-coding-agent')),
    false,
  );
  for (const [packagePath, metadata] of Object.entries(packageLock.packages)) {
    if (!packagePath.endsWith('/brace-expansion')) continue;
    assert.notEqual(metadata.version, '5.0.7', `${packagePath} must not resolve vulnerable brace-expansion 5.0.7`);
  }
});

test('PDF.js is an offline browser build and not a packaged native runtime dependency', () => {
  const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'));
  const packageLock = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package-lock.json'), 'utf8'));
  const browserSource = fs.readFileSync(path.join(__dirname, '..', 'src', 'components', 'sources', 'PdfPageBrowser.tsx'), 'utf8');
  const builderSource = fs.readFileSync(path.join(__dirname, '..', 'electron-builder.config.cjs'), 'utf8');

  assert.equal(packageJson.devDependencies['pdfjs-dist'], '6.2.108');
  assert.equal(packageJson.dependencies['pdfjs-dist'], undefined);
  assert.equal(packageLock.packages['node_modules/pdfjs-dist'].license, 'Apache-2.0');
  assert.equal(packageLock.packages['node_modules/pdfjs-dist'].optionalDependencies['@napi-rs/canvas'], '^1.0.0');
  assert.match(browserSource, /import\('pdfjs-dist'\)/);
  assert.match(browserSource, /pdf\.worker\.min\.mjs\?url/);
  assert.doesNotMatch(builderSource, /pdfjs-dist|napi-rs\/canvas|pdftoppm|poppler/i);
});

test('packaged apps launch only the bundled native sidecar', () => {
  const resourcesPath = path.resolve('/packaged-resources');
  const expected = packagedSidecarPath(resourcesPath, 'darwin', 'arm64');
  const launch = resolveSidecarLaunch({
    isPackaged: true,
    resourcesPath,
    repoRoot: '/source-repository',
    explicitPath: '/tmp/untrusted-override',
    platform: 'darwin',
    arch: 'arm64',
    pathExists: (candidate) => candidate === expected,
  });

  assert.deepEqual(launch, {
    command: expected,
    args: [],
    cwd: resourcesPath,
    source: 'packaged-resource',
  });
  assert.doesNotMatch(JSON.stringify(launch), /cargo|untrusted-override/);
});

test('packaged apps fail closed when the bundled sidecar is absent', () => {
  assert.throws(() => resolveSidecarLaunch({
    isPackaged: true,
    resourcesPath: '/packaged-resources',
    repoRoot: '/source-repository',
    platform: 'linux',
    arch: 'x64',
    pathExists: () => false,
  }), /Packaged Fraia sidecar is missing/);
});

test('packaged apps use only the exact bundled CalculiX runtime', () => {
  const resourcesPath = path.resolve('/packaged-resources');
  const expected = packagedCalculixPath(resourcesPath, 'darwin', 'arm64');
  const runtime = resolveCalculixRuntime({
    isPackaged: true,
    resourcesPath,
    explicitPath: '/opt/homebrew/bin/ccx_2.23',
    developmentResolver: () => '/user-data/runtimes/calculix/ccx',
    platform: 'darwin',
    arch: 'arm64',
    pathExists: (candidate) => candidate === expected,
  });

  assert.deepEqual(runtime, {
    executable: expected,
    source: 'packaged-resource',
  });
  assert.doesNotMatch(JSON.stringify(runtime), /homebrew|user-data/);
});

test('packaged apps fail closed when bundled CalculiX is absent', () => {
  assert.throws(() => resolveCalculixRuntime({
    isPackaged: true,
    resourcesPath: '/packaged-resources',
    explicitPath: '/opt/homebrew/bin/ccx_2.23',
    developmentResolver: () => '/usr/local/bin/ccx',
    platform: 'linux',
    arch: 'x64',
    pathExists: () => false,
  }), /Packaged Fraia CalculiX runtime is missing/);
});

test('development retains explicit and managed CalculiX discovery', () => {
  assert.equal(resolveCalculixRuntime({
    isPackaged: false,
    resourcesPath: '/unused',
    explicitPath: '/tmp/ccx',
    developmentResolver: () => '/usr/local/bin/ccx',
    pathExists: (candidate) => candidate === '/tmp/ccx',
  }).source, 'explicit-development-path');
  assert.deepEqual(resolveCalculixRuntime({
    isPackaged: false,
    resourcesPath: '/unused',
    developmentResolver: () => '/usr/local/bin/ccx',
  }), {
    executable: '/usr/local/bin/ccx',
    source: 'managed-development-runtime',
  });
});

test('development retains explicit-path and Cargo launch modes', () => {
  assert.equal(resolveSidecarLaunch({
    isPackaged: false,
    resourcesPath: '/unused',
    repoRoot: '/source-repository',
    explicitPath: '/tmp/fraia-appd',
  }).source, 'explicit-development-path');
  assert.equal(resolveSidecarLaunch({
    isPackaged: false,
    resourcesPath: '/unused',
    repoRoot: '/source-repository',
  }).source, 'cargo-development');
});

test('the package matrix is deliberately limited to native desktop targets', () => {
  assert.deepEqual([...SUPPORTED_TARGETS].sort(), [
    'darwin-arm64',
    'darwin-x64',
    'linux-arm64',
    'linux-x64',
    'win32-arm64',
    'win32-x64',
  ]);
  assert.equal(nativePlatformArch('win32', 'x64'), 'win32-x64');
  assert.equal(nativePlatformArch('win32', 'arm64'), 'win32-arm64');
  assert.equal(nativePlatformArch('linux', 'arm64'), 'linux-arm64');
  assert.throws(() => nativePlatformArch('freebsd', 'x64'), /does not support/);
  assert.equal(
    packagedCalculixPath('/resources', 'win32', 'arm64'),
    path.join('/resources', 'runtimes', 'calculix', 'win32-arm64', 'ccx.exe'),
  );
});

test('the renderer opens independent Fraia projects through the canonical manifest picker', () => {
  const main = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');
  const preload = fs.readFileSync(path.join(__dirname, '..', 'preload.js'), 'utf8');

  assert.match(main, /ipcMain\.handle\(['"]fraia:pickProjectFile/);
  assert.match(main, /properties: \['openFile', 'openDirectory'\]/);
  assert.match(main, /path\.basename\(selectedPath\) === 'fraia\.project\.json'/);
  assert.match(main, /fs\.existsSync\(projectFilePath\(projectDir\)\)/);
  assert.match(preload, /pickProjectFile: \(\) => ipcRenderer\.invoke\(['"]fraia:pickProjectFile/);
});

test('new projects start immediately in managed unsaved-project storage', () => {
  const main = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');
  const preload = fs.readFileSync(path.join(__dirname, '..', 'preload.js'), 'utf8');

  assert.match(main, /ipcMain\.handle\(['"]fraia:createUntitledProject/);
  assert.match(main, /'unsaved-projects'/);
  assert.match(main, /`untitled-\$\{Date\.now\(\)\}/);
  assert.match(preload, /createUntitledProject: \(\) => ipcRenderer\.invoke\(['"]fraia:createUntitledProject/);
});

test('Save moves an untitled project into one new user project folder', () => {
  const main = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');
  const preload = fs.readFileSync(path.join(__dirname, '..', 'preload.js'), 'utf8');

  assert.match(main, /ipcMain\.handle\(['"]fraia:saveProject/);
  assert.match(main, /callApi\('\/conversations\/unload'/);
  assert.match(main, /copyProjectToNewFolder\(sourceDir, destinationDir\)/);
  assert.match(main, /accelerator: 'CommandOrControl\+S'/);
  assert.match(main, /accelerator: 'CommandOrControl\+Shift\+S'/);
  assert.match(preload, /saveProject: \(payload\) => ipcRenderer\.invoke\('fraia:saveProject'/);
});
