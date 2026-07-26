const fs = require('node:fs');

function binaryArchitecture(filePath) {
  const file = fs.readFileSync(filePath);
  const magic = file.subarray(0, 4).toString('hex');
  if (file.subarray(0, 2).toString('ascii') === 'MZ') {
    const peOffset = file.readUInt32LE(0x3c);
    if (file.subarray(peOffset, peOffset + 4).toString('hex') !== '50450000') throw new Error(`${filePath} has an invalid PE header.`);
    const machine = file.readUInt16LE(peOffset + 4);
    if (machine === 0xaa64) return 'arm64';
    if (machine === 0x8664) return 'x64';
    throw new Error(`${filePath} has unsupported PE machine 0x${machine.toString(16)}.`);
  }
  if (magic === '7f454c46') {
    const machine = file[5] === 1 ? file.readUInt16LE(18) : file.readUInt16BE(18);
    if (machine === 183) return 'arm64';
    if (machine === 62) return 'x64';
    throw new Error(`${filePath} has unsupported ELF machine ${machine}.`);
  }
  const littleEndianMachO = magic === 'cffaedfe';
  const bigEndianMachO = magic === 'feedfacf';
  if (littleEndianMachO || bigEndianMachO) {
    const cpuType = littleEndianMachO ? file.readUInt32LE(4) : file.readUInt32BE(4);
    if (cpuType === 0x0100000c) return 'arm64';
    if (cpuType === 0x01000007) return 'x64';
    throw new Error(`${filePath} has unsupported Mach-O CPU type 0x${cpuType.toString(16)}.`);
  }
  throw new Error(`${filePath} is not a supported native executable.`);
}

function assertBinaryArchitecture(filePath, expectedArchitecture) {
  const actual = binaryArchitecture(filePath);
  if (actual !== expectedArchitecture) {
    throw new Error(`${filePath} architecture ${actual} does not match ${expectedArchitecture}.`);
  }
}

module.exports = { assertBinaryArchitecture, binaryArchitecture };
