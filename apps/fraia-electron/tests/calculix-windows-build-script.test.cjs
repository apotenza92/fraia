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
  assert.match(script, /\$MaximumAttempts = 4/);
  assert.match(script, /Reviewed download failed after/);
  assert.doesNotMatch(script, /Fallback|MirrorUrl/);
  assert.match(script, /\$ReviewedSourceDirectory/);
  assert.match(script, /Copy-Item -LiteralPath \$ReviewedInput -Destination \$Destination/);
  assert.match(script, /Assert-Sha256 -Path \$Destination -Expected \$Entry\.Value\[1\]/);
  assert.match(script, /WinLibs source commit/);
  assert.match(script, /gcc-16\.1\.0\/COPYING3/);
  assert.doesNotMatch(script, /www\.gnu\.org\/licenses/);
});

test('Windows CalculiX vendor build is native, source-built, and reproducible', () => {
  assert.match(script, /native Windows x64 host/);
  assert.match(script, /OSArchitecture -ne "X64"/);
  assert.match(script, /Build-Once/);
  assert.match(script, /\$ControlledBuildDrive = "R:"/);
  assert.match(script, /subst\.exe/);
  assert.match(script, /deterministic build drive .* already in use/);
  assert.match(script, /Failed to remove reviewed deterministic build drive/);
  assert.match(script, /\$BuildOne/);
  assert.match(script, /\$BuildTwo/);
  assert.match(script, /byte-identical/);
  assert.match(script, /SOURCE_DATE_EPOCH/);
  assert.match(script, /--no-insert-timestamp/);
  assert.match(script, /-ffile-prefix-map=/);
  assert.match(script, /-fmacro-prefix-map=/);
  assert.match(script, /-fdebug-prefix-map=/);
  assert.match(script, /-fcanon-prefix-map/);
  assert.match(script, /\$NativePrefixMap = "-ffile-prefix-map=\$\{BuildRoot\}=/);
  assert.match(script, /\$UnixPrefixMap = "-ffile-prefix-map=\$\{BuildRootUnix\}=/);
  assert.match(script, /\$NativeMacroPrefixMap = "-fmacro-prefix-map=\$\{BuildRoot\}=/);
  assert.match(script, /\$UnixMacroPrefixMap = "-fmacro-prefix-map=\$\{BuildRootUnix\}=/);
  assert.match(script, /\$NativeDebugPrefixMap = "-fdebug-prefix-map=\$\{BuildRoot\}=/);
  assert.match(script, /\$UnixDebugPrefixMap = "-fdebug-prefix-map=\$\{BuildRootUnix\}=/);
  assert.match(script, /\[=\[\$\{NativePrefixMap\}\]=\]/);
  assert.match(script, /prefix-map-probe\.f90/);
  assert.match(script, /Pinned gfortran probe source string/);
  assert.match(script, /retained the physical build path despite the controlled source root/);
  assert.match(script, /did not emit the reviewed controlled source path/);
  assert.match(script, /@\(\$WorkRoot, \$WorkRootUnix\)/);
  assert.match(script, /IVinit\(nfront, NULL\)/);
  assert.match(script, /IVinit\(nfront, 0\)/);
  assert.match(script, /correction no longer applies exactly three times/);
  assert.match(script, /list\(FILTER SPOOLES_SOURCES EXCLUDE REGEX `"\x2fMPI\x2f`"\)/);
  assert.match(script, /\[A-Za-z0-9_\.\]\+\[.\]\(\?:c\|f\)/);
  assert.match(script, /listed-but-absent source set/);
  assert.match(script, /\$MissingListedSources -join "`n"\) -ne "mafillmm\.c"/);
  assert.match(script, /Join-Path \$CalculixSource "mafillmm\.f"/);
  assert.match(script, /COMPILE_LANGUAGE:Fortran>:-O2;-g0;-fallow-argument-mismatch;-fopenmp;-cpp>/);
  assert.doesNotMatch(script, /`"-ffile-prefix-map=\$\{BuildRootUnix\}/);
  assert.match(script, /ccx_2\.23\.c", "ccx_2\.23step\.c/);
  assert.match(script, /MinGW output-format correction no longer applies exactly once/);
  assert.match(script, /\$SourceText\.Replace\(\$GlobalWindowsFormatBlock, ""\)/);
  assert.match(script, /readnewmesh\.c void-return correction no longer applies exactly once/);
  assert.match(script, /void \\\*genratiomt/);
  assert.match(script, /genratiomt thread return is not preserved exactly once/);
  assert.doesNotMatch(script, /\$ReadNewMeshSource\.Replace\("return NULL;", "return;"\)/);
  assert.match(script, /reproducibility-failure/);
  assert.match(script, /ccx-build-one\.exe/);
  assert.match(script, /ccx-build-two\.exe/);
  assert.match(script, /No runtime candidate was emitted/);
  assert.match(script, /Move-Item -LiteralPath \$ReproducibilityFailure -Destination \$ResolvedEvidence/);
  assert.match(script, /fraia-calculix-windows-v16/);
  assert.doesNotMatch(script, /calculix_2\.23_4win|ccx_static\.exe/);
});

test('Windows CalculiX vendor build records command output with a typed string collection', () => {
  assert.match(script, /\[string\[\]\]\$Lines = @\(/);
  assert.match(script, /AppendAllLines\(\$LogPath, \$Lines, \$Utf8NoBom\)/);
  assert.match(script, /Console\]::Error\.WriteLine\(\(\$Tail -join \[Environment\]::NewLine\)\)/);
  assert.doesNotMatch(script, /\$Tail \| Write-Error/);
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
