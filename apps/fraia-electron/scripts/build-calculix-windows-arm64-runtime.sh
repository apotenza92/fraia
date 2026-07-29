#!/usr/bin/env bash

set -Eeuo pipefail

CCX_VERSION='2.23'
CCX_SOURCE_URL='https://www.dhondt.de/ccx_2.23.src.tar.bz2'
CCX_SOURCE_SHA256='9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7'
CCX_TEST_URL='https://www.dhondt.de/ccx_2.23.test.tar.bz2'
CCX_TEST_SHA256='be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0'
CCX_GPL2_URL='https://www.dhondt.de/gpl-2.0.txt'
CCX_GPL2_SHA256='8177f97513213526df2cf6184d8ff986c675afb514d4e68a404010521b880643'
SPOOLES_URL='https://www.netlib.org/linalg/spooles/spooles.2.2.tgz'
SPOOLES_SHA256='a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd'
SPOOLES_CORRECTION_URL='https://www.dhondt.de/ccx_2.23.SPOOLEScorrection.tar.bz2'
SPOOLES_CORRECTION_SHA256='15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93'
ARPACK_URL='https://github.com/opencollab/arpack-ng/archive/refs/tags/3.9.1.tar.gz'
ARPACK_SHA256='f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844'
OPENBLAS_URL='https://github.com/OpenMathLib/OpenBLAS/releases/download/v0.3.34/OpenBLAS-0.3.34.tar.gz'
OPENBLAS_SHA256='cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed'
MSYS2_RECIPE_COMMIT='63200aa0d52ebb5cc8874c8813de06ba23d56c27'
MSYS2_RECIPE_URL="https://github.com/msys2/MINGW-packages/archive/${MSYS2_RECIPE_COMMIT}.tar.gz"
MSYS2_RECIPE_SHA256='4732fc54024f98145fd0dda0d109c58a8155e7c48777caecc2c894f8010f9d32'
MSYS2_REPOSITORY='https://repo.msys2.org/mingw/clangarm64'
MSYS2_CLANGARM64_DB_SHA256='a32ceb26e1e830227d4e982bc9a004c5168814b37cb9a602574f0c92dbf192f1'
MSYS2_SETUP_COMMIT='66cd2cce69caa17b53920067426061ca1de3a884'
SOURCE_DATE_EPOCH='1762047462'
MINIMUM_WINDOWS_MAJOR='10'
MINIMUM_WINDOWS_MINOR='0'
WINDOWS_SUBSYSTEM_MAJOR='6'
WINDOWS_SUBSYSTEM_MINOR='0'

usage() {
  printf 'Usage: %s --output <new-directory> --evidence <new-directory>\n' "$0" >&2
}

output=''
evidence=''
while (($#)); do
  case "$1" in
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

if [[ -z "$output" || -z "$evidence" || -e "$output" || -e "$evidence" ]]; then
  usage
  printf 'The output and evidence directories must not already exist.\n' >&2
  exit 2
fi
if [[ "$output" == "$evidence" ]]; then
  printf 'The output and evidence directories must be distinct.\n' >&2
  exit 2
fi
if [[ ${RUNNER_OS:-} != 'Windows' || ${RUNNER_ARCH:-} != 'ARM64' ]]; then
  printf 'win32-arm64 must be built on a native GitHub Windows ARM64 runner.\n' >&2
  exit 1
fi
if [[ ${MSYSTEM:-} != 'CLANGARM64' || ${MINGW_PREFIX:-} != '/clangarm64' ]]; then
  printf 'win32-arm64 must be built in the reviewed MSYS2 CLANGARM64 environment.\n' >&2
  exit 1
fi

for command in \
  awk clang cmake curl cygpath file find flang llvm-ar llvm-objdump llvm-ranlib \
  llvm-readobj llvm-strings ninja pacman patch sed sha256sum tar; do
  command -v "$command" >/dev/null || {
    printf 'Required build command is unavailable: %s\n' "$command" >&2
    exit 1
  }
done

declare -A expected_packages=(
  [mingw-w64-clang-aarch64-clang]='22.1.8-2'
  [mingw-w64-clang-aarch64-flang]='22.1.8-2'
  [mingw-w64-clang-aarch64-llvm-openmp]='22.1.8-1'
  [mingw-w64-clang-aarch64-winpthreads]='14.0.0.r220.gd999af622-1'
  [mingw-w64-clang-aarch64-cmake]='4.4.0-1'
  [mingw-w64-clang-aarch64-ninja]='1.13.2-1'
)
for package_name in "${!expected_packages[@]}"; do
  installed_version=$(pacman -Q "$package_name" | awk '{ print $2 }')
  if [[ "$installed_version" != "${expected_packages[$package_name]}" ]]; then
    printf 'Unreviewed package version for %s: expected %s, received %s.\n' \
      "$package_name" "${expected_packages[$package_name]}" "$installed_version" >&2
    exit 1
  fi
done

clang_executable="$MINGW_PREFIX/bin/clang.exe"
flang_executable="$MINGW_PREFIX/bin/flang.exe"
llvm_ar_executable="$MINGW_PREFIX/bin/llvm-ar.exe"
llvm_ranlib_executable="$MINGW_PREFIX/bin/llvm-ranlib.exe"
omp_fortran_module="$MINGW_PREFIX/include/omp_lib.mod"
if [[
  ! -f "$clang_executable"
  || ! -f "$flang_executable"
  || ! -f "$llvm_ar_executable"
  || ! -f "$llvm_ranlib_executable"
]]; then
  printf 'The reviewed CLANGARM64 compiler executables are unavailable.\n' >&2
  exit 1
fi
if [[ ! -f "$omp_fortran_module" ]]; then
  printf 'The reviewed LLVM OpenMP Fortran module is unavailable: %s\n' \
    "$omp_fortran_module" >&2
  exit 1
fi
clang_header=$(llvm-readobj --file-headers "$(cygpath -w "$clang_executable")")
flang_header=$(llvm-readobj --file-headers "$(cygpath -w "$flang_executable")")
if ! grep -Fq 'Machine: IMAGE_FILE_MACHINE_ARM64 (0xAA64)' <<<"$clang_header" ||
  ! grep -Fq 'Machine: IMAGE_FILE_MACHINE_ARM64 (0xAA64)' <<<"$flang_header"; then
  printf 'The reviewed Clang and Flang executables must themselves be native Windows ARM64.\n' >&2
  exit 1
fi
llvm_ar_cmake=$(cygpath -m "$llvm_ar_executable")
llvm_ranlib_cmake=$(cygpath -m "$llvm_ranlib_executable")
mingw_include_cmake=$(cygpath -m "$MINGW_PREFIX/include")

output=$(cd "$(dirname "$output")" && pwd)/$(basename "$output")
evidence=$(cd "$(dirname "$evidence")" && pwd)/$(basename "$evidence")
case "$output/" in
  "$evidence/"*) printf 'The output directory must not be inside the evidence directory.\n' >&2; exit 2 ;;
esac
case "$evidence/" in
  "$output/"*) printf 'The evidence directory must not be inside the output directory.\n' >&2; exit 2 ;;
