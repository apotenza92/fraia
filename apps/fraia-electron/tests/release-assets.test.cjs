const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { assembleReleaseAssets, expectedReleaseAssetNames } = require('../scripts/release-assets.cjs');

test('channel assets stay isolated while a stable release can carry both application identities', () => {
  const stable = expectedReleaseAssetNames('stable');
  const beta = expectedReleaseAssetNames('beta');
  const stableInclusive = expectedReleaseAssetNames(['stable', 'beta']);
  assert.equal(stable.length, 32);
  assert.equal(beta.length, 32);
  assert.equal(stableInclusive.length, 62);
  assert.ok(stable.includes('Fraia-CalculiX-Corresponding-Source.tar'));
  assert.ok(stable.includes('Fraia-Windows-arm64-Setup.exe'));
  assert.ok(stable.includes('Fraia-Windows-x64-Setup.exe'));
  assert.ok(!stable.some((name) => name.endsWith('.AppImage.blockmap')));
  assert.ok(stable.every((name) => !name.includes('Beta')));
  assert.ok(beta.includes('Fraia-Beta-Windows-arm64-Setup.exe'));
  assert.ok(beta.includes('Fraia-Beta-Windows-x64-Setup.exe'));
  assert.ok(beta.filter((name) => /\.(?:dmg|zip|exe|AppImage|deb|rpm)(?:\.blockmap|\.sha256)?$/.test(name))
    .every((name) => name.startsWith('Fraia-Beta-')));
  for (const arch of ['arm64', 'x64']) {
    assert.ok(stable.includes(`update-stable-darwin-${arch}.yml`));
    assert.ok(stable.includes(`update-stable-win32-${arch}.yml`));
    assert.ok(stable.includes(`update-stable-linux-${arch}.yml`));
    assert.ok(beta.includes(`update-beta-darwin-${arch}.yml`));
    assert.ok(beta.includes(`update-beta-win32-${arch}.yml`));
    assert.ok(beta.includes(`update-beta-linux-${arch}.yml`));
  }
  assert.ok(stableInclusive.includes('Fraia-macOS-arm64.dmg'));
  assert.ok(stableInclusive.includes('Fraia-Beta-macOS-arm64.dmg'));
  assert.equal(stableInclusive.filter((name) => name === 'Fraia-CalculiX-Corresponding-Source.tar').length, 1);
  assert.equal(stableInclusive.filter((name) => name === 'SHA256SUMS').length, 1);
  assert.throws(() => expectedReleaseAssetNames('nightly'), /stable or beta/);
});

test('release assembly rejects collisions and unexpected or missing assets', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-assets-'));
  const input = path.join(root, 'input');
  const output = path.join(root, 'output');
  fs.mkdirSync(input);
  for (const name of expectedReleaseAssetNames('stable').filter((name) => name !== 'SHA256SUMS')) {
    fs.writeFileSync(path.join(input, name), name);
  }
  assert.equal(assembleReleaseAssets('stable', [input], output).length, 32);
  assert.match(fs.readFileSync(path.join(output, 'SHA256SUMS'), 'utf8'), /Fraia-macOS-arm64\.dmg/);
  assert.throws(() => assembleReleaseAssets('stable', [input, input], output), /collision/);
  fs.writeFileSync(path.join(input, 'unexpected.txt'), 'no');
  assert.throws(() => assembleReleaseAssets('stable', [input], output), /Unexpected: unexpected\.txt/);
  fs.rmSync(root, { recursive: true, force: true });
});

test('stable-inclusive release assembly accepts both identities and one shared provenance set', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-stable-inclusive-assets-'));
  const input = path.join(root, 'input');
  const output = path.join(root, 'output');
  fs.mkdirSync(input);

  for (const name of expectedReleaseAssetNames(['stable', 'beta']).filter((name) => name !== 'SHA256SUMS')) {
    fs.writeFileSync(path.join(input, name), name);
  }

  const assembled = assembleReleaseAssets(['stable', 'beta'], [input], output);
  assert.equal(assembled.length, 62);
  assert.ok(fs.existsSync(path.join(output, 'Fraia-macOS-arm64.dmg')));
  assert.ok(fs.existsSync(path.join(output, 'Fraia-Beta-macOS-arm64.dmg')));
  assert.equal(
    assembled.filter((assetPath) => path.basename(assetPath) === 'Fraia-CalculiX-Corresponding-Source.tar').length,
    1,
  );

  fs.rmSync(root, { recursive: true, force: true });
});
