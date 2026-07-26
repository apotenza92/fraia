[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$OutputDirectory,
  [Parameter(Mandatory = $true)]
  [string]$EvidenceDirectory,
  [string]$ReviewedSourceDirectory
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$Utf8NoBom = [Text.UTF8Encoding]::new($false)

$CalculixVersion = "2.23"
$CalculixSourceUrl = "https://www.dhondt.de/ccx_2.23.src.tar.bz2"
$CalculixSourceSha256 = "9c88385c10fb04f5dc6c4e98027a51bebdd8aee3920e05190d6c1dd08357d6e7"
$CalculixTestUrl = "https://www.dhondt.de/ccx_2.23.test.tar.bz2"
$CalculixTestSha256 = "be2259fd9a7b990d0453b30708e1b05f2cd4b6df4a90fa96f0e4abd1ae7beaa0"
$SpoolesUrl = "https://www.netlib.org/linalg/spooles/spooles.2.2.tgz"
$SpoolesSha256 = "a84559a0e987a1e423055ef4fdf3035d55b65bbe4bf915efaa1a35bef7f8c5dd"
$SpoolesCorrectionUrl = "https://www.dhondt.de/ccx_2.23.SPOOLEScorrection.tar.bz2"
$SpoolesCorrectionSha256 = "15528f09312dc7605c0600358f5e6de12945449f249dfcfca7417eed6c220b93"
$ArpackUrl = "https://github.com/opencollab/arpack-ng/archive/refs/tags/3.9.1.tar.gz"
$ArpackSha256 = "f6641deb07fa69165b7815de9008af3ea47eb39b2bb97521fbf74c97aba6e844"
$OpenBlasUrl = "https://github.com/OpenMathLib/OpenBLAS/releases/download/v0.3.34/OpenBLAS-0.3.34.tar.gz"
$OpenBlasSha256 = "cd7e129868320cc2d033afa920e31202dfe0b8066a5b66661900ccc0f197dfed"
$Gpl2Sha256 = "231f7edcc7352d7734a96eef0b8030f77982678c516876fcb81e25b32d68564c"
$Gpl3Sha256 = "8ceb4b9ee5adedde47b31e975c1d90c73ad27b6b165a1dcd80c7c545eb65b903"

$WinLibsTag = "16.1.0posix-14.0.0-ucrt-r3"
$WinLibsCommit = "6e253eff2be383861ae0bf44eccbf6bfef931bf8"
$WinLibsArchiveUrl = "https://github.com/brechtsanders/winlibs_mingw/releases/download/${WinLibsTag}/winlibs-x86_64-posix-seh-gcc-16.1.0-mingw-w64ucrt-14.0.0-r3.zip"
$WinLibsArchiveSha256 = "4273565109cd8ab8ecef1b0dc2a56fd7f5c7ee0885840a1c011b9325160ec0c3"
$WinLibsSourceUrl = "https://github.com/brechtsanders/winlibs_mingw/archive/${WinLibsCommit}.tar.gz"
$WinLibsSourceSha256 = "df21e66d385972cb4cdb2c7fa55da191d0c3841bbf14a76a54bc3a56c199923d"
$GccVersion = "16.1.0"
$GccSourceUrl = "https://ftpmirror.gnu.org/gnu/gcc/gcc-${GccVersion}/gcc-${GccVersion}.tar.xz"
$GccSourceSha256 = "50efb4d94c3397aff3b0d61a5abd748b4dd31d9d3f2ab7be05b171d36a510f79"
$MingwVersion = "14.0.0"
$MingwSourceUrl = "https://github.com/mingw-w64/mingw-w64/archive/refs/tags/v${MingwVersion}.tar.gz"
$MingwSourceSha256 = "d71cc644cd5a37c337f2719f3e0c79d89e8d8d5fb9e2952a62d3fa23623dc137"
$SourceDateEpoch = "1762047462"
$MinimumWindowsMajor = 10
$MinimumWindowsMinor = 0
$ControlledBuildDrive = "R:"

$AllowedSystemImports = @(
  "ADVAPI32.dll",
  "KERNEL32.dll",
  "api-ms-win-crt-conio-l1-1-0.dll",
  "api-ms-win-crt-convert-l1-1-0.dll",
  "api-ms-win-crt-environment-l1-1-0.dll",
  "api-ms-win-crt-filesystem-l1-1-0.dll",
  "api-ms-win-crt-heap-l1-1-0.dll",
  "api-ms-win-crt-locale-l1-1-0.dll",
  "api-ms-win-crt-math-l1-1-0.dll",
  "api-ms-win-crt-multibyte-l1-1-0.dll",
  "api-ms-win-crt-private-l1-1-0.dll",
  "api-ms-win-crt-runtime-l1-1-0.dll",
  "api-ms-win-crt-stdio-l1-1-0.dll",
  "api-ms-win-crt-string-l1-1-0.dll",
  "api-ms-win-crt-time-l1-1-0.dll",
  "api-ms-win-crt-utility-l1-1-0.dll"
)

function Assert-Sha256 {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [Parameter(Mandatory = $true)]
    [string]$Expected
  )

  $Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
  if ($Actual -ne $Expected) {
    throw "SHA-256 mismatch for ${Path}: expected ${Expected}, received ${Actual}."
  }
}

function Invoke-ReviewedDownload {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Url,
    [Parameter(Mandatory = $true)]
    [string]$Destination,
    [Parameter(Mandatory = $true)]
    [string]$Sha256
  )

  $MaximumAttempts = 4
  for ($Attempt = 1; $Attempt -le $MaximumAttempts; $Attempt += 1) {
    try {
      Write-Host "Downloading reviewed input ${Url} (attempt ${Attempt}/${MaximumAttempts})."
      Invoke-WebRequest -Uri $Url -OutFile $Destination
      break
    } catch {
      if ($Attempt -eq $MaximumAttempts) {
        throw "Reviewed download failed after ${MaximumAttempts} attempts: ${Url}. $($_.Exception.Message)"
      }
      Start-Sleep -Seconds (5 * $Attempt)
    }
  }
  Assert-Sha256 -Path $Destination -Expected $Sha256
}

function Invoke-LoggedCommand {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Executable,
    [Parameter(Mandatory = $true)]
    [string[]]$Arguments,
    [Parameter(Mandatory = $true)]
    [string]$LogPath
  )

  [string[]]$Lines = @(& $Executable @Arguments 2>&1 | ForEach-Object { "$_" })
  [IO.File]::AppendAllLines($LogPath, $Lines, $Utf8NoBom)
  if ($LASTEXITCODE -ne 0) {
    $Tail = Get-Content -LiteralPath $LogPath -Tail 120
    [Console]::Error.WriteLine(($Tail -join [Environment]::NewLine))
    throw "Reviewed command failed with exit code ${LASTEXITCODE}: ${Executable}"
  }
}

function Get-PeImports {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Executable,
    [Parameter(Mandatory = $true)]
    [string]$Objdump
  )

  $Output = @(& $Objdump -p $Executable 2>&1)
  if ($LASTEXITCODE -ne 0) {
    throw "objdump failed while inspecting ${Executable}."
  }
  return @(
    $Output |
      ForEach-Object {
        if ($_ -match "^\s*DLL Name:\s*([A-Za-z0-9_.+-]+\.dll)\s*$") {
          $Matches[1]
        }
      } |
      Where-Object { $_ } |
      Sort-Object -Unique
  )
}

