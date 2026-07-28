import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import YAML from 'yaml';

import { assembleUpdateMetadata } from '../scripts/assemble-update-metadata.mjs';

test('one stable package projects byte-identical metadata to stable and beta feeds', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fraia-update-metadata-'));
  try {
    const artifacts = path.join(root, 'artifacts');
    const output = path.join(root, 'feed');
    await fs.mkdir(artifacts);
    const artifactName = 'Fraia-macOS-arm64.zip';
    const artifactPath = path.join(artifacts, artifactName);
    const artifact = Buffer.from('stable Fraia 0.0.1 fixture');
    await fs.writeFile(artifactPath, artifact);
    const input = path.join(root, 'latest-mac.yml');
    await fs.writeFile(input, YAML.stringify({
      version: '0.0.1',
      files: [{
        url: artifactName,
        sha512: createHash('sha512').update(artifact).digest('base64'),
        size: artifact.length,
      }],
      path: artifactName,
      sha512: createHash('sha512').update(artifact).digest('base64'),
      releaseName: 'Fraia 0.0.1',
      releaseNotes: '### Added\n\n- Stable update notes.',
      releaseDate: '2026-07-26T00:00:00.000Z',
    }));

    for (const channel of ['stable', 'beta']) {
      await assembleUpdateMetadata({
        input,
        artifactDir: artifacts,
        outputRoot: output,
        auditOutput: path.join(root, `update-${channel}-darwin-arm64.yml`),
        channel,
        platform: 'darwin',
        arch: 'arm64',
        tag: 'v0.0.1',
        repository: 'apotenza92/fraia',
      });
    }

    const stable = await fs.readFile(path.join(output, 'stable', 'darwin', 'arm64', 'latest-mac.yml'));
    const beta = await fs.readFile(path.join(output, 'beta', 'darwin', 'arm64', 'latest-mac.yml'));
    assert.deepEqual(beta, stable);
    assert.match(
      stable.toString(),
      /https:\/\/github\.com\/apotenza92\/fraia\/releases\/download\/v0\.0\.1\/Fraia-macOS-arm64\.zip/,
    );
    assert.match(stable.toString(), /Stable update notes/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('update metadata rejects prerelease tags', async () => {
  await assert.rejects(
    assembleUpdateMetadata({
      input: 'unused',
      artifactDir: 'unused',
      outputRoot: 'unused',
      auditOutput: 'unused',
      channel: 'beta',
      platform: 'darwin',
      arch: 'arm64',
      tag: 'v0.0.1-beta.1',
      repository: 'apotenza92/fraia',
    }),
    /Invalid stable release tag/,
  );
});
