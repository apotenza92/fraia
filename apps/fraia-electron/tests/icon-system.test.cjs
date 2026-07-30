const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const { readFileSync } = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const applicationDirectory = path.resolve(__dirname, '..');
const repositoryRoot = path.resolve(applicationDirectory, '../..');
const stableSource = path.join(repositoryRoot, 'assets/fraia-icon.svg');
const betaSource = path.join(repositoryRoot, 'assets/fraia-icon-beta.svg');
const stableAdaptiveDirectory = path.join(applicationDirectory, 'build/macos/Fraia.icon');
const betaAdaptiveDirectory = path.join(applicationDirectory, 'build/beta/macos/Fraia Beta.icon');

test('generated stable and beta native icons match their reviewed manifests', () => {
  const output = execFileSync(
    process.execPath,
    [path.join(applicationDirectory, 'scripts/generate-release-icons.mjs'), '--check'],
    { cwd: applicationDirectory, encoding: 'utf8' },
  );
  assert.match(output, /Generated Fraia icon check passed \(30 files\)\./);
});

test('stable and beta icons retain one shared column geometry', () => {
  const sources = [
    stableSource,
    betaSource,
    path.join(stableAdaptiveDirectory, 'Assets/01-artwork.svg'),
    path.join(stableAdaptiveDirectory, 'Assets/01-artwork-dark.svg'),
    path.join(betaAdaptiveDirectory, 'Assets/01-artwork.svg'),
    path.join(betaAdaptiveDirectory, 'Assets/01-artwork-dark.svg'),
  ].map((filePath) => geometryFingerprint(readFileSync(filePath, 'utf8')));

  for (const fingerprint of sources.slice(1)) {
    assert.deepEqual(fingerprint, sources[0]);
  }
  assert.equal(sources[0].filter((shape) => shape.startsWith('path:d=M378 452')).length, 2);
  assert.ok(sources[0].includes('path:d=M328 424H696L682 620H342Z'));
});

test('macOS icon bundles select explicit light and dark artwork', () => {
  for (const directory of [stableAdaptiveDirectory, betaAdaptiveDirectory]) {
    const definition = JSON.parse(readFileSync(path.join(directory, 'icon.json'), 'utf8'));
    const layer = definition.groups?.[0]?.layers?.[0];
    assert.deepEqual(layer['image-name-specializations'], [
      { value: '01-artwork.svg' },
      { appearance: 'dark', value: '01-artwork-dark.svg' },
    ]);
  }
});

test('electron-builder selects the maintained adaptive icon for each channel identity', () => {
  const builderConfig = readFileSync(
    path.join(applicationDirectory, 'electron-builder.config.cjs'),
    'utf8',
  );
  const releaseContract = readFileSync(
    path.join(applicationDirectory, 'release-contract.cjs'),
    'utf8',
  );

  assert.match(builderConfig, /contract\.iconVariant === 'beta'/);
  assert.match(builderConfig, /'Fraia Beta\.icon' : 'Fraia\.icon'/);
  assert.match(builderConfig, /darwinFallback/);
  assert.match(builderConfig, /xcrun', \['actool', '--version'\]/);
  assert.match(builderConfig, /Xcode 26 or newer with actool is required/);
  assert.match(releaseContract, /const CHANNELS = new Set\(\['stable', 'beta'\]\)/);
  assert.match(releaseContract, /appId: 'app\.fraia\.desktop\.beta'/);
});

function geometryFingerprint(svg) {
  const column = extractColumnGroup(svg);
  const shapes = [];
  for (const match of column.matchAll(/<(rect|circle|path)\b([^>]*)\/?>/g)) {
    const [, element, attributes] = match;
    const record = [];
    for (const name of ['d', 'x', 'y', 'width', 'height', 'rx', 'cx', 'cy', 'r']) {
      const value = attributes.match(new RegExp(`\\b${name}="([^"]+)"`))?.[1];
      if (value !== undefined) {
        record.push(`${name}=${value}`);
      }
    }
    if (record.length > 0) {
      shapes.push(`${element}:${record.join(',')}`);
    }
  }
  return shapes;
}

function extractColumnGroup(svg) {
  const start = svg.indexOf('<g id="column">');
  assert.notEqual(start, -1, 'icon source must contain the canonical column group');
  const tags = /<\/?g\b[^>]*>/g;
  tags.lastIndex = start;
  let depth = 0;
  for (const match of svg.matchAll(tags)) {
    if (match.index < start) {
      continue;
    }
    depth += match[0].startsWith('</') ? -1 : 1;
    if (depth === 0) {
      return svg.slice(start, match.index + match[0].length);
    }
  }
  assert.fail('canonical column group must be closed');
}
