# Fraia CalculiX 2.23 linux-x64 build recipe

Build revision: `fraia-calculix-linux-v3`

- Reviewed container: `ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982`
- Ubuntu snapshot: `20260720T000000Z`
- CalculiX source SHA-256: `9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7`
- CalculiX tests SHA-256: `be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0`
- SPOOLES source SHA-256: `a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd`
- SPOOLES correction SHA-256: `15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93`
- ARPACK-NG source SHA-256: `f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844`
- OpenBLAS source SHA-256: `cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed`
- GCC source: `https://ftp.gnu.org/gnu/gcc/gcc-16.1.0/gcc-16.1.0.tar.xz`
- GCC source SHA-256: `50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79`
- GPL-2.0 text from GCC source `COPYING` SHA-256: `231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c`
- GPL-3.0 text from GCC source `COPYING3` SHA-256: `8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903`
- gcc SHA-256: `821af3c74506283c179ca413bb33e6b528805a4dd8a5c09df125e5ad560a9e89`
- gfortran SHA-256: `61bf7aa223e378dba0978c92e951c95a4e8124f8efda72f0e1fd9166a35c6bd4`
- linker SHA-256: `58937fc20c21e147883b4fdaa0fc7438a8e8f2bb886cfcaa4896100ca91139e7`
- Build script SHA-256: `0f438fa92af201c4f840468f46c916a3307bc0110488c8134a4503e3aaf6c58c`
- SOURCE_DATE_EPOCH: `1762047462`
- GLIBC symbol ceiling: `2.35`

Snapshot package versions:

```
binutils=2.38-4ubuntu2.12
build-essential=12.9ubuntu3
bzip2=1.0.8-5build1
cmake=3.22.1-1ubuntu1.22.04.2
curl=7.81.0-1ubuntu1.25
file=1:5.41-3ubuntu0.1
gfortran=4:11.2.0-1ubuntu1
ninja-build=1.10.1-1
patchelf=0.14.3-1```

Reproduce inside the reviewed digest-pinned container on a native matching host:

```sh
./build-calculix-linux-runtime.sh --target linux-x64 --output /absolute/path/to/a-new-runtime-directory --evidence /absolute/path/to/a-new-evidence-directory
```
