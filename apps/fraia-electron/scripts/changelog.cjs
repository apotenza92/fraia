#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const VERSION_HEADING = /^## \[(\d+\.\d+\.\d+)\] - (\d{4}-\d{2}-\d{2})$/gm;

function releaseNotesFromText(source, version) {
  if (typeof source !== 'string') throw new Error('Changelog contents must be text.');
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw new Error(`Invalid release version: ${version}`);

  const matches = [...source.matchAll(VERSION_HEADING)];
  const versions = matches.map((match) => match[1]);
  const duplicates = versions.filter((candidate, index) => versions.indexOf(candidate) !== index);
  if (duplicates.length) throw new Error(`Duplicate changelog version: ${duplicates[0]}`);

  const matchIndex = matches.findIndex((match) => match[1] === version);
  if (matchIndex === -1) throw new Error(`CHANGELOG.md has no release notes for ${version}.`);
  const match = matches[matchIndex];
  const next = matches[matchIndex + 1];
  const body = source.slice(match.index + match[0].length, next?.index ?? source.length).trim();
  if (!body) throw new Error(`CHANGELOG.md release notes for ${version} are empty.`);
  if (!/^### /m.test(body)) {
    throw new Error(`CHANGELOG.md release notes for ${version} require at least one category.`);
  }

  return {
    body,
    date: match[2],
    markdown: `## Fraia ${version}\n\n${body}\n`,
    version,
  };
}

function readReleaseNotes({ changelogPath, version }) {
  const resolved = path.resolve(changelogPath);
  return releaseNotesFromText(fs.readFileSync(resolved, 'utf8'), version);
}

function parseArguments(argv) {
  const values = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith('--') || value === undefined) {
      throw new Error('Arguments must use --name value pairs.');
    }
    values[key.slice(2)] = value;
  }
  return values;
}

function main(argv = process.argv.slice(2)) {
  const args = parseArguments(argv);
  if (!args.version) throw new Error('Missing --version.');
  const changelogPath = args.changelog
    ? path.resolve(args.changelog)
    : path.resolve(__dirname, '..', '..', '..', 'CHANGELOG.md');
  const notes = readReleaseNotes({ changelogPath, version: args.version });
  if (args.output) {
    const output = path.resolve(args.output);
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, notes.markdown, { mode: 0o644 });
  } else {
    process.stdout.write(notes.markdown);
  }
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}

module.exports = {
  readReleaseNotes,
  releaseNotesFromText,
};
