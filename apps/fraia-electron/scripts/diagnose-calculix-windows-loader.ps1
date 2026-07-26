[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$FailureEvidenceDirectory,
  [Parameter(Mandatory = $true)]
  [string]$OutputDirectory
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$Utf8NoBom = [Text.UTF8Encoding]::new($false)

function Write-Lines {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path,
    [Parameter(Mandatory = $true)]
    [AllowEmptyCollection()]
    [string[]]$Lines
  )

  [IO.File]::WriteAllLines($Path, $Lines, $Utf8NoBom)
}

function Get-PeMachine {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Path
  )

  $Stream = [IO.File]::OpenRead($Path)
  try {
    $Reader = [IO.BinaryReader]::new($Stream)
    if ($Reader.ReadUInt16() -ne 0x5a4d) {
      throw "The resolved module is not a PE file: ${Path}"
    }
    $Stream.Position = 0x3c
    $PeOffset = $Reader.ReadUInt32()
    $Stream.Position = $PeOffset
    if ($Reader.ReadUInt32() -ne 0x00004550) {
      throw "The resolved module has no PE signature: ${Path}"
    }
    return ('0x{0:x4}' -f $Reader.ReadUInt16())
  } finally {
    $Stream.Dispose()
  }
}

if (-not $IsWindows -or [Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne "X64") {
  throw "CalculiX win32-x64 loader diagnostics require a native Windows x64 host."
}

$ResolvedEvidence = [IO.Path]::GetFullPath($FailureEvidenceDirectory)
$ResolvedOutput = [IO.Path]::GetFullPath($OutputDirectory)
if (-not (Test-Path -LiteralPath $ResolvedEvidence -PathType Container)) {
  throw "The failure-evidence directory is unavailable: ${ResolvedEvidence}"
}
if (Test-Path -LiteralPath $ResolvedOutput) {
  throw "The diagnostic output directory already exists: ${ResolvedOutput}"
}

$Candidate = Join-Path $ResolvedEvidence "ccx-build-one.exe"
$ChecksumFile = Join-Path $ResolvedEvidence "ccx-build-one.sha256"
$ImportDump = Join-Path $ResolvedEvidence "ccx-build-one-objdump.txt"
foreach ($RequiredPath in @($Candidate, $ChecksumFile, $ImportDump)) {
  if (-not (Test-Path -LiteralPath $RequiredPath -PathType Leaf)) {
    throw "Required retained failure evidence is unavailable: ${RequiredPath}"
  }
}

$ChecksumLine = Get-Content -LiteralPath $ChecksumFile -Raw
if ($ChecksumLine -notmatch "^([a-f0-9]{64})\s+ccx-build-one\.exe\s*$") {
  throw "The retained candidate checksum file is invalid."
}
$ExpectedSha256 = $Matches[1]
$ActualSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $Candidate).Hash.ToLowerInvariant()
if ($ActualSha256 -ne $ExpectedSha256) {
  throw "The retained candidate SHA-256 does not match its failure evidence."
}

$Staging = Join-Path ([IO.Path]::GetTempPath()) "fraia-windows-loader-diagnostic-$([guid]::NewGuid())"
[IO.Directory]::CreateDirectory($Staging) | Out-Null

