const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { assembleReleaseAssets, expectedReleaseAssetNames } = require('../scripts/release-assets.cjs');

test('one stable release carries audit metadata for both update-feed projections', () => {
  const stable = expectedReleaseAssetNames('stable');
  assert.equal(stable.length, 30);
  assert.ok(stable.includes('Fraia-CalculiX-Corresponding-Source.tar'));
  assert.ok(stable.includes('Fraia-Windows-arm64-Setup.exe'));
  assert.ok(stable.includes('Fraia-Windows-x64-Setup.exe'));
  assert.ok(!stable.some((name) => name.endsWith('.AppImage.blockmap')));
  assert.ok(stable.every((name) => !name.includes('Beta')));
  for (const arch of ['arm64', 'x64']) {
    assert.ok(stable.includes(`update-stable-darwin-${arch}.yml`));
    assert.ok(stable.includes(`update-beta-darwin-${arch}.yml`));
  }
  assert.throws(() => expectedReleaseAssetNames('beta'), /stable releases only/);
});

test('release assembly rejects collisions and unexpected or missing assets', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-assets-'));
  const input = path.join(root, 'input');
  const output = path.join(root, 'output');
  fs.mkdirSync(input);
  for (const name of expectedReleaseAssetNames('stable').filter((name) => name !== 'SHA256SUMS')) {
    fs.writeFileSync(path.join(input, name), name);
  }
  assert.equal(assembleReleaseAssets('stable', [input], output).length, 30);
  assert.match(fs.readFileSync(path.join(output, 'SHA256SUMS'), 'utf8'), /Fraia-macOS-arm64\.dmg/);
  assert.throws(() => assembleReleaseAssets('stable', [input, input], output), /collision/);
  fs.writeFileSync(path.join(input, 'unexpected.txt'), 'no');
  assert.throws(() => assembleReleaseAssets('stable', [input], output), /Unexpected: unexpected\.txt/);
  fs.rmSync(root, { recursive: true, force: true });
});
