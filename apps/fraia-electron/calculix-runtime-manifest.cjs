const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { assertBinaryArchitecture } = require('./binary-architecture.cjs');
const {
  SUPPORTED_TARGETS,
  calculixExecutableName,
} = require('./package-boundary.cjs');

const MANIFEST_FILE_NAME = 'runtime-manifest.json';
const SHA256_PATTERN = /^[a-f0-9]{64}$/;

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function requireString(value, label) {
  if (typeof value !== 'string' || value.trim() === '') {
    throw new Error(`${label} must be a non-empty string.`);
  }
  return value;
}

function requireSha256(value, label) {
  requireString(value, label);
  if (!SHA256_PATTERN.test(value)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest.`);
  }
  return value;
}

function requireHttpsUrl(value, label) {
  requireString(value, label);
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    throw new Error(`${label} must be a valid HTTPS URL.`);
  }
  if (parsed.protocol !== 'https:') throw new Error(`${label} must be a valid HTTPS URL.`);
  return value;
}

function resolveManifestFile(directory, relativePath, label) {
  requireString(relativePath, label);
  if (path.isAbsolute(relativePath)) throw new Error(`${label} must be relative to the runtime directory.`);
  const resolved = path.resolve(directory, relativePath);
  const relative = path.relative(path.resolve(directory), resolved);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`${label} must remain inside the runtime directory.`);
  }
  return resolved;
}

function verifyFile(directory, descriptor, label) {
  if (!descriptor || typeof descriptor !== 'object' || Array.isArray(descriptor)) {
    throw new Error(`${label} must be an object.`);
  }
  const filePath = resolveManifestFile(directory, descriptor.path, `${label}.path`);
  requireSha256(descriptor.sha256, `${label}.sha256`);
  if (!fs.existsSync(filePath) || !fs.statSync(filePath).isFile()) {
    throw new Error(`${label} is missing: ${filePath}`);
  }
  const actual = sha256(filePath);
  if (actual !== descriptor.sha256) {
    throw new Error(`${label} SHA-256 mismatch: expected ${descriptor.sha256}, received ${actual}.`);
  }
  return filePath;
}

function parseMacDependencyEntries(output) {
  return output
    .split(/\r?\n/)
    .slice(1)
    .map((line) => line.trim().split(/\s+\(/, 1)[0])
    .filter(Boolean)
    .map((installName) => ({
      installName,
      name: path.basename(installName),
    }));
}

function parseMacDependencies(output) {
  return parseMacDependencyEntries(output).map(({ name }) => name);
}

function parseMacRpaths(output) {
  const rpaths = [];
  let expectsPath = false;
  for (const line of output.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === 'cmd LC_RPATH') {
      expectsPath = true;
      continue;
    }
    if (expectsPath) {
      const match = trimmed.match(/^path\s+(\S+)\s+\(offset\s+\d+\)$/);
      if (match) {
        rpaths.push(match[1]);
        expectsPath = false;
      }
    }
  }
  return rpaths;
}

function parseLinuxDependencyEntries(output) {
  return output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const missing = line.match(/^(\S+)\s+=>\s+not found$/);
      if (missing) {
        return {
          name: missing[1],
          resolvedPath: null,
          notFound: true,
        };
      }
      const linked = line.match(/^(\S+)\s+=>\s+(\S+)(?:\s+\(|$)/);
      if (linked) {
        return {
          name: linked[1],
          resolvedPath: linked[2],
          notFound: false,
        };
      }
      const direct = line.match(/^(\S+)(?:\s+\(|$)/);
      return direct ? {
        name: path.basename(direct[1]),
        resolvedPath: path.isAbsolute(direct[1]) ? direct[1] : null,
        notFound: false,
      } : null;
    })
    .filter(Boolean);
}

function parseLinuxDependencies(output) {
  return parseLinuxDependencyEntries(output).map(({ name }) => name);
}

function parseWindowsDependencies(output) {
  return output
    .split(/\r?\n/)
    .map((line) => line.match(/^\s*(?:DLL Name:\s*)?([A-Za-z0-9_.+-]+\.dll)\s*$/i)?.[1])
    .filter(Boolean);
}

function defaultCommandRunner(command, args) {
  const result = spawnSync(command, args, { encoding: 'utf8' });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} failed with status ${result.status}: ${(result.stderr || '').trim()}`);
  }
  return result.stdout;
}

