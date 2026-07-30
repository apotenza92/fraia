const path = require('node:path');

const CALCULIX_SOURCE_ASSET_NAME = 'Fraia-CalculiX-Corresponding-Source.tar';
const SOURCE_DATE_EPOCH = 1762047462;

const SOURCE_INPUTS = Object.freeze([
  {
    fileName: 'ccx_2.23.src.tar.bz2',
    url: 'https://www.dhondt.de/ccx_2.23.src.tar.bz2',
    sha256: '9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7',
    usedBy: ['darwin', 'linux', 'win32', 'win32-arm64'],
  },
  {
    fileName: 'ccx_2.23.test.tar.bz2',
    url: 'https://www.dhondt.de/ccx_2.23.test.tar.bz2',
    sha256: 'be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0',
    usedBy: ['darwin', 'linux', 'win32', 'win32-arm64'],
  },
  {
    fileName: 'spooles.2.2.tgz',
    url: 'https://www.netlib.org/linalg/spooles/spooles.2.2.tgz',
    sha256: 'a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd',
    usedBy: ['darwin', 'linux', 'win32', 'win32-arm64'],
  },
  {
    fileName: 'ccx_2.23.SPOOLEScorrection.tar.bz2',
    url: 'https://www.dhondt.de/ccx_2.23.SPOOLEScorrection.tar.bz2',
    sha256: '15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93',
    usedBy: ['linux', 'win32', 'win32-arm64'],
  },
  {
    fileName: 'arpack-ng-macos.tar.gz',
    url: 'https://github.com/opencollab/arpack-ng/archive/40329031ae8deb7c1e26baf8353fa384fc37c251.tar.gz',
    sha256: 'bd86b9adf3152bda8a21b3b5faf65a877b209be0f33c4629e2073a073ea5d706',
    usedBy: ['darwin'],
  },
  {
    fileName: 'arpack-ng-3.9.1.tar.gz',
    url: 'https://github.com/opencollab/arpack-ng/archive/refs/tags/3.9.1.tar.gz',
    sha256: 'f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844',
    usedBy: ['linux', 'win32', 'win32-arm64'],
  },
  {
    fileName: 'OpenBLAS-0.3.34.tar.gz',
    url: 'https://github.com/OpenMathLib/OpenBLAS/releases/download/v0.3.34/OpenBLAS-0.3.34.tar.gz',
    sha256: 'cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed',
    usedBy: ['linux', 'win32', 'win32-arm64'],
  },
  {
    fileName: 'gcc-16.1.0.tar.xz',
    url: 'https://ftp.gnu.org/gnu/gcc/gcc-16.1.0/gcc-16.1.0.tar.xz',
    sha256: '50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79',
    usedBy: ['darwin', 'linux', 'win32'],
  },
  {
    fileName: 'homebrew-gcc.rb',
    url: 'https://raw.githubusercontent.com/Homebrew/homebrew-core/1a2659e79c546348874da58b878ce326426749c4/Formula/g/gcc.rb',
    sha256: '5f4b4fe9aab99c021d23b2c1da9025e70b502e275076da12a64fc6196db6f3d3',
    usedBy: ['darwin'],
  },
  {
    fileName: 'homebrew-gcc-16.1.0.diff',
    url: 'https://raw.githubusercontent.com/Homebrew/homebrew-core/1a2659e79c546348874da58b878ce326426749c4/Patches/gcc/gcc-16.1.0.diff',
    sha256: '1593153257db78c270282742088ffe961b44d793f7bbaa458895357094d6f7fc',
    usedBy: ['darwin'],
  },
  {
    fileName: 'winlibs-mingw-source.tar.gz',
    url: 'https://github.com/brechtsanders/winlibs_mingw/archive/6e253eff2be383861ae0bf44eccbf6bfef931bf8.tar.gz',
    sha256: 'df21e66d385972cb4cdb2c7fa55da191d0c3841bbf14a76a54bc3a56c199923d',
    usedBy: ['win32'],
  },
  {
    fileName: 'mingw-w64-v14.0.0.tar.gz',
    url: 'https://github.com/mingw-w64/mingw-w64/archive/refs/tags/v14.0.0.tar.gz',
    sha256: 'd71cc644cd5a37c337f2719f3e0c79d89e8d8d5fb9e2952a62d3fa23623dc137',
    usedBy: ['win32'],
  },
  {
    fileName: 'msys2-mingw-packages-arm64.tar.gz',
    url: 'https://github.com/msys2/MINGW-packages/archive/63200aa0d52ebb5cc8874c8813de06ba23d56c27.tar.gz',
    sha256: '4732fc54024f98145fd0dda0d109c58a8155e7c48777caecc2c894f8010f9d32',
    usedBy: ['win32-arm64'],
  },
]);

const BUILD_RECIPES = Object.freeze([
  {
    platform: 'darwin',
    path: path.join('scripts', 'build-calculix-macos-runtime.sh'),
  },
  {
    platform: 'linux',
    path: path.join('scripts', 'build-calculix-linux-runtime.sh'),
  },
  {
    platform: 'win32',
    path: path.join('scripts', 'build-calculix-windows-runtime.ps1'),
  },
  {
    platform: 'win32-arm64',
    path: path.join('scripts', 'build-calculix-windows-arm64-runtime.sh'),
  },
]);

function correspondingSourceUrl(repository, tag) {
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
    throw new Error(`Invalid GitHub repository: ${repository}`);
  }
  if (!/^v\d+\.\d+\.\d+(?:-beta\.\d+)?$/.test(tag)) {
    throw new Error(`Invalid Fraia release tag: ${tag}`);
  }
  return `https://github.com/${repository}/releases/download/${tag}/${CALCULIX_SOURCE_ASSET_NAME}`;
}

module.exports = {
  BUILD_RECIPES,
  CALCULIX_SOURCE_ASSET_NAME,
  SOURCE_DATE_EPOCH,
  SOURCE_INPUTS,
  correspondingSourceUrl,
};
