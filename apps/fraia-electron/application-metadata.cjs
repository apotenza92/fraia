const path = require('node:path');

const { CHANNEL_IDENTITIES } = require('./release-contract.cjs');
const CHANNELS = new Set(Object.keys(CHANNEL_IDENTITIES));

function resolveApplicationMetadata(packageMetadata = {}) {
  const channel = packageMetadata.fraiaReleaseChannel || 'stable';
  if (!CHANNELS.has(channel)) {
    throw new Error(`Fraia release channel must be stable or beta; received ${channel}.`);
  }

  const identity = CHANNEL_IDENTITIES[channel];
  const { productName, userDataDirectoryName } = identity;
  if (packageMetadata.productName && packageMetadata.productName !== productName) {
    throw new Error(
      `Fraia ${channel} metadata requires productName ${productName}; received ${packageMetadata.productName}.`,
    );
  }

  return Object.freeze({
    channel,
    productName,
    userDataDirectoryName,
  });
}

function resolveUserDataDirectory({
  appDataPath,
  configuredPath,
  metadata,
}) {
  if (configuredPath?.trim()) return path.resolve(configuredPath.trim());
  if (!path.isAbsolute(appDataPath)) throw new Error('appDataPath must be absolute.');
  return path.join(appDataPath, metadata.userDataDirectoryName);
}

module.exports = {
  resolveApplicationMetadata,
  resolveUserDataDirectory,
};
