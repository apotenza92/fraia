# Fraia CalculiX 2.23 darwin-x64 build recipe

Build revision: `fraia-calculix-macos-v5`

- CalculiX source SHA-256: `9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7`
- CalculiX tests SHA-256: `be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0`
- SPOOLES source SHA-256: `a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd`
- ARPACK-NG revision: `40329031ae8deb7c1e26baf8353fa384fc37c251`
- ARPACK-NG source SHA-256: `bd86b9adf3152bda8a21b3b5faf65a877b209be0f33c4629e2073a073ea5d706`
- GPL-2.0 text from GCC source `COPYING` SHA-256: `231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c`
- GPL-3.0 text from GCC source `COPYING3` SHA-256: `8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903`
- GCC source: `https://ftpmirror.gnu.org/gnu/gcc/gcc-16.1.0/gcc-16.1.0.tar.xz`
- GCC source SHA-256: `50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79`
- Homebrew/core revision: `1a2659e79c546348874da58b878ce326426749c4`
- Homebrew GCC formula SHA-256: `5f4b4fe9aab99c021d23b2c1da9025e70b502e275076da12a64fc6196db6f3d3`
- Homebrew GCC patch SHA-256: `1593153257db78c270282742088ffe961b44d793f7bbaa458895357094d6f7fc`
- GCC bottle SHA-256: `74045addfa1423d6ae6c61b1262bf5dceab762da3139a8882d1c3efd4f67407e`
- GMP bottle SHA-256: `d1192da68b2618652f4be0dd9f56b18d2d276481440ae241ce9cc17be0450e07`
- ISL bottle SHA-256: `edae3d6050998a8b6c40d79244d1c73231537371e7a36a3a72f756ed965088be`
- MPFR bottle SHA-256: `ba4a1b8388386e6618de7c7e27199ae8de473373330f5773e2095567a71d76fd`
- libmpc bottle SHA-256: `6c035aa0556baf634ceda0edc4415b6f03d675568873b6ffec4b8c6146639f44`
- zstd bottle SHA-256: `8b2443dfa62b9d28cf0321e0e670bb096b2680fe72739999228291f36018311f`
- gfortran driver SHA-256: `ef46cf36258063e563b5576ac1830e26b7a7bcaaa31280786054dea999fee487`
- gfortran f951 SHA-256: `524a4e00ee656fe87b2f14b828f2ad14a186f0baa0c888d900c0392f4a7253e6`
- gfortran collect2 SHA-256: `cca33f287f5bbbdef8f41ea57add8e954a05ecf5e6aa533aa54e5a2a3e56b8b4`
- Build script SHA-256: `013aab3fdf4ebf75d021b6ef4a5f79a97c3221f518def3a0cc11d9c058517d3d`
- SOURCE_DATE_EPOCH: `1762047462`
- MACOSX_DEPLOYMENT_TARGET: `15.0`
- macOS SDK version: `15.5`
- Native host: macOS `15.7.7` build `24G720`, `Apple clang version 17.0.0 (clang-1700.0.13.5)`

Reproduce on a native matching host with the reviewed toolchain:

```sh
./build-calculix-macos-runtime.sh --target darwin-x64 --output /absolute/path/to/a-new-runtime-directory --evidence /absolute/path/to/a-new-evidence-directory
```
