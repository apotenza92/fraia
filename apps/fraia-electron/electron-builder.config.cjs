const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const {
  calculixExecutableName,
  nativePlatformArch,
  packagedCalculixSourceDirectory,
  sidecarExecutableName,
} = require('./package-boundary.cjs');
const packageMetadata = require('./package.json');
const { validateRuntimeDirectory } = require('./calculix-runtime-manifest.cjs');
const { metadataFileName, releaseContract } = require('./release-contract.cjs');
const { readReleaseNotes } = require('./scripts/changelog.cjs');

const platformArch = nativePlatformArch();
const sidecarName = sidecarExecutableName();
const calculixSourceDirectory = packagedCalculixSourceDirectory(__dirname);
const calculixExecutable = path.join(calculixSourceDirectory, calculixExecutableName());
const contract = releaseContract();
const releaseNotes = readReleaseNotes({
  changelogPath: path.resolve(__dirname, '..', '..', 'CHANGELOG.md'),
  version: packageMetadata.version,
});
const hasSigningKeychain = Boolean(process.env.CSC_KEYCHAIN);
const iconPaths = {
  darwin: path.join(__dirname, 'build', 'macos', 'Fraia.icon'),
  darwinFallback: path.join(__dirname, 'build', 'icon.icns'),
  win32: path.join(__dirname, 'build', 'icon.ico'),
  linux: path.join(__dirname, 'build', 'icons'),
};
const tufRootPath = path.join(__dirname, 'build', 'update-trust', 'root.json');
const adaptiveMacosIconAvailable = contract.platform === 'darwin'
  ? selectAdaptiveMacosIconToolchain()
  : false;
if (process.env.FRAIA_REQUIRE_RELEASE_ICON === '1' && !fs.existsSync(iconPaths[contract.platform])) {
  throw new Error(`A maintained Fraia ${contract.platform} release icon is required at ${iconPaths[contract.platform]}.`);
}
if (
  process.env.FRAIA_REQUIRE_RELEASE_ICON === '1'
  && contract.platform === 'darwin'
  && !adaptiveMacosIconAvailable
) {
  throw new Error('Xcode 26 or newer with actool is required to package Fraia adaptive macOS icons.');
}
if (process.env.FRAIA_REQUIRE_PACKAGED_CALCULIX === '1') {
  validateRuntimeDirectory(calculixSourceDirectory, platformArch);
}
if (process.env.FRAIA_REQUIRE_TUF_ROOT === '1' && !fs.statSync(tufRootPath, { throwIfNoEntry: false })?.isFile()) {
  throw new Error(`A reviewed Fraia TUF trust root is required at ${tufRootPath}.`);
}

