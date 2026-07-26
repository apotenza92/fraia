#!/bin/bash

set -euo pipefail

CCX_VERSION='2.23'
CCX_SOURCE_URL='https://www.dhondt.de/ccx_2.23.src.tar.bz2'
CCX_SOURCE_SHA256='9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7'
CCX_TEST_URL='https://www.dhondt.de/ccx_2.23.test.tar.bz2'
CCX_TEST_SHA256='be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0'
SPOOLES_URL='https://www.netlib.org/linalg/spooles/spooles.2.2.tgz'
SPOOLES_SHA256='a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd'
ARPACK_REVISION='40329031ae8deb7c1e26baf8353fa384fc37c251'
ARPACK_URL="https://github.com/opencollab/arpack-ng/archive/${ARPACK_REVISION}.tar.gz"
ARPACK_SHA256='bd86b9adf3152bda8a21b3b5faf65a877b209be0f33c4629e2073a073ea5d706'
GPL2_SHA256='231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c'
GPL3_SHA256='8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903'
GCC_VERSION='16.1.0'
GCC_SOURCE_URL="https://ftpmirror.gnu.org/gnu/gcc/gcc-${GCC_VERSION}/gcc-${GCC_VERSION}.tar.xz"
GCC_SOURCE_SHA256='50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79'
HOMEBREW_CORE_REVISION='1a2659e79c546348874da58b878ce326426749c4'
HOMEBREW_GCC_FORMULA_URL="https://raw.githubusercontent.com/Homebrew/homebrew-core/${HOMEBREW_CORE_REVISION}/Formula/g/gcc.rb"
HOMEBREW_GCC_FORMULA_SHA256='5f4b4fe9aab99c021d23b2c1da9025e70b502e275076da12a64fc6196db6f3d3'
HOMEBREW_GCC_PATCH_URL="https://raw.githubusercontent.com/Homebrew/homebrew-core/${HOMEBREW_CORE_REVISION}/Patches/gcc/gcc-16.1.0.diff"
HOMEBREW_GCC_PATCH_SHA256='1593153257db78c270282742088ffe961b44d793f7bbaa458895357094d6f7fc'
GCC_ARM64_SEQUOIA_BOTTLE_SHA256='6839eac9682dee9c9ab28ab96c5f6308a3a2d96ed499fbb4c43e10d6cc3691a5'
GCC_X64_SEQUOIA_BOTTLE_SHA256='74045addfa1423d6ae6c61b1262bf5dceab762da3139a8882d1c3efd4f67407e'
GMP_VERSION='6.3.0'
GMP_ARM64_SEQUOIA_BOTTLE_SHA256='6683d73d6677d28e1e8d1b92d6ebfbc068c1d33e19b79114a22a648a99ba5991'
GMP_X64_SEQUOIA_BOTTLE_SHA256='d1192da68b2618652f4be0dd9f56b18d2d276481440ae241ce9cc17be0450e07'
ISL_VERSION='0.27'
ISL_ARM64_SEQUOIA_BOTTLE_SHA256='de143fddb0e20b6b73016ead1e625ebd429db53918200d093e4da98f1e758889'
ISL_X64_SEQUOIA_BOTTLE_SHA256='edae3d6050998a8b6c40d79244d1c73231537371e7a36a3a72f756ed965088be'
MPFR_VERSION='4.2.2'
MPFR_ARM64_SEQUOIA_BOTTLE_SHA256='ed822b7e77645d7c17abb3ee9cc2b2a82a4d0f003acc7615b5df6226031479b2'
MPFR_X64_SEQUOIA_BOTTLE_SHA256='ba4a1b8388386e6618de7c7e27199ae8de473373330f5773e2095567a71d76fd'
LIBMPC_VERSION='1.4.1'
LIBMPC_ARM64_SEQUOIA_BOTTLE_SHA256='e7723a06cf55d69322ada010ad25c6b34627674729e41d89f2526edfa7ba6995'
LIBMPC_X64_SEQUOIA_BOTTLE_SHA256='6c035aa0556baf634ceda0edc4415b6f03d675568873b6ffec4b8c6146639f44'
ZSTD_VERSION='1.5.7_1'
ZSTD_ARM64_SEQUOIA_BOTTLE_SHA256='d72adf48460a8384b256f88061cd7b9df4977df7fa2e0794051d427db754a565'
ZSTD_X64_SEQUOIA_BOTTLE_SHA256='8b2443dfa62b9d28cf0321e0e670bb096b2680fe72739999228291f36018311f'
GFORTRAN_ARM64_DRIVER_SHA256='f0f4f9effd2eec229e9a4ddb64a30343fe2d0fd65ac50aaf70bb842f339e4f7a'
GFORTRAN_ARM64_F951_SHA256='d9a30a7d0564d4d6834d4c7a54691914525351763c72df20902891c557cdee80'
GFORTRAN_ARM64_COLLECT2_SHA256='eeac013cf9a379d609478f2225c9efae61b3fa1ba8913fded18fa928e3d49ce6'
GFORTRAN_X64_DRIVER_SHA256='ef46cf36258063e563b5576ac1830e26b7a7bcaaa31280786054dea999fee487'
GFORTRAN_X64_F951_SHA256='524a4e00ee656fe87b2f14b828f2ad14a186f0baa0c888d900c0392f4a7253e6'
GFORTRAN_X64_COLLECT2_SHA256='cca33f287f5bbbdef8f41ea57add8e954a05ecf5e6aa533aa54e5a2a3e56b8b4'
MACOSX_DEPLOYMENT_TARGET='15.0'
SOURCE_DATE_EPOCH='1762047462'

