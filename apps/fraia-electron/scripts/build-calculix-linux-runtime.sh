#!/bin/bash

set -Eeuo pipefail

CCX_VERSION='2.23'
CCX_SOURCE_URL='https://www.dhondt.de/ccx_2.23.src.tar.bz2'
CCX_SOURCE_SHA256='9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7'
CCX_TEST_URL='https://www.dhondt.de/ccx_2.23.test.tar.bz2'
CCX_TEST_SHA256='be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0'
SPOOLES_URL='https://www.netlib.org/linalg/spooles/spooles.2.2.tgz'
SPOOLES_SHA256='a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd'
SPOOLES_CORRECTION_URL='https://www.dhondt.de/ccx_2.23.SPOOLEScorrection.tar.bz2'
SPOOLES_CORRECTION_SHA256='15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93'
ARPACK_URL='https://github.com/opencollab/arpack-ng/archive/refs/tags/3.9.1.tar.gz'
ARPACK_SHA256='f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844'
OPENBLAS_URL='https://github.com/OpenMathLib/OpenBLAS/releases/download/v0.3.34/OpenBLAS-0.3.34.tar.gz'
OPENBLAS_SHA256='cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed'
GCC_VERSION='16.1.0'
GCC_SOURCE_URL="https://ftpmirror.gnu.org/gnu/gcc/gcc-${GCC_VERSION}/gcc-${GCC_VERSION}.tar.xz"
GCC_SOURCE_SHA256='50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79'
GPL2_SHA256='231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c'
GPL3_SHA256='8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903'
UBUNTU_CONTAINER_IMAGE='ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982'
UBUNTU_SNAPSHOT='20260720T000000Z'
SOURCE_DATE_EPOCH='1762047462'
GLIBC_SYMBOL_CEILING='2.35'

usage() {
  printf 'Usage: %s --target linux-arm64|linux-x64 --output <new-directory> --evidence <new-directory>\n' "$0" >&2
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
  linux-arm64)
    expected_machine='aarch64'
    openblas_target='ARMV8'
    ;;
  linux-x64)
    expected_machine='x86_64'
    openblas_target='CORE2'
    ;;
  *)
    usage
    exit 2
    ;;
esac

if [[ "$(uname -s)" != 'Linux' || "$(uname -m)" != "$expected_machine" ]]; then
  printf 'Target %s must be built natively on Linux %s, received %s-%s.\n' \
    "$target" "$expected_machine" "$(uname -s)" "$(uname -m)" >&2
  exit 1
fi
if [[ ! -r /etc/os-release ]] || ! grep -Fqx 'VERSION_ID="22.04"' /etc/os-release; then
  printf 'The reviewed Linux runtime must be built inside the pinned Ubuntu 22.04 container.\n' >&2
  exit 1
fi

for command in ar awk cmake curl dpkg-query file find gfortran gcc gzip ldd make ninja patchelf readelf ranlib sed shasum strings tar; do
  command -v "$command" >/dev/null || {
    printf 'Required build command is unavailable: %s\n' "$command" >&2
    exit 1
  }
done

output=$(cd "$(dirname "$output")" && pwd)/$(basename "$output")
evidence=$(cd "$(dirname "$evidence")" && pwd)/$(basename "$evidence")
case "$output/" in
  "$evidence/"*) printf 'The output directory must not be inside the evidence directory.\n' >&2; exit 2 ;;
esac
case "$evidence/" in
  "$output/"*) printf 'The evidence directory must not be inside the output directory.\n' >&2; exit 2 ;;
