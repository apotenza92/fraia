const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { CALCULIX_SOURCE_ASSET_NAME } = require('../calculix-source-contract.cjs');

const CHANNELS = new Set(['stable', 'beta']);

function requireChannel(channel) {
  if (!CHANNELS.has(channel)) {
    throw new Error(`Fraia release channel must be stable or beta; received channel: ${channel}`);
  }
}

function normalizeChannels(value) {
  const candidates = Array.isArray(value) ? value : String(value).split(',');
  const channels = [...new Set(candidates.map((channel) => channel.trim()).filter(Boolean))];
  if (channels.length === 0) throw new Error('Fraia release must package at least one channel.');
  channels.forEach(requireChannel);
  return channels;
}

function expectedChannelAssetNames(channel) {
  requireChannel(channel);
  const prefix = channel === 'beta' ? 'Fraia-Beta' : 'Fraia';
  const names = [];
  for (const arch of ['arm64', 'x64']) {
    const mac = `${prefix}-macOS-${arch}`;
    names.push(
      `${mac}.dmg`, `${mac}.dmg.blockmap`, `${mac}.dmg.sha256`,
      `${mac}.zip`, `${mac}.zip.blockmap`, `${mac}.zip.sha256`,
      `notarization-${channel}-${arch}.json`,
      `update-${channel}-darwin-${arch}.yml`,
    );
    const linux = `${prefix}-Linux-${arch}`;
    names.push(
      `${linux}.AppImage`, `${linux}.deb`, `${linux}.rpm`,
      `update-${channel}-linux-${arch}.yml`,
    );
  }
  for (const arch of ['arm64', 'x64']) {
    const windows = `${prefix}-Windows-${arch}-Setup.exe`;
    names.push(
      windows,
      `${windows}.blockmap`,
      `update-${channel}-win32-${arch}.yml`,
    );
  }
  return names.sort();
}

function expectedReleaseAssetNames(channelsValue) {
  const channels = normalizeChannels(channelsValue);
  return [
    'SHA256SUMS',
    CALCULIX_SOURCE_ASSET_NAME,
    ...channels.flatMap(expectedChannelAssetNames),
  ].sort();
}

function actualFileNames(directory) {
  return fs.readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
}

function compareExact(expected, actual, label) {
  const missing = expected.filter((name) => !actual.includes(name));
  const unexpected = actual.filter((name) => !expected.includes(name));
  if (missing.length || unexpected.length) {
    throw new Error(`${label} mismatch. Missing: ${missing.join(', ') || 'none'}. Unexpected: ${unexpected.join(', ') || 'none'}.`);
  }
}

function writeChecksums(directory) {
  const files = actualFileNames(directory).filter((name) => name !== 'SHA256SUMS');
  const lines = files.map((name) => {
    const hash = crypto.createHash('sha256').update(fs.readFileSync(path.join(directory, name))).digest('hex');
    return `${hash}  ${name}`;
  });
  fs.writeFileSync(path.join(directory, 'SHA256SUMS'), `${lines.join('\n')}\n`, { mode: 0o644 });
}

function assembleReleaseAssets(channelsValue, inputDirectories, outputDirectory) {
  const channels = normalizeChannels(channelsValue);
  fs.rmSync(outputDirectory, { recursive: true, force: true });
  fs.mkdirSync(outputDirectory, { recursive: true });
  const seen = new Set();
  for (const directory of inputDirectories) {
    for (const name of actualFileNames(directory)) {
      if (name === 'SHA256SUMS') continue;
      if (seen.has(name)) throw new Error(`Release candidate collision: ${name}`);
      seen.add(name);
      fs.copyFileSync(path.join(directory, name), path.join(outputDirectory, name), fs.constants.COPYFILE_EXCL);
    }
  }
  const expectedWithoutChecksums = expectedReleaseAssetNames(channels).filter((name) => name !== 'SHA256SUMS');
  compareExact(expectedWithoutChecksums, actualFileNames(outputDirectory), 'Release candidate assets');
  writeChecksums(outputDirectory);
  compareExact(expectedReleaseAssetNames(channels), actualFileNames(outputDirectory), 'Public release assets');
  return expectedReleaseAssetNames(channels);
}

function main(argv = process.argv.slice(2)) {
  const channelIndex = argv.indexOf('--channel');
  const channelsIndex = argv.indexOf('--channels');
  const outputIndex = argv.indexOf('--output');
  const inputIndices = argv.flatMap((value, index) => value === '--input' ? [index] : []);
  if ((channelIndex < 0 && channelsIndex < 0) || outputIndex < 0 || inputIndices.length === 0) {
    throw new Error('Usage: release-assets.cjs --channels stable[,beta] --output DIR --input DIR [--input DIR]');
  }
  const names = assembleReleaseAssets(
    argv[(channelsIndex >= 0 ? channelsIndex : channelIndex) + 1],
    inputIndices.map((index) => path.resolve(argv[index + 1])),
    path.resolve(argv[outputIndex + 1]),
  );
  process.stdout.write(`${names.join('\n')}\n`);
}

if (require.main === module) main();

module.exports = {
  assembleReleaseAssets,
  expectedChannelAssetNames,
  expectedReleaseAssetNames,
  normalizeChannels,
};
