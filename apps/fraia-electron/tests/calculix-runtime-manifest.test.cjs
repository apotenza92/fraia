const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const {
  parseLinuxDependencies,
  parseMacDependencies,
  parseWindowsDependencies,
  sha256,
  validateRuntimeDirectory,
} = require('../calculix-runtime-manifest.cjs');

function writeElfX64(filePath) {
  const bytes = Buffer.alloc(64);
  bytes.set([0x7f, 0x45, 0x4c, 0x46], 0);
  bytes[5] = 1;
  bytes.writeUInt16LE(62, 18);
  fs.writeFileSync(filePath, bytes);
  fs.chmodSync(filePath, 0o755);
}

function writeElfArm64(filePath) {
  const bytes = Buffer.alloc(64);
  bytes.set([0x7f, 0x45, 0x4c, 0x46], 0);
  bytes[5] = 1;
  bytes.writeUInt16LE(183, 18);
  fs.writeFileSync(filePath, bytes);
  fs.chmodSync(filePath, 0o755);
}

function writeMachOX64(filePath) {
  const bytes = Buffer.alloc(32);
  bytes.set([0xcf, 0xfa, 0xed, 0xfe], 0);
  bytes.writeUInt32LE(0x01000007, 4);
  fs.writeFileSync(filePath, bytes);
  fs.chmodSync(filePath, 0o755);
}