try {
  Copy-Item -LiteralPath $ChecksumFile -Destination $Staging
  Copy-Item -LiteralPath $ImportDump -Destination $Staging
  $FailureRecord = Join-Path $ResolvedEvidence "FAILURE.txt"
  if (Test-Path -LiteralPath $FailureRecord -PathType Leaf) {
    Copy-Item -LiteralPath $FailureRecord -Destination $Staging
  }

  Write-Lines -Path (Join-Path $Staging "environment.txt") -Lines @(
    "OS: $([Environment]::OSVersion.VersionString)",
    "OS architecture: $([Runtime.InteropServices.RuntimeInformation]::OSArchitecture)",
    "Process architecture: $([Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)",
    "PowerShell: $($PSVersionTable.PSVersion)",
    "Candidate SHA-256: ${ActualSha256}",
    "Candidate bytes: $((Get-Item -LiteralPath $Candidate).Length)"
  )

  $CurrentDll = $null
  $Imports = [Collections.Generic.List[object]]::new()
  foreach ($Line in Get-Content -LiteralPath $ImportDump) {
    if ($Line -match "^\s*DLL Name:\s*([A-Za-z0-9_.+-]+\.dll)\s*$") {
      $CurrentDll = $Matches[1]
      continue
    }
    if ($CurrentDll -and $Line -match "^\s*[0-9a-fA-F]+\s+<none>\s+[0-9a-fA-F]+\s+(\S+)\s*$") {
      $Imports.Add([pscustomobject]@{
        dll = $CurrentDll
        symbol = $Matches[1]
      })
    }
  }
  if ($Imports.Count -eq 0) {
    throw "No named imports were parsed from the retained PE evidence."
  }

  Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class FraiaNativeLoaderDiagnostics {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
    public static extern IntPtr LoadLibraryExW(string fileName, IntPtr file, uint flags);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string procedureName);

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
    public static extern uint GetModuleFileNameW(IntPtr module, StringBuilder fileName, int size);

    [DllImport("kernel32.dll", ExactSpelling = true, SetLastError = true)]
    public static extern bool FreeLibrary(IntPtr module);
}
'@

  $Resolution = [Collections.Generic.List[object]]::new()
  foreach ($Group in $Imports | Group-Object dll | Sort-Object Name) {
    $DllName = $Group.Name
    $Handle = [FraiaNativeLoaderDiagnostics]::LoadLibraryExW($DllName, [IntPtr]::Zero, 0x00000800)
    $LoadError = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
    if ($Handle -eq [IntPtr]::Zero) {
      $Resolution.Add([pscustomobject]@{
        dll = $DllName
        loaded = $false
        loadError = $LoadError
        resolvedPath = $null
        machine = $null
        symbols = @()
      })
      continue
    }
    try {
      $ModulePathBuffer = [Text.StringBuilder]::new(32768)
      $ModulePathLength = [FraiaNativeLoaderDiagnostics]::GetModuleFileNameW(
        $Handle,
        $ModulePathBuffer,
        $ModulePathBuffer.Capacity
      )
      $ResolvedModulePath = if ($ModulePathLength) { $ModulePathBuffer.ToString() } else { $null }
      $Symbols = @(
        foreach ($Import in $Group.Group | Sort-Object symbol) {
          $Address = [FraiaNativeLoaderDiagnostics]::GetProcAddress($Handle, $Import.symbol)
          [pscustomobject]@{
            name = $Import.symbol
            resolved = $Address -ne [IntPtr]::Zero
            error = if ($Address -eq [IntPtr]::Zero) {
              [Runtime.InteropServices.Marshal]::GetLastWin32Error()
            } else {
              0
            }
          }
        }
      )
      $Resolution.Add([pscustomobject]@{
        dll = $DllName
        loaded = $true
        loadError = 0
        resolvedPath = $ResolvedModulePath
        machine = if ($ResolvedModulePath) { Get-PeMachine -Path $ResolvedModulePath } else { $null }
        symbols = $Symbols
      })
    } finally {
      [FraiaNativeLoaderDiagnostics]::FreeLibrary($Handle) | Out-Null
    }
  }

  [IO.File]::WriteAllText(
    (Join-Path $Staging "import-resolution.json"),
    "$($Resolution | ConvertTo-Json -Depth 6)`n",
    $Utf8NoBom
  )

  $MissingResolution = @(
    $Resolution | Where-Object {
      -not $_.loaded -or @($_.symbols | Where-Object { -not $_.resolved }).Count -gt 0
    }
  )
  Write-Lines -Path (Join-Path $Staging "import-resolution-summary.txt") -Lines @(
    "Imported DLLs: $($Resolution.Count)",
    "Unresolved DLLs or symbols: $($MissingResolution.Count)",
    "All resolved module PE machines must be 0x8664."
  )

  $LlvmBin = Join-Path $env:ProgramFiles "LLVM\bin"
  $LlvmReadObj = Join-Path $LlvmBin "llvm-readobj.exe"
  $LlvmObjdump = Join-Path $LlvmBin "llvm-objdump.exe"
  foreach ($Tool in @($LlvmReadObj, $LlvmObjdump)) {
    if (-not (Test-Path -LiteralPath $Tool -PathType Leaf)) {
      throw "The hosted Windows image is missing the reviewed LLVM inspection tool: ${Tool}"
    }
  }
  [string[]]$LlvmHeaders = @(& $LlvmReadObj --file-headers --sections --coff-imports $Candidate 2>&1)
  if ($LASTEXITCODE -ne 0) {
    throw "llvm-readobj failed while inspecting the retained candidate."
  }
  Write-Lines -Path (Join-Path $Staging "llvm-readobj.txt") -Lines $LlvmHeaders
  [string[]]$LlvmDisassembly = @(
    & $LlvmObjdump `
      --disassemble `
      --no-show-raw-insn `
      --symbolize-operands `
      --start-address=0x140001000 `
      --stop-address=0x140002000 `
      $Candidate 2>&1
  )
  if ($LASTEXITCODE -ne 0) {
    throw "llvm-objdump failed while inspecting the retained candidate entry point."
  }
  Write-Lines -Path (Join-Path $Staging "llvm-entry-disassembly.txt") -Lines $LlvmDisassembly

  $CaseDirectory = Join-Path $ResolvedEvidence "runtime-test\case\CalculiX\ccx_2.23\test"
  if (-not (Test-Path -LiteralPath (Join-Path $CaseDirectory "spring1.inp") -PathType Leaf)) {
    throw "The retained official spring1 fixture is unavailable."
  }
  $DirectStdout = Join-Path $Staging "direct-spring1.stdout"
  $DirectStderr = Join-Path $Staging "direct-spring1.stderr"
  $DirectProcess = Start-Process `
    -FilePath $Candidate `
    -ArgumentList "spring1" `
    -WorkingDirectory $CaseDirectory `
    -Wait `
    -PassThru `
    -NoNewWindow `
    -RedirectStandardOutput $DirectStdout `
    -RedirectStandardError $DirectStderr
  $UnsignedExitCode = [BitConverter]::ToUInt32(
    [BitConverter]::GetBytes([int32]$DirectProcess.ExitCode),
    0
  )
  Write-Lines -Path (Join-Path $Staging "direct-process.txt") -Lines @(
    "Signed exit code: $($DirectProcess.ExitCode)",
    "Unsigned exit code: $('0x{0:x8}' -f $UnsignedExitCode)",
    "Standard output bytes: $((Get-Item -LiteralPath $DirectStdout).Length)",
    "Standard error bytes: $((Get-Item -LiteralPath $DirectStderr).Length)"
  )

  $DebuggerRoots = @(
    (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\Debuggers\x64"),
    (Join-Path $env:ProgramFiles "Windows Kits\10\Debuggers\x64")
  )
  $Cdb = $DebuggerRoots |
    ForEach-Object { Join-Path $_ "cdb.exe" } |
    Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
    Select-Object -First 1
  $Gflags = $DebuggerRoots |
    ForEach-Object { Join-Path $_ "gflags.exe" } |
    Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
    Select-Object -First 1
  Write-Lines -Path (Join-Path $Staging "debugger-availability.txt") -Lines @(
    "cdb available: $([bool]$Cdb)",
    "gflags available: $([bool]$Gflags)"
  )

  if ($Cdb) {
    $CandidateName = [IO.Path]::GetFileName($Candidate)
    if ($Gflags) {
      & $Gflags /i $CandidateName +sls | Out-File `
        -LiteralPath (Join-Path $Staging "gflags-enable.txt") `
        -Encoding utf8
    }
    try {
      Push-Location $CaseDirectory
      try {
        & $Cdb `
          -logo (Join-Path $Staging "cdb-loader.log") `
          -c "g; .lastevent; k; lm; q" `
          $Candidate `
          "spring1"
        Write-Lines -Path (Join-Path $Staging "cdb-process.txt") -Lines @(
          "cdb exit code: ${LASTEXITCODE}"
        )
      } finally {
        Pop-Location
      }
    } finally {
      if ($Gflags) {
        & $Gflags /i $CandidateName -sls | Out-File `
          -LiteralPath (Join-Path $Staging "gflags-disable.txt") `
          -Encoding utf8
      }
    }
  }

  $Since = (Get-Date).AddMinutes(-10)
  [string[]]$EventEvidence = @(
    foreach ($LogName in @("Application", "System")) {
      try {
        Get-WinEvent -FilterHashtable @{ LogName = $LogName; StartTime = $Since } -ErrorAction Stop |
          Where-Object {
            $_.Message -match "ccx-build-one|0xc000007b|api-ms-win-crt|side-by-side"
          } |
          ForEach-Object {
            "${LogName} $($_.TimeCreated.ToUniversalTime().ToString('o')) $($_.Id) $($_.ProviderName)"
            $_.Message
            ""
          }
      } catch {
        "${LogName} event query failed: $($_.Exception.Message)"
      }
    }
  )
  Write-Lines -Path (Join-Path $Staging "windows-events.txt") -Lines $EventEvidence

  Move-Item -LiteralPath $Staging -Destination $ResolvedOutput
  Write-Host "Wrote native Windows loader diagnostics for ${ActualSha256} to ${ResolvedOutput}"
} catch {
  $DiagnosticFailure = $_
  if (-not (Test-Path -LiteralPath $ResolvedOutput)) {
    Write-Lines -Path (Join-Path $Staging "DIAGNOSTIC_FAILURE.txt") -Lines @(
      "The native loader diagnostic failed before completing.",
      "",
      $DiagnosticFailure.Exception.Message
    )
    Move-Item -LiteralPath $Staging -Destination $ResolvedOutput
  }
  throw
} finally {
  if (Test-Path -LiteralPath $Staging) {
    $ExpectedPrefix = Join-Path ([IO.Path]::GetTempPath()) "fraia-windows-loader-diagnostic-"
    if (-not $Staging.StartsWith($ExpectedPrefix, [StringComparison]::OrdinalIgnoreCase)) {
      throw "Refusing to clean an unexpected diagnostic path: ${Staging}"
    }
    Remove-Item -LiteralPath $Staging -Recurse -Force
  }
}
