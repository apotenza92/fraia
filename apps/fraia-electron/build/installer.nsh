Var /GLOBAL FraiaInstallerPid

!macro customCheckAppRunning
  System::Call 'Kernel32::GetCurrentProcessId() i.R0'
  StrCpy $FraiaInstallerPid $R0
  ${if} $FraiaInstallerPid == 0
    DetailPrint `Could not identify the "${PRODUCT_NAME}" installer process.`
    SetErrorLevel 2
    Quit
  ${endIf}
  System::Call 'Kernel32::SetEnvironmentVariable(t "FRAIA_NSIS_INSTALL_DIR", t "$INSTDIR") i.R0'
  ${if} $R0 == 0
    DetailPrint `Could not prepare the "${PRODUCT_NAME}" process check.`
    SetErrorLevel 2
    Quit
  ${endIf}
  nsExec::Exec `"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& { $$installDirectory = [Environment]::GetEnvironmentVariable('FRAIA_NSIS_INSTALL_DIR'); if ([string]::IsNullOrWhiteSpace($$installDirectory)) { exit 1 }; $$installPrefix = [IO.Path]::GetFullPath($$installDirectory).TrimEnd('\') + '\'; $$deadline = (Get-Date).AddSeconds(15); do { $$processes = @(Get-CimInstance -ClassName Win32_Process | Where-Object { $$_.ExecutablePath -and $$_.ExecutablePath.StartsWith($$installPrefix, [System.StringComparison]::OrdinalIgnoreCase) -and $$_.ProcessId -ne $FraiaInstallerPid }); if ($$processes.Count -eq 0) { exit 0 }; Start-Sleep -Milliseconds 250 } while ((Get-Date) -lt $$deadline); exit 1 }"`
  Pop $R0
  System::Call 'Kernel32::SetEnvironmentVariable(t "FRAIA_NSIS_INSTALL_DIR", t "") i.R1'
  ${if} $R0 != 0
    DetailPrint `Timed out waiting for "${PRODUCT_NAME}" processes to close.`
    SetErrorLevel 2
    Quit
  ${endIf}
!macroend
