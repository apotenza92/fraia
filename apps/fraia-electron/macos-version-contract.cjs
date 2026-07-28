const { spawnSync } = require('node:child_process');

function compareVersions(left, right) {
  const leftParts = String(left).split('.').map(Number);
  const rightParts = String(right).split('.').map(Number);
  const length = Math.max(leftParts.length, rightParts.length);
  for (let index = 0; index < length; index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return 0;
}

function parseMacosMinimumVersion(output) {
  let command = null;
  for (const line of String(output).split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed === 'cmd LC_BUILD_VERSION') {
      command = 'build';
    } else if (trimmed === 'cmd LC_VERSION_MIN_MACOSX') {
      command = 'legacy';
    } else if (command === 'build' && trimmed.startsWith('minos ')) {
      return trimmed.slice('minos '.length).trim();
    } else if (command === 'legacy' && trimmed.startsWith('version ')) {
      return trimmed.slice('version '.length).trim();
    } else if (trimmed.startsWith('cmd ')) {
      command = null;
    }
  }
  throw new Error('Mach-O load commands do not declare a minimum macOS version.');
}

function assertMacosMinimumVersion(filePath, maximum = '15.0') {
  if (process.platform !== 'darwin') {
    throw new Error('Mach-O minimum-version verification requires macOS.');
  }
  const result = spawnSync('otool', ['-l', filePath], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`otool failed for ${filePath}: ${result.stderr || result.stdout}`);
  }
  const actual = parseMacosMinimumVersion(result.stdout);
  if (compareVersions(actual, maximum) > 0) {
    throw new Error(`${filePath} requires macOS ${actual}, exceeding the reviewed ${maximum} maximum.`);
  }
  return actual;
}

module.exports = {
  assertMacosMinimumVersion,
  compareVersions,
  parseMacosMinimumVersion,
};
