#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { pipeline } = require('node:stream/promises');
const {
  BUILD_RECIPES,
  CALCULIX_SOURCE_ASSET_NAME,
  SOURCE_DATE_EPOCH,
  SOURCE_INPUTS,
} = require('../calculix-source-contract.cjs');

const ROOT_NAME = 'Fraia-CalculiX-Corresponding-Source';
const SHA256_PATTERN = /^[a-f0-9]{64}$/;

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function writeString(header, offset, length, value) {
  const bytes = Buffer.from(value, 'utf8');
  if (bytes.length > length) throw new Error(`Tar field is too long: ${value}`);
  bytes.copy(header, offset);
}

function writeOctal(header, offset, length, value) {
  const encoded = value.toString(8).padStart(length - 1, '0');
  if (encoded.length >= length) throw new Error(`Tar number does not fit: ${value}`);
  writeString(header, offset, length, `${encoded}\0`);
}

function tarHeader(name, size, type = '0', mode = 0o644) {
  const header = Buffer.alloc(512);
  writeString(header, 0, 100, name);
  writeOctal(header, 100, 8, mode);
  writeOctal(header, 108, 8, 0);
  writeOctal(header, 116, 8, 0);
  writeOctal(header, 124, 12, size);
  writeOctal(header, 136, 12, SOURCE_DATE_EPOCH);
  header.fill(0x20, 148, 156);
  writeString(header, 156, 1, type);
  writeString(header, 257, 6, 'ustar\0');
  writeString(header, 263, 2, '00');
  const checksum = header.reduce((sum, byte) => sum + byte, 0);
  writeString(header, 148, 8, `${checksum.toString(8).padStart(6, '0')}\0 `);
  return header;
}

function writeDeterministicTar(outputPath, entries) {
  if (fs.existsSync(outputPath)) throw new Error(`Output already exists: ${outputPath}`);
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  const output = fs.openSync(outputPath, 'wx', 0o644);
  try {
    const directories = new Set();
    for (const entry of entries) {
      const parts = entry.name.split('/');
      for (let index = 1; index < parts.length; index += 1) {
        directories.add(`${parts.slice(0, index).join('/')}/`);
      }
    }
    for (const directory of [...directories].sort()) {
      fs.writeSync(output, tarHeader(directory, 0, '5', 0o755));
    }
    for (const entry of [...entries].sort((left, right) => (
      left.name < right.name ? -1 : left.name > right.name ? 1 : 0
    ))) {
      const size = entry.data ? entry.data.length : fs.statSync(entry.filePath).size;
      fs.writeSync(output, tarHeader(entry.name, size));
      if (entry.data) {
        fs.writeSync(output, entry.data);
      } else {
        const input = fs.openSync(entry.filePath, 'r');
        try {
          const buffer = Buffer.allocUnsafe(1024 * 1024);
          let count;
          while ((count = fs.readSync(input, buffer, 0, buffer.length, null)) > 0) {
            fs.writeSync(output, buffer, 0, count);
          }
        } finally {
          fs.closeSync(input);
        }
      }
      const padding = (512 - (size % 512)) % 512;
      if (padding) fs.writeSync(output, Buffer.alloc(padding));
    }
    fs.writeSync(output, Buffer.alloc(1024));
  } finally {
    fs.closeSync(output);
  }
}

async function download(url, destination) {
  const temporary = `${destination}.partial`;
  let lastError;
  for (let attempt = 1; attempt <= 4; attempt += 1) {
    try {
      if (fs.existsSync(temporary)) fs.rmSync(temporary);
      const response = await fetch(url, {
        redirect: 'follow',
        signal: AbortSignal.timeout(120_000),
      });
      if (!response.ok || !response.body) {
        throw new Error(`Download failed (${response.status}) for ${url}`);
      }
      await pipeline(response.body, fs.createWriteStream(temporary, { flags: 'wx', mode: 0o600 }));
      fs.renameSync(temporary, destination);
      return;
    } catch (error) {
      lastError = error;
      if (fs.existsSync(temporary)) fs.rmSync(temporary);
      if (attempt === 4) break;
      process.stderr.write(`Retrying pinned source download after attempt ${attempt}/4 failed: ${error.message}\n`);
      await new Promise((resolve) => setTimeout(resolve, attempt * 5_000));
    }
  }
  throw new Error(`Pinned source download failed after 4 attempts for ${url}: ${lastError?.message || lastError}`);
}

