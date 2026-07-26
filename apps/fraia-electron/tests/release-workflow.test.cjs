const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const repositoryRoot = path.resolve(__dirname, '..', '..', '..');
const workflowPath = path.join(repositoryRoot, '.github', 'workflows', 'release.yml');
const workflow = fs.readFileSync(workflowPath, 'utf8');
const builder = fs.readFileSync(path.join(__dirname, '..', 'electron-builder.config.cjs'), 'utf8');
const signing = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'build-signed-macos.cjs'), 'utf8');
const updaterTest = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'test-macos-update.cjs'), 'utf8');
const continuousIntegration = fs.readFileSync(path.join(repositoryRoot, '.github', 'workflows', 'ci.yml'), 'utf8');
const mainProcess = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');

function jobSource(jobId) {
  const match = workflow.match(
    new RegExp(`^  ${jobId}:\\n([\\s\\S]*?)(?=^  [A-Za-z0-9_-]+:\\n|(?![\\s\\S]))`, 'm'),
  );
  assert.ok(match, `workflow job ${jobId} is missing`);
  return match[1];
}

test('release is tag-only, native on five solver-backed targets, and uses protected publication boundaries', () => {
  assert.match(workflow, /tags:\n\s+- 'v\*'/);
  assert.doesNotMatch(workflow, /workflow_dispatch/);
  for (const runner of ['macos-15', 'macos-15-intel', 'windows-2025', 'ubuntu-24.04', 'ubuntu-24.04-arm']) {
    assert.match(workflow, new RegExp(runner.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
  assert.doesNotMatch(workflow, /windows-11-arm|win32-arm64|Windows-arm64/);
  for (const environment of [
    'release-signing', 'beta-release', 'stable-release',
  ]) assert.match(workflow, new RegExp(environment));
  assert.match(workflow, /name: \$\{\{ needs\.prepare\.outputs\.channel \}\}-updater-verification/);
  assert.match(workflow, /actions\/attest@[a-f0-9]{40}/);
  assert.match(workflow, /cancel-in-progress: false/);
  assert.match(workflow, /Require maintained release icons/);
  assert.match(workflow, /Require the declared repository license/);
  assert.match(workflow, /run: test -f LICENSE/);
  assert.match(builder, /FRAIA_REQUIRE_RELEASE_ICON/);
  assert.match(workflow, /Require a public updater and binary origin/);
  assert.match(workflow, /SOURCE_REPOSITORY_PRIVATE/);
  assert.match(workflow, /Require reviewed native CalculiX runtime manifests/);
  assert.match(workflow, /FRAIA_REQUIRE_PACKAGED_CALCULIX: '1'/);
  assert.match(builder, /validateRuntimeDirectory/);
  const nativeVerifier = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'verify-native-package.cjs'), 'utf8');
  const macVerifier = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'verify-macos-package.cjs'), 'utf8');
  assert.match(nativeVerifier, /runtime-manifest\.json/);
  assert.match(macVerifier, /runtime-manifest\.json/);
});

test('canonical Apple credentials are isolated from build and followed by credential-free verification', () => {
  for (const name of [
    'APPLE_SIGNING_CERTIFICATE_P12_BASE64', 'APPLE_SIGNING_CERTIFICATE_PASSWORD',
    'APPLE_NOTARYTOOL_KEY_P8_BASE64', 'APPLE_NOTARYTOOL_KEY_ID', 'APPLE_NOTARYTOOL_ISSUER_ID',
    'APPLE_SIGNING_CERTIFICATE_SHA256', 'APPLE_SIGNING_IDENTITY', 'APPLE_TEAM_ID',
  ]) assert.match(workflow, new RegExp(name));
  assert.match(workflow, /APPLE_NOTARYTOOL_KEY_ID: \$\{\{ vars\.APPLE_NOTARYTOOL_KEY_ID \}\}/);
  assert.match(workflow, /APPLE_NOTARYTOOL_ISSUER_ID: \$\{\{ vars\.APPLE_NOTARYTOOL_ISSUER_ID \}\}/);
  assert.doesNotMatch(workflow, /secrets\.APPLE_NOTARYTOOL_(?:KEY|ISSUER)_ID/);
  assert.match(workflow, /Build renderer and native Rust sidecar without release credentials/);
  assert.match(workflow, /Verify signatures and launch without release credentials/);
  assert.match(signing, /pass --skip-build/);
  assert.doesNotMatch(`${workflow}\n${signing}`, /CSC_LINK|APPLE_ID_PASSWORD|app-specific password/i);
  const macVerifier = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'verify-macos-package.cjs'), 'utf8');
  assert.match(macVerifier, /validateSignature\(dmg, expectations/);
  assert.match(macVerifier, /DMG signature lacks a secure timestamp/);
});

