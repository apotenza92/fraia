const fs = require('node:fs');
const path = require('node:path');

const CHANNELS = new Set(['stable', 'beta']);
const VERSION_PATTERN = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-beta\.(0|[1-9]\d*))?$/;
const FEED_METADATA = Object.freeze([
  ['darwin', 'arm64', 'latest-mac.yml'],
  ['darwin', 'x64', 'latest-mac.yml'],
  ['linux', 'arm64', 'latest-linux-arm64.yml'],
  ['linux', 'x64', 'latest-linux.yml'],
  ['win32', 'arm64', 'latest.yml'],
  ['win32', 'x64', 'latest.yml'],
]);

function parseVersion(value) {
  const match = VERSION_PATTERN.exec(value);
  if (!match) throw new Error(`Invalid Fraia semantic version: ${value}`);
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    prerelease: match[4] === undefined ? null : Number(match[4]),
    value,
  };
}

function compareVersions(leftValue, rightValue) {
  const left = parseVersion(leftValue);
  const right = parseVersion(rightValue);
  for (const key of ['major', 'minor', 'patch']) {
    if (left[key] !== right[key]) return left[key] < right[key] ? -1 : 1;
  }
  if (left.prerelease === null && right.prerelease === null) return 0;
  if (left.prerelease === null) return 1;
  if (right.prerelease === null) return -1;
  return Math.sign(left.prerelease - right.prerelease);
}

function requireChannel(channel) {
  if (!CHANNELS.has(channel)) throw new Error(`Fraia release channel must be stable or beta; received ${channel}.`);
}

function readMetadataVersion(filePath) {
  const contents = fs.readFileSync(filePath, 'utf8');
  const version = contents.match(/^version:\s*['"]?([^'"\s]+)['"]?\s*$/m)?.[1];
  if (!version) throw new Error(`Updater metadata does not declare a version: ${filePath}`);
  parseVersion(version);
  return version;
}

function readPublishedChannelVersion(feedRoot, channel) {
  requireChannel(channel);
  const channelRoot = path.join(feedRoot, channel);
  if (!fs.existsSync(channelRoot)) return null;
  const versions = FEED_METADATA.map(([platform, arch, name]) => {
    const filePath = path.join(channelRoot, platform, arch, name);
    if (!fs.existsSync(filePath)) throw new Error(`Published ${channel} feed is incomplete: ${filePath}`);
    return readMetadataVersion(filePath);
  });
  const unique = [...new Set(versions)];
  if (unique.length !== 1) {
    throw new Error(`Published ${channel} feed versions disagree: ${unique.join(', ')}`);
  }
  return unique[0];
}

function releasePolicy({
  tagChannel,
  candidateVersion,
  currentStableVersion = null,
  currentBetaVersion = null,
}) {
  requireChannel(tagChannel);
  const candidate = parseVersion(candidateVersion);
  if (tagChannel === 'stable' && candidate.prerelease !== null) {
    throw new Error(`Stable tags require a final version; received ${candidateVersion}.`);
  }
  if (tagChannel === 'beta' && candidate.prerelease === null) {
    throw new Error(`Beta tags require a beta pre-release; received ${candidateVersion}.`);
  }

  if (tagChannel === 'stable') {
    if (currentStableVersion && compareVersions(candidateVersion, currentStableVersion) <= 0) {
      throw new Error(`Stable ${candidateVersion} must be newer than the published stable ${currentStableVersion}.`);
    }
    const channels = ['stable'];
    if (!currentBetaVersion || compareVersions(candidateVersion, currentBetaVersion) > 0) channels.push('beta');
    return Object.freeze({
      channels: Object.freeze(channels),
      previousBetaVersion: currentBetaVersion,
      previousStableVersion: currentStableVersion,
      promotesBeta: channels.includes('beta'),
    });
  }

  if (currentBetaVersion && compareVersions(candidateVersion, currentBetaVersion) <= 0) {
    throw new Error(`Beta ${candidateVersion} must be newer than the published beta ${currentBetaVersion}.`);
  }
  return Object.freeze({
    channels: Object.freeze(['beta']),
    previousBetaVersion: currentBetaVersion,
    previousStableVersion: currentStableVersion,
    promotesBeta: false,
  });
}

module.exports = {
  CHANNELS,
  FEED_METADATA,
  VERSION_PATTERN,
  compareVersions,
  parseVersion,
  readPublishedChannelVersion,
  releasePolicy,
};
