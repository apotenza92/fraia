#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { readFile, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';

const require = createRequire(import.meta.url);
const { buildBlockMap } = require('app-builder-lib/out/targets/blockmap/blockmap');

function artifactName(value) {
  if (typeof value !== 'string' || !value) throw new Error('Every macOS update file must have a URL.');
  const candidate = /^https?:\/\//.test(value) ? new URL(value).pathname : value;
  const decoded = decodeURIComponent(path.posix.basename(candidate));
  if (!decoded || decoded !== path.posix.basename(decoded) || decoded.includes('\\')) {
    throw new Error(`Unsafe macOS update artifact name: ${value}`);
  }
  return decoded;
}

export async function finalizeMacosUpdateArtifacts({ metadataPath, artifactDir, arch }) {
  if (!['arm64', 'x64'].includes(arch)) throw new Error(`Unsupported macOS release architecture: ${arch}`);
  const metadata = YAML.parse(await readFile(metadataPath, 'utf8'));
  if (!metadata || typeof metadata !== 'object' || !Array.isArray(metadata.files)) {
    throw new Error('macOS update metadata must contain a files array.');
  }

  const expectedNames = new Set([
    `Fraia-macOS-${arch}.dmg`,
    `Fraia-macOS-${arch}.zip`,
  ]);
  const finalizedFiles = [];
  for (const file of metadata.files) {
    const name = artifactName(file.url);
    if (!expectedNames.delete(name)) throw new Error(`Unexpected macOS update artifact: ${name}`);
    const target = path.resolve(artifactDir, name);
    const relative = path.relative(path.resolve(artifactDir), target);
    if (relative.startsWith('..') || path.isAbsolute(relative)) {
      throw new Error(`macOS update artifact escapes its directory: ${name}`);
    }
    const details = await stat(target);
    if (!details.isFile()) throw new Error(`macOS update artifact is not a file: ${name}`);
    const updateInfo = await buildBlockMap(target, 'gzip', `${target}.blockmap`);
    finalizedFiles.push({
      ...file,
      sha512: updateInfo.sha512,
      size: updateInfo.size,
    });
  }
  if (expectedNames.size) {
    throw new Error(`macOS update metadata is missing: ${[...expectedNames].sort().join(', ')}`);
  }

  const finalized = { ...metadata, files: finalizedFiles };
  if (metadata.path) {
    const legacyName = artifactName(metadata.path);
    const legacyFile = finalizedFiles.find((file) => artifactName(file.url) === legacyName);
    if (!legacyFile) throw new Error('Legacy macOS update path does not match a files entry.');
    finalized.sha512 = legacyFile.sha512;
  }
  await writeFile(metadataPath, `${YAML.stringify(finalized, { lineWidth: 0 }).trimEnd()}\n`);

  const metadataHash = createHash('sha256').update(await readFile(metadataPath)).digest('hex');
  return { metadataHash, files: finalizedFiles.map((file) => artifactName(file.url)).sort() };
}

function argumentsFrom(argv) {
  const values = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith('--') || value === undefined) throw new Error('Arguments must use --name value pairs.');
    values[key.slice(2)] = value;
  }
  return values;
}

async function main(argv = process.argv.slice(2)) {
  const args = argumentsFrom(argv);
  for (const name of ['metadata', 'artifact-dir', 'arch']) {
    if (!args[name]) throw new Error(`Missing --${name}.`);
  }
  const result = await finalizeMacosUpdateArtifacts({
    metadataPath: path.resolve(args.metadata),
    artifactDir: path.resolve(args['artifact-dir']),
    arch: args.arch,
  });
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  });
}
