const path = require('node:path');

const CHANNELS = new Set(['stable']);

function resolveApplicationMetadata(packageMetadata = {}) {
  const channel = packageMetadata.fraiaReleaseChannel || 'stable';
  if (!CHANNELS.has(channel)) {
    throw new Error(`Fraia release channel must be stable; received ${channel}.`);
  }

  const productName = 'Fraia';
  if (packageMetadata.productName && packageMetadata.productName !== productName) {
    throw new Error(
      `Fraia ${channel} metadata requires productName ${productName}; received ${packageMetadata.productName}.`,
    );
  }

  return Object.freeze({
    channel,
    productName,
    userDataDirectoryName: productName,
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
