const assert = require('node:assert/strict');
const path = require('node:path');
const test = require('node:test');

const {
  resolvePlaywrightArguments,
  resolveTargetDirectory,
} = require('../scripts/run-electron-tests.cjs');

test('Electron test runner always keeps the project Playwright configuration', () => {
  assert.deepEqual(resolvePlaywrightArguments([]), [
    'test',
    '--config',
    'playwright.electron.config.ts',
  ]);
  assert.deepEqual(resolvePlaywrightArguments([
    'tests/electron/conversation-first.spec.ts',
    '--repeat-each=3',
  ]), [
    'test',
    '--config',
    'playwright.electron.config.ts',
    'tests/electron/conversation-first.spec.ts',
    '--repeat-each=3',
  ]);
});

test('Electron test runner resolves Cargo output from the repository root', () => {
  const repoRoot = path.resolve('/source/fraia');
  assert.equal(resolveTargetDirectory(repoRoot), path.join(repoRoot, 'target'));
  assert.equal(resolveTargetDirectory(repoRoot, '.test-target'), path.join(repoRoot, '.test-target'));
  assert.equal(resolveTargetDirectory(repoRoot, '/tmp/fraia-target'), path.resolve('/tmp/fraia-target'));
});
