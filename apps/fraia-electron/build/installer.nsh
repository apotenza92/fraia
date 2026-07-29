!include "nsProcess.nsh"

!macro customCheckAppRunning
  StrCpy $R1 0

  FraiaWaitForAppExit:
    ${nsProcess::FindProcess} "${APP_EXECUTABLE_FILENAME}" $R0
    ${if} $R0 != 0
      ${nsProcess::Unload}
      Goto FraiaAppExited
    ${endIf}
    IntOp $R1 $R1 + 1
    ${if} $R1 >= 240
      ${nsProcess::Unload}
      DetailPrint `Timed out waiting for "${PRODUCT_NAME}" processes to close.`
      SetErrorLevel 2
      Quit
    ${endIf}
    Sleep 250
    Goto FraiaWaitForAppExit

  FraiaAppExited:
!macroend