esac

runner_temp=$(cygpath -u "${RUNNER_TEMP:?RUNNER_TEMP is required}")
work_root=$(mktemp -d "${runner_temp}/fraia-calculix-windows-arm64.XXXXXX")
chmod 700 "$work_root"
cleanup() {
  case "$work_root" in
    "${runner_temp}"/fraia-calculix-windows-arm64.*) rm -rf -- "$work_root" ;;
    *) printf 'Refusing to remove unexpected work directory: %s\n' "$work_root" >&2 ;;
  esac
}
report_failure() {
  local status=$1
  local log_file
  trap - ERR
  printf 'CalculiX win32-arm64 build failed with status %s. Recent build logs follow.\n' \
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
  if [[ ! -e "$evidence" ]]; then
    mkdir -p "$evidence/logs"
    {
      printf 'The controlled win32-arm64 build failed closed.\n'
      printf 'No runtime candidate was emitted.\n'
      printf 'Exit status: %s\n' "$status"
    } >"$evidence/FAILURE.txt"
    while IFS= read -r log_file; do
      cp "$log_file" "$evidence/logs/$(basename "$(dirname "$log_file")")-$(basename "$log_file")"
    done < <(
      find "$work_root" -type f \
        \( -name '*-build.log' -o -name '*-configure.log' \) \
        -print |
        sort
    )
  fi
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
  printf '%s  %s\n' "$expected_sha256" "$destination" | sha256sum -c -
}

download "$CCX_SOURCE_URL" "$work_root/ccx_2.23.src.tar.bz2" "$CCX_SOURCE_SHA256"
download "$CCX_TEST_URL" "$work_root/ccx_2.23.test.tar.bz2" "$CCX_TEST_SHA256"
download "$CCX_GPL2_URL" "$work_root/GPL-2.0.txt" "$CCX_GPL2_SHA256"
download "$SPOOLES_URL" "$work_root/spooles.2.2.tgz" "$SPOOLES_SHA256"
download \
  "$SPOOLES_CORRECTION_URL" \
  "$work_root/ccx_2.23.SPOOLEScorrection.tar.bz2" \
  "$SPOOLES_CORRECTION_SHA256"
download "$ARPACK_URL" "$work_root/arpack-ng-3.9.1.tar.gz" "$ARPACK_SHA256"
download "$OPENBLAS_URL" "$work_root/OpenBLAS-0.3.34.tar.gz" "$OPENBLAS_SHA256"
download "$MSYS2_RECIPE_URL" "$work_root/msys2-mingw-packages.tar.gz" "$MSYS2_RECIPE_SHA256"
download \
  "$MSYS2_REPOSITORY/clangarm64.db.tar.zst" \
  "$work_root/clangarm64.db.tar.zst" \
  "$MSYS2_CLANGARM64_DB_SHA256"

export LC_ALL=C
export TZ=UTC
export SOURCE_DATE_EPOCH

