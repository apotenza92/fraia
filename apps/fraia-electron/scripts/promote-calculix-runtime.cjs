#!/usr/bin/env node

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const {
  parseLinuxDependencies,
  parseMacDependencyEntries,
  parseWindowsDependencies,
  sha256,
  validateRuntimeDirectory,
} = require('../calculix-runtime-manifest.cjs');
const {
  CALCULIX_SOURCE_ASSET_NAME,
  correspondingSourceUrl,
} = require('../calculix-source-contract.cjs');
const {
  SUPPORTED_TARGETS,
  calculixExecutableName,
} = require('../package-boundary.cjs');

const SHA256_PATTERN = /^[a-f0-9]{64}$/;
const PLATFORM_RECIPE = Object.freeze({
  darwin: 'build-calculix-macos-runtime.sh',
  linux: 'build-calculix-linux-runtime.sh',
  win32: 'build-calculix-windows-runtime.ps1',
});
const LICENSE_IDENTIFIERS = Object.freeze({
  darwin: [
    'GPL-2.0-only',
    'LicenseRef-SPOOLES-Public-Domain',
    'BSD-3-Clause',
    'GPL-3.0-or-later WITH GCC-exception-3.1',
  ],
  linux: [
    'GPL-2.0-only',
    'LicenseRef-SPOOLES-Public-Domain',
    'BSD-3-Clause',
    'GPL-3.0-or-later WITH GCC-exception-3.1',
  ],
  win32: [
    'GPL-2.0-only',
    'LicenseRef-SPOOLES-Public-Domain',
    'BSD-3-Clause',
    'MIT',
    'GPL-3.0-or-later WITH GCC-exception-3.1',
  ],
});

function hashBuffer(buffer) {
  return crypto.createHash('sha256').update(buffer).digest('hex');
}

function verifyChecksumIndex(baseDirectory, indexPath) {
  const lines = fs.readFileSync(indexPath, 'utf8').trim().split(/\r?\n/).filter(Boolean);
  if (lines.length === 0) throw new Error(`Checksum index is empty: ${indexPath}`);
  for (const line of lines) {
    const match = line.match(/^([a-f0-9]{64}) ([ *])(?:\.\/)?(.+)$/);
    if (!match) throw new Error(`Invalid checksum line in ${indexPath}: ${line}`);
    const resolved = path.resolve(baseDirectory, match[3]);
    const relative = path.relative(path.resolve(baseDirectory), resolved);
    if (relative.startsWith('..') || path.isAbsolute(relative)) {
      throw new Error(`Checksum path escapes ${baseDirectory}: ${match[3]}`);
    }
    if (!fs.statSync(resolved).isFile()) throw new Error(`Checksummed file is missing: ${resolved}`);
    const actual = sha256(resolved);
    if (actual !== match[1]) {
      throw new Error(`Checksum mismatch for ${resolved}: expected ${match[1]}, received ${actual}.`);
    }
  }
  return lines.length;
}

function unique(values) {
  return [...new Set(values)];
}

function observedDependencyNames(evidenceDirectory, runtimeDirectory, platform) {
  if (platform === 'darwin') {
    const directory = path.join(evidenceDirectory, 'native');
    return unique(fs.readdirSync(directory)
      .filter((name) => name.endsWith('.dependencies.txt'))
      .sort()
      .flatMap((name) => {
        const owner = name.slice(0, -'.dependencies.txt'.length);
        const entries = parseMacDependencyEntries(fs.readFileSync(path.join(directory, name), 'utf8'));
        if (owner.endsWith('.dylib') && entries[0]?.name === owner) entries.shift();
        return entries.map((entry) => entry.name);
      }));
  }
  if (platform === 'linux') {
    const executable = calculixExecutableName(platform);
    return unique(parseLinuxDependencies(
      fs.readFileSync(path.join(evidenceDirectory, 'native', `${executable}.dependencies.txt`), 'utf8'),
    ));
  }
  if (platform === 'win32') {
    return unique(parseWindowsDependencies(
      fs.readFileSync(path.join(evidenceDirectory, 'native', 'ccx.imports.txt'), 'utf8'),
    ));
  }
  throw new Error(`Unsupported promotion platform: ${platform}`);
}

