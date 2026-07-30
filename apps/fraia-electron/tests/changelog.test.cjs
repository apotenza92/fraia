const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const { readReleaseNotes, releaseNotesFromText } = require('../scripts/changelog.cjs');

const repositoryRoot = path.resolve(__dirname, '..', '..', '..');

test('current package version has authoritative non-empty release notes', () => {
  const packageMetadata = require('../package.json');
  const notes = readReleaseNotes({
    changelogPath: path.join(repositoryRoot, 'CHANGELOG.md'),
    version: packageMetadata.version,
  });
  assert.equal(notes.version, packageMetadata.version);
  assert.match(notes.body, /^### /);
  assert.match(notes.markdown, new RegExp(`^## Fraia ${packageMetadata.version}`));
});

test('release-note extraction is version-bounded and rejects missing or duplicate versions', () => {
  const source = [
    '# Changelog',
    '',
    '## [1.2.0] - 2026-07-28',
    '',
    '### Added',
    '',
    '- Current.',
    '',
    '## [1.1.0] - 2026-07-01',
    '',
    '### Fixed',
    '',
    '- Previous.',
    '',
  ].join('\n');
  const notes = releaseNotesFromText(source, '1.2.0');
  assert.match(notes.body, /Current/);
  assert.doesNotMatch(notes.body, /Previous/);
  assert.throws(() => releaseNotesFromText(source, '1.3.0'), /no release notes/);
  assert.throws(
    () => releaseNotesFromText(`${source}\n## [1.2.0] - 2026-07-29\n\n### Fixed\n\n- Duplicate.\n`, '1.2.0'),
    /Duplicate changelog version/,
  );
});

test('release-note extraction supports numbered beta prereleases', () => {
  const source = [
    '# Changelog',
    '',
    '## [1.2.0-beta.2] - 2026-07-30',
    '',
    '### Changed',
    '',
    '- Separate beta identity.',
    '',
    '## [1.1.0] - 2026-07-01',
    '',
    '### Fixed',
    '',
    '- Previous.',
    '',
  ].join('\n');
  const notes = releaseNotesFromText(source, '1.2.0-beta.2');
  assert.equal(notes.version, '1.2.0-beta.2');
  assert.match(notes.body, /Separate beta identity/);
});

test('electron-builder embeds the same current-version notes used by releases', () => {
  const packageMetadata = require('../package.json');
  const expected = readReleaseNotes({
    changelogPath: path.join(repositoryRoot, 'CHANGELOG.md'),
    version: packageMetadata.version,
  });
  const config = require('../electron-builder.config.cjs');
  const expectedProductName = packageMetadata.version.includes('-beta.') ? 'Fraia Beta' : 'Fraia';
  assert.equal(config.releaseInfo.releaseName, `${expectedProductName} ${packageMetadata.version}`);
  assert.equal(config.releaseInfo.releaseNotes, expected.body);
});
