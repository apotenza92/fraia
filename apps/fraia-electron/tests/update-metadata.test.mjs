import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import YAML from 'yaml';

import { assembleUpdateMetadata } from '../scripts/assemble-update-metadata.mjs';
import { finalizeMacosUpdateArtifacts } from '../scripts/finalize-macos-update-artifacts.mjs';

test('post-stapling macOS metadata and blockmaps match final artifact bytes reproducibly', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fraia-finalize-macos-update-'));
  try {
    const arch = 'arm64';
    const artifacts = path.join(root, 'artifacts');
    await fs.mkdir(artifacts);
    const names = [`Fraia-macOS-${arch}.dmg`, `Fraia-macOS-${arch}.zip`];
    for (const name of names) await fs.writeFile(path.join(artifacts, name), `final ${name} bytes`);
    const metadataPath = path.join(artifacts, 'latest-mac.yml');
    await fs.writeFile(metadataPath, YAML.stringify({
      version: '0.0.2',
      files: names.map((url) => ({ url, sha512: 'stale', size: 1 })),
      path: names[1],
      sha512: 'stale',
    }));

    const first = await finalizeMacosUpdateArtifacts({ metadataPath, artifactDir: artifacts, arch });
    const firstMetadata = await fs.readFile(metadataPath);
    const firstBlockmaps = await Promise.all(names.map((name) => fs.readFile(path.join(artifacts, `${name}.blockmap`))));
    const parsed = YAML.parse(firstMetadata.toString());
    for (const file of parsed.files) {
      const artifact = await fs.readFile(path.join(artifacts, file.url));
      assert.equal(file.sha512, createHash('sha512').update(artifact).digest('base64'));
      assert.equal(file.size, artifact.length);
    }
    assert.equal(parsed.sha512, parsed.files.find((file) => file.url === parsed.path).sha512);

    const second = await finalizeMacosUpdateArtifacts({ metadataPath, artifactDir: artifacts, arch });
    assert.equal(second.metadataHash, first.metadataHash);
    assert.deepEqual(await fs.readFile(metadataPath), firstMetadata);
    for (let index = 0; index < names.length; index += 1) {
      assert.deepEqual(await fs.readFile(path.join(artifacts, `${names[index]}.blockmap`)), firstBlockmaps[index]);
    }
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('post-stapling beta metadata requires beta artifact names', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fraia-finalize-beta-macos-update-'));
  try {
    const arch = 'arm64';
    const names = [`Fraia-Beta-macOS-${arch}.dmg`, `Fraia-Beta-macOS-${arch}.zip`];
    for (const name of names) await fs.writeFile(path.join(root, name), `final ${name} bytes`);
    const metadataPath = path.join(root, 'latest-mac.yml');
    await fs.writeFile(metadataPath, YAML.stringify({
      version: '0.0.4-beta.1',
      files: names.map((url) => ({ url, sha512: 'stale', size: 1 })),
      path: names[1],
      sha512: 'stale',
    }));
    const result = await finalizeMacosUpdateArtifacts({
      metadataPath,
      artifactDir: root,
      arch,
      channel: 'beta',
    });
    assert.deepEqual(result.files, names.sort());
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('stable, beta, and stable-promoted beta packages write only their matching channel metadata', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fraia-update-metadata-'));
  try {
    const artifacts = path.join(root, 'artifacts');
    const output = path.join(root, 'feed');
    await fs.mkdir(artifacts);
    for (const fixture of [
      {
        channel: 'stable',
        tag: 'v0.0.1',
        version: '0.0.1',
        artifactName: 'Fraia-macOS-arm64.zip',
      },
      {
        channel: 'beta',
        tag: 'v0.0.2-beta.1',
        version: '0.0.2-beta.1',
        artifactName: 'Fraia-Beta-macOS-arm64.zip',
      },
      {
        channel: 'beta',
        tag: 'v0.0.2',
        version: '0.0.2',
        artifactName: 'Fraia-Beta-macOS-x64.zip',
        arch: 'x64',
      },
    ]) {
      const artifact = Buffer.from(`${fixture.channel} Fraia fixture`);
      await fs.writeFile(path.join(artifacts, fixture.artifactName), artifact);
      const input = path.join(root, `latest-mac-${fixture.channel}.yml`);
      await fs.writeFile(input, YAML.stringify({
        version: fixture.version,
        files: [{
          url: fixture.artifactName,
          sha512: createHash('sha512').update(artifact).digest('base64'),
          size: artifact.length,
        }],
        path: fixture.artifactName,
        sha512: createHash('sha512').update(artifact).digest('base64'),
        releaseName: `Fraia ${fixture.version}`,
        releaseNotes: `${fixture.channel} update notes.`,
        releaseDate: '2026-07-26T00:00:00.000Z',
      }));
      await assembleUpdateMetadata({
        input,
        artifactDir: artifacts,
        outputRoot: output,
        auditOutput: path.join(root, `update-${fixture.channel}-darwin-arm64.yml`),
        channel: fixture.channel,
        platform: 'darwin',
        arch: fixture.arch || 'arm64',
        tag: fixture.tag,
        repository: 'apotenza92/fraia',
      });
    }

    const stable = (await fs.readFile(path.join(output, 'stable', 'darwin', 'arm64', 'latest-mac.yml'))).toString();
    const beta = (await fs.readFile(path.join(output, 'beta', 'darwin', 'arm64', 'latest-mac.yml'))).toString();
    assert.notEqual(beta, stable);
    assert.match(
      stable,
      /https:\/\/github\.com\/apotenza92\/fraia\/releases\/download\/v0\.0\.1\/Fraia-macOS-arm64\.zip/,
    );
    assert.match(
      beta,
      /https:\/\/github\.com\/apotenza92\/fraia\/releases\/download\/v0\.0\.2-beta\.1\/Fraia-Beta-macOS-arm64\.zip/,
    );
    const promotedBeta = (await fs.readFile(path.join(output, 'beta', 'darwin', 'x64', 'latest-mac.yml'))).toString();
    assert.match(
      promotedBeta,
      /https:\/\/github\.com\/apotenza92\/fraia\/releases\/download\/v0\.0\.2\/Fraia-Beta-macOS-x64\.zip/,
    );
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test('update metadata rejects a tag whose release type does not match the channel', async () => {
  await assert.rejects(
    assembleUpdateMetadata({
      input: 'unused',
      artifactDir: 'unused',
      outputRoot: 'unused',
      auditOutput: 'unused',
      channel: 'stable',
      platform: 'darwin',
      arch: 'arm64',
      tag: 'v0.0.1-beta.1',
      repository: 'apotenza92/fraia',
    }),
    /Invalid stable release tag/,
  );
});
