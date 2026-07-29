const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const test = require('node:test');
const {
  BUILD_RECIPES,
  CALCULIX_SOURCE_ASSET_NAME,
  SOURCE_INPUTS,
  correspondingSourceUrl,
} = require('../calculix-source-contract.cjs');
const { writeDeterministicTar } = require('../scripts/assemble-calculix-corresponding-source.cjs');
const { verifyCorrespondingSource } = require('../scripts/verify-calculix-corresponding-source.cjs');
const { SUPPORTED_TARGETS } = require('../package-boundary.cjs');
const packageMetadata = require('../package.json');

const electronRoot = path.resolve(__dirname, '..');

function digest(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

test('corresponding-source contract covers every runtime platform with pinned public inputs', () => {
  assert.equal(CALCULIX_SOURCE_ASSET_NAME, 'Fraia-CalculiX-Corresponding-Source.tar');
  assert.equal(
    correspondingSourceUrl('apotenza92/fraia', 'v0.0.1'),
    'https://github.com/apotenza92/fraia/releases/download/v0.0.1/Fraia-CalculiX-Corresponding-Source.tar',
  );
  assert.deepEqual(
    BUILD_RECIPES.map(({ platform }) => platform),
    ['darwin', 'linux', 'win32', 'win32-arm64'],
  );
  assert.equal(new Set(SOURCE_INPUTS.map(({ fileName }) => fileName)).size, SOURCE_INPUTS.length);
  for (const source of SOURCE_INPUTS) {
    assert.match(source.url, /^https:\/\//);
    assert.match(source.sha256, /^[a-f0-9]{64}$/);
    assert.ok(source.usedBy.length > 0);
    for (const platform of source.usedBy) {
      const recipe = BUILD_RECIPES.find((candidate) => candidate.platform === platform);
      const script = fs.readFileSync(path.join(electronRoot, recipe.path), 'utf8');
      assert.match(script, new RegExp(source.sha256), `${source.fileName} must remain pinned in ${recipe.path}`);
    }
  }
});

test('reviewed runtime manifests target the current stable release source asset', () => {
  const releaseTag = `v${packageMetadata.version}`;
  const expectedSourceUrl = correspondingSourceUrl('apotenza92/fraia', releaseTag);
  const sourceDigests = new Set();
  for (const target of SUPPORTED_TARGETS) {
    const manifest = JSON.parse(fs.readFileSync(
      path.join(electronRoot, 'runtimes', 'calculix', target, 'runtime-manifest.json'),
      'utf8',
    ));
    assert.equal(manifest.redistribution.sourceUrl, expectedSourceUrl);
    assert.match(manifest.redistribution.sourceSha256, /^[a-f0-9]{64}$/);
    sourceDigests.add(manifest.redistribution.sourceSha256);
  }
  assert.equal(sourceDigests.size, 1);
});

test('tar writer produces byte-identical archives with fixed metadata', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-source-tar-'));
  const input = path.join(root, 'input.txt');
  const first = path.join(root, 'first.tar');
  const second = path.join(root, 'second.tar');
  fs.writeFileSync(input, 'reviewed source\n');
  const entries = [
    { name: 'bundle/z.txt', data: Buffer.from('z\n') },
    { name: 'bundle/a/input.txt', filePath: input },
  ];
  writeDeterministicTar(first, entries);
  writeDeterministicTar(second, [...entries].reverse());
  assert.equal(digest(first), digest(second));
  const listing = spawnSync('tar', ['-tf', first], { encoding: 'utf8' });
  assert.equal(listing.status, 0, listing.stderr);
  assert.deepEqual(
    listing.stdout.trim().split(/\r?\n/),
    ['bundle/', 'bundle/a/', 'bundle/a/input.txt', 'bundle/z.txt'],
  );
  fs.rmSync(root, { recursive: true, force: true });
});

test('one corresponding-source byte stream must match every runtime manifest', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-source-manifests-'));
  const bundlePath = path.join(root, CALCULIX_SOURCE_ASSET_NAME);
  const runtimeRoot = path.join(root, 'runtimes');
  fs.writeFileSync(bundlePath, 'reviewed corresponding source\n');
  const sourceSha256 = digest(bundlePath);
  const sourceUrl = correspondingSourceUrl('apotenza92/fraia', 'v0.0.1');
  for (const target of SUPPORTED_TARGETS) {
    const directory = path.join(runtimeRoot, target);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(
      path.join(directory, 'runtime-manifest.json'),
      `${JSON.stringify({ redistribution: { sourceSha256, sourceUrl } }, null, 2)}\n`,
    );
  }
  assert.deepEqual(
    verifyCorrespondingSource({
      bundlePath,
      repository: 'apotenza92/fraia',
      runtimeRoot,
      tag: 'v0.0.1',
    }),
    { sha256: sourceSha256, sourceUrl },
  );
  fs.appendFileSync(path.join(runtimeRoot, 'darwin-arm64', 'runtime-manifest.json'), 'changed');
  assert.throws(
    () => verifyCorrespondingSource({
      bundlePath,
      repository: 'apotenza92/fraia',
      runtimeRoot,
      tag: 'v0.0.1',
    }),
    /JSON/,
  );
  fs.rmSync(root, { recursive: true, force: true });
});
