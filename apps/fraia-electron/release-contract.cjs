const path = require('node:path');
const { SUPPORTED_TARGETS } = require('./package-boundary.cjs');

const CHANNELS = new Set(['stable', 'beta']);
const PLATFORMS = new Set(['darwin', 'win32', 'linux']);
const ARCHITECTURES = new Set(['arm64', 'x64']);

function requireChoice(label, value, choices) {
  if (!choices.has(value)) {
    throw new Error(`${label} must be one of ${[...choices].join(', ')}; received ${value}.`);
  }
}

function requireTarget(platform, arch) {
  const target = `${platform}-${arch}`;
  if (!SUPPORTED_TARGETS.has(target)) {
    throw new Error(`Fraia release does not support ${target}.`);
  }
}

function releaseContract({
  channel = process.env.FRAIA_RELEASE_CHANNEL || 'stable',
  platform = process.env.FRAIA_RELEASE_PLATFORM || process.platform,
  arch = process.env.FRAIA_RELEASE_ARCH || process.arch,
  outputDir = process.env.FRAIA_RELEASE_OUTPUT_DIR,
  feedBaseUrl = process.env.FRAIA_UPDATE_FEED_BASE_URL
    || 'https://raw.githubusercontent.com/apotenza92/fraia/updates',
} = {}) {
  requireChoice('channel', channel, CHANNELS);
  requireChoice('platform', platform, PLATFORMS);
  requireChoice('architecture', arch, ARCHITECTURES);
  requireTarget(platform, arch);

  const beta = channel === 'beta';
  const productName = beta ? 'Fraia Beta' : 'Fraia';
  const packageName = beta ? 'fraia-electron-beta' : 'fraia-electron';
  const artifactPrefix = beta ? 'Fraia-Beta' : 'Fraia';
  const normalizedFeedBaseUrl = feedBaseUrl.replace(/\/$/, '');

  return {
    appId: beta ? 'app.fraia.desktop.beta' : 'app.fraia.desktop',
    appName: `${productName}.app`,
    arch,
    artifactPrefix,
    channel,
    executableName: platform === 'linux' ? packageName : productName,
    feedUrl: `${normalizedFeedBaseUrl}/${channel}/${platform}/${arch}`,
    outputDir: outputDir
      ? path.resolve(outputDir)
      : path.resolve(__dirname, 'release', channel, platform, arch),
    packageName,
    platform,
    productName,
  };
}

function metadataFileName(platform, arch) {
  requireChoice('platform', platform, PLATFORMS);
  requireChoice('architecture', arch, ARCHITECTURES);
  requireTarget(platform, arch);
  if (platform === 'darwin') return 'latest-mac.yml';
  if (platform === 'win32') return 'latest.yml';
  return arch === 'arm64' ? 'latest-linux-arm64.yml' : 'latest-linux.yml';
}

module.exports = {
  ARCHITECTURES,
  CHANNELS,
  PLATFORMS,
  metadataFileName,
  releaseContract,
};