function fixture() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-calculix-manifest-'));
  const executable = path.join(directory, 'ccx');
  const notices = path.join(directory, 'THIRD_PARTY_NOTICES.txt');
  const dependency = path.join(directory, 'libreviewed.so.1');
  const recipe = path.join(directory, 'BUILD_RECIPE.md');
  writeElfX64(executable);
  fs.writeFileSync(notices, 'CalculiX and bundled dependency notices.\n');
  writeElfX64(dependency);
  fs.writeFileSync(recipe, 'Pinned native build procedure.\n');
  const manifest = {
    schemaVersion: 1,
    target: 'linux-x64',
    calculixVersion: '2.23',
    upstream: {
      sourceUrl: 'https://example.invalid/calculix-2.23.tar.bz2',
      sourceSha256: '1'.repeat(64),
      revision: 'calculix-2.23',
    },
    build: {
      recipe: 'BUILD_RECIPE.md',
      recipeSha256: sha256(recipe),
      revision: 'fraia-calculix-build-v1',
    },
    redistribution: {
      sourceUrl: 'https://example.invalid/fraia-calculix-linux-x64-source.tar.zst',
      sourceSha256: '3'.repeat(64),
      licenseIdentifiers: ['GPL-2.0-only'],
    },
    files: {
      executable: { path: 'ccx', sha256: sha256(executable) },
      notices: { path: 'THIRD_PARTY_NOTICES.txt', sha256: sha256(notices) },
      dependencies: [
        { name: 'libreviewed.so.1', kind: 'bundled', path: 'libreviewed.so.1', sha256: sha256(dependency) },
        { name: 'libc.so.6', kind: 'system' },
      ],
    },
  };
  fs.writeFileSync(path.join(directory, 'runtime-manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  return { directory, manifest };
}

test('reviewed runtime manifest pins source, recipe, files, licenses, and exact dependency closure', (t) => {
  const { directory } = fixture();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const result = validateRuntimeDirectory(directory, 'linux-x64', {
    commandRunner: () => `libreviewed.so.1 => ${path.join(directory, 'libreviewed.so.1')} (0x1)\nlibc.so.6 => /lib/libc.so.6 (0x2)\n`,
  });
  assert.equal(result.manifest.calculixVersion, '2.23');
});

test('runtime manifest rejects altered files and unreviewed native dependencies', (t) => {
  const { directory } = fixture();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.appendFileSync(path.join(directory, 'THIRD_PARTY_NOTICES.txt'), 'altered');
  assert.throws(
    () => validateRuntimeDirectory(directory, 'linux-x64', { inspectDependencies: false }),
    /files\.notices SHA-256 mismatch/,
  );

  const fresh = fixture();
  t.after(() => fs.rmSync(fresh.directory, { recursive: true, force: true }));
  assert.throws(
    () => validateRuntimeDirectory(fresh.directory, 'linux-x64', {
      commandRunner: () => `libreviewed.so.1 => ${path.join(fresh.directory, 'libreviewed.so.1')} (0x1)\nlibsurprise.so.9 => /tmp/libsurprise.so.9 (0x2)\n`,
    }),
    /Undeclared: libsurprise\.so\.9.*Missing: libc\.so\.6/,
  );

  const alteredRecipe = fixture();
  t.after(() => fs.rmSync(alteredRecipe.directory, { recursive: true, force: true }));
  fs.appendFileSync(path.join(alteredRecipe.directory, 'BUILD_RECIPE.md'), 'changed');
  assert.throws(
    () => validateRuntimeDirectory(alteredRecipe.directory, 'linux-x64', {
      inspectDependencies: false,
    }),
    /build\.recipe SHA-256 mismatch/,
  );
});

test('runtime manifest rejects unresolved and externally resolved bundled Linux dependencies', (t) => {
  const unresolved = fixture();
  t.after(() => fs.rmSync(unresolved.directory, { recursive: true, force: true }));
  assert.throws(
    () => validateRuntimeDirectory(unresolved.directory, 'linux-x64', {
      commandRunner: () => 'libreviewed.so.1 => not found\nlibc.so.6 => /lib/libc.so.6 (0x2)\n',
    }),
    /libreviewed\.so\.1 was not found/,
  );

  const external = fixture();
  t.after(() => fs.rmSync(external.directory, { recursive: true, force: true }));
  assert.throws(
    () => validateRuntimeDirectory(external.directory, 'linux-x64', {
      commandRunner: () => 'libreviewed.so.1 => /tmp/libreviewed.so.1 (0x1)\nlibc.so.6 => /lib/libc.so.6 (0x2)\n',
    }),
    /resolved outside the reviewed runtime/,
  );
});

test('runtime manifest rejects a bundled dependency for another architecture', (t) => {
  const { directory, manifest } = fixture();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const dependency = path.join(directory, 'libreviewed.so.1');
  writeElfArm64(dependency);
  manifest.files.dependencies[0].sha256 = sha256(dependency);
  fs.writeFileSync(
    path.join(directory, 'runtime-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  assert.throws(
    () => validateRuntimeDirectory(directory, 'linux-x64', {
      inspectDependencies: false,
    }),
    /architecture arm64 does not match x64/,
  );
});

test('runtime manifest recursively enforces loader-relative macOS dependencies', (t) => {
  const { directory, manifest } = fixture();
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const executable = path.join(directory, 'ccx');
  const oldDependency = path.join(directory, 'libreviewed.so.1');
  const dependency = path.join(directory, 'libreviewed.dylib');
  fs.renameSync(oldDependency, dependency);
  writeMachOX64(executable);
  writeMachOX64(dependency);
  manifest.target = 'darwin-x64';
  manifest.files.executable.sha256 = sha256(executable);
  manifest.files.dependencies = [
    {
      name: 'libreviewed.dylib',
      kind: 'bundled',
      path: 'libreviewed.dylib',
      sha256: sha256(dependency),
    },
    { name: 'libSystem.B.dylib', kind: 'system' },
  ];
  fs.writeFileSync(
    path.join(directory, 'runtime-manifest.json'),
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  const goodOutput = [
    `${executable}:`,
    '\t@loader_path/libreviewed.dylib (compatibility version 1.0.0)',
    '\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0)',
    '',
  ].join('\n');
  const dependencyOutput = [
    `${dependency}:`,
    '\t@rpath/libreviewed.dylib (compatibility version 1.0.0)',
    '\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0)',
    '',
  ].join('\n');
  const commandRunner = (_command, args) => (
    args[0] === '-l'
      ? 'cmd LC_RPATH\n  cmdsize 32\n     path @loader_path (offset 12)\n'
      : args.at(-1) === dependency ? dependencyOutput : goodOutput
  );
  validateRuntimeDirectory(directory, 'darwin-x64', {
    commandRunner,
  });

  assert.throws(
    () => validateRuntimeDirectory(directory, 'darwin-x64', {
      commandRunner: (_command, args) => (
        args[0] === '-l'
          ? 'cmd LC_RPATH\n  cmdsize 32\n     path @loader_path (offset 12)\n'
          : args.at(-1) === dependency
          ? dependencyOutput
          : goodOutput.replace('@loader_path/', '@rpath/')
      ),
    }),
    /must use @loader_path/,
  );
});

test('dependency parsers normalize native loader output', () => {
  assert.deepEqual(
    parseMacDependencies('/tmp/ccx:\n\t@rpath/libgcc_s.1.1.dylib (compatibility version 1.0.0)\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0)\n'),
    ['libgcc_s.1.1.dylib', 'libSystem.B.dylib'],
  );
  assert.deepEqual(
    parseLinuxDependencies('linux-vdso.so.1 (0x1)\nlibgomp.so.1 => /lib/libgomp.so.1 (0x2)\n/lib64/ld-linux-x86-64.so.2 (0x3)\n'),
    ['linux-vdso.so.1', 'libgomp.so.1', 'ld-linux-x86-64.so.2'],
  );
  assert.deepEqual(
    parseWindowsDependencies('    KERNEL32.dll\n    DLL Name: libgfortran-5.dll\n'),
    ['KERNEL32.dll', 'libgfortran-5.dll'],
  );
});
