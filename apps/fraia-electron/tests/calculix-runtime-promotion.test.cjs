const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const {
  LICENSE_IDENTIFIERS,
  observedDependencyNames,
  verifyChecksumIndex,
} = require('../scripts/promote-calculix-runtime.cjs');

test('promotion declares the reviewed Windows licence set', () => {
  assert.deepEqual(LICENSE_IDENTIFIERS.win32, [
    'GPL-2.0-only',
    'LicenseRef-SPOOLES-Public-Domain',
    'BSD-3-Clause',
    'MIT',
    'GPL-3.0-or-later WITH GCC-exception-3.1',
  ]);
});

test('promotion dependency evidence normalizes macOS, Linux, and Windows closure', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-promotion-deps-'));
  const native = path.join(root, 'native');
  fs.mkdirSync(native);
  fs.writeFileSync(
    path.join(native, 'ccx.dependencies.txt'),
    [
      '\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)',
      '\t@loader_path/libgfortran.5.dylib (compatibility version 6.0.0, current version 6.0.0)',
      '',
    ].join('\n'),
  );
  fs.writeFileSync(
    path.join(native, 'libgfortran.5.dylib.dependencies.txt'),
    [
      '\t@loader_path/libgfortran.5.dylib (compatibility version 6.0.0, current version 6.0.0)',
      '\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)',
      '',
    ].join('\n'),
  );
  assert.deepEqual(
    observedDependencyNames(root, root, 'darwin').sort(),
    ['libSystem.B.dylib', 'libgfortran.5.dylib'],
  );
  fs.writeFileSync(
    path.join(native, 'ccx.dependencies.txt'),
    'libc.so.6 => /lib/aarch64-linux-gnu/libc.so.6 (0x1)\n/lib/ld-linux-aarch64.so.1 (0x2)\n',
  );
  assert.deepEqual(
    observedDependencyNames(root, root, 'linux').sort(),
    ['ld-linux-aarch64.so.1', 'libc.so.6'],
  );
  fs.writeFileSync(path.join(native, 'ccx.imports.txt'), 'KERNEL32.dll\napi-ms-win-crt-runtime-l1-1-0.dll\n');
  assert.deepEqual(
    observedDependencyNames(root, root, 'win32').sort(),
    ['KERNEL32.dll', 'api-ms-win-crt-runtime-l1-1-0.dll'],
  );
  fs.rmSync(root, { recursive: true, force: true });
});

test('promotion checksum verification rejects changed and escaping evidence', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-promotion-sums-'));
  const payload = path.join(root, 'payload.txt');
  const index = path.join(root, 'SHA256SUMS');
  fs.writeFileSync(payload, 'reviewed\n');
  fs.writeFileSync(index, 'a9f2d25d1f71f8065e2119e538bde8846570fcdad320388236e99d9e225c290d  payload.txt\n');
  assert.equal(verifyChecksumIndex(root, index), 1);
  fs.appendFileSync(payload, 'changed\n');
  assert.throws(() => verifyChecksumIndex(root, index), /Checksum mismatch/);
  fs.writeFileSync(index, `${'0'.repeat(64)}  ../outside\n`);
  assert.throws(() => verifyChecksumIndex(root, index), /escapes/);
  fs.rmSync(root, { recursive: true, force: true });
});