write_calculix_project() {
  local source_root=$1
  local project_root=$2
  local spooles_root=$3
  local spooles_library=$4
  local arpack_library=$5
  local openblas_library=$6
  local payload_root=$7
  local controlled_root=$8
  local source_name
  local source_root_cmake
  local spooles_root_cmake
  local spooles_library_cmake
  local arpack_library_cmake
  local openblas_library_cmake
  local payload_root_cmake
  source_root_cmake=$(cygpath -m "$source_root")
  spooles_root_cmake=$(cygpath -m "$spooles_root")
  spooles_library_cmake=$(cygpath -m "$spooles_library")
  arpack_library_cmake=$(cygpath -m "$arpack_library")
  openblas_library_cmake=$(cygpath -m "$openblas_library")
  payload_root_cmake=$(cygpath -m "$payload_root")

  mkdir -p "$project_root"
  {
    printf 'cmake_minimum_required(VERSION 3.24)\n'
    printf 'project(FraiaCalculiX C Fortran)\n'
    printf 'set(CCX_SOURCES\n'
    while IFS= read -r source_name; do
      [[ "$source_name" == 'mafillmm.c' ]] && continue
      if [[ ! -f "$source_root/$source_name" ]]; then
        if [[ "$source_name" == 'mafillmm.c' && -f "$source_root/mafillmm.f" ]]; then
          continue
        fi
        printf 'CalculiX Makefile.inc references missing source %s.\n' "$source_name" >&2
        return 1
      fi
      printf '  "%s"\n' "$(cygpath -m "$source_root/$source_name")"
    done < <(
      awk '
        /^[[:space:]]*[A-Za-z0-9_.]+[.][cf][[:space:]]*\\?[[:space:]]*$/ {
          gsub(/[[:space:]\\]/, "", $0);
          print;
        }
      ' "$source_root/Makefile.inc" |
        LC_ALL=C sort -u
    )
    printf ')\n'
    printf 'add_library(ccxcore STATIC ${CCX_SOURCES})\n'
    printf 'target_include_directories(ccxcore PRIVATE "%s")\n' "$spooles_root_cmake"
    printf 'target_compile_definitions(ccxcore PRIVATE ARCH=Linux SPOOLES ARPACK MATRIXSTORAGE NETWORKOUT USE_MT=1)\n'
    printf 'target_compile_options(ccxcore PRIVATE\n'
    printf '  "$<$<COMPILE_LANGUAGE:C>:-O2;-g0;-std=gnu17;-fcommon;-Wno-implicit-function-declaration;-Wno-incompatible-pointer-types;-ffile-prefix-map=%s=/usr/src/fraia-runtime;-fdebug-prefix-map=%s=/usr/src/fraia-runtime>"\n' \
      "$controlled_root" "$controlled_root"
    printf '  "$<$<COMPILE_LANGUAGE:Fortran>:-O2;-g0;-fopenmp;-cpp;-I%s>"\n' \
      "$mingw_include_cmake"
    printf ')\n'
    printf 'add_executable(ccx "%s/ccx_2.23.c")\n' "$source_root_cmake"
    printf 'target_include_directories(ccx PRIVATE "%s")\n' "$spooles_root_cmake"
    printf 'target_compile_definitions(ccx PRIVATE ARCH=Linux SPOOLES ARPACK MATRIXSTORAGE NETWORKOUT USE_MT=1)\n'
    printf 'target_compile_options(ccx PRIVATE -O2 -g0 -std=gnu17 -fcommon -Wno-implicit-function-declaration -Wno-incompatible-pointer-types "-ffile-prefix-map=%s=/usr/src/fraia-runtime" "-fdebug-prefix-map=%s=/usr/src/fraia-runtime")\n' \
      "$controlled_root" "$controlled_root"
    printf 'set_property(TARGET ccx PROPERTY LINKER_LANGUAGE Fortran)\n'
    printf 'target_link_options(ccx PRIVATE\n'
    printf '  -O2 -g0 -fopenmp\n'
    printf '  "SHELL:-Wl,--no-insert-timestamp"\n'
    printf '  "SHELL:-Wl,--major-os-version,%s"\n' "$MINIMUM_WINDOWS_MAJOR"
    printf '  "SHELL:-Wl,--minor-os-version,%s"\n' "$MINIMUM_WINDOWS_MINOR"
    printf '  "SHELL:-Wl,--major-subsystem-version,%s"\n' "$WINDOWS_SUBSYSTEM_MAJOR"
    printf '  "SHELL:-Wl,--minor-subsystem-version,%s"\n' "$WINDOWS_SUBSYSTEM_MINOR"
    printf ')\n'
    printf 'target_link_libraries(ccx PRIVATE\n'
    printf '  "-Wl,--start-group" ccxcore "%s" "%s" "%s"\n' \
      "$spooles_library_cmake" "$arpack_library_cmake" "$openblas_library_cmake"
    printf '  omp winpthread m "-Wl,--end-group"\n'
    printf ')\n'
    printf 'set_target_properties(ccx PROPERTIES OUTPUT_NAME ccx SUFFIX ".exe" RUNTIME_OUTPUT_DIRECTORY "%s")\n' \
      "$payload_root_cmake"
  } >"$project_root/CMakeLists.txt"
}

