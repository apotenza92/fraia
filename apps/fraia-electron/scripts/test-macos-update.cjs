#!/usr/bin/env node

const { spawn, spawnSync } = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');
const YAML = require('yaml');
const { releaseContract } = require('../release-contract.cjs');
const { extractedZip, normalizeFingerprint, verifyApp } = require('./verify-macos-package.cjs');

function option(argv, name, fallback = null) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : fallback;
}

function run(command, args, { cwd, env = process.env, capture = false } = {}) {
  const result = spawnSync(command, args, { cwd, env, encoding: 'utf8', stdio: capture ? ['ignore', 'pipe', 'pipe'] : 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed with status ${result.status}: ${result.stderr || ''}`);
  return result;
}

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function sha512(filePath) {
  return crypto.createHash('sha512').update(fs.readFileSync(filePath)).digest('base64');
}

function verifyPublishedChecksum(zipPath, checksumsPath) {
  const expected = new Map(fs.readFileSync(checksumsPath, 'utf8').trim().split('\n').map((line) => {
    const match = line.match(/^([a-f0-9]{64})\s+\*?(.+)$/i);
    if (!match) throw new Error(`Invalid SHA256SUMS line: ${line}`);
    return [match[2], match[1].toLowerCase()];
  }));
  const name = path.basename(zipPath);
  if (!expected.has(name) || expected.get(name) !== sha256(zipPath)) throw new Error(`Published SHA-256 does not verify ${name}.`);
}

function restrictedEnvironment(overrides) {
  const allowed = ['CI', 'HOME', 'LANG', 'LC_ALL', 'LOGNAME', 'PATH', 'SHELL', 'TEMP', 'TMP', 'TMPDIR', 'USER'];
  return Object.fromEntries([...allowed.flatMap((name) => process.env[name] ? [[name, process.env[name]]] : []), ...Object.entries(overrides)]);
}

function prepareScenarioAssets({ candidateDirectory, metadataPath, scenario, root }) {
  const assets = path.join(root, 'feed');
  fs.mkdirSync(assets);
  for (const entry of fs.readdirSync(candidateDirectory, { withFileTypes: true })) {
    if (entry.isFile()) fs.copyFileSync(path.join(candidateDirectory, entry.name), path.join(assets, entry.name));
  }
  const metadata = YAML.parse(fs.readFileSync(metadataPath, 'utf8'));
  if (!metadata?.version || !Array.isArray(metadata.files) || metadata.files.length === 0) throw new Error('Candidate macOS metadata has no files.');
  const zipFiles = metadata.files.filter((candidate) => decodeURIComponent(path.posix.basename(new URL(candidate.url, 'https://local.invalid/').pathname)).endsWith('.zip'));
  if (zipFiles.length !== 1) throw new Error('Candidate macOS metadata must identify exactly one ZIP.');
  const file = zipFiles[0];
  const zipName = decodeURIComponent(path.posix.basename(new URL(file.url, 'https://local.invalid/').pathname));
  const zipPath = path.join(assets, zipName);
  if (!fs.existsSync(zipPath) || !zipName.endsWith('.zip')) throw new Error('Candidate updater ZIP is missing.');

  if (scenario === 'corrupt') {
    fs.appendFileSync(zipPath, 'corrupt-update-payload');
  } else if (scenario === 'signature') {
    const extraction = path.join(root, 'signature-extraction');
    fs.mkdirSync(extraction);
    run('ditto', ['-x', '-k', zipPath, extraction]);
    const app = fs.readdirSync(extraction).find((name) => name.endsWith('.app'));
    if (!app) throw new Error('Candidate ZIP did not contain an app.');
    const sidecar = path.join(extraction, app, 'Contents', 'Resources', 'sidecar');
    const executable = (() => {
      const matches = [];
      function walk(directory) {
        for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
          const candidate = path.join(directory, entry.name);
          if (entry.isDirectory()) walk(candidate);
          else if (entry.name === 'fraia-appd') matches.push(candidate);
        }
      }
      walk(sidecar);
      if (matches.length !== 1) throw new Error('Candidate ZIP has an unexpected sidecar layout.');
      return matches[0];
    })();
    fs.appendFileSync(executable, 'invalidate-developer-id-signature');
    fs.rmSync(zipPath);
    run('ditto', ['-c', '-k', '--keepParent', path.join(extraction, app), zipPath]);
    file.sha512 = sha512(zipPath);
    file.size = fs.statSync(zipPath).size;
    metadata.sha512 = file.sha512;
    if (metadata.path) metadata.path = zipName;
  }
  return { assets, metadata, version: metadata.version, zipName };
}

function serveFeed(assets, metadata) {
  const requests = [];
  const server = http.createServer((request, response) => {
    const requested = decodeURIComponent(new URL(request.url, 'http://127.0.0.1').pathname.slice(1));
    requests.push(requested);
    if (requested === 'latest-mac.yml') {
      const contents = Buffer.from(YAML.stringify(metadata, { lineWidth: 0 }));
      response.writeHead(200, { 'content-type': 'application/yaml', 'content-length': contents.length });
      response.end(contents);
      return;
    }
    const safeName = path.posix.basename(requested);
    const target = path.join(assets, safeName);
    if (safeName !== requested || !fs.existsSync(target)) {
      response.writeHead(404); response.end(); return;
    }
    const bytes = fs.readFileSync(target);
    const range = request.headers.range?.match(/^bytes=(\d+)-(\d*)$/);
    if (range) {
      const start = Number(range[1]);
      const end = range[2] ? Number(range[2]) : bytes.length - 1;
      response.writeHead(206, {
        'accept-ranges': 'bytes', 'content-range': `bytes ${start}-${end}/${bytes.length}`,
        'content-length': end - start + 1,
      });
      response.end(bytes.subarray(start, end + 1));
    } else {
      response.writeHead(200, { 'accept-ranges': 'bytes', 'content-length': bytes.length });
      response.end(bytes);
    }
  });
  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      const base = `http://127.0.0.1:${port}`;
      for (const file of metadata.files) {
        const name = decodeURIComponent(path.posix.basename(new URL(file.url, 'https://local.invalid/').pathname));
        file.url = `${base}/${encodeURIComponent(name)}`;
      }
      if (metadata.path) {
        const name = decodeURIComponent(path.posix.basename(new URL(metadata.path, 'https://local.invalid/').pathname));
        metadata.path = `${base}/${encodeURIComponent(name)}`;
      }
      resolve({ base, requests, close: () => new Promise((done) => server.close(done)) });
    });
  });
}

