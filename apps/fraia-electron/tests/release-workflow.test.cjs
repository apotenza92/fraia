const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

const repositoryRoot = path.resolve(__dirname, '..', '..', '..');
const workflowPath = path.join(repositoryRoot, '.github', 'workflows', 'release.yml');
const workflow = fs.readFileSync(workflowPath, 'utf8');
const builder = fs.readFileSync(path.join(__dirname, '..', 'electron-builder.config.cjs'), 'utf8');
const signing = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'build-signed-macos.cjs'), 'utf8');
const updaterTest = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'test-macos-update.cjs'), 'utf8');
const continuousIntegration = fs.readFileSync(path.join(repositoryRoot, '.github', 'workflows', 'ci.yml'), 'utf8');
const packagedElectronTest = fs.readFileSync(
  path.join(__dirname, 'electron', 'packaged-app.spec.ts'),
  'utf8',
);
const desktopElectronTest = fs.readFileSync(
  path.join(__dirname, 'electron', 'conversation-first.spec.ts'),
  'utf8',
);
const packagedE2e = fs.readFileSync(
  path.join(__dirname, '..', 'scripts', 'run-packaged-e2e.cjs'),
  'utf8',
);
const signedMacosBuild = fs.readFileSync(
  path.join(__dirname, '..', 'scripts', 'build-signed-macos.cjs'),
  'utf8',
);
const runtimeAudit = fs.readFileSync(
  path.join(repositoryRoot, '.github', 'workflows', 'calculix-runtime-audit.yml'),
  'utf8',
);
const nonmacUpdaterAudit = fs.readFileSync(
  path.join(repositoryRoot, '.github', 'workflows', 'nonmac-updater-audit.yml'),
  'utf8',
);
const tufMetadataRefresh = fs.readFileSync(
  path.join(repositoryRoot, '.github', 'workflows', 'tuf-metadata-refresh.yml'),
  'utf8',
);
const tufSigningAudit = fs.readFileSync(
  path.join(repositoryRoot, '.github', 'workflows', 'tuf-signing-audit.yml'),
  'utf8',
);
const mainProcess = fs.readFileSync(path.join(__dirname, '..', 'main.js'), 'utf8');
const changelog = fs.readFileSync(path.join(repositoryRoot, 'CHANGELOG.md'), 'utf8');

function jobSource(jobId) {
  const match = workflow.match(
    new RegExp(`^  ${jobId}:\\n([\\s\\S]*?)(?=^  [A-Za-z0-9_-]+:\\n|(?![\\s\\S]))`, 'm'),
  );
  assert.ok(match, `workflow job ${jobId} is missing`);
  return match[1];
}

test('inline Node release workflow scripts are valid standalone programs', () => {
  const scripts = [...workflow.matchAll(/^[ \t]*node <<'NODE'\n([\s\S]*?)^[ \t]*NODE$/gm)]
    .map((match) => match[1]);
  assert.equal(scripts.length, 2, 'every inline Node release script must be syntax checked');
  scripts.forEach((script, index) => {
    assert.doesNotThrow(
      () => new vm.Script(script),
      `inline Node release script ${index + 1} must parse without a module or function wrapper`,
    );
  });
});

