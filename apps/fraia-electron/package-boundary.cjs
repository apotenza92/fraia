const fs = require('node:fs');
const path = require('node:path');

const SUPPORTED_TARGETS = new Set([
  'darwin-arm64',
  'darwin-x64',
  'linux-arm64',
  'linux-x64',
  'win32-x64',
]);

function nativePlatformArch(platform = process.platform, arch = process.arch) {
  const target = `${platform}-${arch}`;
  if (!SUPPORTED_TARGETS.has(target)) {
    throw new Error(`Fraia packaging does not support ${platform}-${arch}.`);
  }
  return target;
}

function sidecarExecutableName(platform = process.platform) {
  return platform === 'win32' ? 'fraia-appd.exe' : 'fraia-appd';
}

function calculixExecutableName(platform = process.platform) {
  return platform === 'win32' ? 'ccx.exe' : 'ccx';
}

function packagedCalculixSourceDirectory(appRoot, platform = process.platform, arch = process.arch) {
  return path.join(appRoot, 'runtimes', 'calculix', nativePlatformArch(platform, arch));
}

function packagedCalculixPath(resourcesPath, platform = process.platform, arch = process.arch) {
  return path.join(resourcesPath, 'runtimes', 'calculix', nativePlatformArch(platform, arch), calculixExecutableName(platform));
}

function resolveCalculixRuntime({
  isPackaged,
  resourcesPath,
  explicitPath,
  developmentResolver,
  platform = process.platform,
  arch = process.arch,
  pathExists = fs.existsSync,
}) {
  if (isPackaged) {
    const executable = packagedCalculixPath(resourcesPath, platform, arch);
    if (!pathExists(executable)) {
      throw new Error(`Packaged Fraia CalculiX runtime is missing: ${executable}`);
    }
    return { executable, source: 'packaged-resource' };
  }

  if (explicitPath && pathExists(explicitPath)) {
    return { executable: explicitPath, source: 'explicit-development-path' };
  }

  const executable = developmentResolver?.() || null;
  return {
    executable,
    source: executable ? 'managed-development-runtime' : 'unavailable',
  };
}

function packagedSidecarPath(resourcesPath, platform = process.platform, arch = process.arch) {
  return path.join(
    resourcesPath,
    'sidecar',
    nativePlatformArch(platform, arch),
    sidecarExecutableName(platform),
  );
}

function resolveSidecarLaunch({
  isPackaged,
  resourcesPath,
  repoRoot,
  explicitPath,
  platform = process.platform,
  arch = process.arch,
  pathExists = fs.existsSync,
}) {
  if (isPackaged) {
    const executable = packagedSidecarPath(resourcesPath, platform, arch);
    if (!pathExists(executable)) {
      throw new Error(`Packaged Fraia sidecar is missing: ${executable}`);
    }
    return {
      command: executable,
      args: [],
      cwd: resourcesPath,
      source: 'packaged-resource',
    };
  }

  if (explicitPath) {
    return {
      command: explicitPath,
      args: [],
      cwd: repoRoot,
      source: 'explicit-development-path',
    };
  }

  return {
    command: platform === 'win32' ? 'cargo.exe' : 'cargo',
    args: ['run', '-p', 'fraia-appd', '--'],
    cwd: repoRoot,
    source: 'cargo-development',
  };
}

module.exports = {
  SUPPORTED_TARGETS,
  calculixExecutableName,
  nativePlatformArch,
  packagedCalculixPath,
  packagedCalculixSourceDirectory,
  packagedSidecarPath,
  resolveCalculixRuntime,
  resolveSidecarLaunch,
  sidecarExecutableName,
};