usage() {
  printf 'Usage: %s --target darwin-arm64|darwin-x64 --output <new-directory> --evidence <new-directory>\n' "$0" >&2
}

target=''
output=''
evidence=''
while (($#)); do
  case "$1" in
    --target)
      target=${2:-}
      shift 2
      ;;
    --output)
      output=${2:-}
      shift 2
      ;;
    --evidence)
      evidence=${2:-}
      shift 2
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$target" || -z "$output" || -z "$evidence" || -e "$output" || -e "$evidence" ]]; then
  usage
  printf 'The output and evidence directories must not already exist.\n' >&2
  exit 2
fi
if [[ "$output" == "$evidence" ]]; then
  printf 'The output and evidence directories must be distinct.\n' >&2
  exit 2
fi

case "$target" in
  darwin-arm64)
    expected_machine='arm64'
    gcc_bottle_sha256=$GCC_ARM64_SEQUOIA_BOTTLE_SHA256
    gfortran_driver_sha256=$GFORTRAN_ARM64_DRIVER_SHA256
    gfortran_f951_sha256=$GFORTRAN_ARM64_F951_SHA256
    gfortran_collect2_sha256=$GFORTRAN_ARM64_COLLECT2_SHA256
    gmp_bottle_sha256=$GMP_ARM64_SEQUOIA_BOTTLE_SHA256
    isl_bottle_sha256=$ISL_ARM64_SEQUOIA_BOTTLE_SHA256
    mpfr_bottle_sha256=$MPFR_ARM64_SEQUOIA_BOTTLE_SHA256
    libmpc_bottle_sha256=$LIBMPC_ARM64_SEQUOIA_BOTTLE_SHA256
    zstd_bottle_sha256=$ZSTD_ARM64_SEQUOIA_BOTTLE_SHA256
    ;;
  darwin-x64)
    expected_machine='x86_64'
    gcc_bottle_sha256=$GCC_X64_SEQUOIA_BOTTLE_SHA256
    gfortran_driver_sha256=$GFORTRAN_X64_DRIVER_SHA256
    gfortran_f951_sha256=$GFORTRAN_X64_F951_SHA256
    gfortran_collect2_sha256=$GFORTRAN_X64_COLLECT2_SHA256
    gmp_bottle_sha256=$GMP_X64_SEQUOIA_BOTTLE_SHA256
    isl_bottle_sha256=$ISL_X64_SEQUOIA_BOTTLE_SHA256
    mpfr_bottle_sha256=$MPFR_X64_SEQUOIA_BOTTLE_SHA256
    libmpc_bottle_sha256=$LIBMPC_X64_SEQUOIA_BOTTLE_SHA256
    zstd_bottle_sha256=$ZSTD_X64_SEQUOIA_BOTTLE_SHA256
    ;;
  *)
    usage
    exit 2
    ;;
esac

if [[ "$(uname -s)" != 'Darwin' || "$(uname -m)" != "$expected_machine" ]]; then
  printf 'Target %s must be built natively on %s, received %s-%s.\n' \
    "$target" "$expected_machine" "$(uname -s)" "$(uname -m)" >&2
  exit 1
fi

for command in ar clang codesign curl file install_name_tool make node otool perl ranlib shasum tar xcrun; do
  command -v "$command" >/dev/null || {
    printf 'Required build command is unavailable: %s\n' "$command" >&2
    exit 1
  }
done

verify_toolchain_file() {
  local file_path=$1
  local expected_sha256=$2
  local label=$3
  if [[ ! -f "$file_path" ]]; then
    printf 'Reviewed %s is unavailable: %s\n' "$label" "$file_path" >&2
    exit 1
  fi
  local actual_sha256
  actual_sha256=$(shasum -a 256 "$file_path" | awk '{print $1}')
  if [[ "$actual_sha256" != "$expected_sha256" ]]; then
    printf 'Reviewed %s SHA-256 mismatch: expected %s, received %s.\n' \
      "$label" "$expected_sha256" "$actual_sha256" >&2
    exit 1
  fi
}

output=$(cd "$(dirname "$output")" && pwd)/$(basename "$output")
evidence=$(cd "$(dirname "$evidence")" && pwd)/$(basename "$evidence")
case "$output/" in
  "$evidence/"*) printf 'The output directory must not be inside the evidence directory.\n' >&2; exit 2 ;;
esac
case "$evidence/" in
  "$output/"*) printf 'The evidence directory must not be inside the output directory.\n' >&2; exit 2 ;;
esac
work_root=$(mktemp -d "${TMPDIR:-/tmp}/fraia-calculix-macos.XXXXXX")
chmod 700 "$work_root"
cleanup() {
  local status=$?
  if ((status != 0)); then
    printf 'CalculiX macOS build failed; captured logs follow.\n' >&2
    while IFS= read -r log_file; do
      printf '%s\n' "--- $log_file" >&2
      tail -n 120 "$log_file" >&2 || true
    done < <(find "$work_root" -type f -name '*-build.log' -print 2>/dev/null | sort)
  fi
  case "$work_root" in
    "${TMPDIR:-/tmp}"/fraia-calculix-macos.*) rm -rf -- "$work_root" ;;
    *) printf 'Refusing to remove unexpected work directory: %s\n' "$work_root" >&2 ;;
  esac
  exit "$status"
}
trap cleanup EXIT