function inspectNativeDependencyEntries(executable, platform, commandRunner = defaultCommandRunner) {
  if (platform === 'darwin') {
    const entries = parseMacDependencyEntries(commandRunner('otool', ['-L', executable]));
    if (executable.endsWith('.dylib') && entries[0]?.name === path.basename(executable)) {
      return entries.slice(1);
    }
    return entries;
  }
  if (platform === 'linux') return parseLinuxDependencyEntries(commandRunner('ldd', [executable]));
  if (platform === 'win32') {
    try {
      return parseWindowsDependencies(commandRunner('llvm-objdump', ['-p', executable]))
        .map((name) => ({ name }));
    } catch (llvmError) {
      try {
        return parseWindowsDependencies(commandRunner('dumpbin', ['/DEPENDENTS', executable]))
          .map((name) => ({ name }));
      } catch (dumpbinError) {
        throw new Error(
          `Unable to inspect Windows runtime dependencies with llvm-objdump or dumpbin: `
          + `${llvmError.message}; ${dumpbinError.message}`,
        );
      }
    }
  }
  throw new Error(`Unsupported dependency-inspection platform: ${platform}.`);
}

function inspectNativeDependencies(executable, platform, commandRunner = defaultCommandRunner) {
  return inspectNativeDependencyEntries(executable, platform, commandRunner)
    .map(({ name }) => name);
}

function validateDependencyContract(
  directory,
  dependencies,
  observedEntries,
  platform,
  expectedArchitecture,
) {
  if (!Array.isArray(dependencies)) throw new Error('files.dependencies must be an array.');
  const declaredNames = new Set();
  const declarations = new Map();
  const normalizeName = (name) => platform === 'win32' ? name.toLowerCase() : name;
  for (const [index, dependency] of dependencies.entries()) {
    const label = `files.dependencies[${index}]`;
    if (!dependency || typeof dependency !== 'object' || Array.isArray(dependency)) {
      throw new Error(`${label} must be an object.`);
    }
    const name = normalizeName(requireString(dependency.name, `${label}.name`));
    if (declaredNames.has(name)) throw new Error(`Duplicate declared dependency: ${name}.`);
    declaredNames.add(name);
    declarations.set(name, dependency);
    if (dependency.kind === 'bundled') {
      const bundledPath = verifyFile(directory, dependency, label);
      if (path.basename(bundledPath) !== dependency.name || path.basename(dependency.path) !== dependency.path) {
        throw new Error(`${label} bundled dependencies must be flat files named exactly ${dependency.name}.`);
      }
      assertBinaryArchitecture(bundledPath, expectedArchitecture);
    } else if (dependency.kind === 'system') {
      if ('path' in dependency || 'sha256' in dependency) {
        throw new Error(`${label} system dependencies must not declare local file hashes.`);
      }
    } else {
      throw new Error(`${label}.kind must be bundled or system.`);
    }
  }

  if (observedEntries) {
    const observed = new Set(observedEntries.map(({ name }) => normalizeName(name)));
    const undeclared = [...observed].filter((name) => !declaredNames.has(name)).sort();
    const missing = [...declaredNames].filter((name) => !observed.has(name)).sort();
    if (undeclared.length || missing.length) {
      throw new Error(
        `Native dependency closure differs from the reviewed manifest. `
        + `Undeclared: ${undeclared.join(', ') || 'none'}. `
        + `Missing: ${missing.join(', ') || 'none'}.`,
      );
    }

    const runtimeRoot = path.resolve(directory);
    for (const entry of observedEntries) {
      const name = normalizeName(entry.name);
      const declaration = declarations.get(name);
      if (platform === 'darwin') {
        if (declaration.kind === 'bundled') {
          const expectedLoaderPath = `@loader_path/${declaration.path}`;
          const expectedRunPath = `@rpath/${declaration.path}`;
          const ownerDeclaration = declarations.get(
            normalizeName(path.basename(entry.owner)),
          );
          const isBundledDylibOwner = ownerDeclaration?.kind === 'bundled'
            && entry.owner.endsWith('.dylib');
          if (entry.installName !== expectedLoaderPath
            && !(isBundledDylibOwner && entry.installName === expectedRunPath)) {
            throw new Error(
              `Bundled macOS dependency ${entry.name} must use ${expectedLoaderPath}`
              + `, or ${expectedRunPath} from a bundled dylib, received ${entry.installName}.`,
            );
          }
        } else if (!entry.installName.startsWith('/usr/lib/')
          && !entry.installName.startsWith('/System/Library/Frameworks/')) {
          throw new Error(
            `System macOS dependency ${entry.name} has an unreviewed install name: ${entry.installName}.`,
          );
        }
      }
      if (platform === 'linux') {
        if (entry.notFound) throw new Error(`Linux dependency ${entry.name} was not found by the native loader.`);
        if (declaration.kind === 'bundled') {
          if (!entry.resolvedPath || !path.isAbsolute(entry.resolvedPath)) {
            throw new Error(`Bundled Linux dependency ${entry.name} did not resolve to an absolute path.`);
          }
          const resolved = path.resolve(entry.resolvedPath);
          const relative = path.relative(runtimeRoot, resolved);
          if (relative.startsWith('..') || path.isAbsolute(relative) || path.basename(resolved) !== declaration.path) {
            throw new Error(
              `Bundled Linux dependency ${entry.name} resolved outside the reviewed runtime: ${entry.resolvedPath}.`,
            );
          }
        } else if (entry.resolvedPath) {
          const relative = path.relative(runtimeRoot, path.resolve(entry.resolvedPath));
          if (!relative.startsWith('..') && !path.isAbsolute(relative)) {
            throw new Error(
              `System Linux dependency ${entry.name} unexpectedly resolved inside the reviewed runtime.`,
            );
          }
        }
      }
    }
  }
}

