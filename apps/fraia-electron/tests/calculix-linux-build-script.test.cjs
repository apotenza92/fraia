const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const script = fs.readFileSync(
  path.resolve(__dirname, '../scripts/build-calculix-linux-runtime.sh'),
  'utf8',
);

test('Linux CalculiX vendor build pins every source input', () => {
  for (const digest of [
    '9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7',
    'be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0',
    'a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd',
    '15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93',
    'f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844',
    'cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed',
    '50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79',
    '231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c',
    '8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903',
    '9d6b43ce4d8de0c878bf16b54d8e7a10d9bd42b75178153e3af6a815bdc90f74',
  ]) {
    assert.match(script, new RegExp(digest));
  }
  assert.match(script, /curl --proto '=https' --tlsv1\.2 --fail/);
  assert.match(script, /--retry-all-errors/);
  assert.match(script, /shasum -a 256 -c -/);
  assert.match(script, /gcc-\$\{GCC_VERSION\}\/COPYING3/);
  assert.match(script, /gcc-\$\{GCC_VERSION\}\/COPYING\.RUNTIME/);
  assert.doesNotMatch(script, /\/usr\/share\/doc|COPYING\.RUNTIME\.gz/);
  assert.doesNotMatch(script, /www\.gnu\.org\/licenses/);
  assert.match(script, /BUILD_RECIPE\.md/);
  assert.match(script, /UBUNTU_CONTAINER_IMAGE='ubuntu:22\.04@sha256:0e0a0fc6/);
  assert.match(script, /UBUNTU_SNAPSHOT='20260720T000000Z'/);
  assert.match(script, /dpkg-query -W/);
});

test('Linux CalculiX vendor build requires native Ubuntu 22.04 and reproducible bytes', () => {
  assert.match(script, /VERSION_ID="22\.04"/);
  assert.match(script, /uname -m/);
  assert.match(script, /build_once "\$work_root\/build-one"/);
  assert.match(script, /build_once "\$work_root\/build-two"/);
  assert.match(script, /--build-id=none/);
  assert.doesNotMatch(script, /DYNAMIC_ARCH=/);
  assert.match(script, /openblas_target='ARMV8'/);
  assert.match(script, /openblas_target='CORE2'/);
  assert.match(script, /build_root\/arpack\/#\.\.\/arpack\//);
  assert.match(script, /diff -qr "\$work_root\/build-one\/payload"/);
  assert.match(script, /not byte-identical/);
});

test('Linux CalculiX vendor build closes dependencies and compatibility', () => {
  assert.match(script, /-static-libgfortran -static-libgcc/);
  assert.match(script, /cp -L "\$quadmath_path" "\$build_root\/payload\/libquadmath\.so\.0"/);
  assert.match(script, /patchelf --set-rpath "\\\$ORIGIN"/);
  assert.match(script, /resolved_quadmath/);
  assert.match(script, /libgfortran\|libgomp\|libopenblas\|libgcc_s/);
  assert.match(script, /GLIBC_SYMBOL_CEILING='2\.35'/);
  assert.match(script, /patchelf --print-rpath/);
  assert.match(script, /ldd "\$candidate"/);
});

test('Linux CalculiX vendor build reports captured logs before cleanup', () => {
  assert.match(script, /set -Eeuo pipefail/);
  assert.match(script, /trap 'report_failure \$\?' ERR/);
  assert.match(script, /tail -n 200 "\$log_file"/);
});

test('Linux CalculiX vendor build runs the pinned official solver fixture', () => {
  assert.match(script, /spring1\.inp/);
  assert.match(script, /grep -Fq 'Job finished'/);
  assert.match(script, /spring1\.dat/);
  assert.match(script, /spring1\.frd/);
  assert.match(script, /spring1\.sta/);
});

test('Linux CalculiX vendor build never replaces an existing output directory', () => {
  assert.match(script, /-e "\$output"/);
  assert.doesNotMatch(script, /rm -rf -- "\$output"/);
});