build_once() {
  local build_root=$1
  local prefix="$build_root/prefix"
  local physical_windows_root
  physical_windows_root=$(cygpath -m "$build_root")
  mkdir -p \
    "$build_root/spooles" \
    "$build_root/correction" \
    "$build_root/arpack" \
    "$build_root/openblas" \
    "$build_root/msys2-recipe" \
    "$build_root/payload"
  tar -xjf "$work_root/ccx_2.23.src.tar.bz2" -C "$build_root"
  tar -xzf "$work_root/spooles.2.2.tgz" -C "$build_root/spooles"
  tar -xjf "$work_root/ccx_2.23.SPOOLEScorrection.tar.bz2" -C "$build_root/correction"
  tar -xzf "$work_root/arpack-ng-3.9.1.tar.gz" \
    -C "$build_root/arpack" --strip-components=1
  tar -xzf "$work_root/OpenBLAS-0.3.34.tar.gz" \
    -C "$build_root/openblas" --strip-components=1
  tar -xzf "$work_root/msys2-mingw-packages.tar.gz" \
    -C "$build_root/msys2-recipe" \
    --strip-components=2 \
    "MINGW-packages-${MSYS2_RECIPE_COMMIT}/mingw-w64-calculix-ccx"

  local ccx_source="$build_root/CalculiX/ccx_2.23/src"
  local patch_name
  for patch_name in \
    ccx_mingw.patch \
    ccx_ooc.patch \
    ccx_numeric_format.patch \
    ccx_adapt_main_pastix.patch; do
    patch -d "$ccx_source" -Np1 \
      <"$build_root/msys2-recipe/$patch_name"
  done

  cp \
    "$build_root/correction/CalculiX/ccx_2.23/SPOOLES.2.2/I2Ohash/src/util.c" \
    "$build_root/spooles/I2Ohash/src/util.c"
  sed -i 's/IVinit(nfront, NULL)/IVinit(nfront, 0)/g' \
    "$build_root/spooles/ETree/src/transform.c"

  local spooles_root="$build_root/spooles"
  local spooles_root_cmake
  spooles_root_cmake=$(cygpath -m "$spooles_root")
  mkdir -p "$build_root/spooles-project"
  {
    printf 'cmake_minimum_required(VERSION 3.24)\n'
    printf 'project(FraiaSPOOLES C)\n'
    printf 'file(GLOB_RECURSE SPOOLES_SOURCES CONFIGURE_DEPENDS "%s/*/src/*.c")\n' "$spooles_root_cmake"
    printf 'list(FILTER SPOOLES_SOURCES EXCLUDE REGEX "/MPI/")\n'
    printf 'list(SORT SPOOLES_SOURCES)\n'
    printf 'add_library(spooles STATIC ${SPOOLES_SOURCES})\n'
    printf 'target_include_directories(spooles PUBLIC "%s")\n' "$spooles_root_cmake"
    printf 'target_compile_options(spooles PRIVATE -O2 -g0 -std=gnu17 "-ffile-prefix-map=%s=/usr/src/fraia-runtime" "-fdebug-prefix-map=%s=/usr/src/fraia-runtime")\n' \
      "$physical_windows_root" "$physical_windows_root"
    printf 'set_target_properties(spooles PROPERTIES OUTPUT_NAME spooles PREFIX lib)\n'
  } >"$build_root/spooles-project/CMakeLists.txt"
  cmake \
    -S "$build_root/spooles-project" \
    -B "$build_root/spooles-build" \
    -G Ninja \
    -DCMAKE_C_COMPILER=clang \
    "-DCMAKE_AR=$llvm_ar_cmake" \
    "-DCMAKE_RANLIB=$llvm_ranlib_cmake" \
    -DCMAKE_BUILD_TYPE=Release \
    >"$build_root/spooles-configure.log" 2>&1
  cmake --build "$build_root/spooles-build" --parallel 2 \
    >"$build_root/spooles-build.log" 2>&1
  local spooles_library="$build_root/spooles-build/libspooles.a"
  [[ -f "$spooles_library" ]]

  cmake \
    -S "$build_root/openblas" \
    -B "$build_root/openblas-build" \
    -G Ninja \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_Fortran_COMPILER=flang \
    "-DCMAKE_AR=$llvm_ar_cmake" \
    "-DCMAKE_RANLIB=$llvm_ranlib_cmake" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=OFF \
    -DBUILD_TESTING=OFF \
    -DNO_SHARED=ON \
    -DDYNAMIC_ARCH=OFF \
    -DMINGW64=1 \
    -DCMAKE_SYSTEM_PROCESSOR=ARM64 \
    -DTARGET=ARMV8 \
    -DUSE_OPENMP=OFF \
    -DUSE_THREAD=ON \
    -DNUM_THREADS=64 \
    "-DCMAKE_C_FLAGS_RELEASE=-O2 -g0 -ffile-prefix-map=$physical_windows_root=/usr/src/fraia-runtime -fdebug-prefix-map=$physical_windows_root=/usr/src/fraia-runtime" \
    "-DCMAKE_Fortran_FLAGS_RELEASE=-O2 -g0" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    >"$build_root/openblas-configure.log" 2>&1
  if ! grep -Eq '^#define ARCH_ARM64[[:space:]]+1$' "$build_root/openblas-build/config.h"; then
    printf 'OpenBLAS did not configure ARM64 kernels.\n' >&2
    return 1
  fi
  cmake --build "$build_root/openblas-build" --parallel 2 \
    >"$build_root/openblas-build.log" 2>&1
  cmake --install "$build_root/openblas-build" >>"$build_root/openblas-build.log" 2>&1
  local openblas_library
  openblas_library=$(find "$prefix" -type f -name 'libopenblas*.a' -print | LC_ALL=C sort | head -n 1)
  [[ -n "$openblas_library" && -f "$openblas_library" ]]

  cmake \
    -S "$build_root/arpack" \
    -B "$build_root/arpack-build" \
    -G Ninja \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_Fortran_COMPILER=flang \
    "-DCMAKE_AR=$llvm_ar_cmake" \
    "-DCMAKE_RANLIB=$llvm_ranlib_cmake" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=OFF \
    -DMPI=OFF \
    -DICB=OFF \
    -DEXAMPLES=OFF \
    "-DCMAKE_C_FLAGS_RELEASE=-O2 -g0 -ffile-prefix-map=$physical_windows_root=/usr/src/fraia-runtime -fdebug-prefix-map=$physical_windows_root=/usr/src/fraia-runtime" \
    "-DCMAKE_Fortran_FLAGS_RELEASE=-O2 -g0" \
    -DBLAS_LIBRARIES="$openblas_library" \
    -DLAPACK_LIBRARIES="$openblas_library" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    >"$build_root/arpack-configure.log" 2>&1
  cmake --build "$build_root/arpack-build" --parallel 2 \
    >"$build_root/arpack-build.log" 2>&1
  cmake --install "$build_root/arpack-build" >>"$build_root/arpack-build.log" 2>&1
  local arpack_library
  arpack_library=$(find "$prefix" -type f -name 'libarpack*.a' -print | LC_ALL=C sort | head -n 1)
  [[ -n "$arpack_library" && -f "$arpack_library" ]]

  sed -i \
    '/^void readnewmesh(/,/^}/{s/^[[:space:]]*return NULL;[[:space:]]*$/  return;/;}' \
    "$ccx_source/readnewmesh.c"
  write_calculix_project \
    "$ccx_source" \
    "$build_root/calculix-project" \
    "$spooles_root" \
    "$spooles_library" \
    "$arpack_library" \
    "$openblas_library" \
    "$build_root/payload" \
    "$physical_windows_root"
  cmake \
    -S "$build_root/calculix-project" \
    -B "$build_root/calculix-build" \
    -G Ninja \
    -DCMAKE_C_COMPILER=clang \
    -DCMAKE_Fortran_COMPILER=flang \
    "-DCMAKE_AR=$llvm_ar_cmake" \
    "-DCMAKE_RANLIB=$llvm_ranlib_cmake" \
    -DCMAKE_BUILD_TYPE=Release \
    >"$build_root/calculix-configure.log" 2>&1
  cmake --build "$build_root/calculix-build" --parallel 2 \
    >"$build_root/calculix-build.log" 2>&1
  [[ -f "$build_root/payload/ccx.exe" ]]
  cp "$MINGW_PREFIX/bin/libomp.dll" "$build_root/payload/libomp.dll"
  cp "$MINGW_PREFIX/bin/libwinpthread-1.dll" "$build_root/payload/libwinpthread-1.dll"
}

