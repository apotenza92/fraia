const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const identities = {
  stable: { token: 'fraia', asset: 'Fraia-macOS', app: 'Fraia.app', bundleId: 'app.fraia.desktop', name: 'Fraia', desc: 'AI-assisted structural engineering design and analysis', userData: 'Fraia' },
  beta: { token: 'fraia@beta', asset: 'Fraia-Beta-macOS', app: 'Fraia Beta.app', bundleId: 'app.fraia.desktop.beta', name: 'Fraia Beta', desc: 'Beta channel for Fraia structural engineering design and analysis', userData: 'Fraia Beta' },
};

function digest(file) { return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'); }

function renderCask(identity, version, tag, assetsDirectory) {
  const arm = `${identity.asset}-arm64.zip`;
  const intel = `${identity.asset}-x64.zip`;
  return `cask "${identity.token}" do
  version "${version}"

  on_arm do
    sha256 "${digest(path.join(assetsDirectory, arm))}"

    url "https://github.com/apotenza92/fraia/releases/download/v#{version}/${arm}"
  end
  on_intel do
    sha256 "${digest(path.join(assetsDirectory, intel))}"

    url "https://github.com/apotenza92/fraia/releases/download/v#{version}/${intel}"
  end

  name "${identity.name}"
  desc "${identity.desc}"
  homepage "https://github.com/apotenza92/fraia"

  livecheck do
    skip "Updated by trusted Fraia release automation"
  end

  auto_updates true
  depends_on macos: :sequoia

  app "${identity.app}"

  zap trash: [
    "~/Library/Application Support/${identity.userData}",
    "~/Library/Caches/${identity.bundleId}",
    "~/Library/Caches/${identity.bundleId}.ShipIt",
    "~/Library/Preferences/${identity.bundleId}.plist",
    "~/Library/Saved Application State/${identity.bundleId}.savedState",
  ]
end
`;
}

function buildPublication({ channel, tag, commit, runId, runAttempt, assetsDirectory, outputDirectory }) {
  if (!['stable', 'beta'].includes(channel) || !/^v\d+\.\d+\.\d+(?:-beta\.[1-9]\d*)?$/.test(tag) || !/^[0-9a-f]{40}$/.test(commit)) throw new Error('Invalid release identity');
  if ((channel === 'beta') !== tag.includes('-beta.')) throw new Error('Channel and tag disagree');
  const version = tag.slice(1);
  const channels = channel === 'stable' ? ['stable', 'beta'] : ['beta'];
  fs.mkdirSync(path.join(outputDirectory, 'Casks'), { recursive: true });
  const artifacts = [];
  const casks = [];
  for (const publicationChannel of channels) {
    const identity = identities[publicationChannel];
    const filename = `${identity.token}.rb`;
    casks.push(filename);
    fs.writeFileSync(path.join(outputDirectory, 'Casks', filename), renderCask(identity, version, tag, assetsDirectory));
    for (const architecture of ['arm64', 'x64']) {
      const name = `${identity.asset}-${architecture}.zip`;
      const file = path.join(assetsDirectory, name);
      const size = fs.statSync(file).size;
      if (size <= 0) throw new Error(`Empty release asset: ${name}`);
      artifacts.push({ name, url: `https://github.com/apotenza92/fraia/releases/download/${tag}/${name}`, size, sha256: digest(file), channel: publicationChannel, architecture });
    }
  }
  const manifest = {
    schema_version: 1, product: 'fraia', source_repository: 'apotenza92/fraia', release_tag: tag, release_commit: commit,
    channel, casks, artifacts,
    applications: Object.fromEntries(channels.map(value => [value, identities[value].app])),
    bundle_identifiers: Object.fromEntries(channels.map(value => [value, identities[value].bundleId])),
    architectures: ['arm64', 'x64'], minimum_macos: '15.0',
    native_validation: { workflow_run_id: Number(runId), workflow_run_attempt: Number(runAttempt), jobs: channels.map(value => `Homebrew ${value} (\${{ matrix.architecture }})`) },
  };
  fs.writeFileSync(path.join(outputDirectory, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  const checksumPaths = ['manifest.json', ...casks.map(name => `Casks/${name}`)].sort();
  fs.writeFileSync(path.join(outputDirectory, 'SHA256SUMS'), checksumPaths.map(name => `${digest(path.join(outputDirectory, name))}  ${name}`).join('\n') + '\n');
  return manifest;
}

function option(name) { const index = process.argv.indexOf(name); if (index < 0 || !process.argv[index + 1]) throw new Error(`Missing ${name}`); return process.argv[index + 1]; }
if (require.main === module) buildPublication({ channel: option('--channel'), tag: option('--tag'), commit: process.env.GITHUB_SHA, runId: process.env.GITHUB_RUN_ID, runAttempt: process.env.GITHUB_RUN_ATTEMPT, assetsDirectory: path.resolve(option('--assets')), outputDirectory: path.resolve(option('--output')) });

module.exports = { buildPublication, renderCask };