module.exports = {
  appId: contract.appId,
  productName: contract.productName,
  asar: true,
  compression: 'maximum',
  releaseInfo: {
    releaseName: `Fraia ${packageMetadata.version}`,
    releaseNotes: releaseNotes.body,
  },
  extraMetadata: {
    name: contract.packageName,
    productName: contract.productName,
    fraiaReleaseChannel: contract.channel,
    fraiaTufRepositoryUrl: `${contract.feedUrl}/tuf`,
    fraiaUpdateFeedUrl: contract.feedUrl,
    fraiaUpdateTargetName: metadataFileName(contract.platform, contract.arch),
  },
  directories: {
    output: process.env.FRAIA_RELEASE_OUTPUT_DIR || 'release',
    buildResources: 'build',
  },
  files: [
    'dist/**/*',
    'main.js',
    'preload.js',
    'ai-runtime.cjs',
    'application-metadata.cjs',
    'binary-architecture.cjs',
    'package-boundary.cjs',
    'tuf-update-feed.cjs',
    'update-manager.cjs',
    'scripts/perf-budgets.cjs',
    'package.json',
  ],
  extraResources: [
    {
      from: path.join('.package', 'sidecar', platformArch, sidecarName),
      to: path.join('sidecar', platformArch, sidecarName),
    },
    ...(fs.existsSync(calculixExecutable) ? [{
      from: calculixSourceDirectory,
      to: path.join('runtimes', 'calculix', platformArch),
    }] : []),
    ...(fs.existsSync(tufRootPath) ? [{
      from: tufRootPath,
      to: path.join('update-trust', 'root.json'),
    }] : []),
  ],
  mac: {
    category: 'public.app-category.productivity',
    minimumSystemVersion: '15.0',
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.plist',
    identity: hasSigningKeychain ? undefined : null,
    icon: adaptiveMacosIconAvailable && fs.existsSync(iconPaths.darwin)
      ? iconPaths.darwin
      : (fs.existsSync(iconPaths.darwinFallback) ? iconPaths.darwinFallback : undefined),
    target: [
      { target: 'dmg', arch: [contract.arch] },
      { target: 'zip', arch: [contract.arch] },
    ],
    artifactName: `${contract.artifactPrefix}-macOS-\${arch}.\${ext}`,
  },
  afterSign: process.env.FRAIA_REQUIRE_NOTARIZATION === '1'
    ? 'scripts/notarize-macos.cjs'
    : undefined,
  dmg: { sign: hasSigningKeychain },
  win: {
    icon: fs.existsSync(iconPaths.win32) ? iconPaths.win32 : undefined,
    target: [{ target: 'nsis', arch: [contract.arch] }],
    artifactName: `${contract.artifactPrefix}-Windows-\${arch}-Setup.\${ext}`,
  },
  nsis: {
    oneClick: false,
    perMachine: false,
    allowElevation: true,
    allowToChangeInstallationDirectory: true,
    deleteAppDataOnUninstall: false,
  },
  linux: {
    icon: fs.existsSync(iconPaths.linux) ? iconPaths.linux : undefined,
    category: 'Engineering',
    executableName: contract.packageName,
    maintainer: 'Alex Potenza <apotenza92@users.noreply.github.com>',
    synopsis: 'Structural engineering design and analysis workbench',
    description: 'Rust-backed structural engineering design and analysis workbench.',
    target: [
      { target: 'AppImage', arch: [contract.arch] },
      { target: 'deb', arch: [contract.arch] },
      { target: 'rpm', arch: [contract.arch] },
    ],
    artifactName: `${contract.artifactPrefix}-Linux-${contract.arch}.\${ext}`,
  },
  publish: [{
    provider: 'generic',
    url: contract.feedUrl,
    channel: 'latest',
  }],
};

function selectAdaptiveMacosIconToolchain() {
  if (!fs.existsSync(iconPaths.darwin)) return false;

  const configuredDeveloperDirectory = process.env.DEVELOPER_DIR?.trim();
  if (configuredDeveloperDirectory) {
    return adaptiveMacosActoolVersion(configuredDeveloperDirectory) !== null;
  }

  const candidates = new Set();
  try {
    for (const entry of fs.readdirSync('/Applications')) {
      if (/^Xcode.*\.app$/.test(entry)) {
        candidates.add(path.join('/Applications', entry, 'Contents', 'Developer'));
      }
    }
  } catch {
    // A nonstandard macOS host can still supply DEVELOPER_DIR explicitly.
  }

  let selected = null;
  for (const developerDirectory of candidates) {
    const version = adaptiveMacosActoolVersion(developerDirectory);
    if (version) {
      if (!selected || compareVersions(version, selected.version) > 0) {
        selected = { developerDirectory, version };
      }
    }
  }

  if (!selected) return false;
  process.env.DEVELOPER_DIR = selected.developerDirectory;
  return true;
}

function adaptiveMacosActoolVersion(developerDirectory) {
  try {
    const output = execFileSync('/usr/bin/xcrun', ['actool', '--version'], {
      encoding: 'utf8',
      env: { ...process.env, DEVELOPER_DIR: developerDirectory },
      stdio: ['ignore', 'pipe', 'ignore'],
    });
    const version = output.match(
      /<key>short-bundle-version<\/key>\s*<string>(\d+(?:\.\d+)*)<\/string>/,
    )?.[1];
    return version && Number.parseInt(version, 10) >= 26 ? version : null;
  } catch {
    return null;
  }
}

function compareVersions(left, right) {
  const leftParts = left.split('.').map(Number);
  const rightParts = right.split('.').map(Number);
  const length = Math.max(leftParts.length, rightParts.length);
  for (let index = 0; index < length; index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return 0;
}
