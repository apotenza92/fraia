# Fraia CalculiX 2.23 win32-arm64 build recipe

Build revision: `fraia-calculix-windows-arm64-v1`

- Native host: GitHub `windows-11-arm`
- MSYS2 environment: `CLANGARM64`
- setup-msys2 commit: `66cd2cce69caa17b53920067426061ca1de3a884`
- CLANGARM64 repository database SHA-256: `a32ceb26e1e830227d4e982bc9a004c5168814b37cb9a602574f0c92dbf192f1`
- MSYS2 CalculiX 2.23 recipe commit: `63200aa0d52ebb5cc8874c8813de06ba23d56c27`
- MSYS2 recipe source archive SHA-256: `4732fc54024f98145fd0dda0d109c58a8155e7c48777caecc2c894f8010f9d32`
- Applied MSYS2 patches: `ccx_mingw.patch`, `ccx_ooc.patch`, `ccx_numeric_format.patch`, `ccx_adapt_main_pastix.patch`
- Minimum Windows contract: `10.0`
- Windows console subsystem contract: `6.0`
- CalculiX source SHA-256: `9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7`
- CalculiX tests SHA-256: `be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0`
- SPOOLES source SHA-256: `a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd`
- SPOOLES correction SHA-256: `15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93`
- ARPACK-NG source SHA-256: `f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844`
- OpenBLAS source SHA-256: `cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed`
- Build script SHA-256: `7422501921cb5cf3a25e8206c5b55a0bb70c76a82cf6830da6a66b06c9488e4a`
- SOURCE_DATE_EPOCH: `1762047462`

Pinned package versions:

```
mingw-w64-clang-aarch64-clang 22.1.8-2
mingw-w64-clang-aarch64-clang-libs 22.1.8-2
mingw-w64-clang-aarch64-cmake 4.4.0-1
mingw-w64-clang-aarch64-compiler-rt 22.1.8-2
mingw-w64-clang-aarch64-crt 14.0.0.r220.gd999af622-1
mingw-w64-clang-aarch64-flang 22.1.8-2
mingw-w64-clang-aarch64-flang-rt 22.1.8-2
mingw-w64-clang-aarch64-headers 14.0.0.r220.gd999af622-1
mingw-w64-clang-aarch64-libunwind 22.1.8-1
mingw-w64-clang-aarch64-libwinpthread 14.0.0.r220.gd999af622-1
mingw-w64-clang-aarch64-lld 22.1.8-2
mingw-w64-clang-aarch64-llvm 22.1.8-2
mingw-w64-clang-aarch64-llvm-libs 22.1.8-2
mingw-w64-clang-aarch64-llvm-openmp 22.1.8-1
mingw-w64-clang-aarch64-llvm-tools 22.1.8-2
mingw-w64-clang-aarch64-ninja 1.13.2-1
mingw-w64-clang-aarch64-winpthreads 14.0.0.r220.gd999af622-1
mingw-w64-clang-aarch64-zlib 1.3.2-2
mingw-w64-clang-aarch64-zstd 1.5.7-2
```

Reproduce in the reviewed workflow on a native Windows ARM64 host:

```sh
./scripts/build-calculix-windows-arm64-runtime.sh --output /new/runtime --evidence /new/evidence
```
