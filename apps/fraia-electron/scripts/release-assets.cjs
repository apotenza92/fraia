const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { CALCULIX_SOURCE_ASSET_NAME } = require('../calculix-source-contract.cjs');

function requireChannel(channel) {
  if (channel !== 'stable') throw new Error(`Fraia publishes stable releases only; received channel: ${channel}`);
}

function expectedReleaseAssetNames(channel) {
  requireChannel(channel);
  const prefix = 'Fraia';
  const names = ['SHA256SUMS', CALCULIX_SOURCE_ASSET_NAME];
  for (const arch of ['arm64', 'x64']) {
    const mac = `${prefix}-macOS-${arch}`;
    names.push(
      `${mac}.dmg`, `${mac}.dmg.blockmap`, `${mac}.dmg.sha256`,
      `${mac}.zip`, `${mac}.zip.blockmap`, `${mac}.zip.sha256`,
      `notarization-stable-${arch}.json`,
      `update-stable-darwin-${arch}.yml`,
      `update-beta-darwin-${arch}.yml`,
    );
    const linux = `${prefix}-Linux-${arch}`;
    names.push(`${linux}.AppImage`, `${linux}.AppImage.blockmap`, `${linux}.deb`, `${linux}.rpm`);
  }
  const windows = `${prefix}-Windows-x64-Setup.exe`;
  names.push(windows, `${windows}.blockmap`);
  return names.sort();
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

function assembleReleaseAssets(channel, inputDirectories, outputDirectory) {
  requireChannel(channel);
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
  const expectedWithoutChecksums = expectedReleaseAssetNames(channel).filter((name) => name !== 'SHA256SUMS');
  compareExact(expectedWithoutChecksums, actualFileNames(outputDirectory), 'Release candidate assets');
  writeChecksums(outputDirectory);
  compareExact(expectedReleaseAssetNames(channel), actualFileNames(outputDirectory), 'Public release assets');
  return expectedReleaseAssetNames(channel);
}

function main(argv = process.argv.slice(2)) {
  const channelIndex = argv.indexOf('--channel');
  const outputIndex = argv.indexOf('--output');
  const inputIndices = argv.flatMap((value, index) => value === '--input' ? [index] : []);
  if (channelIndex < 0 || outputIndex < 0 || inputIndices.length === 0) {
    throw new Error('Usage: release-assets.cjs --channel stable --output DIR --input DIR [--input DIR]');
  }
  const names = assembleReleaseAssets(
    argv[channelIndex + 1],
    inputIndices.map((index) => path.resolve(argv[index + 1])),
    path.resolve(argv[outputIndex + 1]),
  );
  process.stdout.write(`${names.join('\n')}\n`);
}

if (require.main === module) main();

module.exports = { assembleReleaseAssets, expectedReleaseAssetNames };
