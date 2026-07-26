#!/usr/bin/env node

const { spawnSync } = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { releaseContract } = require('../release-contract.cjs');
const { normalizeFingerprint } = require('./verify-macos-package.cjs');

function option(argv, name, fallback = null) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : fallback;
}

function run(command, args, { env = process.env, capture = false, allowFailure = false } = {}) {
  const result = spawnSync(command, args, {
    cwd: path.resolve(__dirname, '..'), env, encoding: 'utf8',
    stdio: capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0 && !allowFailure) throw new Error(`${command} ${args.join(' ')} failed (${result.status}).`);
  return result;
}

function decodeBase64(value, label) {
  let encoded = value.trim();
  if (encoded.startsWith("'") && encoded.endsWith("'")) encoded = encoded.slice(1, -1);
  const marker = encoded.indexOf(';base64,');
  if (marker >= 0) encoded = encoded.slice(marker + ';base64,'.length);
  const decoded = Buffer.from(encoded.replace(/\s+/g, ''), 'base64');
  if (!decoded.length) throw new Error(`${label} did not decode to data.`);
  return decoded;
}

function parseJson(result, label) {
  for (const value of [result.stdout, result.stderr]) {
    if (!value?.trim()) continue;
    try { return JSON.parse(value); } catch { /* continue */ }
  }
  throw new Error(`${label} did not return JSON.`);
}

function sha256(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function main(argv = process.argv.slice(2)) {
  if (process.platform !== 'darwin') throw new Error('Signed macOS packages must be built on macOS.');
  if (!argv.includes('--skip-build')) {
    throw new Error('Build and sidecar compilation must run before release credentials are exposed; pass --skip-build.');
  }
  const channel = option(argv, '--channel', process.env.FRAIA_RELEASE_CHANNEL || 'stable');
  const arch = option(argv, '--arch', process.env.FRAIA_RELEASE_ARCH || process.arch);
  if (arch !== process.arch) throw new Error(`Native ${arch} release requires a ${arch} runner; current Node is ${process.arch}.`);
  const outputDir = path.resolve(option(argv, '--output-dir', process.env.FRAIA_RELEASE_OUTPUT_DIR || `release/${channel}/darwin/${arch}`));
  const contract = releaseContract({ channel, platform: 'darwin', arch, outputDir });

  const required = [
    'APPLE_SIGNING_CERTIFICATE_P12_BASE64', 'APPLE_SIGNING_CERTIFICATE_PASSWORD',
    'APPLE_NOTARYTOOL_KEY_ID', 'APPLE_NOTARYTOOL_ISSUER_ID', 'APPLE_NOTARYTOOL_KEY_P8_BASE64',
    'APPLE_SIGNING_IDENTITY', 'APPLE_SIGNING_CERTIFICATE_SHA256', 'APPLE_TEAM_ID',
  ];
  const credentials = {};
  for (const name of required) {
    if (!process.env[name]?.trim()) throw new Error(`Required release variable is missing: ${name}`);
    credentials[name] = process.env[name].trim();
  }
  for (const name of ['APPLE_SIGNING_CERTIFICATE_P12_BASE64', 'APPLE_SIGNING_CERTIFICATE_PASSWORD', 'APPLE_NOTARYTOOL_KEY_P8_BASE64']) delete process.env[name];

  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-signing-'));
  fs.chmodSync(temporaryRoot, 0o700);
  const keychain = path.join(temporaryRoot, 'signing.keychain-db');
  const originalP12 = path.join(temporaryRoot, 'original.p12');
  const passwordFile = path.join(temporaryRoot, 'password');
  const combinedPem = path.join(temporaryRoot, 'combined.pem');
  const importP12 = path.join(temporaryRoot, 'import.p12');
  const apiKey = path.join(temporaryRoot, 'AuthKey.p8');
  const disposablePassword = 'fraia-disposable-import-password';
  let originalKeychains = [];
  let keychainCreated = false;
  try {
    fs.writeFileSync(originalP12, decodeBase64(credentials.APPLE_SIGNING_CERTIFICATE_P12_BASE64, 'P12'), { mode: 0o600 });
    fs.writeFileSync(passwordFile, credentials.APPLE_SIGNING_CERTIFICATE_PASSWORD, { mode: 0o600 });
    const p8 = credentials.APPLE_NOTARYTOOL_KEY_P8_BASE64.includes('BEGIN PRIVATE KEY')
      ? Buffer.from(credentials.APPLE_NOTARYTOOL_KEY_P8_BASE64)
      : decodeBase64(credentials.APPLE_NOTARYTOOL_KEY_P8_BASE64, 'P8');
    fs.writeFileSync(apiKey, p8, { mode: 0o600 });
    run('openssl', ['pkcs12', '-legacy', '-in', originalP12, '-passin', `file:${passwordFile}`, '-nodes', '-out', combinedPem]);
    run('openssl', ['pkcs12', '-legacy', '-export', '-in', combinedPem, '-passout', `pass:${disposablePassword}`, '-out', importP12, '-name', 'Fraia Developer ID']);

    originalKeychains = run('security', ['list-keychains', '-d', 'user'], { capture: true }).stdout
      .split('\n').map((line) => line.trim().replace(/^"(.*)"$/, '$1')).filter(Boolean);
    run('security', ['create-keychain', '-p', '', keychain]);
    keychainCreated = true;
    run('security', ['set-keychain-settings', '-lut', '21600', keychain]);
    run('security', ['unlock-keychain', '-p', '', keychain]);
    run('security', ['import', importP12, '-k', keychain, '-P', disposablePassword, '-T', '/usr/bin/codesign']);
    run('security', ['set-key-partition-list', '-S', 'apple-tool:,apple:,codesign:', '-s', '-k', '', keychain]);
    run('security', ['list-keychains', '-d', 'user', '-s', keychain, ...originalKeychains]);
    const identities = run('security', ['find-identity', '-v', '-p', 'codesigning', keychain], { capture: true }).stdout;
    if (!identities.includes(credentials.APPLE_SIGNING_IDENTITY)) throw new Error('Expected Developer ID identity was not imported.');
    const certificate = run('security', ['find-certificate', '-a', '-c', credentials.APPLE_SIGNING_IDENTITY, '-Z', keychain], { capture: true }).stdout;
    const fingerprints = [...certificate.matchAll(/SHA-256 hash:\s*([A-Fa-f0-9]+)/g)].map((match) => normalizeFingerprint(match[1]));
    const expectedFingerprint = normalizeFingerprint(credentials.APPLE_SIGNING_CERTIFICATE_SHA256);
    if (!fingerprints.includes(expectedFingerprint)) throw new Error('Imported certificate fingerprint does not match the configured fingerprint.');

    fs.rmSync(outputDir, { recursive: true, force: true });
    fs.mkdirSync(outputDir, { recursive: true });
    const releaseEnvironment = {
      ...process.env,
      APPLE_API_KEY: apiKey,
      APPLE_API_KEY_ID: credentials.APPLE_NOTARYTOOL_KEY_ID,
      APPLE_API_ISSUER: credentials.APPLE_NOTARYTOOL_ISSUER_ID,
      APPLE_SIGNING_CERTIFICATE_SHA256: expectedFingerprint,
      APPLE_SIGNING_IDENTITY: credentials.APPLE_SIGNING_IDENTITY,
      APPLE_TEAM_ID: credentials.APPLE_TEAM_ID,
      CSC_IDENTITY_AUTO_DISCOVERY: 'true',
      CSC_KEYCHAIN: keychain,
      CSC_NAME: credentials.APPLE_SIGNING_IDENTITY.replace(/^Developer ID Application:\s*/, ''),
      FRAIA_RELEASE_ARCH: arch,
      FRAIA_RELEASE_CHANNEL: channel,
      FRAIA_RELEASE_OUTPUT_DIR: outputDir,
      FRAIA_RELEASE_PLATFORM: 'darwin',
      FRAIA_REQUIRE_NOTARIZATION: '1',
    };
    run('npx', ['--no-install', 'electron-builder', '--config', 'electron-builder.config.cjs', '--mac', 'dmg', 'zip', `--${arch}`, '--publish', 'never'], { env: releaseEnvironment });

    const stem = `${contract.artifactPrefix}-macOS-${arch}`;
    const dmg = path.join(outputDir, `${stem}.dmg`);
    const zip = path.join(outputDir, `${stem}.zip`);
    const submission = run('xcrun', [
      'notarytool', 'submit', dmg, '--key', apiKey,
      '--key-id', credentials.APPLE_NOTARYTOOL_KEY_ID,
      '--issuer', credentials.APPLE_NOTARYTOOL_ISSUER_ID,
      '--wait', '--output-format', 'json',
    ], { env: releaseEnvironment, capture: true, allowFailure: true });
    const response = parseJson(submission, 'DMG notarization submission');
    if (!response.id) throw new Error('DMG notarization returned no submission identifier.');
    const logResult = run('xcrun', [
      'notarytool', 'log', response.id, '--key', apiKey,
      '--key-id', credentials.APPLE_NOTARYTOOL_KEY_ID,
      '--issuer', credentials.APPLE_NOTARYTOOL_ISSUER_ID,
    ], { env: releaseEnvironment, capture: true });
    const dmgLog = parseJson(logResult, 'DMG notarization log');
    for (const issue of dmgLog.issues || []) console.warn(`Notarization ${issue.severity || 'issue'}: ${issue.message || 'No message'}`);
    if (submission.status !== 0 || response.status !== 'Accepted') throw new Error('DMG notarization was not accepted.');
    if ((dmgLog.issues || []).some((issue) => String(issue.severity).toLowerCase() === 'error')) throw new Error('DMG notarization log contains errors.');
    const appNotary = JSON.parse(fs.readFileSync(path.join(outputDir, 'notarization-app.json'), 'utf8'));
    fs.writeFileSync(path.join(outputDir, `notarization-${channel}-${arch}.json`), `${JSON.stringify({ app: appNotary, dmg: { response, log: dmgLog } }, null, 2)}\n`);
    fs.rmSync(path.join(outputDir, 'notarization-app.json'));
    run('xcrun', ['stapler', 'staple', dmg], { env: releaseEnvironment });
    run('xcrun', ['stapler', 'validate', dmg], { env: releaseEnvironment });
    for (const artifact of [dmg, zip]) fs.writeFileSync(`${artifact}.sha256`, `${sha256(artifact)}  ${path.basename(artifact)}\n`);
    run(process.execPath, ['scripts/verify-macos-package.cjs', '--channel', channel, '--arch', arch, '--output-dir', outputDir, '--skip-launch'], { env: releaseEnvironment });
  } finally {
    if (originalKeychains.length) spawnSync('security', ['list-keychains', '-d', 'user', '-s', ...originalKeychains], { stdio: 'ignore' });
    if (keychainCreated) spawnSync('security', ['delete-keychain', keychain], { stdio: 'ignore' });
    fs.rmSync(temporaryRoot, { recursive: true, force: true });
  }
}

if (require.main === module) main();
