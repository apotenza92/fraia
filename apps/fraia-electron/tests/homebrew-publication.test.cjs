const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { buildPublication } = require('../scripts/build-homebrew-publication.cjs');

function fixture(channel = 'stable') {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-homebrew-'));
  const assets = path.join(root, 'assets'); const output = path.join(root, 'output');
  fs.mkdirSync(assets); fs.mkdirSync(output);
  const prefixes = channel === 'stable' ? ['Fraia-macOS', 'Fraia-Beta-macOS'] : ['Fraia-Beta-macOS'];
  for (const prefix of prefixes) for (const arch of ['arm64', 'x64']) fs.writeFileSync(path.join(assets, `${prefix}-${arch}.zip`), `${prefix}-${arch}`);
  return { root, assets, output };
}

test('stable publication contains both isolated identities and exact local digests', () => {
  const value = fixture();
  const manifest = buildPublication({ channel: 'stable', tag: 'v1.2.3', commit: 'a'.repeat(40), runId: '42', runAttempt: '2', assetsDirectory: value.assets, outputDirectory: value.output });
  assert.deepEqual(manifest.casks, ['fraia.rb', 'fraia@beta.rb']);
  assert.deepEqual(manifest.applications, { stable: 'Fraia.app', beta: 'Fraia Beta.app' });
  assert.equal(manifest.minimum_macos, '15.0');
  for (const artifact of manifest.artifacts) assert.equal(artifact.sha256, crypto.createHash('sha256').update(fs.readFileSync(path.join(value.assets, artifact.name))).digest('hex'));
  assert.match(fs.readFileSync(path.join(value.output, 'Casks/fraia@beta.rb'), 'utf8'), /app "Fraia Beta\.app"/);
  assert.equal(fs.readFileSync(path.join(value.output, 'SHA256SUMS'), 'utf8').trim().split('\n').length, 3);
});

test('beta publication cannot touch the stable cask', () => {
  const value = fixture('beta');
  const manifest = buildPublication({ channel: 'beta', tag: 'v1.2.3-beta.4', commit: 'b'.repeat(40), runId: '1', runAttempt: '1', assetsDirectory: value.assets, outputDirectory: value.output });
  assert.deepEqual(manifest.casks, ['fraia@beta.rb']);
  assert.equal(fs.existsSync(path.join(value.output, 'Casks/fraia.rb')), false);
});

test('rejects a channel and tag mismatch', () => {
  const value = fixture();
  assert.throws(() => buildPublication({ channel: 'beta', tag: 'v1.2.3', commit: 'c'.repeat(40), runId: '1', runAttempt: '1', assetsDirectory: value.assets, outputDirectory: value.output }), /disagree/);
});