function readEventHistory(eventPath) {
  const historyPath = `${eventPath}.jsonl`;
  if (!fs.existsSync(historyPath)) return [];
  return fs.readFileSync(historyPath, 'utf8').trim().split('\n').filter(Boolean).map((line) => JSON.parse(line));
}

async function waitForEvent(eventPath, accepted, timeoutMs = 180_000, predicate = () => true) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    for (const event of readEventHistory(eventPath)) {
      if (accepted.has(event.name) && predicate(event)) return event;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for updater event: ${[...accepted].join(', ')}.`);
}

function isPidAlive(pid) {
  if (!Number.isInteger(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error?.code === 'EPERM';
  }
}

async function stopPid(pid) {
  if (!isPidAlive(pid)) return;
  process.kill(pid, 'SIGTERM');
  const started = Date.now();
  while (Date.now() - started < 10_000 && isPidAlive(pid)) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (isPidAlive(pid)) throw new Error(`Updater process ${pid} did not exit after SIGTERM.`);
}

async function waitForPidExit(pid, timeoutMs = 180_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (!isPidAlive(pid)) return;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Updater process ${pid} did not exit for installation.`);
}

function bundleVersion(appPath) {
  const result = run('/usr/bin/plutil', [
    '-extract',
    'CFBundleShortVersionString',
    'raw',
    '-o',
    '-',
    path.join(appPath, 'Contents', 'Info.plist'),
  ], { capture: true });
  return result.stdout.trim();
}