function inspectNativeDependencyClosure(
  executable,
  directory,
  dependencies,
  platform,
  commandRunner = defaultCommandRunner,
) {
  const nativeFiles = [
    executable,
    ...dependencies
      .filter(({ kind }) => kind === 'bundled')
      .map((dependency) => resolveManifestFile(directory, dependency.path, 'files.dependencies[].path')),
  ];
  const entries = [];
  for (const nativeFile of nativeFiles) {
    for (const entry of inspectNativeDependencyEntries(nativeFile, platform, commandRunner)) {
      entries.push({ ...entry, owner: path.relative(directory, nativeFile) || path.basename(nativeFile) });
    }
  }
  return entries;
}

function validateRuntimeDirectory(directory, expectedTarget, {
  inspectDependencies = true,
  commandRunner = defaultCommandRunner,
} = {}) {
  if (!SUPPORTED_TARGETS.has(expectedTarget)) throw new Error(`Unsupported CalculiX target: ${expectedTarget}.`);
  const manifestPath = path.join(directory, MANIFEST_FILE_NAME);
  if (!fs.existsSync(manifestPath)) throw new Error(`Reviewed CalculiX runtime manifest is missing: ${manifestPath}`);
  let manifest;
  try {
    manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  } catch (error) {
    throw new Error(`CalculiX runtime manifest is invalid JSON: ${error.message}`);
  }
  if (manifest.schemaVersion !== 1) throw new Error('schemaVersion must be 1.');
  if (manifest.target !== expectedTarget) {
    throw new Error(`Runtime manifest target ${manifest.target} does not match ${expectedTarget}.`);
  }
  requireString(manifest.calculixVersion, 'calculixVersion');
  if (!manifest.upstream || typeof manifest.upstream !== 'object') throw new Error('upstream must be an object.');
  requireHttpsUrl(manifest.upstream.sourceUrl, 'upstream.sourceUrl');
  requireSha256(manifest.upstream.sourceSha256, 'upstream.sourceSha256');
  requireString(manifest.upstream.revision, 'upstream.revision');
  if (!manifest.build || typeof manifest.build !== 'object') throw new Error('build must be an object.');
  const buildRecipe = resolveManifestFile(directory, manifest.build.recipe, 'build.recipe');
  requireSha256(manifest.build.recipeSha256, 'build.recipeSha256');
  requireString(manifest.build.revision, 'build.revision');
  if (!fs.existsSync(buildRecipe) || !fs.statSync(buildRecipe).isFile()) {
    throw new Error(`Reviewed build recipe is missing: ${buildRecipe}`);
  }
  const recipeSha256 = sha256(buildRecipe);
  if (recipeSha256 !== manifest.build.recipeSha256) {
    throw new Error(
      `build.recipe SHA-256 mismatch: expected ${manifest.build.recipeSha256}, received ${recipeSha256}.`,
    );
  }
  if (!manifest.redistribution || typeof manifest.redistribution !== 'object') {
    throw new Error('redistribution must be an object.');
  }
  requireHttpsUrl(manifest.redistribution.sourceUrl, 'redistribution.sourceUrl');
  requireSha256(manifest.redistribution.sourceSha256, 'redistribution.sourceSha256');
  if (!Array.isArray(manifest.redistribution.licenseIdentifiers)
    || manifest.redistribution.licenseIdentifiers.length === 0) {
    throw new Error('redistribution.licenseIdentifiers must be a non-empty array.');
  }
  for (const [index, identifier] of manifest.redistribution.licenseIdentifiers.entries()) {
    requireString(identifier, `redistribution.licenseIdentifiers[${index}]`);
  }
  if (!manifest.files || typeof manifest.files !== 'object') throw new Error('files must be an object.');
  const executable = verifyFile(directory, manifest.files.executable, 'files.executable');
  const notices = verifyFile(directory, manifest.files.notices, 'files.notices');
  if (path.basename(executable) !== calculixExecutableName(expectedTarget.split('-')[0])) {
    throw new Error(`files.executable must name ${calculixExecutableName(expectedTarget.split('-')[0])}.`);
  }
  if (path.basename(notices) !== 'THIRD_PARTY_NOTICES.txt' || fs.statSync(notices).size === 0) {
    throw new Error('files.notices must be the non-empty THIRD_PARTY_NOTICES.txt.');
  }
  assertBinaryArchitecture(executable, expectedTarget.split('-')[1]);
  validateDependencyContract(
    directory,
    manifest.files.dependencies,
    null,
    expectedTarget.split('-')[0],
    expectedTarget.split('-')[1],
  );
  const observed = inspectDependencies
    ? inspectNativeDependencyClosure(
      executable,
      directory,
      manifest.files.dependencies,
      expectedTarget.split('-')[0],
      commandRunner,
    )
    : null;
  if (inspectDependencies && expectedTarget.startsWith('darwin-')) {
    const rpaths = parseMacRpaths(commandRunner('otool', ['-l', executable]));
    if (rpaths.length !== 1 || rpaths[0] !== '@loader_path') {
      throw new Error(
        `macOS CalculiX must contain exactly the @loader_path runpath; received `
        + `${rpaths.join(', ') || 'none'}.`,
      );
    }
  }
  validateDependencyContract(
    directory,
    manifest.files.dependencies,
    observed,
    expectedTarget.split('-')[0],
    expectedTarget.split('-')[1],
  );
  return { executable, manifest, manifestPath, notices };
}

module.exports = {
  MANIFEST_FILE_NAME,
  inspectNativeDependencyClosure,
  inspectNativeDependencies,
  parseLinuxDependencyEntries,
  parseLinuxDependencies,
  parseMacDependencyEntries,
  parseMacDependencies,
  parseMacRpaths,
  parseWindowsDependencies,
  sha256,
  validateRuntimeDirectory,
};
