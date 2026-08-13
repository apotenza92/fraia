const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const { readFileSync } = require('node:fs');
const { createRequire } = require('node:module');
const path = require('node:path');
const test = require('node:test');

const applicationDirectory = path.resolve(__dirname, '..');
const repositoryRoot = path.resolve(applicationDirectory, '../..');
const stableSource = path.join(repositoryRoot, 'assets/fraia-icon.svg');
const betaSource = path.join(repositoryRoot, 'assets/fraia-icon-beta.svg');
const stableAdaptiveDirectory = path.join(applicationDirectory, 'build/macos/Fraia.icon');
const betaAdaptiveDirectory = path.join(applicationDirectory, 'build/beta/macos/Fraia Beta.icon');
const requireFromApplication = createRequire(path.join(applicationDirectory, 'package.json'));
const { createCanvas, loadImage } = requireFromApplication('@napi-rs/canvas');

test('generated stable and beta native icons match their reviewed manifests', () => {
  const output = execFileSync(
    process.execPath,
    [path.join(applicationDirectory, 'scripts/generate-release-icons.mjs'), '--check'],
    { cwd: applicationDirectory, encoding: 'utf8' },
  );
  assert.match(output, /Generated Fraia icon check passed \(30 files\)\./);
});

test('stable and beta icons retain one shared Fraia glyph geometry', () => {
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
  assert.ok(sources[0].includes('path:d=M28 70V52Q28 26 54 26H112'));
  assert.ok(sources[0].includes('path:d=M28 70H91'));
  assert.ok(sources[0].includes('path:d=M28 70V113H71.61'));
  assert.ok(sources[0].includes('path:d=M103 18A58 58 0 0 0 66.42 121H87.61L72.43 105A42 42 0 0 1 103 34Z'));
});

test('channel artwork uses only the reviewed Fraia brand colours', () => {
  const artwork = [
    [stableSource, '#24211f'],
    [betaSource, '#b95232'],
    [path.join(stableAdaptiveDirectory, 'Assets/01-artwork.svg'), '#24211f'],
    [path.join(stableAdaptiveDirectory, 'Assets/01-artwork-dark.svg'), '#f5f1eb'],
    [path.join(betaAdaptiveDirectory, 'Assets/01-artwork.svg'), '#b95232'],
    [path.join(betaAdaptiveDirectory, 'Assets/01-artwork-dark.svg'), '#ef795f'],
  ];

  for (const [filePath, colour] of artwork) {
    const svg = readFileSync(filePath, 'utf8').toLowerCase();
    assert.match(svg, new RegExp(`data-channel-color="${colour}"`));
    assert.doesNotMatch(svg, /<rect[^>]+fill=/);
  }
});

test('generated glyph is large, centred, and unclipped at release icon sizes', async () => {
  for (const filePath of [stableSource, betaSource]) {
    for (const size of [16, 32, 128, 512]) {
      const bounds = await opaqueBounds(filePath, size);
      assert.ok(bounds.width / size >= 0.7, `${path.basename(filePath)} is too narrow at ${size}px`);
      assert.ok(bounds.height / size >= 0.8, `${path.basename(filePath)} is too short at ${size}px`);
      assert.ok(bounds.left >= 1 && bounds.top >= 1, `${path.basename(filePath)} clips its top or left edge at ${size}px`);
      assert.ok(bounds.right <= size - 2 && bounds.bottom <= size - 2, `${path.basename(filePath)} clips its bottom or right edge at ${size}px`);
    }
  }
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

test('macOS adaptive artwork declares a full-size intrinsic canvas', () => {
  for (const directory of [stableAdaptiveDirectory, betaAdaptiveDirectory]) {
    for (const fileName of ['01-artwork.svg', '01-artwork-dark.svg']) {
      const svg = readFileSync(path.join(directory, 'Assets', fileName), 'utf8');
      const root = svg.match(/<svg\b[^>]*>/)?.[0];
      assert.ok(root, `${fileName} must contain an SVG root element`);
      assert.match(root, /\bwidth="1024"/);
      assert.match(root, /\bheight="1024"/);
      assert.match(root, /\bviewBox="0 40 1024 1024"/);
      assert.match(
        svg,
        /<g id="fraia-glyph"[^>]+\btransform="translate\(512 552\) scale\(6\.1\) translate\(-64 -69\)"/,
      );
    }
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
  const start = svg.indexOf('<g id="fraia-glyph"');
  assert.notEqual(start, -1, 'icon source must contain the canonical Fraia glyph group');
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
  assert.fail('canonical Fraia glyph group must be closed');
}

async function opaqueBounds(filePath, size) {
  const source = await loadImage(filePath);
  const canvas = createCanvas(size, size);
  const context = canvas.getContext('2d');
  context.drawImage(source, 0, 0, size, size);
  const pixels = context.getImageData(0, 0, size, size).data;
  let left = size;
  let top = size;
  let right = -1;
  let bottom = -1;
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      if (pixels[((y * size) + x) * 4 + 3] < 32) continue;
      left = Math.min(left, x);
      top = Math.min(top, y);
      right = Math.max(right, x);
      bottom = Math.max(bottom, y);
    }
  }
  assert.notEqual(right, -1, `${path.basename(filePath)} must render opaque pixels`);
  return { bottom, height: bottom - top + 1, left, right, top, width: right - left + 1 };
}