build_once "$work_root/build-one"
build_once "$work_root/build-two"

write_intermediate_checksums() {
  local build_root=$1
  local destination=$2
  local file_path
  local file_sha256
  (
    cd "$build_root"
    find \
      spooles-build \
      openblas-build \
      arpack-build \
      calculix-build \
      prefix \
      -type f \
      \( \
        -name '*.a' \
        -o -name '*.mod' \
        -o -name '*.o' \
        -o -name '*.obj' \
      \) \
      -print |
      LC_ALL=C sort |
      while IFS= read -r file_path; do
        file_sha256=$(sha256sum "$file_path" | awk '{ print $1 }')
        printf '%s  %s\n' "$file_sha256" "$file_path"
      done
  ) >"$destination"
}

write_reproducibility_failure_evidence() {
  local first_payload="$work_root/build-one/payload"
  local second_payload="$work_root/build-two/payload"
  local failure_staging="$work_root/reproducibility-failure"
  local native_name
  local native_file

  mkdir -p \
    "$failure_staging/binaries" \
    "$failure_staging/intermediates" \
    "$failure_staging/native"
  {
    printf 'The two controlled native Windows ARM64 builds were not byte-identical.\n'
    printf 'No runtime candidate was emitted or promoted.\n'
    printf 'The files in this directory are diagnostic review evidence only.\n'
  } >"$failure_staging/FAILURE.txt"
  (
    cd "$first_payload"
    sha256sum ./* >"$failure_staging/build-one-SHA256SUMS"
  )
  (
    cd "$second_payload"
    sha256sum ./* >"$failure_staging/build-two-SHA256SUMS"
  )
  diff -u \
    "$failure_staging/build-one-SHA256SUMS" \
    "$failure_staging/build-two-SHA256SUMS" \
    >"$failure_staging/PAYLOAD-DIFF.txt" || true

  cp "$first_payload/ccx.exe" "$failure_staging/binaries/build-one-ccx.exe"
  cp "$second_payload/ccx.exe" "$failure_staging/binaries/build-two-ccx.exe"
  cmp -l "$first_payload/ccx.exe" "$second_payload/ccx.exe" |
    awk '
      NR <= 10000 { print }
      END {
        printf "Total differing bytes: %d\n", NR;
        if (NR > 10000) {
          printf "Only the first 10000 differing byte positions are listed.\n";
        }
      }
    ' >"$failure_staging/CCX-BYTE-DIFFERENCES.txt" || true

  for build_name in build-one build-two; do
    native_file="$work_root/$build_name/payload/ccx.exe"
    llvm-readobj --file-headers --sections --coff-imports "$native_file" \
      >"$failure_staging/native/${build_name}-ccx.readobj.txt"
    llvm-objdump -h -p "$native_file" \
      >"$failure_staging/native/${build_name}-ccx.objdump.txt"
    llvm-strings "$native_file" \
      >"$failure_staging/native/${build_name}-ccx.strings.txt"
    write_intermediate_checksums \
      "$work_root/$build_name" \
      "$failure_staging/intermediates/${build_name}-SHA256SUMS"
  done
  diff -u \
    "$failure_staging/intermediates/build-one-SHA256SUMS" \
    "$failure_staging/intermediates/build-two-SHA256SUMS" \
    >"$failure_staging/intermediates/DIFF.txt" || true
  {
    printf 'Native host: GitHub windows-11-arm\n'
    printf 'Runner architecture: %s\n' "$RUNNER_ARCH"
    printf 'MSYS2 environment: %s\n' "$MSYSTEM"
    printf 'SOURCE_DATE_EPOCH: %s\n' "$SOURCE_DATE_EPOCH"
    clang --version
    flang --version
    cmake --version
    ninja --version
  } >"$failure_staging/TOOLCHAIN.txt"
  cp "$0" "$failure_staging/build-calculix-windows-arm64-runtime.sh"
  (
    cd "$failure_staging"
    find . -type f ! -name EVIDENCE_SHA256SUMS -print |
      LC_ALL=C sort |
      while IFS= read -r file_path; do
        sha256sum "$file_path"
      done >EVIDENCE_SHA256SUMS
  )
  mv "$failure_staging" "$evidence"
}

for payload in "$work_root/build-one/payload" "$work_root/build-two/payload"; do
  if llvm-strings "$payload"/* | grep -F "$work_root"; then
    printf 'The reviewed Windows ARM64 runtime contains an absolute build path: %s\n' \
      "$payload" >&2
    exit 1
  fi
done
if ! diff -qr "$work_root/build-one/payload" "$work_root/build-two/payload"; then
  printf 'The independently built Windows ARM64 runtimes are not byte-identical.\n' >&2
  sha256sum "$work_root/build-one/payload/"* >&2
  sha256sum "$work_root/build-two/payload/"* >&2
  write_reproducibility_failure_evidence
  exit 1
fi

candidate="$work_root/build-one/payload/ccx.exe"
declare -A bundled_dependencies=(
  [libomp.dll]=1
  [libwinpthread-1.dll]=1
)
declare -A allowed_system_dependencies=(
  [ADVAPI32.dll]=1
  [KERNEL32.dll]=1
  [api-ms-win-crt-conio-l1-1-0.dll]=1
  [api-ms-win-crt-convert-l1-1-0.dll]=1
  [api-ms-win-crt-environment-l1-1-0.dll]=1
  [api-ms-win-crt-filesystem-l1-1-0.dll]=1
  [api-ms-win-crt-heap-l1-1-0.dll]=1
  [api-ms-win-crt-locale-l1-1-0.dll]=1
  [api-ms-win-crt-math-l1-1-0.dll]=1
  [api-ms-win-crt-multibyte-l1-1-0.dll]=1
  [api-ms-win-crt-private-l1-1-0.dll]=1
  [api-ms-win-crt-runtime-l1-1-0.dll]=1
  [api-ms-win-crt-stdio-l1-1-0.dll]=1
  [api-ms-win-crt-string-l1-1-0.dll]=1
  [api-ms-win-crt-time-l1-1-0.dll]=1
  [api-ms-win-crt-utility-l1-1-0.dll]=1
)
for native_file in "$work_root/build-one/payload/"*; do
  if ! grep -Fq 'Machine: IMAGE_FILE_MACHINE_ARM64 (0xAA64)' \
    < <(llvm-readobj --file-headers "$native_file"); then
    printf 'The reviewed native file is not PE ARM64: %s\n' "$native_file" >&2
    exit 1
  fi
  while IFS= read -r dependency; do
    if [[ -z ${bundled_dependencies[$dependency]+x} &&
      -z ${allowed_system_dependencies[$dependency]+x} ]]; then
      printf 'Unreviewed native dependency %s in %s.\n' "$dependency" "$native_file" >&2
      exit 1
    fi
  done < <(llvm-objdump -p "$native_file" | awk '/DLL Name:/{ print $3 }' | LC_ALL=C sort -fu)
done
pe_header=$(llvm-objdump -p "$candidate")
for contract in \
  "MajorOSystemVersion      ${MINIMUM_WINDOWS_MAJOR}" \
  "MinorOSystemVersion      ${MINIMUM_WINDOWS_MINOR}" \
  "MajorSubsystemVersion   ${WINDOWS_SUBSYSTEM_MAJOR}" \
  "MinorSubsystemVersion   ${WINDOWS_SUBSYSTEM_MINOR}"; do
  grep -Fq "$contract" <<<"$pe_header" || {
    printf 'The reviewed Windows version contract is missing: %s\n' "$contract" >&2
    exit 1
  }
done

mkdir -p "$work_root/runtime-test/case"
tar -xjf "$work_root/ccx_2.23.test.tar.bz2" \
  -C "$work_root/runtime-test/case" \
  ./CalculiX/ccx_2.23/test/spring1.inp
spring_case="$work_root/runtime-test/case/CalculiX/ccx_2.23/test"
(
  cd "$spring_case"
  "$(cygpath -w "$candidate")" spring1 \
    >"$work_root/runtime-test/spring1.stdout" \
    2>"$work_root/runtime-test/spring1.stderr"
)
test -s "$spring_case/spring1.dat"
test -s "$spring_case/spring1.frd"
test -s "$spring_case/spring1.sta"
grep -Fq 'Job finished' "$work_root/runtime-test/spring1.stdout"

runtime_staging="$work_root/runtime-output"
mkdir -p "$runtime_staging/licenses"
cp "$work_root/build-one/payload/"* "$runtime_staging/"
cp "$work_root/GPL-2.0.txt" "$runtime_staging/licenses/GPL-2.0.txt"
cp "$work_root/build-one/arpack/COPYING" \
  "$runtime_staging/licenses/ARPACK-BSD-3-Clause.txt"
cp "$work_root/build-one/openblas/LICENSE" \
  "$runtime_staging/licenses/OpenBLAS-BSD-3-Clause.txt"
cp "$MINGW_PREFIX/share/licenses/openmp/LICENSE" \
  "$runtime_staging/licenses/LLVM-OpenMP-Apache-2.0-WITH-LLVM-exception.txt"
flang_license=$(find "$MINGW_PREFIX/share/licenses" -path '*flang*' -type f -print | LC_ALL=C sort | head -n 1)
[[ -n "$flang_license" && -f "$flang_license" ]]
cp "$flang_license" \
  "$runtime_staging/licenses/LLVM-Flang-Apache-2.0-WITH-LLVM-exception.txt"
cp "$MINGW_PREFIX/share/licenses/winpthreads/COPYING" \
  "$runtime_staging/licenses/winpthreads-MIT-AND-BSD-3-Clause.txt"
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
  printf 'Statically linked LLVM Flang runtime and bundled LLVM OpenMP: Apache-2.0 WITH LLVM-exception.\n'
  printf 'See licenses/LLVM-Flang-Apache-2.0-WITH-LLVM-exception.txt and licenses/LLVM-OpenMP-Apache-2.0-WITH-LLVM-exception.txt.\n'
  printf 'Bundled winpthreads: MIT AND BSD-3-Clause. See licenses/winpthreads-MIT-AND-BSD-3-Clause.txt.\n\n'
  printf 'Corresponding-source publication is recorded separately in runtime-manifest.json.\n'
} >"$runtime_staging/THIRD_PARTY_NOTICES.txt"

build_script_sha256=$(sha256sum "$0" | awk '{ print $1 }')
package_versions=$(
  pacman -Q |
    awk '$1 ~ /^mingw-w64-clang-aarch64-(clang|flang|llvm|lld|compiler-rt|crt|headers|libc\\+\\+|libunwind|libwinpthread|winpthreads|cmake|ninja|zlib|zstd)/ { print }' |
    LC_ALL=C sort
)
{
  printf '# Fraia CalculiX %s win32-arm64 build recipe\n\n' "$CCX_VERSION"
  printf 'Build revision: `fraia-calculix-windows-arm64-v1`\n\n'
  printf -- '- Native host: GitHub `windows-11-arm`\n'
  printf -- '- MSYS2 environment: `CLANGARM64`\n'
  printf -- '- setup-msys2 commit: `%s`\n' "$MSYS2_SETUP_COMMIT"
  printf -- '- CLANGARM64 repository database SHA-256: `%s`\n' "$MSYS2_CLANGARM64_DB_SHA256"
  printf -- '- MSYS2 CalculiX 2.23 recipe commit: `%s`\n' "$MSYS2_RECIPE_COMMIT"
  printf -- '- MSYS2 recipe source archive SHA-256: `%s`\n' "$MSYS2_RECIPE_SHA256"
  printf -- '- Applied MSYS2 patches: `ccx_mingw.patch`, `ccx_ooc.patch`, `ccx_numeric_format.patch`, `ccx_adapt_main_pastix.patch`\n'
  printf -- '- Minimum Windows contract: `%s.%s`\n' "$MINIMUM_WINDOWS_MAJOR" "$MINIMUM_WINDOWS_MINOR"
  printf -- '- Windows console subsystem contract: `%s.%s`\n' "$WINDOWS_SUBSYSTEM_MAJOR" "$WINDOWS_SUBSYSTEM_MINOR"
  printf -- '- CalculiX source SHA-256: `%s`\n' "$CCX_SOURCE_SHA256"
  printf -- '- CalculiX tests SHA-256: `%s`\n' "$CCX_TEST_SHA256"
  printf -- '- SPOOLES source SHA-256: `%s`\n' "$SPOOLES_SHA256"
  printf -- '- SPOOLES correction SHA-256: `%s`\n' "$SPOOLES_CORRECTION_SHA256"
  printf -- '- ARPACK-NG source SHA-256: `%s`\n' "$ARPACK_SHA256"
  printf -- '- OpenBLAS source SHA-256: `%s`\n' "$OPENBLAS_SHA256"
  printf -- '- Build script SHA-256: `%s`\n' "$build_script_sha256"
  printf -- '- SOURCE_DATE_EPOCH: `%s`\n\n' "$SOURCE_DATE_EPOCH"
  printf 'Pinned package versions:\n\n```\n%s\n```\n\n' "$package_versions"
  printf 'Reproduce in the reviewed workflow on a native Windows ARM64 host:\n\n'
  printf '```sh\n'
  printf './scripts/build-calculix-windows-arm64-runtime.sh --output /new/runtime --evidence /new/evidence\n'
  printf '```\n'
} >"$runtime_staging/BUILD_RECIPE.md"
(
  cd "$runtime_staging"
  find . -type f ! -name SHA256SUMS -print |
    LC_ALL=C sort |
    while IFS= read -r file_path; do
      sha256sum "$file_path"
    done >SHA256SUMS
)

evidence_staging="$work_root/review-evidence"
mkdir -p \
  "$evidence_staging/native" \
  "$evidence_staging/reproducibility" \
  "$evidence_staging/solver" \
  "$evidence_staging/source-inputs" \
  "$evidence_staging/toolchain"
cp "$0" "$evidence_staging/source-inputs/build-calculix-windows-arm64-runtime.sh"
cp \
  "$work_root/ccx_2.23.src.tar.bz2" \
  "$work_root/ccx_2.23.test.tar.bz2" \
  "$work_root/GPL-2.0.txt" \
  "$work_root/spooles.2.2.tgz" \
  "$work_root/ccx_2.23.SPOOLEScorrection.tar.bz2" \
  "$work_root/arpack-ng-3.9.1.tar.gz" \
  "$work_root/OpenBLAS-0.3.34.tar.gz" \
  "$work_root/msys2-mingw-packages.tar.gz" \
  "$work_root/clangarm64.db.tar.zst" \
  "$evidence_staging/source-inputs/"
(
  cd "$evidence_staging/source-inputs"
  sha256sum ./* >SHA256SUMS
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
  sha256sum ./* >SHA256SUMS
)
for build_name in build-one build-two; do
  (
    cd "$work_root/$build_name/payload"
    sha256sum ./* >"$evidence_staging/reproducibility/${build_name}-SHA256SUMS"
  )
done
cmp \
  "$evidence_staging/reproducibility/build-one-SHA256SUMS" \
  "$evidence_staging/reproducibility/build-two-SHA256SUMS"
printf 'The two independently source-built native payloads are byte-identical.\n' \
  >"$evidence_staging/reproducibility/RESULT.txt"
for native_file in "$work_root/build-one/payload/"*; do
  native_name=$(basename "$native_file")
  file -b "$native_file" >"$evidence_staging/native/${native_name}.file.txt"
  llvm-readobj --file-headers "$native_file" \
    >"$evidence_staging/native/${native_name}.pe-header.txt"
  llvm-objdump -p "$native_file" \
    >"$evidence_staging/native/${native_name}.dependencies.txt"
done
printf '%s\n' "$package_versions" >"$evidence_staging/toolchain/PACKAGES.txt"
for tool_name in clang cmake flang llvm-ar llvm-objdump llvm-ranlib lld-link ninja; do
  tool_path=$(command -v "$tool_name")
  if [[ ! -f "$tool_path" && -f "${tool_path}.exe" ]]; then
    tool_path="${tool_path}.exe"
  fi
  [[ -f "$tool_path" ]]
  printf '%s  %s\n' "$(sha256sum "$tool_path" | awk '{ print $1 }')" "$tool_name"
done >"$evidence_staging/toolchain/SHA256SUMS"
{
  printf 'Native host: GitHub windows-11-arm\n'
  printf 'Runner architecture: %s\n' "$RUNNER_ARCH"
  printf 'MSYS2 environment: %s\n' "$MSYSTEM"
  printf 'setup-msys2 commit: %s\n' "$MSYS2_SETUP_COMMIT"
  printf 'CLANGARM64 repository database SHA-256: %s\n' "$MSYS2_CLANGARM64_DB_SHA256"
  printf 'SOURCE_DATE_EPOCH: %s\n' "$SOURCE_DATE_EPOCH"
  clang --version
  flang --version
  cmake --version
  ninja --version
} >"$evidence_staging/toolchain/ENVIRONMENT.txt"
cp "$runtime_staging/BUILD_RECIPE.md" "$evidence_staging/BUILD_RECIPE.md"
cp "$runtime_staging/SHA256SUMS" "$evidence_staging/RUNTIME_SHA256SUMS"
cp "$runtime_staging/THIRD_PARTY_NOTICES.txt" "$evidence_staging/THIRD_PARTY_NOTICES.txt"
{
  printf '# Fraia CalculiX %s win32-arm64 review evidence\n\n' "$CCX_VERSION"
  printf 'This directory is review evidence, not a promotable runtime.\n'
  printf 'Promote only the separately emitted runtime after this evidence, corresponding-source publication, and runtime-manifest review all pass.\n'
} >"$evidence_staging/README.md"
(
  cd "$evidence_staging"
  find . -type f ! -name EVIDENCE_SHA256SUMS -print |
    LC_ALL=C sort |
    while IFS= read -r file_path; do
      sha256sum "$file_path"
    done >EVIDENCE_SHA256SUMS
)

mv "$evidence_staging" "$evidence"
mv "$runtime_staging" "$output"
printf 'Built and independently reproduced win32-arm64 CalculiX %s runtime at %s\n' \
  "$CCX_VERSION" "$output"
printf 'Wrote independently reviewable win32-arm64 evidence at %s\n' "$evidence"
