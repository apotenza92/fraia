const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { binaryArchitecture } = require('../binary-architecture.cjs');
const {
  compareVersions,
  parseMacosMinimumVersion,
} = require('../macos-version-contract.cjs');

test('single-architecture fat Mach-O payloads retain exact target identity', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-fat-macho-'));
  const arm64 = path.join(root, 'arm64.dylib');
  const x64 = path.join(root, 'x64.dylib');
  const universal = path.join(root, 'universal.dylib');
  for (const [filePath, cpuType, count] of [
    [arm64, 0x0100000c, 1],
    [x64, 0x01000007, 1],
    [universal, 0x0100000c, 2],
  ]) {
    const bytes = Buffer.alloc(32);
    bytes.writeUInt32BE(0xcafebabe, 0);
    bytes.writeUInt32BE(count, 4);
    bytes.writeUInt32BE(cpuType, 8);
    fs.writeFileSync(filePath, bytes);
  }
  assert.equal(binaryArchitecture(arm64), 'arm64');
  assert.equal(binaryArchitecture(x64), 'x64');
  assert.throws(() => binaryArchitecture(universal), /exactly one Mach-O architecture/);
  fs.rmSync(root, { recursive: true, force: true });
});

test('macOS minimum-version evidence accepts modern and legacy Mach-O load commands', () => {
  assert.equal(parseMacosMinimumVersion(`
Load command 9
      cmd LC_BUILD_VERSION
  cmdsize 32
 platform 1
    minos 15.0
      sdk 26.1
`), '15.0');
  assert.equal(parseMacosMinimumVersion(`
Load command 8
      cmd LC_VERSION_MIN_MACOSX
  cmdsize 16
  version 11.0
      sdk 15.5
`), '11.0');
  assert.equal(compareVersions('15.0', '15.0.0'), 0);
  assert.ok(compareVersions('15.1', '15.0') > 0);
  assert.throws(
    () => parseMacosMinimumVersion('Load command 0\n      cmd LC_SEGMENT_64'),
    /do not declare a minimum macOS version/,
  );
});