esac
work_root=$(mktemp -d "${TMPDIR:-/tmp}/fraia-calculix-linux.XXXXXX")
chmod 700 "$work_root"
cleanup() {
  case "$work_root" in
    "${TMPDIR:-/tmp}"/fraia-calculix-linux.*) rm -rf -- "$work_root" ;;
    *) printf 'Refusing to remove unexpected work directory: %s\n' "$work_root" >&2 ;;
  esac
}
report_failure() {
  local status=$1
  local log_file
  trap - ERR
  printf 'CalculiX runtime build failed with status %s. Recent build logs follow.\n' \
    "$status" >&2
  while IFS= read -r log_file; do
    printf '\n===== %s =====\n' "${log_file#"$work_root"/}" >&2
    tail -n 200 "$log_file" >&2 || true
  done < <(
    find "$work_root" -type f \
      \( -name '*-build.log' -o -name '*-configure.log' \) \
      -print |
      sort
  )
  exit "$status"
}
trap cleanup EXIT
trap 'report_failure $?' ERR

download() {
  local url=$1
  local destination=$2
  local expected_sha256=$3
  curl --proto '=https' --tlsv1.2 --fail --location --retry 3 --retry-all-errors \
    "$url" -o "$destination"
  printf '%s  %s\n' "$expected_sha256" "$destination" | shasum -a 256 -c -
}

download "$CCX_SOURCE_URL" "$work_root/ccx_2.23.src.tar.bz2" "$CCX_SOURCE_SHA256"
download "$CCX_TEST_URL" "$work_root/ccx_2.23.test.tar.bz2" "$CCX_TEST_SHA256"
download "$SPOOLES_URL" "$work_root/spooles.2.2.tgz" "$SPOOLES_SHA256"
download \
  "$SPOOLES_CORRECTION_URL" \
  "$work_root/ccx_2.23.SPOOLEScorrection.tar.bz2" \
  "$SPOOLES_CORRECTION_SHA256"
download "$ARPACK_URL" "$work_root/arpack-ng-3.9.1.tar.gz" "$ARPACK_SHA256"
download "$OPENBLAS_URL" "$work_root/OpenBLAS-0.3.34.tar.gz" "$OPENBLAS_SHA256"
download "$GCC_SOURCE_URL" "$work_root/gcc-${GCC_VERSION}.tar.xz" "$GCC_SOURCE_SHA256"
tar -xJOf "$work_root/gcc-${GCC_VERSION}.tar.xz" \
  "gcc-${GCC_VERSION}/COPYING" >"$work_root/GPL-2.0.txt"
printf '%s  %s\n' "$GPL2_SHA256" "$work_root/GPL-2.0.txt" | shasum -a 256 -c -
tar -xJOf "$work_root/gcc-${GCC_VERSION}.tar.xz" \
  "gcc-${GCC_VERSION}/COPYING3" >"$work_root/GPL-3.0.txt"
printf '%s  %s\n' "$GPL3_SHA256" "$work_root/GPL-3.0.txt" | shasum -a 256 -c -

export LC_ALL=C.UTF-8
export TZ=UTC
export SOURCE_DATE_EPOCH

