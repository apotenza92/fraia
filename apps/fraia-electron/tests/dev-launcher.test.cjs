const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const {
  acquireLaunchLock,
  releaseLaunchLock,
  resolveLaunchConfig,
} = require('../scripts/start-dev.cjs');

test('development launcher uses one strict, identifiable server configuration', () => {
  const lockPath = path.join(os.tmpdir(), 'fraia-launcher-config-test.lock');
  assert.deepEqual(resolveLaunchConfig([], {
    FRAIA_DEV_LOCK_PATH: lockPath,
    FRAIA_DEV_SERVER_PORT: '5187',
  }), {
    clean: false,
    freshGuide: false,
    host: '127.0.0.1',
    port: 5187,
    serverUrl: 'http://127.0.0.1:5187',
    lockPath,
  });
  assert.equal(resolveLaunchConfig(['--clean', '--fresh-guide'], { FRAIA_DEV_LOCK_PATH: lockPath }).clean, true);
  assert.throws(
    () => resolveLaunchConfig([], { FRAIA_DEV_LOCK_PATH: lockPath, FRAIA_DEV_SERVER_PORT: '80' }),
    /between 1024 and 65535/,
  );
});

test('development launcher rejects a second live owner and recovers a stale lock', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-launch-lock-test-'));
  const lockPath = path.join(root, 'fraia.lock');
  try {
    acquireLaunchLock(lockPath, process.pid);
    assert.throws(() => acquireLaunchLock(lockPath, process.pid), /already running/);
    releaseLaunchLock(lockPath, process.pid);
    assert.equal(fs.existsSync(lockPath), false);

    fs.writeFileSync(lockPath, '{"pid":99999999}\n');
    acquireLaunchLock(lockPath, process.pid);
    assert.equal(JSON.parse(fs.readFileSync(lockPath, 'utf8')).pid, process.pid);
  } finally {
    releaseLaunchLock(lockPath, process.pid);
    fs.rmSync(root, { recursive: true, force: true });
  }
});
