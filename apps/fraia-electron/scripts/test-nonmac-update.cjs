#!/usr/bin/env node

const { createHash } = require('node:crypto');
const { spawn, spawnSync } = require('node:child_process');
const fs = require('node:fs');
const http = require('node:http');
const os = require('node:os');
const path = require('node:path');
const asar = require('@electron/asar');
const YAML = require('yaml');
const {
  resolveApplicationMetadata,
  resolveUserDataDirectory,
} = require('../application-metadata.cjs');
const { metadataFileName, releaseContract } = require('../release-contract.cjs');
const { createTestRepositoryMetadata } = require('./test-tuf-repository.cjs');

function option(argv, name) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : null;
}

function run(command, args, { env = process.env } = {}) {
  const result = spawnSync(command, args, { env, encoding: 'utf8', stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} failed with status ${result.status}.`);
}

function digest(filePath, algorithm, encoding) {
  return createHash(algorithm).update(fs.readFileSync(filePath)).digest(encoding);
}

function artifactName(value) {
  if (typeof value !== 'string' || !value) throw new Error('Updater metadata contains an invalid artifact URL.');
  const candidate = /^https?:\/\//.test(value) ? new URL(value).pathname : value;
  const decoded = decodeURIComponent(path.posix.basename(candidate));
  if (!decoded || decoded !== path.posix.basename(decoded) || decoded.includes('\\')) {
    throw new Error(`Unsafe updater artifact name: ${value}`);
  }
  return decoded;
}

function prepareSignedTarget({ baseUrl, candidateDirectory, candidateMetadata }) {
  const metadata = YAML.parse(fs.readFileSync(candidateMetadata, 'utf8'));
  if (!metadata?.version || !Array.isArray(metadata.files) || metadata.files.length === 0) {
    throw new Error('Candidate updater metadata is incomplete.');
  }
  const names = new Set();
  metadata.files = metadata.files.map((file) => {
    const name = artifactName(file.url);
    if (names.has(name)) throw new Error(`Duplicate updater artifact: ${name}`);
    names.add(name);
    const candidate = path.join(candidateDirectory, name);
    if (!fs.statSync(candidate, { throwIfNoEntry: false })?.isFile()) {
      throw new Error(`Candidate updater artifact is missing: ${name}`);
    }
    if (digest(candidate, 'sha512', 'base64') !== file.sha512) {
      throw new Error(`Candidate updater SHA-512 does not match: ${name}`);
    }
    if (file.size !== undefined && fs.statSync(candidate).size !== file.size) {
      throw new Error(`Candidate updater size does not match: ${name}`);
    }
    return { ...file, url: `${baseUrl}/assets/${encodeURIComponent(name)}` };
  });
  if (metadata.path) {
    const name = artifactName(metadata.path);
    if (!names.has(name)) throw new Error('Legacy updater path does not match a files entry.');
    metadata.path = `${baseUrl}/assets/${encodeURIComponent(name)}`;
  }
  return {
    artifactNames: names,
    bytes: Buffer.from(`${YAML.stringify(metadata, { lineWidth: 0 }).trimEnd()}\n`),
    version: metadata.version,
  };
}

function serveFile(request, response, filePath) {
  const bytes = fs.readFileSync(filePath);
  const range = request.headers.range?.match(/^bytes=(\d+)-(\d*)$/);
  if (!range) {
    response.writeHead(200, {
      'Accept-Ranges': 'bytes',
      'Content-Length': bytes.length,
    });
    response.end(request.method === 'HEAD' ? undefined : bytes);
    return;
  }
  const start = Number(range[1]);
  const end = range[2] ? Math.min(Number(range[2]), bytes.length - 1) : bytes.length - 1;
  if (!Number.isSafeInteger(start) || start < 0 || start > end || start >= bytes.length) {
    response.writeHead(416, { 'Content-Range': `bytes */${bytes.length}` }).end();
    return;
  }
  response.writeHead(206, {
    'Accept-Ranges': 'bytes',
    'Content-Length': end - start + 1,
    'Content-Range': `bytes ${start}-${end}/${bytes.length}`,
  });
  response.end(request.method === 'HEAD' ? undefined : bytes.subarray(start, end + 1));
}

async function createServer({
  candidateDirectory,
  candidateMetadata,
  privateKeyPath,
  rootPath,
  targetName,
}) {
  const requests = [];
  let repositoryMetadata;
  let signedTarget;
  const server = http.createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, 'http://127.0.0.1').pathname);
    requests.push(pathname);
    if (!['GET', 'HEAD'].includes(request.method)) {
      response.writeHead(405).end();
      return;
    }
    const metadataMatch = pathname.match(/^\/tuf\/metadata\/([^/]+)$/);
    if (metadataMatch) {
      if (metadataMatch[1] === '2.root.json') {
        response.writeHead(404).end();
        return;
      }
      const bytes = repositoryMetadata?.[metadataMatch[1]];
      if (!bytes) {
        response.writeHead(404).end();
        return;
      }
      response.writeHead(200, { 'Content-Length': bytes.length });
      response.end(request.method === 'HEAD' ? undefined : bytes);
      return;
    }
    if (pathname === `/tuf/targets/${encodeURIComponent(targetName)}`) {
      response.writeHead(200, { 'Content-Length': signedTarget.bytes.length });
      response.end(request.method === 'HEAD' ? undefined : signedTarget.bytes);
      return;
    }
    const assetMatch = pathname.match(/^\/assets\/([^/]+)$/);
    if (assetMatch && signedTarget.artifactNames.has(assetMatch[1])) {
      serveFile(request, response, path.join(candidateDirectory, assetMatch[1]));
      return;
    }
    response.writeHead(404).end();
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  const baseUrl = `http://127.0.0.1:${server.address().port}`;
  signedTarget = prepareSignedTarget({ baseUrl, candidateDirectory, candidateMetadata });
  repositoryMetadata = createTestRepositoryMetadata({
    privateKeyPath,
    rootPath,
    targetBytes: signedTarget.bytes,
    targetName,
  });
  return {
    baseUrl,
    close: () => new Promise((resolve, reject) => {
      server.close((error) => error ? reject(error) : resolve());
    }),
    requests,
    targetBytes: signedTarget.bytes,
    version: signedTarget.version,
  };
}

function readEvents(eventPath) {
  const history = `${eventPath}.jsonl`;
  if (!fs.existsSync(history)) return [];
  return fs.readFileSync(history, 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse);
}

async function waitForEvent(eventPath, accepted, timeoutMs = 300_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const event = readEvents(eventPath).find((candidate) => accepted.has(candidate.name));
    if (event) return event;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for updater event: ${[...accepted].join(', ')}.`);
}

function installedPackageVersion(executable) {
  const archivePath = path.join(path.dirname(executable), 'resources', 'app.asar');
  asar.uncache(archivePath);
  const packageBytes = asar.extractFile(
    archivePath,
    'package.json',
  );
  return JSON.parse(packageBytes.toString('utf8')).version;
}

function installedPackageDigest(executable) {
  return digest(
    path.join(path.dirname(executable), 'resources', 'app.asar'),
    'sha256',
    'hex',
  );
}

function windowsProcessIds(executable) {
  return windowsProcessIdsMatching(
    '$_.ExecutablePath -eq $env:FRAIA_AUDIT_EXECUTABLE',
    { FRAIA_AUDIT_EXECUTABLE: executable },
  );
}

function windowsProcessIdsWithin(directory) {
  return windowsProcessIdsMatching(
    '$_.ExecutablePath -and $_.ExecutablePath.StartsWith($env:FRAIA_AUDIT_DIRECTORY, [System.StringComparison]::OrdinalIgnoreCase)',
    { FRAIA_AUDIT_DIRECTORY: `${path.resolve(directory)}${path.sep}` },
  );
}

function windowsProcessIdsMatching(predicate, environment) {
  const result = spawnSync(
    'powershell.exe',
    [
      '-NoProfile',
      '-NonInteractive',
      '-Command',
      [
        '$processes = @(Get-CimInstance Win32_Process |',
        `Where-Object { ${predicate} } |`,
        'Select-Object -ExpandProperty ProcessId);',
        'ConvertTo-Json -Compress -InputObject $processes',
      ].join(' '),
    ],
    {
      encoding: 'utf8',
      env: { ...process.env, ...environment },
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`Could not inspect the relaunched Windows process: ${result.stderr.trim()}`);
  }
  const parsed = JSON.parse(result.stdout.trim() || '[]');
  return (Array.isArray(parsed) ? parsed : [parsed])
    .map(Number)
    .filter((pid) => Number.isInteger(pid) && pid > 0);
}

async function waitForInstalledWindowsPackage({
  candidateArchive,
  eventPath,
  executable,
  version,
  timeoutMs = 180_000,
}) {
  const expectedDigest = digest(candidateArchive, 'sha256', 'hex');
  const deadline = Date.now() + timeoutMs;
  let lastDigest = '<unreadable>';
  let lastVersion = '<unreadable>';
  let lastReadError = null;
  while (Date.now() < deadline) {
    const error = readEvents(eventPath).find((event) => event.name === 'error');
    if (error) throw new Error(`Native updater failed: ${error.message || '<missing error>'}`);
    try {
      lastDigest = installedPackageDigest(executable);
      lastVersion = installedPackageVersion(executable);
      if (
        lastDigest === expectedDigest
        && lastVersion === version
      ) {
        return expectedDigest;
      }
    } catch (error) {
      lastReadError = String(error?.message || error);
      if (Date.now() >= deadline) throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(
    `Timed out waiting for byte-exact installation of the Windows ${version} package. `
    + `Expected app.asar SHA-256 ${expectedDigest}; last installed SHA-256 ${lastDigest}; `
    + `last installed package version ${lastVersion}; last read error ${lastReadError || 'none'}.`,
  );
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
  if (process.platform === 'win32') {
    const result = spawnSync('taskkill.exe', ['/PID', String(pid), '/T', '/F'], {
      encoding: 'utf8',
    });
    if (result.error) throw result.error;
    if (result.status !== 0 && isPidAlive(pid)) {
      throw new Error(`Could not stop updater process ${pid}: ${result.stderr.trim()}`);
    }
  } else {
    process.kill(pid);
  }
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline && isPidAlive(pid)) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (isPidAlive(pid)) throw new Error(`Updater process ${pid} did not stop.`);
}

function findExactlyOne(root, predicate, label) {
  const matches = [];
  function walk(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const candidate = path.join(directory, entry.name);
      if (entry.isDirectory()) walk(candidate);
      else if (entry.isFile() && predicate(candidate)) matches.push(candidate);
    }
  }
  walk(root);
  if (matches.length !== 1) throw new Error(`Expected exactly one ${label}; found ${matches.length}.`);
  return matches[0];
}

function waitForPathRemoval(target, { timeoutMs = 60_000, intervalMs = 250 } = {}) {
  const deadline = Date.now() + timeoutMs;
  const waitBuffer = new Int32Array(new SharedArrayBuffer(4));
  while (fs.existsSync(target)) {
    const remaining = deadline - Date.now();
    if (remaining <= 0) {
      throw new Error(`Timed out waiting for the native uninstaller to remove ${target}.`);
    }
    Atomics.wait(waitBuffer, 0, 0, Math.min(intervalMs, remaining));
  }
}

function restrictedEnvironment(overrides) {
  const allowed = process.platform === 'win32'
    ? [
      'ALLUSERSPROFILE', 'APPDATA', 'CommonProgramFiles', 'CommonProgramW6432',
      'ComSpec', 'HOMEDRIVE', 'HOMEPATH', 'LOCALAPPDATA', 'NUMBER_OF_PROCESSORS',
      'OS', 'Path', 'PATHEXT', 'PROCESSOR_ARCHITECTURE', 'ProgramData',
      'ProgramFiles', 'ProgramW6432', 'SystemDrive', 'SystemRoot', 'TEMP', 'TMP',
      'USERDOMAIN', 'USERNAME', 'USERPROFILE', 'windir',
    ]
    : ['DBUS_SESSION_BUS_ADDRESS', 'DISPLAY', 'HOME', 'LANG', 'LC_ALL', 'PATH', 'SHELL', 'TEMP', 'TMP', 'TMPDIR', 'USER', 'XAUTHORITY'];
  return Object.fromEntries([
    ...allowed.flatMap((name) => process.env[name] ? [[name, process.env[name]]] : []),
    ...Object.entries(overrides),
  ]);
}

function writeEvidence({
  arch,
  candidateDirectory,
  candidateMetadata,
  eventPath,
  evidenceDirectory,
  failure,
  installedCandidateDigest,
  installedCandidateVersion,
  platform,
  previousArtifact,
  projectBytes,
  projectPath,
  credentialBytes,
  credentialPath,
  rootPath,
  server,
}) {
  const staging = path.join(path.dirname(evidenceDirectory), `.fraia-updater-evidence-${process.pid}`);
  fs.rmSync(staging, { recursive: true, force: true });
  fs.mkdirSync(staging, { recursive: true });
  const result = failure
    ? `Native updater audit failed closed: ${failure.message || failure}\n`
    : 'Native updater audit installed and relaunched the candidate while preserving user data.\n';
  fs.writeFileSync(path.join(staging, failure ? 'FAILURE.txt' : 'RESULT.txt'), result);
  fs.writeFileSync(path.join(staging, 'ENVIRONMENT.txt'), [
    `Platform: ${platform}`,
    `Architecture: ${arch}`,
    `Previous artifact: ${path.basename(previousArtifact)}`,
    `Candidate metadata: ${path.basename(candidateMetadata)}`,
    `Candidate version: ${server?.version || '<not reached>'}`,
    'Trust: ephemeral loopback-only TUF test root',
    '',
  ].join('\n'));
  fs.copyFileSync(rootPath, path.join(staging, 'root.json'));
  fs.copyFileSync(candidateMetadata, path.join(staging, path.basename(candidateMetadata)));
  if (fs.existsSync(`${eventPath}.jsonl`)) {
    fs.copyFileSync(`${eventPath}.jsonl`, path.join(staging, 'updater-events.jsonl'));
  }
  if (server) {
    fs.writeFileSync(path.join(staging, 'REQUESTS.txt'), `${server.requests.join('\n')}\n`);
    fs.writeFileSync(path.join(staging, 'signed-update-target.yml'), server.targetBytes);
  }
  fs.writeFileSync(path.join(staging, 'MIGRATION.txt'), [
    `Installed candidate version: ${installedCandidateVersion || '<not verified>'}`,
    `Installed candidate app.asar SHA-256: ${installedCandidateDigest || '<not verified>'}`,
    `Project before: ${createHash('sha256').update(projectBytes).digest('hex')}`,
    `Project after: ${fs.existsSync(projectPath) ? digest(projectPath, 'sha256', 'hex') : '<missing>'}`,
    `AI data before: ${createHash('sha256').update(credentialBytes).digest('hex')}`,
    `AI data after: ${fs.existsSync(credentialPath) ? digest(credentialPath, 'sha256', 'hex') : '<missing>'}`,
    '',
  ].join('\n'));
  const artifacts = [previousArtifact, ...fs.readdirSync(candidateDirectory)
    .map((name) => path.join(candidateDirectory, name))
    .filter((candidate) => fs.statSync(candidate).isFile())];
  fs.writeFileSync(path.join(staging, 'PACKAGE_SHA256SUMS'), artifacts
    .map((artifact) => `${digest(artifact, 'sha256', 'hex')}  ${path.basename(artifact)}`)
    .sort()
    .join('\n')
    .concat('\n'));
  const evidenceFiles = fs.readdirSync(staging).sort();
  fs.writeFileSync(path.join(staging, 'EVIDENCE_SHA256SUMS'), evidenceFiles
    .map((name) => `${digest(path.join(staging, name), 'sha256', 'hex')}  ${name}`)
    .join('\n')
    .concat('\n'));
  fs.renameSync(staging, evidenceDirectory);
}

async function main(argv = process.argv.slice(2)) {
  if (!['win32', 'linux'].includes(process.platform)) {
    throw new Error('Fraia non-macOS updater tests require a native Windows or Linux runner.');
  }
  const arch = option(argv, '--arch') || process.arch;
  if (arch !== process.arch) throw new Error(`Updater audit requires native ${arch}; current Node is ${process.arch}.`);
  const previousArtifact = path.resolve(option(argv, '--previous-artifact'));
  const candidateDirectory = path.resolve(option(argv, '--candidate-directory'));
  const candidateMetadata = path.resolve(option(argv, '--candidate-metadata'));
  const evidenceDirectory = path.resolve(option(argv, '--evidence'));
  const privateKeyPath = path.resolve(option(argv, '--private-key'));
  const rootPath = path.resolve(option(argv, '--root'));
  if (fs.existsSync(evidenceDirectory)) throw new Error('Updater evidence directory must not already exist.');
  for (const required of [previousArtifact, candidateDirectory, candidateMetadata, privateKeyPath, rootPath]) {
    if (!fs.existsSync(required)) throw new Error(`Updater audit input is missing: ${required}`);
  }

  const contract = releaseContract({ channel: 'stable', platform: process.platform, arch });
  const targetName = metadataFileName(process.platform, arch);
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-nonmac-updater-'));
  const userData = process.platform === 'win32'
    ? resolveUserDataDirectory({
      appDataPath: process.env.APPDATA,
      metadata: resolveApplicationMetadata(),
    })
    : path.join(temporaryRoot, 'user-data');
  if (process.platform === 'win32' && fs.existsSync(userData)) {
    throw new Error(`Native updater audit requires an unused Windows user-data directory: ${userData}`);
  }
  const projectPath = path.join(userData, 'projects', 'preservation-marker.fraia');
  const credentialPath = path.join(userData, 'ai', 'preservation-marker.bin');
  const eventPath = path.join(temporaryRoot, 'events', 'updater.json');
  const projectBytes = Buffer.from('{"schemaVersion":1,"name":"Preserved updater model"}\n');
  const credentialBytes = Buffer.from([0x46, 0x52, 0x41, 0x49, 0x41]);
  fs.mkdirSync(path.dirname(projectPath), { recursive: true });
  fs.mkdirSync(path.dirname(credentialPath), { recursive: true });
  fs.writeFileSync(projectPath, projectBytes);
  fs.writeFileSync(credentialPath, credentialBytes);

  let child;
  let relaunchedPid;
  let server;
  let installedExecutable;
  let installedAppImage;
  let installedCandidateDigest;
  let installedCandidateVersion;
  let failure;
  try {
    if (process.platform === 'win32') {
      const installDirectory = path.join(process.env.LOCALAPPDATA, 'Programs', contract.productName);
      if (fs.existsSync(installDirectory)) {
        throw new Error(`Native updater audit requires an unused Windows install directory: ${installDirectory}`);
      }
      run(previousArtifact, ['/S']);
      installedExecutable = path.join(installDirectory, `${contract.productName}.exe`);
      if (!fs.existsSync(installedExecutable)) throw new Error('The previous Windows package did not install Fraia.');
    } else {
      installedAppImage = path.join(temporaryRoot, path.basename(previousArtifact));
      fs.copyFileSync(previousArtifact, installedAppImage);
      fs.chmodSync(installedAppImage, 0o755);
      installedExecutable = installedAppImage;
    }

    server = await createServer({
      candidateDirectory,
      candidateMetadata,
      privateKeyPath,
      rootPath,
      targetName,
    });
    const environment = restrictedEnvironment({
      APPIMAGE_EXTRACT_AND_RUN: process.platform === 'linux' ? '1' : undefined,
      FRAIA_DEFAULT_PROJECT_DIR: path.join(userData, 'projects', 'default'),
      FRAIA_DISABLE_CALCULIX_RUNTIME: '1',
      FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
      FRAIA_E2E_EXPECT_VERSION: server.version,
      FRAIA_E2E_INSTALL_UPDATE: '1',
      FRAIA_E2E_TUF_REPOSITORY_URL: `${server.baseUrl}/tuf`,
      FRAIA_E2E_UPDATER: '1',
      FRAIA_FAKE_AI_RUNTIME: '1',
      FRAIA_UPDATER_EVENT_PATH: eventPath,
      ...(process.platform === 'linux' ? { FRAIA_USER_DATA_DIR: userData } : {}),
    });
    for (const [name, value] of Object.entries(environment)) {
      if (value === undefined) delete environment[name];
    }
    child = spawn(installedExecutable, [], { env: environment, stdio: 'inherit' });
    const event = await waitForEvent(
      eventPath,
      new Set([
        process.platform === 'win32' ? 'update-downloaded' : 'updated-runtime-launched',
        'error',
      ]),
    );
    if (event.name === 'error') throw new Error(`Native updater failed: ${event.message || '<missing error>'}`);
    if (process.platform === 'win32') {
      if (event.currentVersion === server.version) {
        throw new Error('Windows updater audit did not start from the previous version.');
      }
      const candidateArchive = path.join(
        candidateDirectory,
        'win-unpacked',
        'resources',
        'app.asar',
      );
      if (!fs.existsSync(candidateArchive)) {
        throw new Error('Candidate Windows ASAR is missing from the package output.');
      }
      installedCandidateDigest = await waitForInstalledWindowsPackage({
        candidateArchive,
        eventPath,
        executable: installedExecutable,
        version: server.version,
      });
      installedCandidateVersion = installedPackageVersion(installedExecutable);
      for (const pid of windowsProcessIds(installedExecutable)) await stopPid(pid);
      await stopPid(child.pid);
      child = spawn(
        installedExecutable,
        [],
        { env: restrictedEnvironment({}), stdio: 'inherit' },
      );
      await new Promise((resolve) => setTimeout(resolve, 3_000));
      relaunchedPid = child.pid;
      if (!isPidAlive(relaunchedPid)) {
        throw new Error('The installed Windows candidate did not remain running after a normal user launch.');
      }
    } else {
      if (event.currentVersion !== server.version) throw new Error('Updated runtime reported the wrong version.');
      relaunchedPid = event.pid;
    }
    if (!Number.isInteger(relaunchedPid) || relaunchedPid <= 0) {
      throw new Error('Updated runtime reported an invalid process ID.');
    }
    if (!fs.readFileSync(projectPath).equals(projectBytes)) throw new Error('Updater changed existing project data.');
    if (!fs.readFileSync(credentialPath).equals(credentialBytes)) throw new Error('Updater changed existing AI data.');
    const persistedRoot = path.join(userData, 'update-trust', 'metadata', 'root.json');
    if (JSON.parse(fs.readFileSync(persistedRoot, 'utf8')).signed.version !== 1) {
      throw new Error('Updated runtime did not retain its TUF root trust.');
    }
    if (!server.requests.includes(`/tuf/targets/${encodeURIComponent(targetName)}`)) {
      throw new Error('Updater did not request the TUF-authenticated updater metadata.');
    }
    if (!server.requests.some((request) => request.startsWith('/assets/'))) {
      throw new Error('Updater did not request a candidate package.');
    }
    if (process.platform === 'linux') {
      const candidateName = artifactName(YAML.parse(fs.readFileSync(candidateMetadata, 'utf8')).files[0].url);
      if (digest(installedAppImage, 'sha256', 'hex') !== digest(path.join(candidateDirectory, candidateName), 'sha256', 'hex')) {
        throw new Error('AppImage updater did not replace the installed bytes.');
      }
    }
  } catch (error) {
    failure = error;
    throw error;
  } finally {
    let cleanupFailure;
    try {
      await stopPid(relaunchedPid);
      await stopPid(child?.pid);
      if (server) await server.close();
      if (process.platform === 'win32' && installedExecutable && fs.existsSync(path.dirname(installedExecutable))) {
        const installDirectory = path.dirname(installedExecutable);
        const sidecar = path.join(
          installDirectory,
          'resources',
          'sidecar',
          `win32-${arch}`,
          'fraia-appd.exe',
        );
        for (const executable of [installedExecutable, sidecar]) {
          if (!fs.existsSync(executable)) continue;
          for (const pid of windowsProcessIds(executable)) await stopPid(pid);
        }
        for (const pid of windowsProcessIdsWithin(installDirectory)) await stopPid(pid);
        const uninstaller = findExactlyOne(
          installDirectory,
          (candidate) => /^uninstall.*\.exe$/i.test(path.basename(candidate)),
          'NSIS uninstaller',
        );
        run(uninstaller, ['/S']);
        waitForPathRemoval(installDirectory);
      }
    } catch (error) {
      cleanupFailure = error;
      process.stderr.write(`Native updater cleanup failed: ${error.stack || error}\n`);
    }
    writeEvidence({
      arch,
      candidateDirectory,
      candidateMetadata,
      credentialBytes,
      credentialPath,
      eventPath,
      evidenceDirectory,
      failure: failure || cleanupFailure,
      installedCandidateDigest,
      installedCandidateVersion,
      platform: process.platform,
      previousArtifact,
      projectBytes,
      projectPath,
      rootPath,
      server,
    });
    fs.rmSync(temporaryRoot, { recursive: true, force: true, maxRetries: 20, retryDelay: 250 });
    if (process.platform === 'win32') {
      fs.rmSync(userData, { recursive: true, force: true, maxRetries: 20, retryDelay: 250 });
      fs.rmSync(path.join(process.env.LOCALAPPDATA, 'fraia-electron-updater'), {
        recursive: true,
        force: true,
        maxRetries: 20,
        retryDelay: 250,
      });
    }
    if (!failure && cleanupFailure) throw cleanupFailure;
  }
}

if (require.main === module) {
  main().catch((error) => {
    process.stderr.write(`${error.stack || error}\n`);
    process.exitCode = 1;
  });
}

module.exports = {
  artifactName,
  installedPackageDigest,
  installedPackageVersion,
  prepareSignedTarget,
  waitForPathRemoval,
  waitForInstalledWindowsPackage,
  windowsProcessIds,
  windowsProcessIdsWithin,
};
