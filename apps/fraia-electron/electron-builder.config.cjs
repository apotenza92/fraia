const fs = require('node:fs');
const path = require('node:path');
const {
  calculixExecutableName,
  nativePlatformArch,
  packagedCalculixSourceDirectory,
  sidecarExecutableName,
} = require('./package-boundary.cjs');
const { validateRuntimeDirectory } = require('./calculix-runtime-manifest.cjs');
const { releaseContract } = require('./release-contract.cjs');

const platformArch = nativePlatformArch();
const sidecarName = sidecarExecutableName();
const calculixSourceDirectory = packagedCalculixSourceDirectory(__dirname);
const calculixExecutable = path.join(calculixSourceDirectory, calculixExecutableName());
const contract = releaseContract();
const hasSigningKeychain = Boolean(process.env.CSC_KEYCHAIN);
const iconPaths = {
  darwin: path.join(__dirname, 'build', 'icon.icns'),
  win32: path.join(__dirname, 'build', 'icon.ico'),
  linux: path.join(__dirname, 'build', 'icons'),
};
if (process.env.FRAIA_REQUIRE_RELEASE_ICON === '1' && !fs.existsSync(iconPaths[contract.platform])) {
  throw new Error(`A maintained Fraia ${contract.platform} release icon is required at ${iconPaths[contract.platform]}.`);
}
if (process.env.FRAIA_REQUIRE_PACKAGED_CALCULIX === '1') {
  validateRuntimeDirectory(calculixSourceDirectory, platformArch);
}

module.exports = {
  appId: contract.appId,
  productName: contract.productName,
  asar: true,
  compression: 'maximum',
  extraMetadata: {
    name: contract.packageName,
    productName: contract.productName,
    fraiaReleaseChannel: contract.channel,
    fraiaUpdateFeedUrl: contract.feedUrl,
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
  ],
  mac: {
    category: 'public.app-category.productivity',
    hardenedRuntime: true,
    gatekeeperAssess: false,
    entitlements: 'build/entitlements.mac.plist',
    entitlementsInherit: 'build/entitlements.mac.plist',
    identity: hasSigningKeychain ? undefined : null,
    icon: fs.existsSync(iconPaths.darwin) ? iconPaths.darwin : undefined,
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
    artifactName: `${contract.artifactPrefix}-Linux-\${arch}.\${ext}`,
  },
  publish: [{
    provider: 'generic',
    url: contract.feedUrl,
    channel: 'latest',
  }],
};
