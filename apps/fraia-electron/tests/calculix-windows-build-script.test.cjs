const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const script = fs.readFileSync(
  path.resolve(__dirname, '../scripts/build-calculix-windows-runtime.ps1'),
  'utf8',
);

test('Windows CalculiX vendor build pins every source and toolchain input', () => {
  for (const hash of [
    '9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7',
    'be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0',
    'a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd',
    '15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93',
    'f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844',
    'cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed',
    '4273565109cd8ab8ecef1b0dc2a56fd7f5c7ee0885840a1c011b9325160ec0c3',
    'df21e66d385972cb4cdb2c7fa55da191d0c3841bbf14a76a54bc3a56c199923d',
    '50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79',
    'd71cc644cd5a37c337f2719f3e0c79d89e8d8d5fb9e2952a62d3fa23623dc137',
    '231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c',
    '8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903',
  ]) {
    assert.match(script, new RegExp(hash));
  }
  assert.match(script, /16\.1\.0posix-14\.0\.0-ucrt-r3/);
  assert.match(script, /6e253eff2be383861ae0bf44eccbf6bfef931bf8/);
  assert.match(script, /Assert-Sha256/);
  assert.match(script, /WinLibs source commit/);
  assert.match(script, /gcc-16\.1\.0\/COPYING3/);
  assert.doesNotMatch(script, /www\.gnu\.org\/licenses/);
});

test('Windows CalculiX vendor build is native, source-built, and reproducible', () => {
  assert.match(script, /native Windows x64 host/);
  assert.match(script, /OSArchitecture -ne "X64"/);
  assert.match(script, /Build-Once/);
  assert.match(script, /\$BuildOne/);
  assert.match(script, /\$BuildTwo/);
  assert.match(script, /byte-identical/);
  assert.match(script, /SOURCE_DATE_EPOCH/);
  assert.match(script, /--no-insert-timestamp/);
  assert.match(script, /-ffile-prefix-map=/);
  assert.doesNotMatch(script, /calculix_2\.23_4win|ccx_static\.exe/);
});

test('Windows CalculiX vendor build enforces architecture, Windows 10, and dependency closure', () => {
  assert.match(script, /\$Machine -ne 0x8664/);
  assert.match(script, /MajorOSystemVersion/);
  assert.match(script, /MajorSubsystemVersion/);
  assert.match(script, /MinimumWindowsMajor = 10/);
  assert.match(script, /Get-PeImports/);
  assert.match(script, /AllowedSystemImports/);
  assert.match(script, /UnexpectedImports/);
  assert.match(script, /-static-libgcc/);
  assert.match(script, /-static-libgfortran/);
  assert.match(script, /winpthreads/);
  assert.match(script, /Absolute build-path scan: pass/);
});

test('Windows CalculiX vendor build bundles notices and independent review evidence', () => {
  for (const expected of [
    'THIRD_PARTY_NOTICES.txt',
    'CALCULIX-LICENSE-NOTICE.txt',
    'SPOOLES-NOTICE.txt',
    'ARPACK-BSD-3-Clause.txt',
    'OpenBLAS-BSD-3-Clause.txt',
    'GCC-Runtime-Library-Exception-3.1.txt',
    'winpthreads-MIT.txt',
    'BUILD_RECIPE.md',
    'RUNTIME_SHA256SUMS',
    'EVIDENCE_SHA256SUMS',
    'source-inputs',
    'reproducibility',
    'toolchain',
  ]) {
    assert.match(script, new RegExp(expected.replaceAll('.', '\\.')));
  }
});

test('Windows CalculiX vendor build runs the pinned official solver fixture', () => {
  assert.match(script, /spring1\.inp/);
  assert.match(script, /Start-Process/);
  assert.match(script, /"dat", "frd", "sta"/);
  assert.match(script, /"Job finished"/);
  assert.match(script, /spring1\.stdout/);
  assert.match(script, /spring1\.stderr/);
});

test('Windows CalculiX vendor build never replaces an output or evidence directory', () => {
  assert.match(script, /The output directory already exists/);
  assert.match(script, /The evidence directory already exists/);
  assert.match(script, /must not contain one another/);
  assert.doesNotMatch(script, /Remove-Item -LiteralPath \$ResolvedOutput/);
  assert.doesNotMatch(script, /Remove-Item -LiteralPath \$ResolvedEvidence/);
});
