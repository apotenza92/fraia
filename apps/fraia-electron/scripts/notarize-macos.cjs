const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

function run(command, args, { capture = false, allowFailure = false } = {}) {
  const result = spawnSync(command, args, {
    encoding: 'utf8',
    stdio: capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
  });
  if (result.error) throw result.error;
  if (result.status !== 0 && !allowFailure) throw new Error(`${command} failed with status ${result.status}.`);
  return result;
}

function parseJson(result, label) {
  for (const value of [result.stdout, result.stderr]) {
    if (!value?.trim()) continue;
    try { return JSON.parse(value); } catch { /* continue */ }
  }
  throw new Error(`${label} did not return JSON.`);
}

module.exports = async function notarizeMacApp(context) {
  const required = ['APPLE_API_KEY', 'APPLE_API_KEY_ID', 'APPLE_API_ISSUER', 'FRAIA_RELEASE_ARCH', 'FRAIA_RELEASE_CHANNEL', 'FRAIA_RELEASE_OUTPUT_DIR'];
  for (const name of required) if (!process.env[name]?.trim()) throw new Error(`Missing ${name} for notarization.`);
  const appPath = path.join(context.appOutDir, `${context.packager.appInfo.productFilename}.app`);
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-notary-'));
  const upload = path.join(temporaryRoot, 'Fraia.zip');
  try {
    run('ditto', ['-c', '-k', '--keepParent', appPath, upload]);
    const submission = run('xcrun', [
      'notarytool', 'submit', upload,
      '--key', process.env.APPLE_API_KEY,
      '--key-id', process.env.APPLE_API_KEY_ID,
      '--issuer', process.env.APPLE_API_ISSUER,
      '--wait', '--output-format', 'json',
    ], { capture: true, allowFailure: true });
    const response = parseJson(submission, 'App notarization submission');
    if (!response.id) throw new Error('App notarization returned no submission identifier.');
    const logResult = run('xcrun', [
      'notarytool', 'log', response.id,
      '--key', process.env.APPLE_API_KEY,
      '--key-id', process.env.APPLE_API_KEY_ID,
      '--issuer', process.env.APPLE_API_ISSUER,
    ], { capture: true });
    const log = parseJson(logResult, 'App notarization log');
    for (const issue of log.issues || []) console.warn(`Notarization ${issue.severity || 'issue'}: ${issue.message || 'No message'}`);
    if (submission.status !== 0 || response.status !== 'Accepted') {
      throw new Error(`App notarization was not accepted: ${submission.stdout || submission.stderr}`);
    }
    const errors = (log.issues || []).filter((issue) => String(issue.severity).toLowerCase() === 'error');
    if (errors.length) throw new Error(`App notarization log contains ${errors.length} error issue(s).`);
    fs.mkdirSync(process.env.FRAIA_RELEASE_OUTPUT_DIR, { recursive: true });
    fs.writeFileSync(path.join(process.env.FRAIA_RELEASE_OUTPUT_DIR, 'notarization-app.json'), `${JSON.stringify({ response, log }, null, 2)}\n`);
    run('xcrun', ['stapler', 'staple', appPath]);
    run('xcrun', ['stapler', 'validate', appPath]);
  } finally {
    fs.rmSync(temporaryRoot, { recursive: true, force: true });
  }
};