build_once() {
  local build_root=$1
  local prefix="$build_root/prefix"
  mkdir -p \
    "$build_root/spooles" \
    "$build_root/correction" \
    "$build_root/arpack" \
    "$build_root/openblas" \
    "$build_root/payload"
  tar -xjf "$work_root/ccx_2.23.src.tar.bz2" -C "$build_root"
  tar -xzf "$work_root/spooles.2.2.tgz" -C "$build_root/spooles"
  tar -xjf "$work_root/ccx_2.23.SPOOLEScorrection.tar.bz2" \
    -C "$build_root/correction"
  tar -xzf "$work_root/arpack-ng-3.9.1.tar.gz" \
    -C "$build_root/arpack" --strip-components=1
  tar -xzf "$work_root/OpenBLAS-0.3.34.tar.gz" \
    -C "$build_root/openblas" --strip-components=1

  cp \
    "$build_root/correction/CalculiX/ccx_2.23/SPOOLES.2.2/I2Ohash/src/util.c" \
    "$build_root/spooles/I2Ohash/src/util.c"
  sed -i 's#/usr/lang-4\.0/bin/cc#gcc#g' "$build_root/spooles/Make.inc"
  sed -i 's/drawTree\.c/tree.c/g' "$build_root/spooles/Tree/src/makeGlobalLib"
  sed -i 's/IVinit(nfront, NULL)/IVinit(nfront, 0)/g' \
    "$build_root/spooles/ETree/src/transform.c"
  make -C "$build_root/spooles" lib >"$build_root/spooles-build.log" 2>&1
  make -C "$build_root/spooles/MT/src" makeLib >>"$build_root/spooles-build.log" 2>&1

  make -C "$build_root/openblas" -j2 \
    NO_SHARED=1 \
    TARGET="$openblas_target" \
    USE_OPENMP=0 \
    USE_THREAD=1 \
    NUM_THREADS=64 \
    CFLAGS="-O2 -g0 -ffile-prefix-map=$build_root=/usr/src/fraia-runtime" \
    FFLAGS="-O2 -g0 -fallow-argument-mismatch -ffile-prefix-map=$build_root=/usr/src/fraia-runtime" \
    >"$build_root/openblas-build.log" 2>&1
  make -C "$build_root/openblas" \
    NO_SHARED=1 \
    PREFIX="$prefix" \
    install >>"$build_root/openblas-build.log" 2>&1

  cmake \
    -S "$build_root/arpack" \
    -B "$build_root/arpack-build" \
    -G Ninja \
    -DBUILD_SHARED_LIBS=OFF \
    -DMPI=OFF \
    -DICB=OFF \
    -DEXAMPLES=OFF \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_C_FLAGS_RELEASE="-O2 -g0 -ffile-prefix-map=$build_root=/usr/src/fraia-runtime" \
    -DCMAKE_Fortran_FLAGS_RELEASE="-O2 -g0 -fallow-argument-mismatch -ffile-prefix-map=$build_root=/usr/src/fraia-runtime" \
    -DBLAS_LIBRARIES="$prefix/lib/libopenblas.a" \
    -DLAPACK_LIBRARIES="$prefix/lib/libopenblas.a" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    >"$build_root/arpack-configure.log" 2>&1
  sed -i \
    "s#$build_root/arpack/#../arpack/#g" \
    "$build_root/arpack-build/build.ninja"
  if grep -Fq "$build_root/arpack/" "$build_root/arpack-build/build.ninja"; then
    printf 'ARPACK retained an absolute source path in its Ninja build graph.\n' >&2
    exit 1
  fi
  cmake --build "$build_root/arpack-build" --parallel 2 >"$build_root/arpack-build.log" 2>&1
  cmake --install "$build_root/arpack-build" >>"$build_root/arpack-build.log" 2>&1

  local ccx_source="$build_root/CalculiX/ccx_2.23/src"
  sed -i -E 's/(ccx_2\.23: \$\(OCCXMAIN\) ccx_2\.23\.a)[[:space:]]+\$\(LIBS\)/\1/' \
    "$ccx_source/Makefile"
  sed -i -E 's#\./date\.pl;[[:space:]]*##g' "$ccx_source/Makefile"
  sed -i 's/return NULL;/return;/' "$ccx_source/readnewmesh.c"
  make -C "$ccx_source" -j2 ccx_2.23 \
    CC=gcc \
    FC=gfortran \
    CFLAGS="-O2 -g0 -ffile-prefix-map=$build_root=/usr/src/fraia-runtime -I$build_root/spooles -DARCH=Linux -DSPOOLES -DARPACK -DMATRIXSTORAGE -DNETWORKOUT -DUSE_MT=1" \
    FFLAGS="-O2 -g0 -fallow-argument-mismatch -ffile-prefix-map=$build_root=/usr/src/fraia-runtime -fopenmp -cpp" \
    LIBS="$build_root/spooles/spooles.a $prefix/lib/libarpack.a $prefix/lib/libopenblas.a -Wl,-Bstatic -lgomp -Wl,-Bdynamic -Wl,--build-id=none -static-libgfortran -static-libgcc -lpthread -ldl -lm -lc" \
    >"$build_root/ccx-build.log" 2>&1
  cp "$ccx_source/ccx_2.23" "$build_root/payload/ccx"
  chmod 755 "$build_root/payload/ccx"
  local quadmath_path
  quadmath_path=$(
    ldd "$build_root/payload/ccx" |
      awk '$1 == "libquadmath.so.0" && $2 == "=>" { print $3; exit }'
  )
  if [[ -n "$quadmath_path" ]]; then
    if [[ "$target" != 'linux-x64' || ! -f "$quadmath_path" ]]; then
      printf 'Unexpected dynamic libquadmath dependency for %s: %s\n' \
        "$target" "${quadmath_path:-missing}" >&2
      exit 1
    fi
    cp -L "$quadmath_path" "$build_root/payload/libquadmath.so.0"
    chmod 755 "$build_root/payload/libquadmath.so.0"
    patchelf --set-rpath "\$ORIGIN" "$build_root/payload/ccx"
  fi
}