download() {
  local url=$1
  local destination=$2
  local expected_sha256=$3
  curl --proto '=https' --tlsv1.2 --fail --location --retry 3 --retry-all-errors \
    "$url" -o "$destination"
  printf '%s  %s\n' "$expected_sha256" "$destination" | shasum -a 256 -c -
}

download_homebrew_bottle() {
  local formula=$1
  local expected_sha256=$2
  local destination=$3
  local registry_token
  local auth_config="$work_root/${formula}-registry-auth.conf"
  registry_token=$(
    curl --silent --show-error --fail \
      "https://ghcr.io/token?service=ghcr.io&scope=repository:homebrew/core/${formula}:pull" |
      node -e "
        let value = '';
        process.stdin.setEncoding('utf8');
        process.stdin.on('data', (chunk) => { value += chunk; });
        process.stdin.on('end', () => {
          const token = JSON.parse(value).token;
          if (!token) process.exit(1);
          process.stdout.write(token);
        });
      "
  )
  (
    umask 077
    printf 'header = "Authorization: Bearer %s"\n' "$registry_token" >"$auth_config"
  )
  chmod 600 "$auth_config"
  unset registry_token
  if ! curl --silent --show-error --fail --location \
    --config "$auth_config" \
    "https://ghcr.io/v2/homebrew/core/${formula}/blobs/sha256:${expected_sha256}" \
    -o "$destination"; then
    rm -f -- "$auth_config"
    return 1
  fi
  rm -f -- "$auth_config"
  printf '%s  %s\n' "$expected_sha256" "$destination" | shasum -a 256 -c -
}

download "$CCX_SOURCE_URL" "$work_root/ccx_2.23.src.tar.bz2" "$CCX_SOURCE_SHA256"
download "$CCX_TEST_URL" "$work_root/ccx_2.23.test.tar.bz2" "$CCX_TEST_SHA256"
download "$SPOOLES_URL" "$work_root/spooles.2.2.tgz" "$SPOOLES_SHA256"
download "$ARPACK_URL" "$work_root/arpack-ng-3.9.1.tar.gz" "$ARPACK_SHA256"
download "$GCC_SOURCE_URL" "$work_root/gcc-${GCC_VERSION}.tar.xz" "$GCC_SOURCE_SHA256"
tar -xJOf "$work_root/gcc-${GCC_VERSION}.tar.xz" \
  "gcc-${GCC_VERSION}/COPYING" >"$work_root/GPL-2.0.txt"
printf '%s  %s\n' "$GPL2_SHA256" "$work_root/GPL-2.0.txt" | shasum -a 256 -c -
tar -xJOf "$work_root/gcc-${GCC_VERSION}.tar.xz" \
  "gcc-${GCC_VERSION}/COPYING3" >"$work_root/GPL-3.0.txt"
printf '%s  %s\n' "$GPL3_SHA256" "$work_root/GPL-3.0.txt" | shasum -a 256 -c -
download \
  "$HOMEBREW_GCC_FORMULA_URL" \
  "$work_root/homebrew-gcc.rb" \
  "$HOMEBREW_GCC_FORMULA_SHA256"
download \
  "$HOMEBREW_GCC_PATCH_URL" \
  "$work_root/homebrew-gcc-${GCC_VERSION}.diff" \
  "$HOMEBREW_GCC_PATCH_SHA256"

download_homebrew_bottle gcc "$gcc_bottle_sha256" "$work_root/gcc-bottle.tar.gz"
mkdir "$work_root/gcc-bottle"
tar -xzf "$work_root/gcc-bottle.tar.gz" -C "$work_root/gcc-bottle"
gcc_runtime_directory=$(
  find "$work_root/gcc-bottle" -type d -path '*/lib/gcc/current' -print -quit
)
if [[ -z "$gcc_runtime_directory" ]]; then
  printf 'The pinned GCC bottle did not contain lib/gcc/current.\n' >&2
  exit 1
fi
gfortran_driver=$(
  find "$work_root/gcc-bottle" -type f -path "*/gcc/$GCC_VERSION/bin/gfortran-16" \
    -print -quit
)
if [[ -z "$gfortran_driver" ]]; then
  printf 'The pinned GCC bottle did not contain gfortran %s.\n' "$GCC_VERSION" >&2
  exit 1
fi
verify_toolchain_file "$gfortran_driver" "$gfortran_driver_sha256" 'gfortran driver'
if ! "$gfortran_driver" --version | head -1 | grep -Fq "$GCC_VERSION"; then
  printf 'The pinned bottle does not provide reviewed gfortran %s.\n' "$GCC_VERSION" >&2
  exit 1
fi
gfortran_f951=$("$gfortran_driver" -print-prog-name=f951)
gfortran_collect2=$("$gfortran_driver" -print-prog-name=collect2)
verify_toolchain_file "$gfortran_f951" "$gfortran_f951_sha256" 'gfortran f951'
verify_toolchain_file "$gfortran_collect2" "$gfortran_collect2_sha256" 'gfortran collect2'