function Get-PeHeaderValue {
  param(
    [Parameter(Mandatory = $true)]
    [string[]]$Header,
    [Parameter(Mandatory = $true)]
    [string]$Name
  )

  $Match = $Header | Where-Object { $_ -match "^\s*${Name}\s+([0-9]+)" } | Select-Object -First 1
  if (-not $Match -or $Match -notmatch "^\s*${Name}\s+([0-9]+)") {
    throw "PE header field ${Name} is unavailable."
  }
  return [int]$Matches[1]
}

function Get-RelativeUnixPath {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Root,
    [Parameter(Mandatory = $true)]
    [string]$Path
  )

  return ([IO.Path]::GetRelativePath($Root, $Path) -replace "\\", "/")
}

if (-not $IsWindows -or [Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne "X64") {
  throw "win32-x64 must be built on a native Windows x64 host."
}

$ResolvedOutput = [IO.Path]::GetFullPath($OutputDirectory)
$ResolvedEvidence = [IO.Path]::GetFullPath($EvidenceDirectory)
if ($ResolvedOutput -eq $ResolvedEvidence) {
  throw "The runtime and evidence directories must be distinct."
}
if (Test-Path -LiteralPath $ResolvedOutput) {
  throw "The output directory already exists: ${ResolvedOutput}"
}
if (Test-Path -LiteralPath $ResolvedEvidence) {
  throw "The evidence directory already exists: ${ResolvedEvidence}"
}
if ($ResolvedOutput.StartsWith("${ResolvedEvidence}\", [StringComparison]::OrdinalIgnoreCase) -or
    $ResolvedEvidence.StartsWith("${ResolvedOutput}\", [StringComparison]::OrdinalIgnoreCase)) {
  throw "The runtime and evidence directories must not contain one another."
}

$WorkRoot = Join-Path ([IO.Path]::GetTempPath()) "fraia-calculix-windows-$([guid]::NewGuid())"
[IO.Directory]::CreateDirectory($WorkRoot) | Out-Null
$WorkRootUnix = $WorkRoot -replace "\\", "/"

try {
  $Downloads = Join-Path $WorkRoot "downloads"
  [IO.Directory]::CreateDirectory($Downloads) | Out-Null
  $ReviewedSourceRoot = if ($ReviewedSourceDirectory) {
    $CandidateRoot = [IO.Path]::GetFullPath($ReviewedSourceDirectory)
    if (-not (Test-Path -LiteralPath $CandidateRoot -PathType Container)) {
      throw "The reviewed source directory is unavailable: ${CandidateRoot}"
    }
    $CandidateRoot
  } else {
    $null
  }
  $Inputs = [ordered]@{
    "ccx_2.23.src.tar.bz2" = @($CalculixSourceUrl, $CalculixSourceSha256)
    "ccx_2.23.test.tar.bz2" = @($CalculixTestUrl, $CalculixTestSha256)
    "spooles.2.2.tgz" = @($SpoolesUrl, $SpoolesSha256)
    "ccx_2.23.SPOOLEScorrection.tar.bz2" = @($SpoolesCorrectionUrl, $SpoolesCorrectionSha256)
    "arpack-ng-3.9.1.tar.gz" = @($ArpackUrl, $ArpackSha256)
    "OpenBLAS-0.3.34.tar.gz" = @($OpenBlasUrl, $OpenBlasSha256)
    "winlibs.zip" = @($WinLibsArchiveUrl, $WinLibsArchiveSha256)
    "winlibs-source.tar.gz" = @($WinLibsSourceUrl, $WinLibsSourceSha256)
    "gcc-16.1.0.tar.xz" = @($GccSourceUrl, $GccSourceSha256)
    "mingw-w64-v14.0.0.tar.gz" = @($MingwSourceUrl, $MingwSourceSha256)
  }
  foreach ($Entry in $Inputs.GetEnumerator()) {
    $Destination = Join-Path $Downloads $Entry.Key
    $ReviewedInput = if ($ReviewedSourceRoot) {
      Join-Path $ReviewedSourceRoot $Entry.Key
    } else {
      $null
    }
    if ($ReviewedInput -and (Test-Path -LiteralPath $ReviewedInput -PathType Leaf)) {
      Copy-Item -LiteralPath $ReviewedInput -Destination $Destination
      Assert-Sha256 -Path $Destination -Expected $Entry.Value[1]
    } else {
      Invoke-ReviewedDownload `
        -Url $Entry.Value[0] `
        -Destination $Destination `
        -Sha256 $Entry.Value[1]
    }
  }

  $ToolchainRoot = Join-Path $WorkRoot "toolchain"
  Expand-Archive -LiteralPath (Join-Path $Downloads "winlibs.zip") -DestinationPath $ToolchainRoot
  $ToolBin = Join-Path $ToolchainRoot "mingw64\bin"
  $Tool = @{}
  foreach ($Name in @("ar", "cmake", "gcc", "gfortran", "mingw32-make", "ninja", "objdump", "ranlib", "strings")) {
    $Tool[$Name] = Join-Path $ToolBin "${Name}.exe"
    if (-not (Test-Path -LiteralPath $Tool[$Name] -PathType Leaf)) {
      throw "The pinned WinLibs archive does not contain ${Name}.exe."
    }
  }
  $env:Path = "${ToolBin};${env:Path}"
  $env:LC_ALL = "C"
  $env:TZ = "UTC"
  $env:SOURCE_DATE_EPOCH = $SourceDateEpoch

  $GccVersionOutput = (& $Tool["gcc"] --version | Select-Object -First 1)
  $GfortranVersionOutput = (& $Tool["gfortran"] --version | Select-Object -First 1)
  if ($GccVersionOutput -notmatch [regex]::Escape($GccVersion) -or
      $GfortranVersionOutput -notmatch [regex]::Escape($GccVersion)) {
    throw "The pinned WinLibs archive does not provide reviewed GCC and GFortran ${GccVersion}."
  }

  function Build-Once {
    param(
      [Parameter(Mandatory = $true)]
      [string]$PhysicalBuildRoot
    )

    [IO.Directory]::CreateDirectory($PhysicalBuildRoot) | Out-Null
    $SubstLog = Join-Path $PhysicalBuildRoot "subst.log"
    if (Test-Path -LiteralPath "${ControlledBuildDrive}\") {
      throw "The reviewed deterministic build drive ${ControlledBuildDrive} is already in use."
    }
    Invoke-LoggedCommand -Executable "subst.exe" -Arguments @(
      $ControlledBuildDrive, $PhysicalBuildRoot
    ) -LogPath $SubstLog
    try {
      $BuildRoot = "${ControlledBuildDrive}\"
      foreach ($Name in @("calculix", "spooles", "correction", "arpack", "openblas", "payload", "logs")) {
        [IO.Directory]::CreateDirectory((Join-Path $BuildRoot $Name)) | Out-Null
      }
    Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
      "-xjf", (Join-Path $Downloads "ccx_2.23.src.tar.bz2"), "-C", $BuildRoot
    ) -LogPath (Join-Path $BuildRoot "logs\extract.log")
    Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
      "-xzf", (Join-Path $Downloads "spooles.2.2.tgz"), "-C", (Join-Path $BuildRoot "spooles")
    ) -LogPath (Join-Path $BuildRoot "logs\extract.log")
    Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
      "-xjf", (Join-Path $Downloads "ccx_2.23.SPOOLEScorrection.tar.bz2"), "-C", (Join-Path $BuildRoot "correction")
    ) -LogPath (Join-Path $BuildRoot "logs\extract.log")
    Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
      "-xzf", (Join-Path $Downloads "arpack-ng-3.9.1.tar.gz"), "-C", (Join-Path $BuildRoot "arpack"), "--strip-components=1"
    ) -LogPath (Join-Path $BuildRoot "logs\extract.log")
    Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
      "-xzf", (Join-Path $Downloads "OpenBLAS-0.3.34.tar.gz"), "-C", (Join-Path $BuildRoot "openblas"), "--strip-components=1"
    ) -LogPath (Join-Path $BuildRoot "logs\extract.log")

    Copy-Item `
      -LiteralPath (Join-Path $BuildRoot "correction\CalculiX\ccx_2.23\SPOOLES.2.2\I2Ohash\src\util.c") `
      -Destination (Join-Path $BuildRoot "spooles\I2Ohash\src\util.c")
    $SpoolesTransform = Join-Path $BuildRoot "spooles\ETree\src\transform.c"
    $SpoolesTransformSource = [IO.File]::ReadAllText($SpoolesTransform)
    $LegacyInitializer = "IVinit(nfront, NULL)"
    if ([regex]::Matches($SpoolesTransformSource, [regex]::Escape($LegacyInitializer)).Count -ne 3) {
      throw "The reviewed SPOOLES integer-initializer correction no longer applies exactly three times."
    }
    [IO.File]::WriteAllText(
      $SpoolesTransform,
      $SpoolesTransformSource.Replace($LegacyInitializer, "IVinit(nfront, 0)"),
      $Utf8NoBom
    )
    $BuildRootUnix = $BuildRoot -replace "\\", "/"
    $NativePrefixMap = "-ffile-prefix-map=${BuildRoot}=/usr/src/fraia-runtime"
    $UnixPrefixMap = "-ffile-prefix-map=${BuildRootUnix}=/usr/src/fraia-runtime"
    $NativeMacroPrefixMap = "-fmacro-prefix-map=${BuildRoot}=/usr/src/fraia-runtime"
    $UnixMacroPrefixMap = "-fmacro-prefix-map=${BuildRootUnix}=/usr/src/fraia-runtime"
    $NativeDebugPrefixMap = "-fdebug-prefix-map=${BuildRoot}=/usr/src/fraia-runtime"
    $UnixDebugPrefixMap = "-fdebug-prefix-map=${BuildRootUnix}=/usr/src/fraia-runtime"
    $PrefixMapProbeSource = Join-Path $BuildRoot "prefix-map-probe.f90"
    $PrefixMapProbeObject = Join-Path $BuildRoot "prefix-map-probe.obj"
    [IO.File]::WriteAllLines(
      $PrefixMapProbeSource,
      @(
        "subroutine prefix_map_probe(name)",
        "  character(len=*), intent(in) :: name",
        "  print *, name",
        "end subroutine prefix_map_probe"
      ),
      $Utf8NoBom
    )
    Invoke-LoggedCommand -Executable $Tool["gfortran"] -Arguments @(
      "-c",
      "-O2",
      "-g0",
      $NativePrefixMap,
      $UnixPrefixMap,
      $NativeMacroPrefixMap,
      $UnixMacroPrefixMap,
      $NativeDebugPrefixMap,
      $UnixDebugPrefixMap,
      "-fcanon-prefix-map",
      $PrefixMapProbeSource,
      "-o",
      $PrefixMapProbeObject
    ) -LogPath (Join-Path $BuildRoot "logs\prefix-map-probe.log")
    [string[]]$PrefixMapProbeStrings = @(
      & $Tool["strings"] $PrefixMapProbeObject 2>&1 | ForEach-Object { "$_" }
    )
    if ($LASTEXITCODE -ne 0) {
      throw "strings failed while inspecting the gfortran prefix-map probe."
    }
    foreach ($ProbeSourceString in @(
      $PrefixMapProbeStrings |
        Where-Object { $_ -match "prefix-map-probe[.]f90" }
    )) {
      Write-Host "Pinned gfortran probe source string: ${ProbeSourceString}"
    }
    if ($PrefixMapProbeStrings |
      Select-String -SimpleMatch -Pattern @($PhysicalBuildRoot, ($PhysicalBuildRoot -replace "\\", "/")) -Quiet) {
      throw "The pinned gfortran retained the physical build path despite the controlled source root."
    }
    if (-not ($PrefixMapProbeStrings |
      Select-String -SimpleMatch -Pattern @(
        "/usr/src/fraia-runtime/prefix-map-probe.f90",
        "${ControlledBuildDrive}\prefix-map-probe.f90",
        "${ControlledBuildDrive}/prefix-map-probe.f90"
      ) -Quiet)) {
      throw "The pinned gfortran did not emit the reviewed controlled source path."
    }
    $SpoolesRootUnix = (Join-Path $BuildRoot "spooles") -replace "\\", "/"
    $SpoolesProject = Join-Path $BuildRoot "spooles-project"
    $SpoolesBuild = Join-Path $BuildRoot "spooles-build"
    [IO.Directory]::CreateDirectory($SpoolesProject) | Out-Null
    [IO.File]::WriteAllLines(
      (Join-Path $SpoolesProject "CMakeLists.txt"),
      @(
        "cmake_minimum_required(VERSION 3.24)",
        "project(FraiaSPOOLES C)",
        "file(GLOB_RECURSE SPOOLES_SOURCES CONFIGURE_DEPENDS `"${SpoolesRootUnix}/*/src/*.c`")",
        "list(FILTER SPOOLES_SOURCES EXCLUDE REGEX `"/MPI/`")",
        "list(SORT SPOOLES_SOURCES)",
        "add_library(spooles STATIC `${SPOOLES_SOURCES})",
        "target_include_directories(spooles PUBLIC `"${SpoolesRootUnix}`")",
        "target_compile_options(spooles PRIVATE -O2 -g0 -std=gnu17 [=[${NativePrefixMap}]=] ${UnixPrefixMap} [=[${NativeMacroPrefixMap}]=] ${UnixMacroPrefixMap} [=[${NativeDebugPrefixMap}]=] ${UnixDebugPrefixMap} -fcanon-prefix-map)",
        "set_target_properties(spooles PROPERTIES OUTPUT_NAME spooles PREFIX lib)"
      ),
      $Utf8NoBom
    )
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "-S", $SpoolesProject,
      "-B", $SpoolesBuild,
      "-G", "Ninja",
      "-DCMAKE_MAKE_PROGRAM=$($Tool["ninja"])",
      "-DCMAKE_C_COMPILER=$($Tool["gcc"])",
      "-DCMAKE_AR=$($Tool["ar"])",
      "-DCMAKE_RANLIB=$($Tool["ranlib"])",
      "-DCMAKE_BUILD_TYPE=Release"
    ) -LogPath (Join-Path $BuildRoot "logs\spooles-configure.log")
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "--build", $SpoolesBuild, "--parallel", "2"
    ) -LogPath (Join-Path $BuildRoot "logs\spooles-build.log")
    $SpoolesLibrary = Get-ChildItem -LiteralPath $SpoolesBuild -Recurse -File -Filter "libspooles.a" |
      Select-Object -First 1
    if (-not $SpoolesLibrary) {
      throw "The reviewed SPOOLES build did not produce a static library."
    }

    $OpenBlasBuild = Join-Path $BuildRoot "openblas-build"
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "-S", (Join-Path $BuildRoot "openblas"),
      "-B", $OpenBlasBuild,
      "-G", "Ninja",
      "-DCMAKE_MAKE_PROGRAM=$($Tool["ninja"])",
      "-DCMAKE_C_COMPILER=$($Tool["gcc"])",
      "-DCMAKE_Fortran_COMPILER=$($Tool["gfortran"])",
      "-DCMAKE_AR=$($Tool["ar"])",
      "-DCMAKE_RANLIB=$($Tool["ranlib"])",
      "-DCMAKE_BUILD_TYPE=Release",
      "-DBUILD_SHARED_LIBS=OFF",
      "-DNO_SHARED=ON",
      "-DDYNAMIC_ARCH=OFF",
      "-DTARGET=CORE2",
      "-DUSE_OPENMP=OFF",
      "-DUSE_THREAD=ON",
      "-DNUM_THREADS=64",
      "-DCMAKE_C_FLAGS_RELEASE=-O2 -g0 ${NativePrefixMap} ${UnixPrefixMap} ${NativeMacroPrefixMap} ${UnixMacroPrefixMap} ${NativeDebugPrefixMap} ${UnixDebugPrefixMap} -fcanon-prefix-map",
      "-DCMAKE_Fortran_FLAGS_RELEASE=-O2 -g0 -fallow-argument-mismatch ${NativePrefixMap} ${UnixPrefixMap} ${NativeMacroPrefixMap} ${UnixMacroPrefixMap} ${NativeDebugPrefixMap} ${UnixDebugPrefixMap} -fcanon-prefix-map"
    ) -LogPath (Join-Path $BuildRoot "logs\openblas-configure.log")
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "--build", $OpenBlasBuild, "--parallel", "2"
    ) -LogPath (Join-Path $BuildRoot "logs\openblas-build.log")
    $OpenBlasLibrary = Get-ChildItem -LiteralPath $OpenBlasBuild -Recurse -File -Filter "libopenblas*.a" |
      Sort-Object FullName |
      Select-Object -First 1
    if (-not $OpenBlasLibrary) {
      throw "The reviewed OpenBLAS build did not produce a static library."
    }

    $ArpackBuild = Join-Path $BuildRoot "arpack-build"
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "-S", (Join-Path $BuildRoot "arpack"),
      "-B", $ArpackBuild,
      "-G", "Ninja",
      "-DCMAKE_MAKE_PROGRAM=$($Tool["ninja"])",
      "-DCMAKE_C_COMPILER=$($Tool["gcc"])",
      "-DCMAKE_Fortran_COMPILER=$($Tool["gfortran"])",
      "-DCMAKE_AR=$($Tool["ar"])",
      "-DCMAKE_RANLIB=$($Tool["ranlib"])",
      "-DCMAKE_BUILD_TYPE=Release",
      "-DBUILD_SHARED_LIBS=OFF",
      "-DMPI=OFF",
      "-DICB=OFF",
      "-DEXAMPLES=OFF",
      "-DCMAKE_C_FLAGS_RELEASE=-O2 -g0 ${NativePrefixMap} ${UnixPrefixMap} ${NativeMacroPrefixMap} ${UnixMacroPrefixMap} ${NativeDebugPrefixMap} ${UnixDebugPrefixMap} -fcanon-prefix-map",
      "-DCMAKE_Fortran_FLAGS_RELEASE=-O2 -g0 -fallow-argument-mismatch ${NativePrefixMap} ${UnixPrefixMap} ${NativeMacroPrefixMap} ${UnixMacroPrefixMap} ${NativeDebugPrefixMap} ${UnixDebugPrefixMap} -fcanon-prefix-map",
      "-DBLAS_LIBRARIES=$($OpenBlasLibrary.FullName)",
      "-DLAPACK_LIBRARIES=$($OpenBlasLibrary.FullName)"
    ) -LogPath (Join-Path $BuildRoot "logs\arpack-configure.log")
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "--build", $ArpackBuild, "--parallel", "2"
    ) -LogPath (Join-Path $BuildRoot "logs\arpack-build.log")
    $ArpackLibrary = Get-ChildItem -LiteralPath $ArpackBuild -Recurse -File -Filter "libarpack*.a" |
      Sort-Object FullName |
      Select-Object -First 1
    if (-not $ArpackLibrary) {
      throw "The reviewed ARPACK-NG build did not produce a static library."
    }

    $CalculixSource = Join-Path $BuildRoot "CalculiX\ccx_2.23\src"
    $CalculixSourceUnix = $CalculixSource -replace "\\", "/"
    $GlobalWindowsFormatBlock = "#ifdef __WIN32`n_set_output_format(_TWO_DIGIT_EXPONENT);`n#endif`n"
    foreach ($SourceName in @("ccx_2.23.c", "ccx_2.23step.c")) {
      $SourcePath = Join-Path $CalculixSource $SourceName
      $SourceText = [IO.File]::ReadAllText($SourcePath).Replace("`r`n", "`n")
      if ([regex]::Matches($SourceText, [regex]::Escape($GlobalWindowsFormatBlock)).Count -ne 1) {
        throw "The reviewed MinGW output-format correction no longer applies exactly once to ${SourceName}."
      }
      [IO.File]::WriteAllText(
        $SourcePath,
        $SourceText.Replace($GlobalWindowsFormatBlock, ""),
        $Utf8NoBom
      )
    }
    $ReadNewMesh = Join-Path $CalculixSource "readnewmesh.c"
    $ReadNewMeshSource = [IO.File]::ReadAllText($ReadNewMesh).Replace("`r`n", "`n")
    $VoidReadNewMeshReturn = "  return NULL;`n`n}`n`n/* subroutine for multithreading of calcenergy */"
    if ([regex]::Matches($ReadNewMeshSource, [regex]::Escape($VoidReadNewMeshReturn)).Count -ne 1) {
      throw "The reviewed readnewmesh.c void-return correction no longer applies exactly once."
    }
    $ReadNewMeshSource = $ReadNewMeshSource.Replace(
      $VoidReadNewMeshReturn,
      "  return;`n`n}`n`n/* subroutine for multithreading of calcenergy */"
    )
    if ([regex]::Matches(
      $ReadNewMeshSource,
      "(?s)void \*genratiomt\(ITG \*i\)\{.*?return NULL;\s*\}"
    ).Count -ne 1) {
      throw "The reviewed genratiomt thread return is not preserved exactly once."
    }
    [IO.File]::WriteAllText(
      $ReadNewMesh,
      $ReadNewMeshSource,
      $Utf8NoBom
    )
    $MakefileInc = Get-Content -LiteralPath (Join-Path $CalculixSource "Makefile.inc") -Raw
    $ListedSources = @(
      [regex]::Matches($MakefileInc, "(?m)^\s*([A-Za-z0-9_.]+[.](?:c|f))\s*\\?\s*$") |
        ForEach-Object { $_.Groups[1].Value } |
        Sort-Object -Unique
    )
    if ($ListedSources.Count -lt 100) {
      throw "CalculiX Makefile.inc yielded an unexpectedly small source set."
    }
    $MissingListedSources = @(
      $ListedSources |
        Where-Object { -not (Test-Path -LiteralPath (Join-Path $CalculixSource $_) -PathType Leaf) }
    )
    if (($MissingListedSources -join "`n") -ne "mafillmm.c" -or
        -not (Test-Path -LiteralPath (Join-Path $CalculixSource "mafillmm.f") -PathType Leaf)) {
      throw "CalculiX Makefile.inc has an unexpected listed-but-absent source set: $($MissingListedSources -join ', ')."
    }
    $ListedSources = @($ListedSources | Where-Object { $_ -ne "mafillmm.c" })
    $CalculixSourceLines = @()
    foreach ($SourceName in $ListedSources) {
      $SourcePath = Join-Path $CalculixSource $SourceName
      $SourceUnix = $SourcePath -replace "\\", "/"
      $CalculixSourceLines += "  `"${SourceUnix}`""
    }
    $CalculixProject = Join-Path $BuildRoot "calculix-project"
    $CalculixBuild = Join-Path $BuildRoot "calculix-build"
    [IO.Directory]::CreateDirectory($CalculixProject) | Out-Null
    $CalculixCmake = @(
      "cmake_minimum_required(VERSION 3.24)",
      "project(FraiaCalculiX C Fortran)",
      "set(CCX_SOURCES"
    )
    $CalculixCmake += $CalculixSourceLines
    $CalculixCmake += @(
      ")",
      "add_library(ccxcore STATIC `${CCX_SOURCES})",
      "target_include_directories(ccxcore PRIVATE `"${SpoolesRootUnix}`")",
      "target_compile_definitions(ccxcore PRIVATE ARCH=Linux SPOOLES ARPACK MATRIXSTORAGE NETWORKOUT USE_MT=1)",
      "target_compile_options(ccxcore PRIVATE",
      "  [=[${NativePrefixMap}]=]",
      "  ${UnixPrefixMap}",
      "  [=[${NativeMacroPrefixMap}]=]",
      "  ${UnixMacroPrefixMap}",
      "  [=[${NativeDebugPrefixMap}]=]",
      "  ${UnixDebugPrefixMap}",
      "  -fcanon-prefix-map",
      "  `$<`$<COMPILE_LANGUAGE:C>:-O2;-g0;-std=gnu17>",
      "  `$<`$<COMPILE_LANGUAGE:Fortran>:-O2;-g0;-fallow-argument-mismatch;-fopenmp;-cpp>",
      ")",
      "add_executable(ccx `"${CalculixSourceUnix}/ccx_2.23.c`")",
      "target_include_directories(ccx PRIVATE `"${SpoolesRootUnix}`")",
      "target_compile_definitions(ccx PRIVATE ARCH=Linux SPOOLES ARPACK MATRIXSTORAGE NETWORKOUT USE_MT=1)",
      "target_compile_options(ccx PRIVATE -O2 -g0 -std=gnu17 [=[${NativePrefixMap}]=] ${UnixPrefixMap} [=[${NativeMacroPrefixMap}]=] ${UnixMacroPrefixMap} [=[${NativeDebugPrefixMap}]=] ${UnixDebugPrefixMap} -fcanon-prefix-map)",
      "set_property(TARGET ccx PROPERTY LINKER_LANGUAGE Fortran)",
      "target_link_options(ccx PRIVATE",
      "  -O2 -g0 -fopenmp -static -static-libgcc -static-libgfortran",
      "  `"SHELL:-Wl,--no-insert-timestamp`"",
      "  `"SHELL:-Wl,--major-os-version,${MinimumWindowsMajor}`"",
      "  `"SHELL:-Wl,--minor-os-version,${MinimumWindowsMinor}`"",
      "  `"SHELL:-Wl,--major-subsystem-version,${MinimumWindowsMajor}`"",
      "  `"SHELL:-Wl,--minor-subsystem-version,${MinimumWindowsMinor}`"",
      ")",
      "target_link_libraries(ccx PRIVATE",
      "  `"-Wl,--start-group`" ccxcore `"$($SpoolesLibrary.FullName -replace "\\", "/")`"",
      "  `"$($ArpackLibrary.FullName -replace "\\", "/")`" `"$($OpenBlasLibrary.FullName -replace "\\", "/")`"",
      "  gomp quadmath winpthread m `"-Wl,--end-group`"",
      ")",
      "set_target_properties(ccx PROPERTIES",
      "  OUTPUT_NAME ccx",
      "  SUFFIX `".exe`"",
      "  RUNTIME_OUTPUT_DIRECTORY `"$($BuildRoot -replace "\\", "/")/payload`"",
      ")"
    )
    [IO.File]::WriteAllLines((Join-Path $CalculixProject "CMakeLists.txt"), $CalculixCmake, $Utf8NoBom)
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "-S", $CalculixProject,
      "-B", $CalculixBuild,
      "-G", "Ninja",
      "-DCMAKE_MAKE_PROGRAM=$($Tool["ninja"])",
      "-DCMAKE_C_COMPILER=$($Tool["gcc"])",
      "-DCMAKE_Fortran_COMPILER=$($Tool["gfortran"])",
      "-DCMAKE_AR=$($Tool["ar"])",
      "-DCMAKE_RANLIB=$($Tool["ranlib"])",
      "-DCMAKE_BUILD_TYPE=Release"
    ) -LogPath (Join-Path $BuildRoot "logs\calculix-configure.log")
    Invoke-LoggedCommand -Executable $Tool["cmake"] -Arguments @(
      "--build", $CalculixBuild, "--parallel", "2"
    ) -LogPath (Join-Path $BuildRoot "logs\calculix-build.log")
    $ControlledCandidate = Join-Path $BuildRoot "payload\ccx.exe"
    if (-not (Test-Path -LiteralPath $ControlledCandidate -PathType Leaf)) {
      throw "The reviewed CalculiX source build did not produce ccx.exe."
    }
    return (Join-Path $PhysicalBuildRoot "payload\ccx.exe")
    } finally {
      [string[]]$SubstOutput = @(
        & subst.exe $ControlledBuildDrive "/d" 2>&1 | ForEach-Object { "$_" }
      )
      [IO.File]::AppendAllLines($SubstLog, $SubstOutput, $Utf8NoBom)
      if ($LASTEXITCODE -ne 0) {
        throw "Failed to remove reviewed deterministic build drive ${ControlledBuildDrive}."
      }
    }
  }

  $BuildOne = Join-Path $WorkRoot "build-one"
  $BuildTwo = Join-Path $WorkRoot "build-two"
  $CandidateOne = Build-Once -PhysicalBuildRoot $BuildOne
  $CandidateTwo = Build-Once -PhysicalBuildRoot $BuildTwo
  $CandidateOneSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $CandidateOne).Hash.ToLowerInvariant()
  $CandidateTwoSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $CandidateTwo).Hash.ToLowerInvariant()
  if ($CandidateOneSha -ne $CandidateTwoSha) {
    $ReproducibilityFailure = Join-Path $WorkRoot "reproducibility-failure"
    [IO.Directory]::CreateDirectory($ReproducibilityFailure) | Out-Null
    Copy-Item -LiteralPath $CandidateOne `
      -Destination (Join-Path $ReproducibilityFailure "ccx-build-one.exe")
    Copy-Item -LiteralPath $CandidateTwo `
      -Destination (Join-Path $ReproducibilityFailure "ccx-build-two.exe")
    foreach ($CandidateBuild in @(
      @("one", $CandidateOne),
      @("two", $CandidateTwo)
    )) {
      [string[]]$PeDump = @(& $Tool["objdump"] -x $CandidateBuild[1] 2>&1 | ForEach-Object { "$_" })
      if ($LASTEXITCODE -ne 0) {
        throw "objdump failed while recording the build-$($CandidateBuild[0]) reproducibility evidence."
      }
      [IO.File]::WriteAllLines(
        (Join-Path $ReproducibilityFailure "ccx-build-$($CandidateBuild[0])-objdump.txt"),
        $PeDump,
        $Utf8NoBom
      )
    }
    [IO.File]::WriteAllLines(
      (Join-Path $ReproducibilityFailure "REPRODUCIBILITY_FAILURE.txt"),
      @(
        "The two complete native Windows builds were not byte-identical.",
        "No runtime candidate was emitted.",
        "",
        "${CandidateOneSha}  ccx-build-one.exe",
        "${CandidateTwoSha}  ccx-build-two.exe",
        "",
        "build-one bytes: $((Get-Item -LiteralPath $CandidateOne).Length)",
        "build-two bytes: $((Get-Item -LiteralPath $CandidateTwo).Length)"
      ),
      $Utf8NoBom
    )
    Move-Item -LiteralPath $ReproducibilityFailure -Destination $ResolvedEvidence
    throw "The two native Windows builds are not byte-identical: ${CandidateOneSha}, ${CandidateTwoSha}."
  }

  foreach ($Candidate in @($CandidateOne, $CandidateTwo)) {
    $Header = [IO.File]::ReadAllBytes($Candidate)
    if ($Header.Length -lt 64 -or $Header[0] -ne 0x4d -or $Header[1] -ne 0x5a) {
      throw "The source-built CalculiX executable has no valid DOS header."
    }
    $PeOffset = [BitConverter]::ToUInt32($Header, 0x3c)
    if ($PeOffset + 6 -gt $Header.Length) {
      throw "The source-built CalculiX executable has an invalid PE offset."
    }
    $Machine = [BitConverter]::ToUInt16($Header, [int]$PeOffset + 4)
    if ($Machine -ne 0x8664) {
      throw "The source-built CalculiX executable is not PE x64."
    }
    $PeHeader = @(& $Tool["objdump"] -p $Candidate 2>&1)
    if ($LASTEXITCODE -ne 0) {
      throw "objdump failed while inspecting ${Candidate}."
    }
    if ((Get-PeHeaderValue -Header $PeHeader -Name "MajorOSystemVersion") -ne $MinimumWindowsMajor -or
        (Get-PeHeaderValue -Header $PeHeader -Name "MinorOSystemVersion") -ne $MinimumWindowsMinor -or
        (Get-PeHeaderValue -Header $PeHeader -Name "MajorSubsystemVersion") -ne $MinimumWindowsMajor -or
        (Get-PeHeaderValue -Header $PeHeader -Name "MinorSubsystemVersion") -ne $MinimumWindowsMinor) {
      throw "The source-built CalculiX executable does not declare the reviewed Windows 10 minimum."
    }
    $ObservedImports = @(Get-PeImports -Executable $Candidate -Objdump $Tool["objdump"])
    $UnexpectedImports = @($ObservedImports | Where-Object { $_ -notin $AllowedSystemImports })
    if ($UnexpectedImports.Count) {
      throw "The source-built CalculiX executable has unreviewed imports: $($UnexpectedImports -join ', ')."
    }
    if ($ObservedImports.Count -eq 0) {
      throw "The source-built CalculiX executable reported no native imports."
    }
    $EmbeddedStrings = @(& $Tool["strings"] $Candidate 2>&1)
    if ($LASTEXITCODE -ne 0) {
      throw "strings failed while inspecting ${Candidate}."
    }
    if ($EmbeddedStrings |
      Select-String -SimpleMatch -Pattern @($WorkRoot, $WorkRootUnix) -Quiet) {
      throw "The source-built CalculiX executable contains an absolute build path."
    }
  }

  $ImportsOne = @(Get-PeImports -Executable $CandidateOne -Objdump $Tool["objdump"])
  $ImportsTwo = @(Get-PeImports -Executable $CandidateTwo -Objdump $Tool["objdump"])
  if (($ImportsOne -join "`n") -ne ($ImportsTwo -join "`n")) {
    throw "The two native Windows builds have different dependency closures."
  }

  $RuntimeTestRoot = Join-Path $WorkRoot "runtime-test"
  $CaseRoot = Join-Path $RuntimeTestRoot "case"
  [IO.Directory]::CreateDirectory($CaseRoot) | Out-Null
  Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
    "-xjf", (Join-Path $Downloads "ccx_2.23.test.tar.bz2"), "-C", $CaseRoot,
    "./CalculiX/ccx_2.23/test/spring1.inp"
  ) -LogPath (Join-Path $RuntimeTestRoot "extract.log")
  $CaseDirectory = Join-Path $CaseRoot "CalculiX\ccx_2.23\test"
  $StandardOutput = Join-Path $RuntimeTestRoot "spring1.stdout"
  $StandardError = Join-Path $RuntimeTestRoot "spring1.stderr"
  $Process = Start-Process `
    -FilePath $CandidateOne `
    -ArgumentList "spring1" `
    -WorkingDirectory $CaseDirectory `
    -Wait `
    -PassThru `
    -NoNewWindow `
    -RedirectStandardOutput $StandardOutput `
    -RedirectStandardError $StandardError
  if ($Process.ExitCode -ne 0) {
    throw "The official spring1 solve failed with exit code $($Process.ExitCode)."
  }
  foreach ($Extension in @("dat", "frd", "sta")) {
    $Result = Join-Path $CaseDirectory "spring1.${Extension}"
    if (-not (Test-Path -LiteralPath $Result -PathType Leaf) -or (Get-Item $Result).Length -eq 0) {
      throw "The official spring1 solve did not produce ${Result}."
    }
  }
  if (-not (Select-String -LiteralPath $StandardOutput -SimpleMatch "Job finished" -Quiet)) {
    throw "The official spring1 solve did not report completion."
  }

  $RuntimeStaging = Join-Path $WorkRoot "runtime"
  $EvidenceStaging = Join-Path $WorkRoot "evidence"
  [IO.Directory]::CreateDirectory((Join-Path $RuntimeStaging "licenses")) | Out-Null
  Copy-Item -LiteralPath $CandidateOne -Destination (Join-Path $RuntimeStaging "ccx.exe")
  Copy-Item -LiteralPath (Join-Path $BuildOne "arpack\COPYING") `
    -Destination (Join-Path $RuntimeStaging "licenses\ARPACK-BSD-3-Clause.txt")
  Copy-Item -LiteralPath (Join-Path $BuildOne "openblas\LICENSE") `
    -Destination (Join-Path $RuntimeStaging "licenses\OpenBLAS-BSD-3-Clause.txt")

  $LicenseExtractRoot = Join-Path $WorkRoot "license-source"
  [IO.Directory]::CreateDirectory($LicenseExtractRoot) | Out-Null
  Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
    "-xJf", (Join-Path $Downloads "gcc-16.1.0.tar.xz"), "-C", $LicenseExtractRoot,
    "gcc-16.1.0/COPYING",
    "gcc-16.1.0/COPYING3",
    "gcc-16.1.0/COPYING.RUNTIME"
  ) -LogPath (Join-Path $LicenseExtractRoot "extract.log")
  Assert-Sha256 -Path (Join-Path $LicenseExtractRoot "gcc-16.1.0\COPYING") `
    -Expected $Gpl2Sha256
  Assert-Sha256 -Path (Join-Path $LicenseExtractRoot "gcc-16.1.0\COPYING3") `
    -Expected $Gpl3Sha256
  Copy-Item -LiteralPath (Join-Path $LicenseExtractRoot "gcc-16.1.0\COPYING") `
    -Destination (Join-Path $RuntimeStaging "licenses\GPL-2.0.txt")
  Copy-Item -LiteralPath (Join-Path $LicenseExtractRoot "gcc-16.1.0\COPYING3") `
    -Destination (Join-Path $RuntimeStaging "licenses\GPL-3.0.txt")
  Invoke-LoggedCommand -Executable "tar.exe" -Arguments @(
    "-xzf", (Join-Path $Downloads "mingw-w64-v14.0.0.tar.gz"), "-C", $LicenseExtractRoot,
    "mingw-w64-14.0.0/mingw-w64-libraries/winpthreads/COPYING"
  ) -LogPath (Join-Path $LicenseExtractRoot "extract.log")
  Copy-Item -LiteralPath (Join-Path $LicenseExtractRoot "gcc-16.1.0\COPYING.RUNTIME") `
    -Destination (Join-Path $RuntimeStaging "licenses\GCC-Runtime-Library-Exception-3.1.txt")
  Copy-Item -LiteralPath (Join-Path $LicenseExtractRoot "mingw-w64-14.0.0\mingw-w64-libraries\winpthreads\COPYING") `
    -Destination (Join-Path $RuntimeStaging "licenses\winpthreads-MIT.txt")

  [IO.File]::WriteAllLines(
    (Join-Path $RuntimeStaging "licenses\CALCULIX-LICENSE-NOTICE.txt"),
    @(
      "CalculiX ${CalculixVersion}",
      "Copyright (C) 1998-2025 Guido Dhondt and contributors.",
      "",
      "The CalculiX source headers license the program under version 2 of the",
      "GNU General Public License. The full text is in GPL-2.0.txt.",
      "",
      "Source: ${CalculixSourceUrl}",
      "SHA-256: ${CalculixSourceSha256}"
    ),
    $Utf8NoBom
  )
  [IO.File]::WriteAllLines(
    (Join-Path $RuntimeStaging "licenses\SPOOLES-NOTICE.txt"),
    @(
      "SPOOLES 2.2",
      "",
      "The SPOOLES 2.2 reference manual and release page state that this",
      "release of the package is totally within the public domain.",
      "",
      "The source also contains Harwell-Boeing File I/O in C, version 1.0,",
      "from the National Institute of Standards and Technology, with this notice:",
      "",
      "Permission to use, copy, modify, and distribute this software and its",
      "documentation for any purpose and without fee is hereby granted provided",
      "that the above copyright notice appear in all copies and that both the",
      "copyright notice and this permission notice appear in supporting documentation.",
      "",
      "Neither the Author nor the Institution (National Institute of Standards",
      "and Technology) make any representations about the suitability of this",
      "software for any purpose. This software is provided `"as is`" without",
      "expressed or implied warranty.",
      "",
      "Source: ${SpoolesUrl}",
      "SHA-256: ${SpoolesSha256}"
    ),
    $Utf8NoBom
  )
  [IO.File]::WriteAllLines(
    (Join-Path $RuntimeStaging "THIRD_PARTY_NOTICES.txt"),
    @(
      "Fraia CalculiX ${CalculixVersion} native runtime notices",
      "",
      "CalculiX: GPL-2.0-only. See licenses/CALCULIX-LICENSE-NOTICE.txt and licenses/GPL-2.0.txt.",
      "SPOOLES 2.2: public domain with included NIST notice. See licenses/SPOOLES-NOTICE.txt.",
      "ARPACK-NG: BSD-3-Clause. See licenses/ARPACK-BSD-3-Clause.txt.",
      "OpenBLAS: BSD-3-Clause. See licenses/OpenBLAS-BSD-3-Clause.txt.",
      "Statically linked GCC runtime libraries: GPL-3.0-or-later WITH GCC-exception-3.1.",
      "See licenses/GPL-3.0.txt and licenses/GCC-Runtime-Library-Exception-3.1.txt.",
      "Statically linked winpthreads: MIT. See licenses/winpthreads-MIT.txt.",
      "",
      "Corresponding-source publication is recorded separately in runtime-manifest.json."
    ),
    $Utf8NoBom
  )

  $ScriptSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $PSCommandPath).Hash.ToLowerInvariant()
  $ToolHashes = [ordered]@{}
  foreach ($Name in @("ar", "cmake", "gcc", "gfortran", "ninja", "objdump", "ranlib", "strings")) {
    $ToolHashes[$Name] = (Get-FileHash -Algorithm SHA256 -LiteralPath $Tool[$Name]).Hash.ToLowerInvariant()
  }
  $Recipe = @(
    "# Fraia CalculiX ${CalculixVersion} win32-x64 build recipe",
    "",
    "Build revision: ``fraia-calculix-windows-v16``",
    "",
    "- Native host: ``$([Environment]::OSVersion.VersionString)``",
    "- Minimum Windows contract: ``Windows ${MinimumWindowsMajor}.${MinimumWindowsMinor}``",
    "- WinLibs tag: ``${WinLibsTag}``",
    "- WinLibs source commit: ``${WinLibsCommit}``",
    "- WinLibs x64 UCRT archive SHA-256: ``${WinLibsArchiveSha256}``",
    "- WinLibs source archive SHA-256: ``${WinLibsSourceSha256}``",
    "- CalculiX source SHA-256: ``${CalculixSourceSha256}``",
    "- CalculiX tests SHA-256: ``${CalculixTestSha256}``",
    "- SPOOLES source SHA-256: ``${SpoolesSha256}``",
    "- SPOOLES correction SHA-256: ``${SpoolesCorrectionSha256}``",
    "- ARPACK-NG source SHA-256: ``${ArpackSha256}``",
    "- OpenBLAS source SHA-256: ``${OpenBlasSha256}``",
    "- GCC source SHA-256: ``${GccSourceSha256}``",
    "- MinGW-w64 source SHA-256: ``${MingwSourceSha256}``",
    "- GPL-2.0 text from GCC source ``COPYING`` SHA-256: ``${Gpl2Sha256}``",
    "- GPL-3.0 text from GCC source ``COPYING3`` SHA-256: ``${Gpl3Sha256}``",
    "- Build script SHA-256: ``${ScriptSha256}``",
    "- SOURCE_DATE_EPOCH: ``${SourceDateEpoch}``",
    "- Controlled compiler source root: ``${ControlledBuildDrive}\`` (temporary ``subst`` mapping)",
    "",
    "Reproduce on a native Windows x64 host:",
    "",
    "``````powershell",
    ".\build-calculix-windows-runtime.ps1 -OutputDirectory C:\new\runtime -EvidenceDirectory C:\new\evidence",
    "``````"
  )
  [IO.File]::WriteAllLines((Join-Path $RuntimeStaging "BUILD_RECIPE.md"), $Recipe, $Utf8NoBom)

  $RuntimeChecksumLines = @(
    Get-ChildItem -LiteralPath $RuntimeStaging -Recurse -File |
      Where-Object { $_.Name -ne "SHA256SUMS" } |
      Sort-Object FullName |
      ForEach-Object {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
        $Relative = Get-RelativeUnixPath -Root $RuntimeStaging -Path $_.FullName
        "${Hash}  ./${Relative}"
      }
  )
  [IO.File]::WriteAllLines((Join-Path $RuntimeStaging "SHA256SUMS"), $RuntimeChecksumLines, $Utf8NoBom)

  foreach ($Directory in @(
    "native", "reproducibility", "solver", "source-inputs", "toolchain"
  )) {
    [IO.Directory]::CreateDirectory((Join-Path $EvidenceStaging $Directory)) | Out-Null
  }
  Copy-Item -LiteralPath $PSCommandPath `
    -Destination (Join-Path $EvidenceStaging "source-inputs\build-calculix-windows-runtime.ps1")
  foreach ($Entry in $Inputs.GetEnumerator()) {
    Copy-Item -LiteralPath (Join-Path $Downloads $Entry.Key) `
      -Destination (Join-Path $EvidenceStaging "source-inputs\$($Entry.Key)")
  }
  $SourceChecksums = @(
    Get-ChildItem -LiteralPath (Join-Path $EvidenceStaging "source-inputs") -File |
      Sort-Object Name |
      ForEach-Object {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
        "${Hash}  $($_.Name)"
      }
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "source-inputs\SHA256SUMS"),
    $SourceChecksums,
    $Utf8NoBom
  )

  foreach ($Name in @("inp", "dat", "frd", "sta")) {
    Copy-Item -LiteralPath (Join-Path $CaseDirectory "spring1.${Name}") `
      -Destination (Join-Path $EvidenceStaging "solver\spring1.${Name}")
  }
  Copy-Item -LiteralPath $StandardOutput -Destination (Join-Path $EvidenceStaging "solver\spring1.stdout")
  Copy-Item -LiteralPath $StandardError -Destination (Join-Path $EvidenceStaging "solver\spring1.stderr")
  $SolverChecksums = @(
    Get-ChildItem -LiteralPath (Join-Path $EvidenceStaging "solver") -File |
      Sort-Object Name |
      ForEach-Object {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
        "${Hash}  $($_.Name)"
      }
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "solver\SHA256SUMS"),
    $SolverChecksums,
    $Utf8NoBom
  )

  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "reproducibility\build-one-SHA256SUMS"),
    @("${CandidateOneSha}  ccx.exe"),
    $Utf8NoBom
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "reproducibility\build-two-SHA256SUMS"),
    @("${CandidateTwoSha}  ccx.exe"),
    $Utf8NoBom
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "reproducibility\RESULT.txt"),
    @("The two independently source-built native payloads are byte-identical."),
    $Utf8NoBom
  )

  $NativeHeader = @(& $Tool["objdump"] -p $CandidateOne 2>&1 | ForEach-Object { "$_" })
  [IO.File]::WriteAllLines((Join-Path $EvidenceStaging "native\ccx.pe-header.txt"), $NativeHeader, $Utf8NoBom)
  [IO.File]::WriteAllLines((Join-Path $EvidenceStaging "native\ccx.imports.txt"), $ImportsOne, $Utf8NoBom)
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "native\ccx.contract.txt"),
    @(
      "PE machine: 0x8664 (x86-64)",
      "Minimum Windows: ${MinimumWindowsMajor}.${MinimumWindowsMinor}",
      "Bundled native libraries: none; compiler, OpenMP, winpthreads, SPOOLES, ARPACK-NG, and OpenBLAS are statically linked.",
      "All observed dynamic imports are in the reviewed Windows system allowlist.",
      "Absolute build-path scan: pass"
    ),
    $Utf8NoBom
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "toolchain\ENVIRONMENT.txt"),
    @(
      "Native host: $([Environment]::OSVersion.VersionString)",
      "OS architecture: $([Runtime.InteropServices.RuntimeInformation]::OSArchitecture)",
      "PowerShell: $($PSVersionTable.PSVersion)",
      "WinLibs tag: ${WinLibsTag}",
      "WinLibs source commit: ${WinLibsCommit}",
      "Minimum Windows: ${MinimumWindowsMajor}.${MinimumWindowsMinor}",
      "SOURCE_DATE_EPOCH: ${SourceDateEpoch}",
      "gcc: ${GccVersionOutput}",
      "gfortran: ${GfortranVersionOutput}"
    ),
    $Utf8NoBom
  )
  $ToolChecksumLines = @(
    $ToolHashes.GetEnumerator() |
      ForEach-Object { "$($_.Value)  $($_.Key).exe" }
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "toolchain\SHA256SUMS"),
    $ToolChecksumLines,
    $Utf8NoBom
  )
  Copy-Item -LiteralPath (Join-Path $RuntimeStaging "BUILD_RECIPE.md") `
    -Destination (Join-Path $EvidenceStaging "BUILD_RECIPE.md")
  Copy-Item -LiteralPath (Join-Path $RuntimeStaging "SHA256SUMS") `
    -Destination (Join-Path $EvidenceStaging "RUNTIME_SHA256SUMS")
  Copy-Item -LiteralPath (Join-Path $RuntimeStaging "THIRD_PARTY_NOTICES.txt") `
    -Destination (Join-Path $EvidenceStaging "THIRD_PARTY_NOTICES.txt")
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "README.md"),
    @(
      "# Fraia CalculiX ${CalculixVersion} win32-x64 review evidence",
      "",
      "This directory is review evidence, not a promotable runtime.",
      "Promote only the separately emitted runtime after this evidence, corresponding-source publication, and runtime-manifest review all pass."
    ),
    $Utf8NoBom
  )
  $EvidenceChecksums = @(
    Get-ChildItem -LiteralPath $EvidenceStaging -Recurse -File |
      Where-Object { $_.Name -ne "EVIDENCE_SHA256SUMS" } |
      Sort-Object FullName |
      ForEach-Object {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
        $Relative = Get-RelativeUnixPath -Root $EvidenceStaging -Path $_.FullName
        "${Hash}  ./${Relative}"
      }
  )
  [IO.File]::WriteAllLines(
    (Join-Path $EvidenceStaging "EVIDENCE_SHA256SUMS"),
    $EvidenceChecksums,
    $Utf8NoBom
  )

  Move-Item -LiteralPath $EvidenceStaging -Destination $ResolvedEvidence
  Move-Item -LiteralPath $RuntimeStaging -Destination $ResolvedOutput
  Write-Host "Built and independently reproduced win32-x64 CalculiX ${CalculixVersion} runtime at ${ResolvedOutput}"
  Write-Host "Wrote independently reviewable win32-x64 evidence at ${ResolvedEvidence}"
}
finally {
  if (Test-Path -LiteralPath $WorkRoot) {
    $ExpectedPrefix = Join-Path ([IO.Path]::GetTempPath()) "fraia-calculix-windows-"
    if ($WorkRoot.StartsWith($ExpectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
      Remove-Item -LiteralPath $WorkRoot -Recurse -Force
    } else {
      Write-Warning "Refusing to remove unexpected work directory: ${WorkRoot}"
    }
  }
}