build_once "$work_root/build-one"
build_once "$work_root/build-two"
for payload in \
  "$work_root/build-one/payload" \
  "$work_root/build-two/payload"; do
  if strings "$payload"/* | grep -F "$work_root"; then
    printf 'The reviewed Linux runtime contains an absolute build path: %s\n' \
      "$payload" >&2
    exit 1
  fi
done
if ! diff -qr "$work_root/build-one/payload" "$work_root/build-two/payload"; then
  printf 'The independently built Linux runtimes are not byte-identical.\n' >&2
  shasum -a 256 "$work_root/build-one/payload/"* >&2
  shasum -a 256 "$work_root/build-two/payload/"* >&2
  exit 1
fi

candidate="$work_root/build-one/payload/ccx"
quadmath="$work_root/build-one/payload/libquadmath.so.0"
case "$target" in
  linux-arm64)
    readelf -hW "$candidate" | grep -Fq 'Machine:                           AArch64'
    [[ ! -e "$quadmath" ]]
    ;;
  linux-x64)
    readelf -hW "$candidate" | grep -Fq 'Machine:                           Advanced Micro Devices X86-64'
    readelf -hW "$quadmath" | grep -Fq 'Machine:                           Advanced Micro Devices X86-64'
    ;;
esac
expected_rpath=''
if [[ -e "$quadmath" ]]; then
  expected_rpath=\$ORIGIN
fi
if [[ "$(patchelf --print-rpath "$candidate")" != "$expected_rpath" ]]; then
  printf 'The reviewed Linux runtime has an unexpected RPATH or RUNPATH.\n' >&2
  exit 1
fi
if ldd "$candidate" | grep -Fq 'not found'; then
  printf 'The reviewed Linux runtime has an unresolved native dependency.\n' >&2
  exit 1
fi
if ldd "$candidate" | grep -E 'libgfortran|libgomp|libopenblas|libgcc_s'; then
  printf 'The reviewed Linux runtime did not statically close its compiler and solver libraries.\n' >&2
  exit 1
fi
if [[ -e "$quadmath" ]]; then
  resolved_quadmath=$(
    ldd "$candidate" |
      awk '$1 == "libquadmath.so.0" && $2 == "=>" { print $3; exit }'
  )
  if [[ "$resolved_quadmath" != "$quadmath" ]]; then
    printf 'Bundled libquadmath did not resolve from the reviewed payload: %s\n' \
      "${resolved_quadmath:-missing}" >&2
    exit 1
  fi
elif ldd "$candidate" | grep -Fq 'libquadmath'; then
  printf 'The reviewed Linux runtime has an undeclared libquadmath dependency.\n' >&2
  exit 1
fi
for native_file in "$work_root/build-one/payload/"*; do
  if readelf --version-info "$native_file" |
    grep -Eo 'GLIBC_[0-9]+\.[0-9]+' |
    sort -Vu |
    awk -F_ -v ceiling="$GLIBC_SYMBOL_CEILING" '
      function newer(left, right, a, b) {
        split(left, a, ".");
        split(right, b, ".");
        return (a[1] > b[1]) || (a[1] == b[1] && a[2] > b[2]);
      }
      newer($2, ceiling) { exit 1 }
    '; then
    :
  else
    printf 'The reviewed Linux runtime exceeds the GLIBC_%s symbol ceiling: %s\n' \
      "$GLIBC_SYMBOL_CEILING" "$native_file" >&2
    exit 1
  fi
done

mkdir -p "$work_root/runtime-test/case"
tar -xjf "$work_root/ccx_2.23.test.tar.bz2" \
  -C "$work_root/runtime-test/case" \
  ./CalculiX/ccx_2.23/test/spring1.inp
spring_case="$work_root/runtime-test/case/CalculiX/ccx_2.23/test"
(
  cd "$spring_case"
  "$candidate" spring1 \
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
cp "$work_root/build-one/openblas/LICENSE" \
  "$runtime_staging/licenses/OpenBLAS-BSD-3-Clause.txt"
gcc_runtime_exception=$(
  find /usr/share/doc -type f -name 'COPYING.RUNTIME.gz' -print -quit
)
if [[ -z "$gcc_runtime_exception" ]]; then
  printf 'The reviewed GCC package did not provide COPYING.RUNTIME.gz.\n' >&2
  exit 1
fi
gzip -cd "$gcc_runtime_exception" \
  >"$runtime_staging/licenses/GCC-Runtime-Library-Exception-3.1.txt"
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
  printf 'OpenBLAS: BSD-3-Clause. See licenses/OpenBLAS-BSD-3-Clause.txt.\n'
  printf 'Bundled or statically linked GCC runtime libraries: GPL-3.0-or-later WITH GCC-exception-3.1.\n'
  printf 'See licenses/GPL-3.0.txt and licenses/GCC-Runtime-Library-Exception-3.1.txt.\n\n'
  printf 'Corresponding-source publication is recorded separately in runtime-manifest.json.\n'
} >"$runtime_staging/THIRD_PARTY_NOTICES.txt"
build_script_sha256=$(shasum -a 256 "$0" | awk '{print $1}')
gcc_sha256=$(shasum -a 256 "$(command -v gcc)" | awk '{print $1}')
gfortran_sha256=$(shasum -a 256 "$(command -v gfortran)" | awk '{print $1}')
linker_sha256=$(shasum -a 256 "$(command -v ld)" | awk '{print $1}')
package_versions=$(
  dpkg-query -W -f='${Package}=${Version}\n' \
    binutils build-essential bzip2 cmake curl file gfortran ninja-build patchelf |
    LC_ALL=C sort
)
{
  printf '# Fraia CalculiX %s %s build recipe\n\n' "$CCX_VERSION" "$target"
  printf 'Build revision: `fraia-calculix-linux-v3`\n\n'
  printf -- '- Reviewed container: `%s`\n' "$UBUNTU_CONTAINER_IMAGE"
  printf -- '- Ubuntu snapshot: `%s`\n' "$UBUNTU_SNAPSHOT"
  printf -- '- CalculiX source SHA-256: `%s`\n' "$CCX_SOURCE_SHA256"
  printf -- '- CalculiX tests SHA-256: `%s`\n' "$CCX_TEST_SHA256"
  printf -- '- SPOOLES source SHA-256: `%s`\n' "$SPOOLES_SHA256"
  printf -- '- SPOOLES correction SHA-256: `%s`\n' "$SPOOLES_CORRECTION_SHA256"
  printf -- '- ARPACK-NG source SHA-256: `%s`\n' "$ARPACK_SHA256"
  printf -- '- OpenBLAS source SHA-256: `%s`\n' "$OPENBLAS_SHA256"
  printf -- '- GCC source: `%s`\n' "$GCC_SOURCE_URL"
  printf -- '- GCC source SHA-256: `%s`\n' "$GCC_SOURCE_SHA256"
  printf -- '- GPL-2.0 text from GCC source `COPYING` SHA-256: `%s`\n' "$GPL2_SHA256"
  printf -- '- GPL-3.0 text from GCC source `COPYING3` SHA-256: `%s`\n' "$GPL3_SHA256"
  printf -- '- gcc SHA-256: `%s`\n' "$gcc_sha256"
  printf -- '- gfortran SHA-256: `%s`\n' "$gfortran_sha256"
  printf -- '- linker SHA-256: `%s`\n' "$linker_sha256"
  printf -- '- Build script SHA-256: `%s`\n' "$build_script_sha256"
  printf -- '- SOURCE_DATE_EPOCH: `%s`\n' "$SOURCE_DATE_EPOCH"
  printf -- '- GLIBC symbol ceiling: `%s`\n\n' "$GLIBC_SYMBOL_CEILING"
  printf 'Snapshot package versions:\n\n```\n%s```\n\n' "$package_versions"
  printf 'Reproduce inside the reviewed digest-pinned container on a native matching host:\n\n'
  printf '```sh\n'
  printf './build-calculix-linux-runtime.sh --target %s --output /absolute/path/to/a-new-runtime-directory --evidence /absolute/path/to/a-new-evidence-directory\n' \
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
cp "$0" "$evidence_staging/source-inputs/build-calculix-linux-runtime.sh"
cp \
  "$work_root/ccx_2.23.src.tar.bz2" \
  "$work_root/ccx_2.23.test.tar.bz2" \
  "$work_root/spooles.2.2.tgz" \
  "$work_root/ccx_2.23.SPOOLEScorrection.tar.bz2" \
  "$work_root/arpack-ng-3.9.1.tar.gz" \
  "$work_root/OpenBLAS-0.3.34.tar.gz" \
  "$work_root/gcc-${GCC_VERSION}.tar.xz" \
  "$work_root/GPL-2.0.txt" \
  "$work_root/GPL-3.0.txt" \
  "$evidence_staging/source-inputs/"
(
  cd "$evidence_staging/source-inputs"
  shasum -a 256 ./* >SHA256SUMS
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

for native_file in "$work_root/build-one/payload/"*; do
  native_name=$(basename "$native_file")
  file -b "$native_file" >"$evidence_staging/native/${native_name}.file.txt"
  ldd "$native_file" >"$evidence_staging/native/${native_name}.dependencies.txt"
  readelf -hW "$native_file" >"$evidence_staging/native/${native_name}.elf-header.txt"
  readelf --version-info "$native_file" \
    >"$evidence_staging/native/${native_name}.symbol-versions.txt"
  readelf -dW "$native_file" >"$evidence_staging/native/${native_name}.dynamic.txt"
done

printf '%s\n' "$package_versions" >"$evidence_staging/toolchain/PACKAGES.txt"
for tool_name in ar cmake gcc gfortran ld make ninja patchelf ranlib; do
  tool_path=$(command -v "$tool_name")
  printf '%s  %s\n' \
    "$(shasum -a 256 "$tool_path" | awk '{print $1}')" \
    "$tool_name"
done >"$evidence_staging/toolchain/SHA256SUMS"
{
  printf 'Reviewed container: %s\n' "$UBUNTU_CONTAINER_IMAGE"
  printf 'Ubuntu snapshot: %s\n' "$UBUNTU_SNAPSHOT"
  printf 'GLIBC symbol ceiling: %s\n' "$GLIBC_SYMBOL_CEILING"
  printf 'SOURCE_DATE_EPOCH: %s\n' "$SOURCE_DATE_EPOCH"
} >"$evidence_staging/toolchain/ENVIRONMENT.txt"
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