async function requireSourceInputs(cacheDirectory, offline = false) {
  fs.mkdirSync(cacheDirectory, { recursive: true });
  const resolved = [];
  for (const source of SOURCE_INPUTS) {
    if (!SHA256_PATTERN.test(source.sha256)) throw new Error(`Invalid source hash: ${source.fileName}`);
    const filePath = path.join(cacheDirectory, source.fileName);
    if (!fs.existsSync(filePath)) {
      if (offline) throw new Error(`Offline source input is missing: ${filePath}`);
      await download(source.url, filePath);
    }
    const actual = sha256(filePath);
    if (actual !== source.sha256) {
      throw new Error(`Source SHA-256 mismatch for ${source.fileName}: expected ${source.sha256}, received ${actual}.`);
    }
    resolved.push({ ...source, filePath });
  }
  return resolved;
}

function recipeEntries(electronRoot) {
  return BUILD_RECIPES.map((recipe) => {
    const filePath = path.join(electronRoot, recipe.path);
    if (!fs.statSync(filePath).isFile()) throw new Error(`Build recipe is missing: ${filePath}`);
    return {
      ...recipe,
      filePath,
      sha256: sha256(filePath),
      archivePath: `${ROOT_NAME}/recipes/${path.basename(filePath)}`,
    };
  });
}

function sourceManifest(sources, recipes) {
  return {
    schemaVersion: 1,
    assetName: CALCULIX_SOURCE_ASSET_NAME,
    sourceDateEpoch: SOURCE_DATE_EPOCH,
    targets: [
      'darwin-arm64',
      'darwin-x64',
      'linux-arm64',
      'linux-x64',
      'win32-arm64',
      'win32-x64',
    ],
    sources: sources.map(({ fileName, sha256: digest, url, usedBy }) => ({
      path: `sources/${fileName}`,
      sha256: digest,
      url,
      usedBy,
    })),
    buildRecipes: recipes.map(({ platform, sha256: digest, archivePath }) => ({
      platform,
      path: archivePath.slice(`${ROOT_NAME}/`.length),
      sha256: digest,
    })),
  };
}

async function assembleCorrespondingSource({
  cacheDirectory,
  electronRoot = path.resolve(__dirname, '..'),
  offline = false,
  outputPath,
}) {
  if (path.basename(outputPath) !== CALCULIX_SOURCE_ASSET_NAME) {
    throw new Error(`Corresponding-source output must be named ${CALCULIX_SOURCE_ASSET_NAME}.`);
  }
  const sources = await requireSourceInputs(cacheDirectory, offline);
  const recipes = recipeEntries(electronRoot);
  const manifest = sourceManifest(sources, recipes);
  const readme = [
    'Fraia CalculiX corresponding source',
    '',
    'This deterministic archive contains every SHA-256-pinned upstream source',
    'archive and every reviewed build recipe used for Fraia CalculiX runtimes',
    'on darwin-arm64, darwin-x64, linux-arm64, linux-x64, win32-arm64,',
    'and win32-x64.',
    '',
    'SOURCE_MANIFEST.json records the public origin and digest of each input.',
    'The runtime directories contain the resulting notices, licences, native',
    'dependency closure, and target-specific build and solver evidence.',
    '',
  ].join('\n');
  const entries = [
    {
      name: `${ROOT_NAME}/README.txt`,
      data: Buffer.from(readme, 'utf8'),
    },
    {
      name: `${ROOT_NAME}/SOURCE_MANIFEST.json`,
      data: Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`, 'utf8'),
    },
    {
      name: `${ROOT_NAME}/LICENSE-FRAIA.txt`,
      filePath: path.resolve(electronRoot, '..', '..', 'LICENSE'),
    },
    ...recipes.map((recipe) => ({
      name: recipe.archivePath,
      filePath: recipe.filePath,
    })),
    ...sources.map((source) => ({
      name: `${ROOT_NAME}/sources/${source.fileName}`,
      filePath: source.filePath,
    })),
  ];
  writeDeterministicTar(outputPath, entries);
  return {
    assetName: CALCULIX_SOURCE_ASSET_NAME,
    sha256: sha256(outputPath),
    sourceCount: sources.length,
  };
}

function parseArguments(argv) {
  const value = (name) => {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : null;
  };
  const cacheDirectory = value('--cache');
  const outputPath = value('--output');
  if (!cacheDirectory || !outputPath) {
    throw new Error(`Usage: ${path.basename(process.argv[1])} --cache DIR --output ${CALCULIX_SOURCE_ASSET_NAME} [--offline]`);
  }
  return {
    cacheDirectory: path.resolve(cacheDirectory),
    offline: argv.includes('--offline'),
    outputPath: path.resolve(outputPath),
  };
}

async function main() {
  const result = await assembleCorrespondingSource(parseArguments(process.argv.slice(2)));
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

if (require.main === module) {
  main().catch((error) => {
    process.stderr.write(`${error.stack || error.message}\n`);
    process.exitCode = 1;
  });
}

module.exports = {
  assembleCorrespondingSource,
  sourceManifest,
  tarHeader,
  writeDeterministicTar,
};