test('private-source and package prerequisites fail before any secret-bearing job can run', () => {
  const prepare = jobSource('prepare');
  const validate = jobSource('validate');
  const packageMacos = jobSource('package-macos');
  assert.match(prepare, /SOURCE_REPOSITORY_PRIVATE/);
  assert.match(prepare, /if \[ "\$SOURCE_REPOSITORY_PRIVATE" != false \]/);
  assert.doesNotMatch(prepare, /secrets\.|^    environment:/m);
  assert.match(validate, /needs: prepare/);
  assert.match(validate, /run: test -f LICENSE/);
  assert.match(validate, /test -f apps\/fraia-electron\/build\/icon\.icns/);
  assert.match(validate, /verify-calculix-runtimes\.cjs --all --skip-dependency-inspection/);
  assert.doesNotMatch(validate, /secrets\.|^    environment:/m);
  assert.match(packageMacos, /needs: \[prepare, validate\]/);
  assert.match(packageMacos, /environment: release-signing/);
  assert.match(packageMacos, /secrets\.APPLE_SIGNING_CERTIFICATE_P12_BASE64/);
});

test('publication and bootstrap contracts are exact and fail closed', () => {
  assert.match(workflow, /MACOS_UPDATER_BOOTSTRAP_TAG !== process\.env\.GITHUB_REF_NAME/);
  assert.match(workflow, /APPLE_PRIOR_SIGNING_CERTIFICATE_SHA256/);
  assert.match(updaterTest, /priorExpectations/);
  assert.match(workflow, /needs: \[prepare, assemble, attest\]/);
  assert.match(workflow, /comm -23 existing-assets\.txt expected-assets\.txt/);
  assert.match(workflow, /cmp "publish\/assets\/\$name" "existing-release\/\$name"/);
  assert.doesNotMatch(workflow, /gh release (?:delete|upload)[^\n]*(?:--clobber|-R)/);
  assert.match(updaterTest, /updated-runtime-launched/);
  assert.match(updaterTest, /'gh', \['attestation', 'verify'/);
  assert.match(updaterTest, /sha512\|checksum\|digest\|integrity/);
  assert.match(updaterTest, /signature\|code sign\|signed/);
  assert.match(updaterTest, /server\.requests\.includes\(prepared\.zipName\)/);
  const publishRelease = workflow.indexOf('- name: Publish verified release');
  const verifyPublic = workflow.indexOf('- name: Verify public release before sealing updater feed');
  const sealFeed = workflow.indexOf('- name: Prepare sealed updater-feed publication bundle');
  const uploadFeed = workflow.indexOf('- name: Upload sealed updater-feed publication bundle');
  assert.ok(
    publishRelease >= 0 && publishRelease < verifyPublic && verifyPublic < sealFeed && sealFeed < uploadFeed,
    'the release must be public and byte-verified before its updater feed is sealed for manual publication',
  );
  assert.match(workflow, /Publication: manual, after the explicit release gate/);
  assert.match(workflow, /fraia-update-feed-publication-\$\{\{ github\.ref_name \}\}/);
  assert.doesNotMatch(workflow, /git (?:commit|push)/);
});

test('manual CI keeps deterministic checks default and native package preflight explicitly opt-in', () => {
  assert.match(continuousIntegration, /workflow_dispatch:\n\s+inputs:/);
  assert.doesNotMatch(continuousIntegration, /^\s{2}pull_request:/m);
  assert.doesNotMatch(continuousIntegration, /^\s{2}push:/m);
  assert.match(continuousIntegration, /run_native_package_preflight:/);
  assert.match(continuousIntegration, /default: false/);
  assert.match(continuousIntegration, /if: \$\{\{ inputs\.run_native_package_preflight \}\}/);
  for (const runner of ['macos-15', 'macos-15-intel', 'windows-2025', 'ubuntu-24.04', 'ubuntu-24.04-arm']) {
    assert.match(continuousIntegration, new RegExp(runner.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
  assert.doesNotMatch(continuousIntegration, /windows-11-arm|win32-arm64/);
  assert.match(continuousIntegration, /verify-calculix-runtimes\.cjs/);
  assert.match(continuousIntegration, /--target "\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}"/);
  assert.match(continuousIntegration, /npm run test:package/);
  assert.match(continuousIntegration, /FRAIA_REQUIRE_PACKAGED_CALCULIX: '1'/);
  assert.doesNotMatch(continuousIntegration, /build-calculix-(?:macos|linux)-runtime|build-calculix-windows-runtime/);
  assert.doesNotMatch(continuousIntegration, /curl |wget |actions\/download-artifact/);
});

test('native updater menu exposes manual checking and every supported persisted frequency', () => {
  for (const label of ['Never', 'On Startup', 'Hourly', 'Every 6 Hours', 'Every 12 Hours', 'Daily', 'Weekly']) {
    assert.match(mainProcess, new RegExp(label));
  }
  assert.match(mainProcess, /checkNow/);
  assert.match(mainProcess, /setFrequency/);
});

test('packaged updater code is shipped and Windows and Linux remain update-disabled', () => {
  assert.match(builder, /'update-manager\.cjs'/);
  const updater = fs.readFileSync(path.join(__dirname, '..', 'update-manager.cjs'), 'utf8');
  assert.match(updater, /platform !== 'darwin'/);
  assert.match(updater, /SIX_HOURS_MS/);
  assert.match(updater, /loopback-only/);
});

test('all third-party workflow actions are pinned to full commit SHAs', () => {
  const uses = [...`${workflow}\n${continuousIntegration}`.matchAll(/^\s*uses:\s*([^\s#]+)/gm)].map((match) => match[1]);
  assert.ok(uses.length > 0);
  for (const action of uses) assert.match(action, /^[^@]+@[a-f0-9]{40}$/);
});
