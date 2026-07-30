# Fraia CalculiX 2.23 darwin-arm64 build recipe

Build revision: `fraia-calculix-macos-v5`

- CalculiX source SHA-256: `9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7`
- CalculiX tests SHA-256: `be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0`
- SPOOLES source SHA-256: `a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd`
- ARPACK-NG revision: `40329031ae8deb7c1e26baf8353fa384fc37c251`
- ARPACK-NG source SHA-256: `bd86b9adf3152bda8a21b3b5faf65a877b209be0f33c4629e2073a073ea5d706`
- GPL-2.0 text from GCC source `COPYING` SHA-256: `231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c`
- GPL-3.0 text from GCC source `COPYING3` SHA-256: `8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903`
- GCC source: `https://ftp.gnu.org/gnu/gcc/gcc-16.1.0/gcc-16.1.0.tar.xz`
- GCC source SHA-256: `50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79`
- Homebrew/core revision: `1a2659e79c546348874da58b878ce326426749c4`
- Homebrew GCC formula SHA-256: `5f4b4fe9aab99c021d23b2c1da9025e70b502e275076da12a64fc6196db6f3d3`
- Homebrew GCC patch SHA-256: `1593153257db78c270282742088ffe961b44d793f7bbaa458895357094d6f7fc`
- GCC bottle SHA-256: `6839eac9682dee9c9ab28ab96c5f6308a3a2d96ed499fbb4c43e10d6cc3691a5`
- GMP bottle SHA-256: `6683d73d6677d28e1e8d1b92d6ebfbc068c1d33e19b79114a22a648a99ba5991`
- ISL bottle SHA-256: `de143fddb0e20b6b73016ead1e625ebd429db53918200d093e4da98f1e758889`
- MPFR bottle SHA-256: `ed822b7e77645d7c17abb3ee9cc2b2a82a4d0f003acc7615b5df6226031479b2`
- libmpc bottle SHA-256: `e7723a06cf55d69322ada010ad25c6b34627674729e41d89f2526edfa7ba6995`
- zstd bottle SHA-256: `d72adf48460a8384b256f88061cd7b9df4977df7fa2e0794051d427db754a565`
- gfortran driver SHA-256: `f0f4f9effd2eec229e9a4ddb64a30343fe2d0fd65ac50aaf70bb842f339e4f7a`
- gfortran f951 SHA-256: `d9a30a7d0564d4d6834d4c7a54691914525351763c72df20902891c557cdee80`
- gfortran collect2 SHA-256: `eeac013cf9a379d609478f2225c9efae61b3fa1ba8913fded18fa928e3d49ce6`
- Build script SHA-256: `452274f3452339e100c0107badd829bc73513698da84cea97992b2564d1c74ca`
- SOURCE_DATE_EPOCH: `1762047462`
- MACOSX_DEPLOYMENT_TARGET: `15.0`
- macOS SDK version: `15.5`
- Native host: macOS `15.7.7` build `24G720`, `Apple clang version 17.0.0 (clang-1700.0.13.5)`

Reproduce on a native matching host with the reviewed toolchain:

```sh
./build-calculix-macos-runtime.sh --target darwin-arm64 --output /absolute/path/to/a-new-runtime-directory --evidence /absolute/path/to/a-new-evidence-directory
```
