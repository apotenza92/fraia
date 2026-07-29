const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const repositoryRoot = path.resolve(__dirname, '..', '..', '..');
const script = fs.readFileSync(
  path.join(__dirname, '..', 'scripts', 'build-calculix-windows-arm64-runtime.sh'),
  'utf8',
);
const workflow = fs.readFileSync(
  path.join(repositoryRoot, '.github', 'workflows', 'calculix-runtime-audit.yml'),
  'utf8',
);
const boundary = fs.readFileSync(path.join(__dirname, '..', 'package-boundary.cjs'), 'utf8');

test('Windows ARM64 audit pins source, recipe, repository, and toolchain inputs', () => {
  for (const expected of [
    '9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7',
    'be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0',
    'a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd',
    '15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93',
    'f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844',
    'cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed',
    '63200aa0d52ebb5cc8874c8813de06ba23d56c27',
    '4732fc54024f98145fd0dda0d109c58a8155e7c48777caecc2c894f8010f9d32',
    'a32ceb26e1e830227d4e982bc9a004c5168814b37cb9a602574f0c92dbf192f1',
    '66cd2cce69caa17b53920067426061ca1de3a884',
  ]) {
    assert.match(script, new RegExp(expected));
  }
  assert.match(script, /expected_packages/);
  assert.match(script, /22\.1\.8-2/);
  assert.match(script, /sha256sum -c/);
  assert.doesNotMatch(script, /calculix_2\.23_4win|ccx_static\.exe/);
});

test('Windows ARM64 audit is native, source-built, reproducible, and fail-closed', () => {
  assert.match(script, /RUNNER_ARCH:-} != 'ARM64'/);
  assert.match(script, /MSYSTEM:-} != 'CLANGARM64'/);
  assert.match(script, /patch sed sha256sum tar/);
  assert.match(script, /clang\.exe/);
  assert.match(script, /flang\.exe/);
  assert.match(script, /cygpath -w "\$clang_executable"/);
  assert.match(script, /llvm_ar_cmake=\$\(cygpath -m "\$llvm_ar_executable"\)/);
  assert.match(script, /"-DCMAKE_AR=\$llvm_ar_cmake"/);
  assert.match(script, /"-DCMAKE_RANLIB=\$llvm_ranlib_cmake"/);
  assert.match(script, /omp_fortran_module="\$MINGW_PREFIX\/include\/omp_lib\.mod"/);
  assert.match(script, /mingw_include_cmake=\$\(cygpath -m "\$MINGW_PREFIX\/include"\)/);
  assert.match(script, /COMPILE_LANGUAGE:Fortran>:-O2;-g0;-fopenmp;-cpp;-I%s/);
  assert.match(script, /cygpath -u "\$\{RUNNER_TEMP:/);
  assert.match(script, /source_root_cmake=\$\(cygpath -m "\$source_root"\)/);
  assert.match(script, /spooles_root_cmake=\$\(cygpath -m "\$spooles_root"\)/);
  assert.match(script, /-DMINGW64=1/);
  assert.match(script, /-DCMAKE_SYSTEM_PROCESSOR=ARM64/);
  assert.match(script, /-DBUILD_TESTING=OFF/);
  for (const patchName of [
    'ccx_mingw.patch',
    'ccx_ooc.patch',
    'ccx_numeric_format.patch',
    'ccx_adapt_main_pastix.patch',
  ]) {
    assert.match(script, new RegExp(patchName.replaceAll('.', '\\.')));
  }
  assert.match(script, /patch -d "\$ccx_source" -Np1/);
  assert.match(script, /-fcommon/);
  assert.match(script, /\^#define ARCH_ARM64/);
  assert.match(script, /Machine: IMAGE_FILE_MACHINE_ARM64 \(0xAA64\)/);
  assert.match(script, /canonical_build_root='\/c\/usr\/src\/fraia-runtime'/);
  assert.match(script, /build_once "\$canonical_build_root"/);
  assert.match(script, /mv "\$canonical_build_root" "\$work_root\/build-one"/);
  assert.match(script, /mv "\$canonical_build_root" "\$work_root\/build-two"/);
  assert.match(script, /forbidden_machine_paths/);
  assert.match(script, /GITHUB_WORKSPACE USERPROFILE/);
  assert.match(script, /llvm-strings "\$payload"\/\*/);
  assert.match(script, /not byte-identical/);
  assert.match(script, /--no-insert-timestamp/);
  assert.match(script, /-ffile-prefix-map=/);
  assert.doesNotMatch(script, /COMPILE_LANGUAGE:Fortran>[^\n]*ffile-prefix-map/);
  assert.doesNotMatch(script, /CMAKE_Fortran_FLAGS_RELEASE=[^\n]*ffile-prefix-map/);
  assert.match(script, /SOURCE_DATE_EPOCH/);
  assert.match(script, /FAILURE\.txt/);
  assert.match(script, /No runtime candidate was emitted/);
  assert.match(script, /write_reproducibility_failure_evidence/);
  assert.match(script, /CCX-BYTE-DIFFERENCES\.txt/);
  assert.match(script, /PAYLOAD-DIFF\.txt/);
  assert.match(script, /intermediates\/DIFF\.txt/);
  assert.match(script, /build-one-ccx\.exe/);
  assert.match(script, /build-two-ccx\.exe/);
  assert.match(script, /llvm-readobj --file-headers --sections --coff-imports/);
  assert.match(script, /llvm-objdump -h -p/);
  assert.match(script, /EVIDENCE_SHA256SUMS/);
  assert.match(script, /spring1\.inp/);
  assert.match(script, /Job finished/);
});