compiler_support_directory=$(dirname "$gfortran_f951")
install_compiler_dependency() {
  local formula=$1
  local version=$2
  local expected_sha256=$3
  local library_name=$4
  local archive="$work_root/${formula}-bottle.tar.gz"
  local extracted="$work_root/${formula}-bottle"
  download_homebrew_bottle "$formula" "$expected_sha256" "$archive"
  mkdir "$extracted"
  tar -xzf "$archive" -C "$extracted"
  local source_path
  source_path=$(
    find "$extracted" \( -type f -o -type l \) \
      -path "*/${formula}/${version}/lib/${library_name}" -print -quit
  )
  if [[ -z "$source_path" ]]; then
    printf 'The pinned %s bottle did not contain %s.\n' "$formula" "$library_name" >&2
    exit 1
  fi
  cp -L "$source_path" "$compiler_support_directory/$library_name"
}

install_compiler_dependency gmp "$GMP_VERSION" "$gmp_bottle_sha256" libgmp.10.dylib
install_compiler_dependency isl "$ISL_VERSION" "$isl_bottle_sha256" libisl.23.dylib
install_compiler_dependency mpfr "$MPFR_VERSION" "$mpfr_bottle_sha256" libmpfr.6.dylib
install_compiler_dependency libmpc "$LIBMPC_VERSION" "$libmpc_bottle_sha256" libmpc.3.dylib
install_compiler_dependency zstd "$ZSTD_VERSION" "$zstd_bottle_sha256" libzstd.1.dylib

gfortran_f951_signature_status='verified'
for owner in "$gfortran_f951" "$compiler_support_directory"/*.dylib; do
  while IFS= read -r dependency; do
    case "$dependency" in
      /usr/lib/*|/System/Library/*)
        continue
        ;;
    esac
    dependency_basename=$(basename "$dependency")
    if [[ ! -f "$compiler_support_directory/$dependency_basename" ]]; then
      printf 'Pinned compiler dependency did not resolve: %s -> %s\n' \
        "$owner" "$dependency" >&2
      exit 1
    fi
  done < <(
    otool -L "$owner" |
      tail -n +2 |
      sed -E 's/^[[:space:]]*([^[:space:]]+).*/\1/'
  )
  if ! signature_output=$(codesign --verify --strict "$owner" 2>&1); then
    if [[ "$owner" != "$gfortran_f951" ]] ||
      ! grep -Fq 'code object is not signed at all' <<<"$signature_output"; then
      printf '%s\n' "$signature_output" >&2
      exit 1
    fi
    # Homebrew's reviewed Intel bottle leaves this exact, SHA-pinned compiler
    # helper unsigned. It is an ephemeral build input, never a bundled runtime
    # file; accepting its reviewed unsigned state preserves its source bytes.
    gfortran_f951_signature_status='unsigned, exact SHA-256 verified'
  fi
done
export DYLD_LIBRARY_PATH="$compiler_support_directory"
"$gfortran_f951" --version >/dev/null

macos_sdk=$(xcrun --sdk macosx --show-sdk-path)
macos_sdk_version=$(xcrun --sdk macosx --show-sdk-version)
if [[ "$macos_sdk" == *[[:space:]]* ]]; then
  printf 'The reviewed build requires a macOS SDK path without whitespace: %s\n' \
    "$macos_sdk" >&2
  exit 1
fi

export LC_ALL=C
export TZ=UTC
export ZERO_AR_DATE=1
export SOURCE_DATE_EPOCH
export MACOSX_DEPLOYMENT_TARGET