test('updater resolution uses the exact highest-SemVer feed predecessor across release classifications', () => {
  const resolver = [...workflow.matchAll(/^[ \t]*node <<'NODE'\n([\s\S]*?)^[ \t]*NODE$/gm)]
    .map((match) => match[1])
    .find((script) => script.includes('EXPECTED_PREVIOUS_VERSION'));
  assert.ok(resolver, 'updater release resolver is missing');

  const runResolver = (releasePages, environment) => {
    let output = '';
    new vm.Script(resolver).runInNewContext({
      process: {
        env: {
          GITHUB_OUTPUT: 'output',
          RELEASE_CHANNEL: 'stable',
          RELEASE_ARCH: 'x64',
          EXPECTED_PREVIOUS_VERSION: '0.0.2',
          ...environment,
        },
      },
      require: (specifier) => {
        assert.equal(specifier, 'node:fs');
        return {
          appendFileSync: (file, value) => {
            assert.equal(file, 'output');
            output += value;
          },
          readFileSync: (file, encoding) => {
            assert.equal(file, 'releases.json');
            assert.equal(encoding, 'utf8');
            return JSON.stringify(releasePages);
          },
        };
      },
    });
    return output;
  };

  assert.equal(
    runResolver([[
      {
        assets: [{ name: 'Fraia-macOS-x64.zip' }],
        draft: false,
        prerelease: false,
        published_at: '2026-07-28T00:00:00Z',
        tag_name: 'v0.0.2',
      },
    ]], {
      GITHUB_REF_NAME: 'v0.0.3',
    }),
    'tag=v0.0.2\nasset=Fraia-macOS-x64.zip\n',
  );
  assert.throws(
    () => runResolver([], {
      GITHUB_REF_NAME: 'v0.0.3',
    }),
    /has no matching Fraia-macOS-x64\.zip release asset/,
  );
  assert.equal(
    runResolver([[
      {
        assets: [{ name: 'Fraia-Beta-macOS-x64.zip' }],
        draft: false,
        prerelease: false,
        published_at: '2026-07-29T00:00:00Z',
        tag_name: 'v0.0.4',
      },
    ]], {
      EXPECTED_PREVIOUS_VERSION: '0.0.4',
      GITHUB_REF_NAME: 'v0.0.5-beta.1',
      RELEASE_CHANNEL: 'beta',
    }),
    'tag=v0.0.4\nasset=Fraia-Beta-macOS-x64.zip\n',
  );
});