async function waitForBundleVersion(appPath, expectedVersion, timeoutMs = 180_000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    try {
      if (bundleVersion(appPath) === expectedVersion) return;
    } catch { /* Squirrel may be atomically replacing the bundle */ }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for Squirrel.Mac to install ${expectedVersion}.`);
}

async function waitForVerifiedApp(appPath, contract, expectations, expectedVersion, timeoutMs = 180_000) {
  const started = Date.now();
  let lastError;
  while (Date.now() - started < timeoutMs) {
    try {
      const verified = verifyApp(appPath, contract, expectations);
      if (verified.version === expectedVersion) return verified;
    } catch (error) {
      lastError = error;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(
    `Timed out waiting for the complete signed ${expectedVersion} application bundle: ${lastError?.message || 'version did not match'}`,
  );
}

function executablePids(executable) {
  const result = run('ps', ['-axo', 'pid=,command='], { capture: true });
  return result.stdout.split('\n').flatMap((line) => {
    const match = line.match(/^\s*(\d+)\s+(.+)$/);
    if (!match) return [];
    const command = match[2];
    return command === executable || command.startsWith(`${executable} `)
      ? [Number(match[1])]
      : [];
  });
}

async function stopExecutableProcesses(executable) {
  if (!executable) return;
  for (const pid of executablePids(executable)) {
    if (pid !== process.pid) await stopPid(pid);
  }
}

async function main(argv = process.argv.slice(2)) {
  if (process.platform !== 'darwin') throw new Error('Fraia updater tests require a native macOS runner.');
  const scenario = option(argv, '--scenario');
  if (!['valid', 'corrupt', 'signature'].includes(scenario)) throw new Error('Scenario must be valid, corrupt, or signature.');
  const channel = option(argv, '--channel');
  const arch = option(argv, '--arch', process.arch);
  if (arch !== process.arch) throw new Error('Updater test architecture must match the native runner.');
  const previousZip = path.resolve(option(argv, '--previous-zip'));
  const previousChecksums = path.resolve(option(argv, '--previous-checksums'));
  const candidateDirectory = path.resolve(option(argv, '--candidate-directory'));
  const metadataPath = path.resolve(option(argv, '--candidate-metadata'));
  const repository = option(argv, '--repository', 'apotenza92/fraia');
  verifyPublishedChecksum(previousZip, previousChecksums);
  run('gh', ['attestation', 'verify', previousZip, '--repo', repository]);

  const contract = releaseContract({ channel, platform: 'darwin', arch });
  const currentExpectations = {
    fingerprint: normalizeFingerprint(process.env.APPLE_SIGNING_CERTIFICATE_SHA256),
    identity: process.env.APPLE_SIGNING_IDENTITY?.trim(),
    teamId: process.env.APPLE_TEAM_ID?.trim(),
  };
  const priorExpectations = {
    ...currentExpectations,
    fingerprint: normalizeFingerprint(
      process.env.APPLE_PRIOR_SIGNING_CERTIFICATE_SHA256
        || process.env.APPLE_SIGNING_CERTIFICATE_SHA256,
    ),
  };
  if (!currentExpectations.identity || !currentExpectations.teamId) throw new Error('Updater verification requires signing identity variables.');
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-updater-'));
  let child;
  let executable;
  let server;
  let relaunchedPid;
  try {
    const installedApp = path.join(root, contract.appName);
    extractedZip(previousZip, (directory) => {
      const previousApp = path.join(directory, contract.appName);
      verifyApp(previousApp, contract, priorExpectations);
      run('ditto', [previousApp, installedApp]);
    });
    const previous = verifyApp(installedApp, contract, priorExpectations);
    const userData = path.join(root, 'user-data');
    fs.mkdirSync(userData);
    const persistenceMarker = path.join(userData, 'fraia-updater-persistence');
    fs.writeFileSync(persistenceMarker, 'preserve-me');
    const eventPath = path.join(root, 'event.json');
    const prepared = prepareScenarioAssets({ candidateDirectory, metadataPath, scenario, root });
    if (prepared.version === previous.version) throw new Error('N-1 updater test requires different previous and candidate versions.');
    server = await serveFeed(prepared.assets, prepared.metadata);
    executable = path.join(installedApp, 'Contents', 'MacOS', contract.productName);
    const runtimeEnvironment = restrictedEnvironment({
      FRAIA_DEFAULT_PROJECT_DIR: path.join(userData, 'project'),
      FRAIA_DISABLE_CALCULIX_RUNTIME: '1',
      FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
      FRAIA_E2E_EXPECT_VERSION: prepared.version,
      FRAIA_E2E_INSTALL_UPDATE: '1',
      FRAIA_E2E_UPDATER: '1',
      FRAIA_FAKE_AI_RUNTIME: '1',
      FRAIA_UPDATE_FEED_URL: server.base,
      FRAIA_UPDATER_EVENT_PATH: eventPath,
      FRAIA_USER_DATA_DIR: userData,
    });
    child = spawn(executable, [], {
      env: runtimeEnvironment,
      stdio: 'inherit',
    });
    const accepted = scenario === 'valid' ? new Set(['update-downloaded']) : new Set(['error']);
    let event = await waitForEvent(eventPath, accepted);
    if (scenario === 'valid') {
      if (event.version !== prepared.version) throw new Error('Updater downloaded the wrong candidate version.');
      const previousPid = child.pid;
      await waitForPidExit(previousPid);
      child = null;
      await waitForBundleVersion(installedApp, prepared.version);
      await waitForVerifiedApp(installedApp, contract, currentExpectations, prepared.version);
      child = spawn(executable, [], { env: runtimeEnvironment, stdio: 'inherit' });
      event = await waitForEvent(
        eventPath,
        new Set(['updated-runtime-launched']),
        180_000,
        (candidate) => candidate.pid === child.pid,
      );
      if (event.currentVersion !== prepared.version) throw new Error('Relaunched runtime reported the wrong version.');
      if (path.resolve(event.executablePath) !== path.resolve(executable)) throw new Error('Updated runtime launched from an unexpected executable path.');
      if (!Number.isInteger(event.pid) || event.pid <= 0) throw new Error('Updated runtime did not report a valid process ID.');
      relaunchedPid = event.pid;
    } else {
      const expectedError = scenario === 'corrupt'
        ? /sha512|checksum|digest|integrity/i
        : /signature|code sign|signed/i;
      if (!expectedError.test(String(event.message || ''))) {
        throw new Error(`${scenario} updater scenario failed for an unrelated reason: ${event.message || '<missing message>'}`);
      }
      const unchanged = verifyApp(installedApp, contract, priorExpectations);
      if (unchanged.version !== previous.version) throw new Error('Rejected update modified the installed app.');
    }
    if (!server.requests.includes('latest-mac.yml')) throw new Error('Updater did not request candidate metadata.');
    if (!server.requests.includes(prepared.zipName)) throw new Error('Updater did not request the candidate ZIP payload.');
    if (fs.readFileSync(persistenceMarker, 'utf8') !== 'preserve-me') throw new Error('Updater did not preserve user data.');
  } finally {
    await stopPid(relaunchedPid);
    await stopPid(child?.pid);
    await stopExecutableProcesses(executable);
    if (server) await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
}

if (require.main === module) main().catch((error) => {
  process.stderr.write(`${error.stack || error}\n`);
  process.exitCode = 1;
});
