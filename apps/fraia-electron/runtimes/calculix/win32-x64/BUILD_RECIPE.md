# Fraia CalculiX 2.23 win32-x64 build recipe

Build revision: `fraia-calculix-windows-v20`

- Native host: `Microsoft Windows NT 10.0.26100.0`
- Minimum Windows contract: `Windows 10.0`
- Windows console subsystem contract: `6.0`
- WinLibs tag: `16.1.0posix-14.0.0-ucrt-r3`
- WinLibs source commit: `6e253eff2be383861ae0bf44eccbf6bfef931bf8`
- WinLibs x64 UCRT archive SHA-256: `4273565109cd8ab8ecef1b0dc2a56fd7f5c7ee0885840a1c011b9325160ec0c3`
- WinLibs source archive SHA-256: `df21e66d385972cb4cdb2c7fa55da191d0c3841bbf14a76a54bc3a56c199923d`
- CalculiX source SHA-256: `9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7`
- CalculiX tests SHA-256: `be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0`
- SPOOLES source SHA-256: `a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd`
- SPOOLES correction SHA-256: `15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93`
- ARPACK-NG source SHA-256: `f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844`
- OpenBLAS source SHA-256: `cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed`
- GCC source SHA-256: `50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79`
- MinGW-w64 source SHA-256: `d71cc644cd5a37c337f2719f3e0c79d89e8d8d5fb9e2952a62d3fa23623dc137`
- GPL-2.0 text from GCC source `COPYING` SHA-256: `231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c`
- GPL-3.0 text from GCC source `COPYING3` SHA-256: `8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903`
- winpthreads `COPYING` SHA-256: `63263614cdd29f2f93cba85e992f041b31f9fc7b4033692f31269489a8a1b177`
- Build script SHA-256: `c2fd52b24b6b5bffcc77b713aa909ee550f03a4eb35f5be9cce6d98c907383ed`
- SOURCE_DATE_EPOCH: `1762047462`
- Controlled compiler source root: `R:\` (temporary `subst` mapping)

Reproduce on a native Windows x64 host:

```powershell
.\build-calculix-windows-runtime.ps1 -OutputDirectory C:\new\runtime -EvidenceDirectory C:\new\evidence
```
