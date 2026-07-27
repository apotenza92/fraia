const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const {
  SUPPORTED_TARGETS,
  nativePlatformArch,
  packagedCalculixPath,
  packagedSidecarPath,
  resolveCalculixRuntime,
  resolveSidecarLaunch,
} = require('../package-boundary.cjs');

test('production uses the reviewed low-level Pi runtime without coding-agent shrinkwrap', () => {
  const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package.json'), 'utf8'));
  const packageLock = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'package-lock.json'), 'utf8'));

  assert.equal(packageJson.dependencies['@earendil-works/pi-ai'], '0.82.1');
  assert.equal(packageJson.dependencies['@earendil-works/pi-agent-core'], '0.82.1');
  assert.equal(packageJson.dependencies['@earendil-works/pi-coding-agent'], undefined);
  assert.deepEqual(Object.keys(packageJson.dependencies).sort(), [
    '@earendil-works/pi-agent-core',
    '@earendil-works/pi-ai',
    'electron-updater',
    'typebox',
  ]);
  for (const dependency of Object.keys(packageJson.dependencies)) {
    assert.equal(
      packageLock.packages[`node_modules/${dependency}`]?.license,
      'MIT',
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
    'win32-x64',
  ]);
  assert.equal(nativePlatformArch('win32', 'x64'), 'win32-x64');
  assert.equal(nativePlatformArch('linux', 'arm64'), 'linux-arm64');
  assert.throws(() => nativePlatformArch('freebsd', 'x64'), /does not support/);
  assert.throws(() => nativePlatformArch('win32', 'arm64'), /does not support win32-arm64/);
  assert.throws(() => packagedCalculixPath('/resources', 'win32', 'arm64'), /does not support win32-arm64/);
});