test('stable-inclusive and beta releases are owner-authorized tags and native on six solver-backed targets', () => {
  assert.match(workflow, /tags:\n\s+- 'v\*'/);
  assert.doesNotMatch(workflow, /workflow_dispatch/);
  for (const runner of ['macos-15', 'macos-15-intel', 'windows-11-arm', 'windows-2025', 'ubuntu-24.04', 'ubuntu-24.04-arm']) {
    assert.match(workflow, new RegExp(runner.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
  assert.match(workflow, /artifactPrefix/);
  assert.match(workflow, /Require an owner-authorized release actor/);
  assert.match(workflow, /AUTHORIZED_RELEASE_ACTOR: apotenza92/);
  assert.match(workflow, /RELEASE_ACTOR: \$\{\{ github\.actor \}\}/);
  for (const environment of [
    'release-signing',
  ]) assert.match(workflow, new RegExp(environment));
  assert.match(workflow, /\$\{\{ matrix\.channel \}\}-updater-verification/);
  assert.match(workflow, /channel: \$\{\{ fromJSON\(needs\.prepare\.outputs\.channels\) \}\}/);
  assert.match(workflow, /readPublishedChannelVersion/);
  assert.match(workflow, /releasePolicy/);
  assert.doesNotMatch(workflow, /Approve stable release projection to Beta/);
  assert.doesNotMatch(workflow, /name: \$\{\{ needs\.prepare\.outputs\.environment \}\}/);
  assert.match(workflow, /vX\.Y\.Z-beta\.N/);
  assert.match(workflow, /\^v\\d\+\\\.\\d\+\\\.\\d\+-beta\\\.\\d\+\$/);
  assert.match(workflow, /actions\/attest@[a-f0-9]{40}/);
  assert.match(workflow, /cancel-in-progress: false/);
  assert.match(workflow, /Require maintained release icons/);
  assert.match(workflow, /Require the declared repository license/);
  assert.match(workflow, /run: test -f LICENSE/);
  assert.match(builder, /FRAIA_REQUIRE_RELEASE_ICON/);
  assert.match(builder, /minimumSystemVersion: '15\.0'/);
  assert.match(workflow, /Require a public updater and binary origin/);
  assert.match(workflow, /SOURCE_REPOSITORY_PRIVATE/);
  assert.match(workflow, /Require reviewed native CalculiX runtime manifests/);
  assert.match(workflow, /Assemble deterministic CalculiX corresponding source/);
  assert.match(workflow, /verify-calculix-corresponding-source\.cjs/);
  assert.match(workflow, /Fraia-CalculiX-Corresponding-Source\.tar/);
  assert.match(workflow, /FRAIA_REQUIRE_PACKAGED_CALCULIX: '1'/);
  assert.match(builder, /validateRuntimeDirectory/);
  const nativeVerifier = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'verify-native-package.cjs'), 'utf8');
  const macVerifier = fs.readFileSync(path.join(__dirname, '..', 'scripts', 'verify-macos-package.cjs'), 'utf8');
  assert.match(nativeVerifier, /runtime-manifest\.json/);
  assert.match(nativeVerifier, /waitForPathRemoval\(install\)/);
  assert.match(nativeVerifier, /maxRetries: 20/);
  assert.match(macVerifier, /runtime-manifest\.json/);
  assert.match(macVerifier, /LSMinimumSystemVersion/);
  assert.match(macVerifier, /reviewed macOS 15\.0 minimum/);
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
  assert.match(workflow, /Build reviewed native sidecar once/);
  assert.match(workflow, /Upload validated renderer/);
  assert.match(workflow, /Verify signatures and launch without release credentials/);
  assert.match(signing, /pass --skip-build/);
  assert.match(signing, /finalize-macos-update-artifacts\.mjs/);
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
  assert.match(validate, /cd apps\/fraia-electron/);
  assert.match(validate, /test -f build\/icon\.icns/);
  assert.match(validate, /verify-calculix-runtimes\.cjs --all --skip-dependency-inspection/);
  assert.doesNotMatch(validate, /secrets\.|^    environment:/m);
  assert.match(packageMacos, /needs: \[prepare, validate, build-sidecar\]/);
  assert.match(packageMacos, /environment: release-signing/);
  assert.match(packageMacos, /secrets\.APPLE_SIGNING_CERTIFICATE_P12_BASE64/);
  assert.match(validate, /npm run check:icons/);
  assert.match(validate, /npm run verify:import-runtimes:built/);
  assert.match(validate, /build\/macos\/Fraia\.icon\/Assets\/01-artwork-dark\.svg/);
});

test('release packages reuse one validated renderer and one native sidecar per target', () => {
  const validate = jobSource('validate');
  const sidecars = jobSource('build-sidecar');
  assert.match(validate, /name: fraia-validated-renderer/);
  assert.match(sidecars, /name: Native sidecar \$\{\{ matrix\.platform \}\} \$\{\{ matrix\.arch \}\}/);
  assert.equal((sidecars.match(/platform: (?:darwin|win32|linux)/g) || []).length, 6);
  assert.match(sidecars, /tar -czf "\$RUNNER_TEMP\/native-sidecar\.tar\.gz"/);
  for (const job of ['package-macos', 'package-windows', 'package-linux']) {
    const source = jobSource(job);
    assert.match(source, /needs: \[prepare, validate, build-sidecar\]/);
    assert.match(source, /name: fraia-validated-renderer/);
    assert.match(source, /Download reviewed native sidecar/);
    assert.match(source, /tar -xzf "\$RUNNER_TEMP\/native-sidecar\/native-sidecar\.tar\.gz"/);
    assert.doesNotMatch(source, /npm run build(?:\s|$)|npm run build:sidecar/);
  }
});

test('large release assets bypass updater-feed signing and public feed comparison', () => {
  const assemble = jobSource('assemble');
  const attest = jobSource('attest');
  const seal = jobSource('seal-tuf');
  const publish = jobSource('publish');
  const verify = jobSource('verify-publication');
  assert.match(assemble, /name: fraia-release-assets[\s\S]*path: publish\/assets/);
  assert.match(assemble, /name: fraia-update-feeds-unsigned[\s\S]*path: publish\/feed/);
  assert.match(attest, /name: fraia-release-assets[\s\S]*path: publish\/assets/);
  assert.doesNotMatch(seal, /fraia-release-assets|path: publish\/assets/);
  assert.match(seal, /name: fraia-update-feeds-unsigned[\s\S]*path: publish\/feed/);
  assert.match(seal, /name: fraia-update-feeds-sealed[\s\S]*path: publish\/feed/);
  assert.match(publish, /name: fraia-release-assets[\s\S]*name: fraia-update-feeds-sealed/);
  assert.match(verify, /name: fraia-update-feeds-sealed[\s\S]*path: expected\/feed/);
  assert.doesNotMatch(verify, /name: fraia-release-assets/);
});

test('each publication atomically advances only higher-SemVer identity-isolated updater feeds', () => {
  assert.match(workflow, /EXPECTED_PREVIOUS_VERSION/);
  assert.match(workflow, /expectedTag = `v\$\{process\.env\.EXPECTED_PREVIOUS_VERSION\}`/);
  assert.match(workflow, /APPLE_PRIOR_SIGNING_CERTIFICATE_SHA256/);
  assert.match(updaterTest, /priorExpectations/);
  assert.match(workflow, /needs: \[prepare, seal-tuf, attest\]/);
  assert.match(workflow, /environment: update-signing/);
  assert.match(workflow, /sign-tuf-update-repository\.cjs/);
  for (const role of ['TARGETS', 'SNAPSHOT', 'TIMESTAMP']) {
    assert.match(workflow, new RegExp(`secrets\\.FRAIA_TUF_${role}_PRIVATE_KEY_PEM`));
  }
  assert.doesNotMatch(workflow, /FRAIA_TUF_ROOT_PRIVATE_KEY/);
  assert.match(
    workflow,
    /gh release view "\$TAG" --repo "\$REPOSITORY" --json assets --jq '\.assets\[\]\.name'/,
  );
  assert.doesNotMatch(workflow, /gh api "repos\/\$REPOSITORY\/releases\/tags\/\$TAG"/);
  assert.match(workflow, /comm -23 existing-assets\.txt expected-assets\.txt/);
  assert.match(workflow, /cmp "publish\/assets\/\$name" "existing-release\/\$name"/);
  assert.doesNotMatch(workflow, /gh release (?:delete|upload)[^\n]*(?:--clobber|-R)/);
  assert.match(
    workflow,
    /name: fraia-update-feed-publication-\$\{\{ github\.ref_name \}\}[\s\S]{0,200}include-hidden-files: true/,
  );
  assert.ok(
    (workflow.match(/--retry 10 --retry-all-errors --retry-delay 2 "\$RELEASE_BASE\/\$filename"/g) || []).length >= 2,
    'both public release download passes must retry transient network failures',
  );
  assert.match(updaterTest, /updated-runtime-launched/);
  assert.match(updaterTest, /await waitForPidExit\(previousPid\)/);
  assert.match(updaterTest, /await waitForBundleVersion\(installedApp, prepared\.version\)/);
  assert.match(updaterTest, /await waitForVerifiedApp\(installedApp, contract, currentExpectations, prepared\.version\)/);
  assert.match(updaterTest, /candidate\.pid === child\.pid/);
  assert.match(updaterTest, /fs\.realpathSync\.native\(event\.executablePath\)/);
  assert.match(updaterTest, /'gh', \['attestation', 'verify'/);
  assert.match(updaterTest, /sha512\|checksum\|digest\|integrity/);
  assert.match(updaterTest, /signature\|code sign\|signed/);
  assert.match(updaterTest, /server\.requests\.includes\(prepared\.zipName\)/);
  const publishRelease = workflow.indexOf('- name: Publish verified release');
  const verifyPublic = workflow.indexOf('- name: Verify public release before sealing updater feed');
  const sealFeed = workflow.indexOf('- name: Prepare sealed updater-feed publication bundle');
  const uploadFeed = workflow.indexOf('- name: Upload sealed updater-feed publication bundle');
  const publishFeeds = workflow.indexOf('- name: Publish only the eligible updater channels atomically');
  assert.ok(
    publishRelease >= 0
      && publishRelease < verifyPublic
      && verifyPublic < sealFeed
      && sealFeed < uploadFeed
      && uploadFeed < publishFeeds,
    'the release must be public and byte-verified before its updater feed is published',
  );
  assert.match(workflow, /printf 'Publication: owner-authorized tag workflow by %s\\n' "\$\{\{ github\.actor \}\}"/);
  assert.match(workflow, /rm -rf feed-publication\n\s+mkdir -p feed-publication\n\s+touch feed-publication\/\.nojekyll/);
  assert.match(workflow, /mkdir -p "feed-publication\/feed\/\$CHANNEL"/);
  assert.match(workflow, /rm -rf "update-site\/\$CHANNEL"/);
  assert.match(workflow, /git add -- "\$\{PATHS\[@\]\}"/);
  assert.doesNotMatch(workflow, /for FEED_CHANNEL in stable beta|byte-identical stable and beta/);
  assert.match(workflow, /--channels "\$\{\{ needs\.prepare\.outputs\.channels_csv \}\}"/);
  assert.match(workflow, /uses: \.\/\.github\/workflows\/nonmac-updater-audit\.yml/);
  assert.match(workflow, /previous_version: \$\{\{ matrix\.channel == 'stable'/);
  assert.match(workflow, /git push origin HEAD:updates/);
  assert.doesNotMatch(workflow, /git add -A/);
  assert.match(workflow, /fraia-update-feed-publication-\$\{\{ github\.ref_name \}\}/);
  assert.match(workflow, /changelog\.cjs/);
  assert.match(workflow, /--notes-file release-notes\.md/);
  assert.doesNotMatch(workflow, /--generate-notes/);
  assert.match(workflow, /\.body' public-release\.json/);
  assert.match(builder, /releaseInfo/);
  assert.match(builder, /releaseNotes/);
  assert.match(changelog, /^# Changelog/m);
});

test('publication revalidates the reviewed Homebrew auxiliary asset explicitly', () => {
  const publish = jobSource('publish');
  assert.match(
    publish,
    /release-assets\.cjs[\s\S]*--allow-auxiliary-asset homebrew-publication\.tar\.gz/,
  );
});

test('Homebrew native validation installs the candidate from a disposable local tap', () => {
  const validation = jobSource('homebrew-native-validation');
  assert.match(validation, /timeout-minutes: 20/);
  assert.match(validation, /for attempt in 1 2 3 4/);
  assert.match(validation, /gh release download[\s\S]*gh attestation verify/);
  assert.match(validation, /failed after \$attempt attempts/);
  assert.match(validation, /Library\/Taps\/fraia-validation\/homebrew-release/);
  assert.match(validation, /HOMEBREW_NO_AUTO_UPDATE/);
  assert.match(validation, /run_with_timeout 720 brew install --verbose --cask "\$token"/);
  assert.match(validation, /run_with_timeout 120 brew uninstall --cask "\$token"/);
  assert.match(validation, /trap cleanup EXIT/);
  assert.match(validation, /rm -rf "\$\(brew --repository\)\/Library\/Taps\/fraia-validation"/);
  assert.doesNotMatch(validation, /brew install --cask "\$cask"/);
});

test('published TUF metadata refreshes on a trusted schedule without changing targets', () => {
  assert.match(workflow, /group: fraia-updater-publication/);
  assert.match(tufMetadataRefresh, /schedule:/);
  assert.match(tufMetadataRefresh, /cron: '17 3 \* \* 1'/);
  assert.match(tufMetadataRefresh, /if: github\.ref == 'refs\/heads\/main'/);
  assert.match(tufMetadataRefresh, /environment: update-signing/);
  assert.match(tufMetadataRefresh, /group: fraia-updater-publication/);
  assert.match(tufMetadataRefresh, /--previous-metadata "\$FEED\/tuf\/metadata"/);
  assert.match(tufMetadataRefresh, /--target "\$FEED\/\$TARGET_NAME"/);
  assert.match(tufMetadataRefresh, /git add -- stable\/win32 stable\/linux beta\/win32 beta\/linux/);
  assert.doesNotMatch(tufMetadataRefresh, /git add -A|gh release|APPLE_/);
});

test('protected production TUF keys are auditable without exposing private material', () => {
  assert.match(tufSigningAudit, /workflow_dispatch:/);
  assert.match(tufSigningAudit, /environment: update-signing/);
  assert.match(tufSigningAudit, /sign-tuf-update-repository\.cjs/);
  assert.match(tufSigningAudit, /sha256sum --check --strict SHA256SUMS/);
  assert.match(tufSigningAudit, /db88d4445135c02065824de9d035803bfc0b2b7a6eb0e5bb2fc57556e39d478e/);
  for (const role of ['TARGETS', 'SNAPSHOT', 'TIMESTAMP']) {
    assert.match(tufSigningAudit, new RegExp(`secrets\\.FRAIA_TUF_${role}_PRIVATE_KEY_PEM`));
  }
  assert.doesNotMatch(tufSigningAudit, /FRAIA_TUF_ROOT_PRIVATE_KEY|gh release|contents: write/);
});

test('manual CI keeps deterministic checks default and native package preflight explicitly opt-in', () => {
  assert.match(continuousIntegration, /workflow_dispatch:\n\s+inputs:/);
  assert.doesNotMatch(continuousIntegration, /^\s{2}pull_request:/m);
  assert.doesNotMatch(continuousIntegration, /^\s{2}push:/m);
  assert.match(continuousIntegration, /run_native_package_preflight:/);
  assert.match(continuousIntegration, /release_channel:[\s\S]*?options:\n\s+- stable\n\s+- beta/);
  assert.match(continuousIntegration, /default: false/);
  assert.match(continuousIntegration, /if: \$\{\{ inputs\.run_native_package_preflight \}\}/);
  assert.match(continuousIntegration, /run_macos_signing_audit:/);
  assert.match(continuousIntegration, /uses: \.\/\.github\/workflows\/macos-signing-audit\.yml/);
  assert.match(continuousIntegration, /channel: \$\{\{ inputs\.release_channel \}\}/);
  assert.match(continuousIntegration, /run_tuf_signing_audit:/);
  assert.match(continuousIntegration, /uses: \.\/\.github\/workflows\/tuf-signing-audit\.yml/);
  for (const runner of ['macos-26', 'macos-26-intel', 'windows-11-arm', 'windows-2025', 'ubuntu-24.04', 'ubuntu-24.04-arm']) {
    assert.match(continuousIntegration, new RegExp(runner.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  }
  assert.match(continuousIntegration, /platform: win32\n\s+arch: arm64\n\s+runner: windows-11-arm/);
  assert.match(continuousIntegration, /verify-calculix-runtimes\.cjs/);
  assert.match(continuousIntegration, /--target "\$\{\{ matrix\.platform \}\}-\$\{\{ matrix\.arch \}\}"/);
  assert.match(continuousIntegration, /npm run test:package/);
  assert.match(continuousIntegration, /--linux AppImage deb rpm --\$\{\{ matrix\.arch \}\} --publish never/);
  assert.match(continuousIntegration, /--win nsis --\$\{\{ matrix\.arch \}\} --publish never/);
  assert.match(continuousIntegration, /verify-native-package\.cjs/);
  assert.match(continuousIntegration, /FRAIA_REQUIRE_PACKAGED_CALCULIX: '1'/);
  assert.doesNotMatch(continuousIntegration, /build-calculix-(?:macos|linux)-runtime|build-calculix-windows-runtime/);
  assert.doesNotMatch(continuousIntegration, /curl |wget |actions\/download-artifact/);
});

test('native package checks pin the reviewed macOS icon toolchain and deterministic Linux renderer', () => {
  for (const source of [workflow, continuousIntegration]) {
    assert.match(source, /DEVELOPER_DIR: \/Applications\/Xcode_26\.1\.1\.app\/Contents\/Developer/);
    assert.match(source, /FRAIA_XCODE_VERSION: '26\.1\.1'/);
    assert.match(source, /test "\$\(xcodebuild -version \| sed -n '1p'\)" = "Xcode \$FRAIA_XCODE_VERSION"/);
    assert.match(source, /test "\$\(xcrun --find actool\)" = "\$DEVELOPER_DIR\/usr\/bin\/actool"/);
  }
  assert.match(continuousIntegration, /runner: macos-26/);
  assert.match(continuousIntegration, /runner: macos-26-intel/);
  assert.match(workflow, /runs-on: \$\{\{ matrix\.arch == 'arm64' && 'macos-26' \|\| 'macos-26-intel' \}\}/);
  const updaterMacos = jobSource('test-macos-updater');
  assert.match(updaterMacos, /macos-15/);
  assert.match(updaterMacos, /macos-15-intel/);
  for (const source of [packagedElectronTest, desktopElectronTest]) {
    assert.match(source, /["']--use-gl=angle["'], ["']--use-angle=swiftshader["'], ["']--enable-unsafe-swiftshader["']/);
  }
  assert.match(packagedElectronTest, /"--no-sandbox"/);
  assert.match(packagedElectronTest, /"XAUTHORITY"/);
  assert.match(packagedE2e, /entry\.replaceAll\('\\\\', '\/'\)/);
  assert.match(packagedE2e, /packagePath\.split\('\/'\)\.join\(path\.sep\)/);
  assert.match(packagedE2e, /require\.resolve\('@playwright\/test\/cli'\)/);
  assert.match(packagedE2e, /spawnSync\(process\.execPath/);
  assert.match(packagedE2e, /assertMacosMinimumVersion/);
  assert.match(packagedElectronTest, /test\.setTimeout\(300_000\)/);
  assert.match(packagedElectronTest, /\[packaged-e2e\]/);
  assert.match(continuousIntegration, /AssetCatalogAgent-AssetRuntime/);
  assert.match(continuousIntegration, /Retrying once after a verified Xcode AssetCatalogAgent infrastructure crash/);
  assert.match(signedMacosBuild, /AssetCatalogAgent-AssetRuntime/);
  assert.match(signedMacosBuild, /runElectronBuilderWithActoolRetry/);
  assert.match(signedMacosBuild, /A timestamp was expected but was not found/);
  assert.match(signedMacosBuild, /attempt <= 3/);
});

test('native runtime audit uses Bash for strict Linux container execution', () => {
  const linuxJob = runtimeAudit.match(/^  linux:\n([\s\S]*?)\n  [A-Za-z0-9_-]+:\n/m)?.[1]
    ?? runtimeAudit.match(/^  linux:\n([\s\S]*)$/m)?.[1];
  assert.ok(linuxJob, 'Linux runtime audit job is missing');
  assert.match(linuxJob, /working-directory: apps\/fraia-electron\n\s+shell: bash\n\s+run: \|\n\s+set -euo pipefail/);
});

test('Windows runtime audit reuses only hash-reviewed source evidence from an exact run', () => {
  assert.match(runtimeAudit, /reviewed_source_run_id:/);
  assert.match(runtimeAudit, /actions: read/);
  assert.match(runtimeAudit, /win32-x64 requires reviewed_source_run_id/);
  assert.match(runtimeAudit, /actions\/download-artifact@[a-f0-9]{40}/);
  assert.match(runtimeAudit, /github-token: \$\{\{ github\.token \}\}/);
  assert.match(runtimeAudit, /run-id: \$\{\{ inputs\.reviewed_source_run_id \}\}/);
  assert.match(runtimeAudit, /-ReviewedSourceDirectory/);
});

test('native updater menu exposes manual checking and every supported persisted frequency', () => {
  for (const label of ['Never', 'On Startup', 'Hourly', 'Every 6 Hours', 'Every 12 Hours', 'Daily', 'Weekly']) {
    assert.match(mainProcess, new RegExp(label));
  }
  assert.match(mainProcess, /label: 'Check for Updates…'/);
  assert.match(mainProcess, /label: 'Automatic Checks'/);
  assert.doesNotMatch(mainProcess, /label: 'Updates'/);
  assert.match(mainProcess, /checkNow/);
  assert.match(mainProcess, /setFrequency/);
});

test('application menu delegates reload and whole-window zoom shortcuts to Electron roles', () => {
  for (const role of ['reload', 'forceReload', 'resetZoom', 'zoomIn', 'zoomOut', 'togglefullscreen']) {
    assert.match(mainProcess, new RegExp(`role: ['"]${role}['"]`));
  }
  const explicitCommandAccelerators = [...mainProcess.matchAll(
    /accelerator:\s*['"]((?:Cmd|Command|Ctrl|Control)[^'"]*)['"]/g,
  )].map((match) => match[1]);
  assert.deepEqual(explicitCommandAccelerators, [
    'CommandOrControl+S',
    'CommandOrControl+Shift+S',
  ]);
});

test('packaged updater code ships TUF verification for Windows and Linux', () => {
  assert.match(builder, /'update-manager\.cjs'/);
  assert.match(builder, /'tuf-update-feed\.cjs'/);
  assert.match(builder, /'release-contract\.cjs'/);
  assert.match(builder, /FRAIA_REQUIRE_TUF_ROOT/);
  assert.equal(
    [...workflow.matchAll(/FRAIA_REQUIRE_TUF_ROOT: '1'/g)].length,
    2,
    'Windows and Linux release packages must fail closed without production TUF trust',
  );
  const updater = fs.readFileSync(path.join(__dirname, '..', 'update-manager.cjs'), 'utf8');
  assert.match(updater, /createTufVerifiedUpdateFeed/);
  assert.match(updater, /platform !== 'darwin'/);
  assert.match(updater, /linux-package-manager/);
  assert.match(updater, /allowPrerelease = channel === 'beta'/);
  assert.match(updater, /loopback-only/);
  assert.match(updater, /download-progress/);
  assert.match(mainProcess, /broadcastUpdateStatus/);
  const updateDialog = fs.readFileSync(
    path.join(__dirname, '..', 'src', 'components', 'updates', 'UpdateDialog.tsx'),
    'utf8',
  );
  assert.match(updateDialog, /Restart and update/);
  assert.match(updateDialog, />\s*Later\s*</);
  assert.match(updateDialog, /ProgressLabel/);
  assert.match(updateDialog, /formatEta/);
});

test('packaged sidecar startup stays fail-fast while Cargo development startup has build headroom', () => {
  assert.match(mainProcess, /waitForHealth\(timeoutMs = app\.isPackaged \? 3000 : 60_000\)/);
});

test('packaged main process includes every explicitly required local module', () => {
  const localModules = [...mainProcess.matchAll(/require\(['"]\.\/([^'"]+)['"]\)/g)]
    .map((match) => match[1]);
  for (const localModule of localModules) {
    assert.match(
      builder,
      new RegExp(`['"]${localModule.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}['"]`),
      `${localModule} must be included in the packaged application`,
    );
  }
});

test('all third-party workflow actions are pinned to full commit SHAs', () => {
  const uses = [
    ...`${workflow}\n${continuousIntegration}\n${runtimeAudit}\n${nonmacUpdaterAudit}\n${tufMetadataRefresh}\n${tufSigningAudit}`
      .matchAll(/^\s*uses:\s*([^\s#]+)/gm),
  ].map((match) => match[1]);
  assert.ok(uses.length > 0);
  for (const action of uses) {
    if (action.startsWith('./')) {
      assert.match(action, /^\.\/\.github\/workflows\/[A-Za-z0-9_.-]+\.yml$/);
    } else {
      assert.match(action, /^[^@]+@[a-f0-9]{40}$/);
    }
  }
});
