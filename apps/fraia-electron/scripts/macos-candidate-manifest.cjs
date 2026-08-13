#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function files(root) {
  const result = [];
  function walk(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(absolute);
      else if (entry.isFile()) result.push(path.relative(root, absolute).split(path.sep).join('/'));
    }
  }
  walk(root);
  return result.sort();
}

function create({ root, output, repository, commit, version }) {
  const manifest = {
    schemaVersion: 1,
    repository,
    commit,
    version,
    channels: ['stable', 'beta'],
    targets: ['darwin-arm64', 'darwin-x64'],
    files: Object.fromEntries(files(root).map((name) => [name, sha256(path.join(root, name))])),
  };
  fs.writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

function verify({ root, manifestPath, repository, commit, version }) {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  if (manifest.schemaVersion !== 1) throw new Error('Unsupported macOS candidate manifest schema.');
  if (manifest.repository !== repository) throw new Error('macOS candidate repository does not match.');
  if (manifest.commit !== commit) throw new Error('macOS candidate commit does not match the release tag.');
  if (manifest.version !== version) throw new Error('macOS candidate version does not match the release tag.');
  if (JSON.stringify(manifest.channels) !== JSON.stringify(['stable', 'beta'])) throw new Error('macOS candidate channels are incomplete.');
  if (JSON.stringify(manifest.targets) !== JSON.stringify(['darwin-arm64', 'darwin-x64'])) throw new Error('macOS candidate targets are incomplete.');
  const actual = files(root).filter((name) => name !== path.relative(root, manifestPath).split(path.sep).join('/'));
  const expected = Object.keys(manifest.files).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error('macOS candidate file set does not match its sealed manifest.');
  for (const name of expected) {
    if (sha256(path.join(root, name)) !== manifest.files[name]) throw new Error(`macOS candidate digest does not match: ${name}`);
  }
  return manifest;
}

function options(argv) {
  const values = {};
  for (let index = 0; index < argv.length; index += 2) values[argv[index]] = argv[index + 1];
  return values;
}

if (require.main === module) {
  const [command, ...argv] = process.argv.slice(2);
  const args = options(argv);
  const common = {
    root: path.resolve(args['--root']),
    repository: args['--repository'],
    commit: args['--commit'],
    version: args['--version'],
  };
  if (command === 'create') create({ ...common, output: path.resolve(args['--output']) });
  else if (command === 'verify') verify({ ...common, manifestPath: path.resolve(args['--manifest']) });
  else throw new Error('Usage: macos-candidate-manifest.cjs create|verify --root DIR --repository OWNER/REPO --commit SHA --version VERSION --output|--manifest FILE');
}

module.exports = { create, verify };
