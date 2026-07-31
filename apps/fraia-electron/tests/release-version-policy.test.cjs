const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const {
  FEED_METADATA,
  compareVersions,
  readPublishedChannelVersion,
  releasePolicy,
} = require('../release-version-policy.cjs');

test('Fraia release versions use stable-inclusive Semantic Versioning precedence', () => {
  assert.equal(compareVersions('1.0.0-beta.1', '1.0.0-beta.2'), -1);
  assert.equal(compareVersions('1.0.0-beta.2', '1.0.0'), -1);
  assert.equal(compareVersions('1.0.0', '1.1.0-beta.1'), -1);
  assert.equal(compareVersions('1.1.0-beta.1', '1.0.1'), 1);
  assert.equal(compareVersions('1.0.0', '1.0.0'), 0);
  assert.throws(() => compareVersions('1.0.0-rc.1', '1.0.0'), /Invalid Fraia semantic version/);
  assert.throws(() => compareVersions('01.0.0', '1.0.0'), /Invalid Fraia semantic version/);
  assert.throws(() => compareVersions('1.0.0-beta.01', '1.0.0'), /Invalid Fraia semantic version/);
});

test('a stable tag advances beta only when the final version is the newest beta candidate', () => {
  assert.deepEqual(
    releasePolicy({
      tagChannel: 'stable',
      candidateVersion: '1.0.0',
      currentStableVersion: '0.9.1',
      currentBetaVersion: '1.0.0-beta.2',
    }),
    {
      channels: ['stable', 'beta'],
      previousBetaVersion: '1.0.0-beta.2',
      previousStableVersion: '0.9.1',
      promotesBeta: true,
    },
  );
  assert.deepEqual(
    releasePolicy({
      tagChannel: 'stable',
      candidateVersion: '1.0.1',
      currentStableVersion: '1.0.0',
      currentBetaVersion: '1.1.0-beta.1',
    }),
    {
      channels: ['stable'],
      previousBetaVersion: '1.1.0-beta.1',
      previousStableVersion: '1.0.0',
      promotesBeta: false,
    },
  );
});

test('beta tags advance from either a final or pre-release beta-identity package without downgrading', () => {
  assert.deepEqual(
    releasePolicy({
      tagChannel: 'beta',
      candidateVersion: '1.1.0-beta.1',
      currentStableVersion: '1.0.0',
      currentBetaVersion: '1.0.0',
    }),
    {
      channels: ['beta'],
      previousBetaVersion: '1.0.0',
      previousStableVersion: '1.0.0',
      promotesBeta: false,
    },
  );
  assert.throws(
    () => releasePolicy({
      tagChannel: 'beta',
      candidateVersion: '1.0.1-beta.1',
      currentBetaVersion: '1.1.0-beta.1',
    }),
    /must be newer/,
  );
  assert.throws(
    () => releasePolicy({
      tagChannel: 'stable',
      candidateVersion: '1.0.0',
      currentStableVersion: '1.0.0',
    }),
    /must be newer/,
  );
});

test('all native feed documents must agree before release policy is resolved', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-release-feed-'));
  try {
    assert.equal(readPublishedChannelVersion(root, 'beta'), null);
    for (const [platform, arch, name] of FEED_METADATA) {
      const directory = path.join(root, 'beta', platform, arch);
      fs.mkdirSync(directory, { recursive: true });
      fs.writeFileSync(path.join(directory, name), 'version: 1.0.0-beta.2\n');
    }
    assert.equal(readPublishedChannelVersion(root, 'beta'), '1.0.0-beta.2');
    fs.writeFileSync(path.join(root, 'beta', 'win32', 'x64', 'latest.yml'), 'version: 1.0.0\n');
    assert.throws(() => readPublishedChannelVersion(root, 'beta'), /versions disagree/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
