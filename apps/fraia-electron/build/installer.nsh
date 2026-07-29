!macro customCheckAppRunning
  ${GetProcessInfo} 0 $pid $1 $2 $3 $4
  nsExec::Exec `"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "& { $$deadline = (Get-Date).AddSeconds(15); do { $$processes = @(Get-CimInstance -ClassName Win32_Process | Where-Object { $$_.ExecutablePath -and $$_.ExecutablePath.StartsWith('$INSTDIR\', [System.StringComparison]::OrdinalIgnoreCase) -and $$_.ProcessId -ne $pid }); if ($$processes.Count -eq 0) { exit 0 }; Start-Sleep -Milliseconds 250 } while ((Get-Date) -lt $$deadline); exit 1 }"`
  Pop $R0
  ${if} $R0 != 0
    DetailPrint `Timed out waiting for "${PRODUCT_NAME}" processes to close.`
    SetErrorLevel 2
    Quit
  ${endIf}
!macroend