test('Windows ARM64 audit verifies platform contract, closure, and notices', () => {
  assert.match(script, /MINIMUM_WINDOWS_MAJOR='10'/);
  assert.match(script, /WINDOWS_SUBSYSTEM_MAJOR='6'/);
  assert.match(script, /--major-os-version/);
  assert.match(script, /expected_pe_versions/);
  assert.match(script, /\$1 == header_name/);
  assert.match(script, /header_value.*expected_pe_versions/s);
  assert.doesNotMatch(script, /MajorOSystemVersion {2,}/);
  assert.match(script, /allowed_system_dependencies/);
  assert.match(script, /bundled_dependencies/);
  assert.match(script, /libomp\.dll/);
  assert.match(script, /libwinpthread-1\.dll/);
  assert.match(script, /llvm-objdump -p/);
  for (const expected of [
    'THIRD_PARTY_NOTICES.txt',
    'CALCULIX-LICENSE-NOTICE.txt',
    'SPOOLES-NOTICE.txt',
    'ARPACK-BSD-3-Clause.txt',
    'OpenBLAS-BSD-3-Clause.txt',
    'LLVM-Flang-Apache-2.0-WITH-LLVM-exception.txt',
    'LLVM-OpenMP-Apache-2.0-WITH-LLVM-exception.txt',
    'winpthreads-MIT-AND-BSD-3-Clause.txt',
    'RUNTIME_SHA256SUMS',
    'EVIDENCE_SHA256SUMS',
  ]) {
    assert.match(script, new RegExp(expected.replaceAll('.', '\\.')));
  }
});

test('Windows ARM64 audit remains native and review-only at build time', () => {
  assert.match(workflow, /- win32-arm64/);
  assert.match(workflow, /runs-on: windows-11-arm/);
  assert.match(workflow, /runner\.arch.*ARM64/);
  assert.match(workflow, /msystem: CLANGARM64/);
  assert.match(workflow, /build-calculix-windows-arm64-runtime\.sh/);
  assert.match(workflow, /calculix-win32-arm64/);
  assert.match(boundary, /'win32-arm64'/);
});