function dependencyDeclarations(evidenceDirectory, runtimeDirectory, platform) {
  const runtimeFiles = new Set(
    fs.readdirSync(runtimeDirectory, { withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => entry.name),
  );
  return observedDependencyNames(evidenceDirectory, runtimeDirectory, platform)
    .sort((left, right) => left.toLowerCase().localeCompare(right.toLowerCase(), 'en'))
    .map((name) => {
      if (runtimeFiles.has(name)) {
        return {
          name,
          kind: 'bundled',
          path: name,
          sha256: sha256(path.join(runtimeDirectory, name)),
        };
      }
      return { name, kind: 'system' };
    });
}

function parseBuildRevision(recipePath) {
  const recipe = fs.readFileSync(recipePath, 'utf8');
  const revision = recipe.match(/^Build revision: `([^`]+)`$/m)?.[1];
  if (!revision) throw new Error(`Build revision is missing from ${recipePath}.`);
  return revision;
}

function verifyCandidate(candidateDirectory, repositoryRoot, target) {
  const runtimeDirectory = path.join(candidateDirectory, 'runtime');
  const evidenceDirectory = path.join(candidateDirectory, 'evidence');
  if (fs.existsSync(path.join(runtimeDirectory, 'runtime-manifest.json'))) {
    throw new Error('The review candidate must not supply its own runtime-manifest.json.');
  }
  const evidenceCount = verifyChecksumIndex(
    evidenceDirectory,
    path.join(evidenceDirectory, 'EVIDENCE_SHA256SUMS'),
  );
  const sourceCount = verifyChecksumIndex(
    path.join(evidenceDirectory, 'source-inputs'),
    path.join(evidenceDirectory, 'source-inputs', 'SHA256SUMS'),
  );
  const runtimeCount = verifyChecksumIndex(
    runtimeDirectory,
    path.join(runtimeDirectory, 'SHA256SUMS'),
  );
  const runtimeIndex = fs.readFileSync(path.join(runtimeDirectory, 'SHA256SUMS'));
  const evidenceRuntimeIndex = fs.readFileSync(path.join(evidenceDirectory, 'RUNTIME_SHA256SUMS'));
  if (!runtimeIndex.equals(evidenceRuntimeIndex)) {
    throw new Error('Candidate runtime checksum index differs from the evidence copy.');
  }
  const buildOne = fs.readFileSync(path.join(evidenceDirectory, 'reproducibility', 'build-one-SHA256SUMS'));
  const buildTwo = fs.readFileSync(path.join(evidenceDirectory, 'reproducibility', 'build-two-SHA256SUMS'));
  if (!buildOne.equals(buildTwo)) throw new Error('The two candidate build indexes are not byte-identical.');
  const solverStdout = fs.readFileSync(path.join(evidenceDirectory, 'solver', 'spring1.stdout'));
  const solverStderr = fs.readFileSync(path.join(evidenceDirectory, 'solver', 'spring1.stderr'));
  if (solverStderr.length !== 0 || !solverStdout.includes(Buffer.from('Job finished'))) {
    throw new Error('The official spring1 solver evidence did not finish cleanly.');
  }
  const platform = target.split('-')[0];
  const recipeName = PLATFORM_RECIPE[platform];
  const reviewedRecipe = path.join(evidenceDirectory, 'source-inputs', recipeName);
  const repositoryRecipe = path.join(repositoryRoot, 'apps', 'fraia-electron', 'scripts', recipeName);
  if (!fs.readFileSync(reviewedRecipe).equals(fs.readFileSync(repositoryRecipe))) {
    throw new Error(`Candidate recipe does not match the current reviewed recipe: ${recipeName}`);
  }
  return {
    evidenceCount,
    evidenceDirectory,
    evidenceIndexSha256: sha256(path.join(evidenceDirectory, 'EVIDENCE_SHA256SUMS')),
    runtimeCount,
    runtimeDirectory,
    runtimeIndexSha256: hashBuffer(runtimeIndex),
    solver: {
      stderrSha256: hashBuffer(solverStderr),
      stdoutSha256: hashBuffer(solverStdout),
    },
    sourceCount,
  };
}

function promoteRuntime({
  candidateDirectory,
  outputDirectory,
  repository,
  repositoryRoot,
  sourceSha256,
  tag,
  target,
}) {
  if (!SUPPORTED_TARGETS.has(target)) throw new Error(`Unsupported CalculiX target: ${target}`);
  if (!SHA256_PATTERN.test(sourceSha256)) throw new Error('Corresponding-source SHA-256 is invalid.');
  if (fs.existsSync(outputDirectory)) throw new Error(`Promotion output already exists: ${outputDirectory}`);
  const verified = verifyCandidate(candidateDirectory, repositoryRoot, target);
  const platform = target.split('-')[0];
  const staging = `${outputDirectory}.staging-${process.pid}`;
  if (fs.existsSync(staging)) throw new Error(`Promotion staging path already exists: ${staging}`);
  fs.mkdirSync(path.dirname(outputDirectory), { recursive: true });
  fs.cpSync(verified.runtimeDirectory, staging, { recursive: true, errorOnExist: true });
  try {
    const recipePath = path.join(staging, 'BUILD_RECIPE.md');
    const executableName = calculixExecutableName(platform);
    const manifest = {
      schemaVersion: 1,
      target,
      calculixVersion: '2.23',
      upstream: {
        sourceUrl: 'https://www.dhondt.de/ccx_2.23.src.tar.bz2',
        sourceSha256: '9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7',
        revision: 'calculix-2.23',
      },
      build: {
        recipe: 'BUILD_RECIPE.md',
        recipeSha256: sha256(recipePath),
        revision: parseBuildRevision(recipePath),
      },
      redistribution: {
        sourceUrl: correspondingSourceUrl(repository, tag),
        sourceSha256,
        sourceAsset: CALCULIX_SOURCE_ASSET_NAME,
        licenseIdentifiers: LICENSE_IDENTIFIERS[platform],
      },
      files: {
        executable: {
          path: executableName,
          sha256: sha256(path.join(staging, executableName)),
        },
        notices: {
          path: 'THIRD_PARTY_NOTICES.txt',
          sha256: sha256(path.join(staging, 'THIRD_PARTY_NOTICES.txt')),
        },
        dependencies: dependencyDeclarations(
          verified.evidenceDirectory,
          staging,
          platform,
        ),
      },
    };
    fs.writeFileSync(
      path.join(staging, 'runtime-manifest.json'),
      `${JSON.stringify(manifest, null, 2)}\n`,
      { flag: 'wx', mode: 0o644 },
    );
    validateRuntimeDirectory(staging, target, { inspectDependencies: false });
    fs.renameSync(staging, outputDirectory);
    return {
      ...verified,
      executableSha256: manifest.files.executable.sha256,
      manifest,
      manifestSha256: sha256(path.join(outputDirectory, 'runtime-manifest.json')),
      outputDirectory,
    };
  } catch (error) {
    fs.rmSync(staging, { recursive: true, force: true });
    throw error;
  }
}

function parseArguments(argv) {
  const value = (name) => {
    const index = argv.indexOf(name);
    return index >= 0 ? argv[index + 1] : null;
  };
  const required = {
    candidateDirectory: value('--candidate'),
    outputDirectory: value('--output'),
    repository: value('--repository'),
    sourceSha256: value('--source-sha256'),
    tag: value('--tag'),
    target: value('--target'),
  };
  if (Object.values(required).some((entry) => !entry)) {
    throw new Error(
      'Usage: promote-calculix-runtime.cjs --candidate DIR --output NEW_DIR '
      + '--repository OWNER/REPO --source-sha256 HASH --tag vX.Y.Z --target TARGET',
    );
  }
  return {
    ...required,
    candidateDirectory: path.resolve(required.candidateDirectory),
    outputDirectory: path.resolve(required.outputDirectory),
    repositoryRoot: path.resolve(__dirname, '..', '..', '..'),
  };
}

function main() {
  const result = promoteRuntime(parseArguments(process.argv.slice(2)));
  process.stdout.write(`${JSON.stringify({
    evidenceCount: result.evidenceCount,
    evidenceIndexSha256: result.evidenceIndexSha256,
    executableSha256: result.executableSha256,
    manifestSha256: result.manifestSha256,
    outputDirectory: result.outputDirectory,
    runtimeCount: result.runtimeCount,
    runtimeIndexSha256: result.runtimeIndexSha256,
    solver: result.solver,
    sourceCount: result.sourceCount,
  })}\n`);
}

if (require.main === module) main();

module.exports = {
  dependencyDeclarations,
  LICENSE_IDENTIFIERS,
  observedDependencyNames,
  promoteRuntime,
  verifyCandidate,
  verifyChecksumIndex,
};