build_once() {
  local build_root=$1
  mkdir -p \
    "$build_root/spooles" \
    "$build_root/arpack" \
    "$build_root/arpack-objects" \
    "$build_root/payload"
  tar -xjf "$work_root/ccx_2.23.src.tar.bz2" -C "$build_root"
  tar -xzf "$work_root/spooles.2.2.tgz" -C "$build_root/spooles"
  tar -xzf "$work_root/arpack-ng-3.9.1.tar.gz" \
    -C "$build_root/arpack" --strip-components=1

  perl -0pi -e 's#/usr/lang-4\.0/bin/cc#clang#g' "$build_root/spooles/Make.inc"
  perl -0pi -e 's/drawTree\.c/tree.c/g' \
    "$build_root/spooles/Tree/src/makeGlobalLib"
  perl -0pi -e 's/IVinit\(nfront, NULL\)/IVinit(nfront, 0)/g' \
    "$build_root/spooles/ETree/src/transform.c"
  make -C "$build_root/spooles" lib >"$build_root/spooles-build.log" 2>&1
  make -C "$build_root/spooles/MT/src" makeLib >>"$build_root/spooles-build.log" 2>&1

  (
    cd "$build_root/arpack-objects"
    "$gfortran_driver" \
      -O2 \
      -g0 \
      -isysroot "$macos_sdk" \
      -ffile-prefix-map=..=/usr/src/fraia-runtime \
      -c \
      ../arpack/SRC/*.f \
      ../arpack/UTIL/*.f \
      ../arpack/dbgini.f \
      ../arpack/staini.f \
      >"$build_root/arpack-build.log" 2>&1
    ar rcs "$build_root/libarpack.a" ./*.o
    ranlib "$build_root/libarpack.a"
  )

  local ccx_source="$build_root/CalculiX/ccx_2.23/src"
  perl -0pi -e 's/(ccx_2\.23: \$\(OCCXMAIN\) ccx_2\.23\.a)\s+\$\(LIBS\)/$1/' \
    "$ccx_source/Makefile"
  perl -0pi -e 's#\./date\.pl;\s*##g' "$ccx_source/Makefile"
  perl -0pi -e 's/\$\(FC\)  -Wall -O2 -o/\$\(FC\) \$\(FFLAGS\) -o/' \
    "$ccx_source/Makefile"
  perl -0pi -e 's/return NULL;/return;/' "$ccx_source/readnewmesh.c"
  make -C "$ccx_source" ccx_2.23 \
    CC=clang \
    FC="$gfortran_driver" \
    CFLAGS="-O2 -g0 -mmacosx-version-min=$MACOSX_DEPLOYMENT_TARGET -ffile-prefix-map=$build_root=/usr/src/fraia-runtime -I$build_root/spooles -DARCH=Linux -DSPOOLES -DARPACK -DMATRIXSTORAGE -DUSE_MT=1" \
    FFLAGS="-O2 -g0 -isysroot $macos_sdk -mmacosx-version-min=$MACOSX_DEPLOYMENT_TARGET -ffile-prefix-map=$build_root=/usr/src/fraia-runtime -fopenmp -cpp" \
    LIBS="-L$gcc_runtime_directory -Wl,-search_paths_first -Wl,-no_adhoc_codesign -Wl,-rpath,@loader_path $build_root/spooles/spooles.a $build_root/libarpack.a -framework Accelerate -lpthread -lm -lc" \
    >"$build_root/ccx-build.log" 2>&1
  cp "$ccx_source/ccx_2.23" "$build_root/payload/ccx"
  chmod 755 "$build_root/payload/ccx"

  local queue="$build_root/payload/ccx"
  local seen=''
  while [[ -n "$queue" ]]; do
    local owner=${queue%%$'\n'*}
    if [[ "$queue" == "$owner" ]]; then
      queue=''
    else
      queue=${queue#*$'\n'}
    fi
    [[ "$seen" != *"|$owner|"* ]] || continue
    seen="$seen|$owner|"
    while IFS= read -r dependency; do
      [[ -n "$dependency" ]] || continue
      local basename=''
      local source_path=''
      case "$dependency" in
        /usr/lib/*|/System/Library/Frameworks/*|@loader_path/*)
          continue
          ;;
        @rpath/*|/opt/homebrew/*|/usr/local/*|@@HOMEBREW_PREFIX@@/*)
          basename=$(basename "$dependency")
          source_path="$gcc_runtime_directory/$basename"
          ;;
        *)
          printf 'Unreviewed dependency: %s -> %s\n' "$owner" "$dependency" >&2
          exit 1
          ;;
      esac
      [[ -f "$source_path" ]] || {
        printf 'Pinned GCC dependency did not resolve: %s\n' "$dependency" >&2
        exit 1
      }
      local destination="$build_root/payload/$basename"
      if [[ ! -f "$destination" ]]; then
        cp "$source_path" "$destination"
        chmod 755 "$destination"
        queue=${queue:+"$queue"$'\n'}$destination
      fi
    done < <(
      otool -L "$owner" |
        tail -n +2 |
        sed -E 's/^[[:space:]]*([^[:space:]]+).*/\1/'
    )
  done

  local owner
  # Preserve each Mach-O's link-edit layout while install_name_tool rewrites it.
  # The completed payload is deliberately stripped of signatures only after all
  # loader commands are final, then a disposable copy is signed for solver QA.
  while IFS= read -r runpath; do
    [[ -n "$runpath" && "$runpath" != '@loader_path' ]] || continue
    install_name_tool -delete_rpath "$runpath" "$build_root/payload/ccx"
  done < <(
    otool -l "$build_root/payload/ccx" |
      awk '/LC_RPATH/{found=1; next} found && /path /{print $2; found=0}'
  )
  for owner in "$build_root"/payload/*; do
    local owner_basename
    owner_basename=$(basename "$owner")
    if [[ "$owner_basename" == *.dylib ]]; then
      install_name_tool -id "@loader_path/$owner_basename" "$owner"
    fi
    while IFS= read -r dependency; do
      [[ -n "$dependency" ]] || continue
      if [[ "$owner_basename" == *.dylib && "$(basename "$dependency")" == "$owner_basename" ]]; then
        continue
      fi
      case "$dependency" in
        /usr/lib/*|/System/Library/Frameworks/*|@loader_path/*)
          continue
          ;;
        @rpath/*)
          if [[ "$owner_basename" == *.dylib ]]; then
            continue
          fi
          install_name_tool \
            -change "$dependency" "@loader_path/$(basename "$dependency")" "$owner"
          ;;
        /opt/homebrew/*|/usr/local/*|@@HOMEBREW_PREFIX@@/*)
          if [[ "$owner_basename" == *.dylib ]]; then
            printf 'Bundled GCC library has an unreviewed load dependency: %s -> %s\n' \
              "$owner" "$dependency" >&2
            exit 1
          fi
          install_name_tool \
            -change "$dependency" "@loader_path/$(basename "$dependency")" "$owner"
          ;;
        *)
          printf 'Unreviewed dependency after copying: %s -> %s\n' \
            "$owner" "$dependency" >&2
          exit 1
          ;;
      esac
    done < <(
      otool -L "$owner" |
        tail -n +2 |
        sed -E 's/^[[:space:]]*([^[:space:]]+).*/\1/'
    )
    if [[ "$owner_basename" == *.dylib ]]; then
      local install_id
      install_id=$(
        otool -D "$owner" |
          tail -n 1 |
          sed -E 's/^[[:space:]]*//'
      )
      if [[ "$install_id" != "@loader_path/$owner_basename" ]]; then
        printf 'Bundled GCC library has an unreviewed install ID: %s -> %s\n' \
          "$owner" "$install_id" >&2
        exit 1
      fi
    fi
  done
  if [[ "$(
    otool -l "$build_root/payload/ccx" |
      awk '/LC_RPATH/{found=1; next} found && /path /{print $2; found=0}'
  )" != '@loader_path' ]]; then
    printf 'CalculiX must contain exactly the @loader_path runpath.\n' >&2
    exit 1
  fi
  for owner in "$build_root"/payload/*; do
    codesign --remove-signature "$owner" >/dev/null 2>&1 || true
  done
}

build_once "$work_root/build-one"
build_once "$work_root/build-two"

for candidate in "$work_root/build-one/payload/"*; do
  name=$(basename "$candidate")
  cmp "$candidate" "$work_root/build-two/payload/$name"
  if [[ "$(file -b "$candidate")" != *"$expected_machine"* ]]; then
    printf 'Payload architecture mismatch: %s\n' "$candidate" >&2
    exit 1
  fi
  minos=$(
    otool -l "$candidate" |
      awk '/LC_BUILD_VERSION/{found=1} found&&/minos/{print $2; exit}'
  )
  if [[ -z "$minos" ]]; then
    printf 'Payload has no macOS minimum-version declaration: %s\n' "$candidate" >&2
    exit 1
  fi
  if ! node -e "
    const parse = (value) => value.split('.').map(Number);
    const [actual, maximum] = process.argv.slice(1).map(parse);
    const width = Math.max(actual.length, maximum.length);
    for (let index = 0; index < width; index += 1) {
      const difference = (actual[index] || 0) - (maximum[index] || 0);
      if (difference > 0) process.exit(1);
      if (difference < 0) process.exit(0);
    }
  " "$minos" "$MACOSX_DEPLOYMENT_TARGET"; then
    printf 'Payload minimum macOS %s exceeds the reviewed %s ceiling: %s\n' \
      "$minos" "$MACOSX_DEPLOYMENT_TARGET" "$candidate" >&2
    exit 1
  fi
done

mkdir -p "$work_root/runtime-test/payload" "$work_root/runtime-test/case"
cp "$work_root/build-one/payload/"* "$work_root/runtime-test/payload/"
for dylib in "$work_root/runtime-test"/payload/*.dylib; do
  codesign --force --sign - "$dylib"
done
codesign --force --sign - "$work_root/runtime-test/payload/ccx"
for signed_target in "$work_root/runtime-test"/payload/*; do
  codesign --verify --strict --verbose=2 "$signed_target"
done
tar -xjf "$work_root/ccx_2.23.test.tar.bz2" \
  -C "$work_root/runtime-test/case" \
  ./CalculiX/ccx_2.23/test/spring1.inp
spring_case="$work_root/runtime-test/case/CalculiX/ccx_2.23/test"
(
  cd "$spring_case"
  "$work_root/runtime-test/payload/ccx" spring1 \
    >"$work_root/runtime-test/spring1.stdout" \
    2>"$work_root/runtime-test/spring1.stderr"
)
test -s "$spring_case/spring1.dat"
test -s "$spring_case/spring1.frd"
test -s "$spring_case/spring1.sta"
grep -Fq 'Job finished' "$work_root/runtime-test/spring1.stdout"

runtime_staging="$work_root/runtime-output"
mkdir "$runtime_staging"
cp "$work_root/build-one/payload/"* "$runtime_staging/"
mkdir "$runtime_staging/licenses"
cp "$work_root/GPL-2.0.txt" "$runtime_staging/licenses/GPL-2.0.txt"
cp "$work_root/GPL-3.0.txt" "$runtime_staging/licenses/GPL-3.0.txt"
cp "$work_root/build-one/arpack/COPYING" \
  "$runtime_staging/licenses/ARPACK-BSD-3-Clause.txt"
gcc_bottle_prefix=$(cd "$(dirname "$gfortran_driver")/.." && pwd)
cp "$gcc_bottle_prefix/COPYING.RUNTIME" \
  "$runtime_staging/licenses/GCC-Runtime-Library-Exception-3.1.txt"
{
  printf 'CalculiX %s\n' "$CCX_VERSION"
  printf 'Copyright (C) 1998-2025 Guido Dhondt and contributors.\n\n'
  printf 'The CalculiX source headers license the program under version 2 of the\n'
  printf 'GNU General Public License. The full text is in GPL-2.0.txt.\n\n'
  printf 'Source: %s\n' "$CCX_SOURCE_URL"
  printf 'SHA-256: %s\n' "$CCX_SOURCE_SHA256"
} >"$runtime_staging/licenses/CALCULIX-LICENSE-NOTICE.txt"
{
  printf 'SPOOLES 2.2\n\n'
  printf 'The SPOOLES 2.2 reference manual and release page state that this\n'
  printf 'release of the package is totally within the public domain.\n\n'
  printf 'The source also contains Harwell-Boeing File I/O in C, version 1.0,\n'
  printf 'from the National Institute of Standards and Technology, with this notice:\n\n'
  printf 'Permission to use, copy, modify, and distribute this software and its\n'
  printf 'documentation for any purpose and without fee is hereby granted provided\n'
  printf 'that the above copyright notice appear in all copies and that both the\n'
  printf 'copyright notice and this permission notice appear in supporting documentation.\n\n'
  printf 'Neither the Author nor the Institution (National Institute of Standards\n'
  printf 'and Technology) make any representations about the suitability of this\n'
  printf 'software for any purpose. This software is provided "as is" without\n'
  printf 'expressed or implied warranty.\n\n'
  printf 'Source: %s\n' "$SPOOLES_URL"
  printf 'SHA-256: %s\n' "$SPOOLES_SHA256"
} >"$runtime_staging/licenses/SPOOLES-NOTICE.txt"
{
  printf 'Fraia CalculiX %s native runtime notices\n\n' "$CCX_VERSION"
  printf 'CalculiX: GPL-2.0-only. See licenses/CALCULIX-LICENSE-NOTICE.txt and licenses/GPL-2.0.txt.\n'
  printf 'SPOOLES 2.2: public domain with included NIST notice. See licenses/SPOOLES-NOTICE.txt.\n'
  printf 'ARPACK-NG: BSD-3-Clause. See licenses/ARPACK-BSD-3-Clause.txt.\n'
  printf 'Bundled GCC runtime libraries: GPL-3.0-or-later WITH GCC-exception-3.1.\n'
  printf 'See licenses/GPL-3.0.txt and licenses/GCC-Runtime-Library-Exception-3.1.txt.\n\n'
  printf 'Corresponding-source publication is recorded separately in runtime-manifest.json.\n'
} >"$runtime_staging/THIRD_PARTY_NOTICES.txt"
build_script_sha256=$(shasum -a 256 "$0" | awk '{print $1}')
clang_version=$(clang --version | head -1)
macos_version=$(sw_vers -productVersion)
macos_build=$(sw_vers -buildVersion)
{
  printf '# Fraia CalculiX %s %s build recipe\n\n' "$CCX_VERSION" "$target"
  printf 'Build revision: `fraia-calculix-macos-v5`\n\n'
  printf -- '- CalculiX source SHA-256: `%s`\n' "$CCX_SOURCE_SHA256"
  printf -- '- CalculiX tests SHA-256: `%s`\n' "$CCX_TEST_SHA256"
  printf -- '- SPOOLES source SHA-256: `%s`\n' "$SPOOLES_SHA256"
  printf -- '- ARPACK-NG revision: `%s`\n' "$ARPACK_REVISION"
  printf -- '- ARPACK-NG source SHA-256: `%s`\n' "$ARPACK_SHA256"
  printf -- '- GPL-2.0 text from GCC source `COPYING` SHA-256: `%s`\n' "$GPL2_SHA256"
  printf -- '- GPL-3.0 text from GCC source `COPYING3` SHA-256: `%s`\n' "$GPL3_SHA256"
  printf -- '- GCC source: `%s`\n' "$GCC_SOURCE_URL"
  printf -- '- GCC source SHA-256: `%s`\n' "$GCC_SOURCE_SHA256"
  printf -- '- Homebrew/core revision: `%s`\n' "$HOMEBREW_CORE_REVISION"
  printf -- '- Homebrew GCC formula SHA-256: `%s`\n' "$HOMEBREW_GCC_FORMULA_SHA256"
  printf -- '- Homebrew GCC patch SHA-256: `%s`\n' "$HOMEBREW_GCC_PATCH_SHA256"
  printf -- '- GCC bottle SHA-256: `%s`\n' "$gcc_bottle_sha256"
  printf -- '- GMP bottle SHA-256: `%s`\n' "$gmp_bottle_sha256"
  printf -- '- ISL bottle SHA-256: `%s`\n' "$isl_bottle_sha256"
  printf -- '- MPFR bottle SHA-256: `%s`\n' "$mpfr_bottle_sha256"
  printf -- '- libmpc bottle SHA-256: `%s`\n' "$libmpc_bottle_sha256"
  printf -- '- zstd bottle SHA-256: `%s`\n' "$zstd_bottle_sha256"
  printf -- '- gfortran driver SHA-256: `%s`\n' "$gfortran_driver_sha256"
  printf -- '- gfortran f951 SHA-256: `%s`\n' "$gfortran_f951_sha256"
  printf -- '- gfortran collect2 SHA-256: `%s`\n' "$gfortran_collect2_sha256"
  printf -- '- Build script SHA-256: `%s`\n' "$build_script_sha256"
  printf -- '- SOURCE_DATE_EPOCH: `%s`\n' "$SOURCE_DATE_EPOCH"
  printf -- '- MACOSX_DEPLOYMENT_TARGET: `%s`\n' "$MACOSX_DEPLOYMENT_TARGET"
  printf -- '- macOS SDK version: `%s`\n' "$macos_sdk_version"
  printf -- '- Native host: macOS `%s` build `%s`, `%s`\n\n' \
    "$macos_version" "$macos_build" "$clang_version"
  printf 'Reproduce on a native matching host with the reviewed toolchain:\n\n'
  printf '```sh\n'
  printf './build-calculix-macos-runtime.sh --target %s --output /absolute/path/to/a-new-runtime-directory --evidence /absolute/path/to/a-new-evidence-directory\n' \
    "$target"
  printf '```\n'
} >"$runtime_staging/BUILD_RECIPE.md"
(
  cd "$runtime_staging"
  find . -type f ! -name SHA256SUMS -print |
    LC_ALL=C sort |
    while IFS= read -r file_path; do
      shasum -a 256 "$file_path"
    done >SHA256SUMS
)

evidence_staging="$work_root/review-evidence"
mkdir -p \
  "$evidence_staging/native" \
  "$evidence_staging/reproducibility" \
  "$evidence_staging/solver" \
  "$evidence_staging/source-inputs" \
  "$evidence_staging/toolchain"
cp "$0" "$evidence_staging/source-inputs/build-calculix-macos-runtime.sh"
cp \
  "$work_root/ccx_2.23.src.tar.bz2" \
  "$work_root/ccx_2.23.test.tar.bz2" \
  "$work_root/spooles.2.2.tgz" \
  "$work_root/arpack-ng-3.9.1.tar.gz" \
  "$work_root/gcc-${GCC_VERSION}.tar.xz" \
  "$work_root/homebrew-gcc.rb" \
  "$work_root/homebrew-gcc-${GCC_VERSION}.diff" \
  "$work_root/GPL-2.0.txt" \
  "$work_root/GPL-3.0.txt" \
  "$evidence_staging/source-inputs/"
find "$work_root" -maxdepth 1 -type f -name '*-bottle.tar.gz' -exec \
  cp {} "$evidence_staging/source-inputs/" \;
(
  cd "$evidence_staging/source-inputs"
  find . -type f ! -name SHA256SUMS -print |
    LC_ALL=C sort |
    while IFS= read -r file_path; do
      shasum -a 256 "$file_path"
    done >SHA256SUMS
)

cp \
  "$spring_case/spring1.inp" \
  "$spring_case/spring1.dat" \
  "$spring_case/spring1.frd" \
  "$spring_case/spring1.sta" \
  "$work_root/runtime-test/spring1.stdout" \
  "$work_root/runtime-test/spring1.stderr" \
  "$evidence_staging/solver/"
(
  cd "$evidence_staging/solver"
  shasum -a 256 ./* >SHA256SUMS
)

for build_name in build-one build-two; do
  (
    cd "$work_root/$build_name/payload"
    shasum -a 256 ./* >"$evidence_staging/reproducibility/${build_name}-SHA256SUMS"
  )
done
cmp \
  "$evidence_staging/reproducibility/build-one-SHA256SUMS" \
  "$evidence_staging/reproducibility/build-two-SHA256SUMS"
printf 'The two independently built native payloads are byte-identical.\n' \
  >"$evidence_staging/reproducibility/RESULT.txt"

for native_file in "$work_root/runtime-test/payload/"*; do
  native_name=$(basename "$native_file")
  file -b "$native_file" >"$evidence_staging/native/${native_name}.file.txt"
  otool -L "$native_file" |
    tail -n +2 >"$evidence_staging/native/${native_name}.dependencies.txt"
  otool -l "$native_file" >"$evidence_staging/native/${native_name}.load-commands.txt"
  codesign --verify --strict --verbose=4 "$native_file" \
    >"$evidence_staging/native/${native_name}.codesign.txt" 2>&1
done

{
  printf 'Native host: macOS %s build %s\n' "$macos_version" "$macos_build"
  printf 'macOS SDK version: %s\n' "$macos_sdk_version"
  printf 'MACOSX_DEPLOYMENT_TARGET: %s\n' "$MACOSX_DEPLOYMENT_TARGET"
  printf 'clang version: %s\n' "$clang_version"
  printf 'gfortran f951 signature: %s\n' "$gfortran_f951_signature_status"
} >"$evidence_staging/toolchain/ENVIRONMENT.txt"
for tool_name in ar clang codesign install_name_tool ld make otool ranlib xcrun; do
  tool_path=$(command -v "$tool_name")
  printf '%s  %s\n' \
    "$(shasum -a 256 "$tool_path" | awk '{print $1}')" \
    "$tool_name"
done >"$evidence_staging/toolchain/SHA256SUMS"
printf '%s  gfortran-16\n' "$gfortran_driver_sha256" \
  >>"$evidence_staging/toolchain/SHA256SUMS"
printf '%s  f951\n' "$gfortran_f951_sha256" \
  >>"$evidence_staging/toolchain/SHA256SUMS"
printf '%s  collect2\n' "$gfortran_collect2_sha256" \
  >>"$evidence_staging/toolchain/SHA256SUMS"
cp "$runtime_staging/BUILD_RECIPE.md" "$evidence_staging/BUILD_RECIPE.md"
cp "$runtime_staging/SHA256SUMS" "$evidence_staging/RUNTIME_SHA256SUMS"
cp "$runtime_staging/THIRD_PARTY_NOTICES.txt" "$evidence_staging/THIRD_PARTY_NOTICES.txt"
{
  printf '# Fraia CalculiX %s %s review evidence\n\n' "$CCX_VERSION" "$target"
  printf 'This directory is review evidence, not a promotable runtime.\n'
  printf 'Promote only the separately emitted runtime after this evidence, corresponding-source publication, and runtime-manifest review all pass.\n'
} >"$evidence_staging/README.md"
(
  cd "$evidence_staging"
  find . -type f ! -name EVIDENCE_SHA256SUMS -print |
    LC_ALL=C sort |
    while IFS= read -r file_path; do
      shasum -a 256 "$file_path"
    done >EVIDENCE_SHA256SUMS
)
mv "$evidence_staging" "$evidence"
mv "$runtime_staging" "$output"
printf 'Built and independently reproduced %s CalculiX %s runtime at %s\n' \
  "$target" "$CCX_VERSION" "$output"
printf 'Wrote independently reviewable %s evidence at %s\n' "$target" "$evidence"
