const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const script = fs.readFileSync(
  path.resolve(__dirname, '../scripts/build-calculix-macos-runtime.sh'),
  'utf8',
);

test('macOS CalculiX vendor build pins every downloaded input', () => {
  for (const digest of [
    '9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7',
    'be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0',
    'a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd',
    'bd86b9adf3152bda8a21b3b5faf65a877b209be0f33c4629e2073a073ea5d706',
    '231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c',
    '8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903',
    '50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79',
    '5f4b4fe9aab99c021d23b2c1da9025e70b502e275076da12a64fc6196db6f3d3',
    '1593153257db78c270282742088ffe961b44d793f7bbaa458895357094d6f7fc',
    '6839eac9682dee9c9ab28ab96c5f6308a3a2d96ed499fbb4c43e10d6cc3691a5',
    '74045addfa1423d6ae6c61b1262bf5dceab762da3139a8882d1c3efd4f67407e',
    '6683d73d6677d28e1e8d1b92d6ebfbc068c1d33e19b79114a22a648a99ba5991',
    'd1192da68b2618652f4be0dd9f56b18d2d276481440ae241ce9cc17be0450e07',
    'de143fddb0e20b6b73016ead1e625ebd429db53918200d093e4da98f1e758889',
    'edae3d6050998a8b6c40d79244d1c73231537371e7a36a3a72f756ed965088be',
    'ed822b7e77645d7c17abb3ee9cc2b2a82a4d0f003acc7615b5df6226031479b2',
    'ba4a1b8388386e6618de7c7e27199ae8de473373330f5773e2095567a71d76fd',
    'e7723a06cf55d69322ada010ad25c6b34627674729e41d89f2526edfa7ba6995',
    '6c035aa0556baf634ceda0edc4415b6f03d675568873b6ffec4b8c6146639f44',
    'd72adf48460a8384b256f88061cd7b9df4977df7fa2e0794051d427db754a565',
    '8b2443dfa62b9d28cf0321e0e670bb096b2680fe72739999228291f36018311f',
    'f0f4f9effd2eec229e9a4ddb64a30343fe2d0fd65ac50aaf70bb842f339e4f7a',
    'd9a30a7d0564d4d6834d4c7a54691914525351763c72df20902891c557cdee80',
    'eeac013cf9a379d609478f2225c9efae61b3fa1ba8913fded18fa928e3d49ce6',
    'ef46cf36258063e563b5576ac1830e26b7a7bcaaa31280786054dea999fee487',
    '524a4e00ee656fe87b2f14b828f2ad14a186f0baa0c888d900c0392f4a7253e6',
    'cca33f287f5bbbdef8f41ea57add8e954a05ecf5e6aa533aa54e5a2a3e56b8b4',
  ]) {
    assert.match(script, new RegExp(digest));
  }
  assert.match(script, /curl --proto '=https' --tlsv1\.2 --fail/);
  assert.match(script, /--retry-all-errors/);
  assert.match(script, /shasum -a 256 -c -/);
  assert.match(script, /gcc-\$\{GCC_VERSION\}\/COPYING3/);
  assert.doesNotMatch(script, /www\.gnu\.org\/licenses/);
  assert.match(script, /gcc\/\$GCC_VERSION\/bin\/gfortran-16/);
  assert.match(script, /"\$gfortran_driver" -print-prog-name=f951/);
  assert.match(script, /"\$gfortran_driver" -print-prog-name=collect2/);
  assert.doesNotMatch(script, /command -v gfortran/);
  assert.match(script, /install_compiler_dependency gmp/);
  assert.match(script, /install_compiler_dependency zstd/);
  assert.match(script, /export DYLD_LIBRARY_PATH="\$compiler_support_directory"/);
  const compilerRelocationBlock = script.slice(
    script.indexOf('for owner in "$gfortran_f951"'),
    script.indexOf('macos_sdk='),
  );
  assert.doesNotMatch(compilerRelocationBlock, /install_name_tool/);
  assert.match(script, /install_name_tool -id "@loader_path/);
  assert.match(script, /codesign --verify --strict "\$owner"/);
  assert.match(script, /unsigned, exact SHA-256 verified/);
  assert.match(script, /"\$owner" != "\$gfortran_f951"/);
  assert.match(script, /chmod 600 "\$auth_config"/);
  assert.match(script, /--config "\$auth_config"/);
  assert.match(script, /unset registry_token/);
  assert.match(script, /rm -f -- "\$auth_config"/);
  assert.doesNotMatch(script, /-H "Authorization: Bearer \$registry_token"/);
  assert.match(script, /Reviewed %s SHA-256 mismatch/);
});

test('macOS CalculiX vendor build is native, reproducible, and solver-tested', () => {
  assert.match(script, /uname -m/);
  assert.match(script, /build_once "\$work_root\/build-one"/);
  assert.match(script, /build_once "\$work_root\/build-two"/);
  assert.match(script, /cmp "\$candidate" "\$work_root\/build-two\/payload\/\$name"/);
  assert.match(script, /MACOSX_DEPLOYMENT_TARGET='15\.0'/);
  assert.match(script, /xcrun --sdk macosx --show-sdk-path/);
  assert.match(script, /FFLAGS="[^"]*-isysroot \$macos_sdk/);
  assert.doesNotMatch(script, /-isysroot=\$macos_sdk/);
  assert.match(script, /minimum macOS %s exceeds the reviewed %s ceiling/);
  assert.match(script, /-Wl,-rpath,@loader_path/);
  assert.match(script, /-Wl,-no_adhoc_codesign/);
  assert.match(script, /install_name_tool -delete_rpath/);
  assert.match(script, /stripped of signatures only after all/);
  assert.match(script, /owner_basename" == \*\.dylib/);
  assert.match(
    script,
    /install_name_tool -id "@loader_path\/\$owner_basename" "\$owner"/,
  );
  assert.match(script, /Bundled GCC library has an unreviewed install ID/);
  assert.match(script, /exactly the @loader_path runpath/);
  assert.match(script, /@loader_path/);
  assert.match(script, /spring1\.inp/);
  assert.match(script, /grep -Fq 'Job finished'/);
  assert.match(script, /BUILD_RECIPE\.md/);
  assert.match(script, /THIRD_PARTY_NOTICES\.txt/);
  assert.match(script, /CALCULIX-LICENSE-NOTICE\.txt/);
  assert.match(script, /SPOOLES-NOTICE\.txt/);
  assert.match(script, /ARPACK-BSD-3-Clause\.txt/);
  assert.match(script, /GCC-Runtime-Library-Exception-3\.1\.txt/);
  assert.match(script, /find \. -type f ! -name SHA256SUMS/);
  assert.match(script, /gfortran driver SHA-256/);
  assert.match(script, /Build script SHA-256/);
  assert.match(script, /CalculiX macOS build failed; captured logs follow/);
  assert.match(script, /tail -n 120/);
});

test('macOS CalculiX vendor build never replaces an existing output directory', () => {
  assert.match(script, /-e "\$output"/);
  assert.doesNotMatch(script, /rm -rf -- "\$output"/);
});
