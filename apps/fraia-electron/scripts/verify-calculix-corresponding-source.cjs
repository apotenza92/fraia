#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const {
  CALCULIX_SOURCE_ASSET_NAME,
  correspondingSourceUrl,
} = require('../calculix-source-contract.cjs');
const { SUPPORTED_TARGETS } = require('../package-boundary.cjs');

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function value(argv, name) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : null;
}

function verifyCorrespondingSource({
  bundlePath,
  repository,
  runtimeRoot,
  tag,
}) {
  if (path.basename(bundlePath) !== CALCULIX_SOURCE_ASSET_NAME) {
    throw new Error(`Corresponding-source bundle must be named ${CALCULIX_SOURCE_ASSET_NAME}.`);
  }
  if (!fs.statSync(bundlePath).isFile()) throw new Error(`Missing source bundle: ${bundlePath}`);
  const expectedUrl = correspondingSourceUrl(repository, tag);
  const expectedSha256 = sha256(bundlePath);
  for (const target of SUPPORTED_TARGETS) {
    const manifestPath = path.join(runtimeRoot, target, 'runtime-manifest.json');
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    if (manifest.redistribution?.sourceUrl !== expectedUrl) {
      throw new Error(`${manifestPath} redistribution.sourceUrl does not match ${expectedUrl}.`);
    }
    if (manifest.redistribution?.sourceSha256 !== expectedSha256) {
      throw new Error(`${manifestPath} redistribution.sourceSha256 does not match ${expectedSha256}.`);
    }
  }
  return { sha256: expectedSha256, sourceUrl: expectedUrl };
}

function main(argv = process.argv.slice(2)) {
  const bundlePath = value(argv, '--bundle');
  const repository = value(argv, '--repository');
  const runtimeRoot = value(argv, '--runtime-root');
  const tag = value(argv, '--tag');
  if (!bundlePath || !repository || !runtimeRoot || !tag) {
    throw new Error('Usage: verify-calculix-corresponding-source.cjs --bundle FILE --repository OWNER/REPO --runtime-root DIR --tag vX.Y.Z');
  }
  const result = verifyCorrespondingSource({
    bundlePath: path.resolve(bundlePath),
    repository,
    runtimeRoot: path.resolve(runtimeRoot),
    tag,
  });
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

if (require.main === module) main();

module.exports = { verifyCorrespondingSource };
