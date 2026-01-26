; DevPort Manager - Environment Setup
; Handles PATH injection and environment variable management

;--------------------------------
; Variables for PATH management
Var AddToSystemPath

;--------------------------------
; Function to add DevPort to system PATH (optional)
; This is an advanced option - not enabled by default
Function AddToSystemPath
    ; Read current PATH
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

    ; Check if already in PATH
    ${StrContains} $1 "${INSTALL_PATH}\runtime\nodejs" $0
    ${If} $1 != ""
        ; Already in PATH, skip
        Return
    ${EndIf}

    ; Build DevPort PATH entries
    StrCpy $1 "${INSTALL_PATH}\runtime\nodejs"
    StrCpy $1 "$1;${INSTALL_PATH}\runtime\php"
    StrCpy $1 "$1;${INSTALL_PATH}\runtime\git\bin"
    StrCpy $1 "$1;${INSTALL_PATH}\tools\composer"

    ; Append to existing PATH
    StrCpy $0 "$1;$0"

    ; Write new PATH
    WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" $0

    ; Notify system of environment change
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
FunctionEnd

;--------------------------------
; Function to remove DevPort from system PATH
Function un.RemoveFromSystemPath
    ; Read current PATH
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"

    ; Remove DevPort entries
    ${WordReplace} $0 "${INSTALL_PATH}\runtime\nodejs;" "" "+" $0
    ${WordReplace} $0 "${INSTALL_PATH}\runtime\php;" "" "+" $0
    ${WordReplace} $0 "${INSTALL_PATH}\runtime\git\bin;" "" "+" $0
    ${WordReplace} $0 "${INSTALL_PATH}\tools\composer;" "" "+" $0

    ; Also handle case without trailing semicolon
    ${WordReplace} $0 ";${INSTALL_PATH}\runtime\nodejs" "" "+" $0
    ${WordReplace} $0 ";${INSTALL_PATH}\runtime\php" "" "+" $0
    ${WordReplace} $0 ";${INSTALL_PATH}\runtime\git\bin" "" "+" $0
    ${WordReplace} $0 ";${INSTALL_PATH}\tools\composer" "" "+" $0

    ; Clean up any double semicolons
    ${WordReplace} $0 ";;" ";" "+" $0

    ; Write cleaned PATH
    WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" $0

    ; Notify system of environment change
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
FunctionEnd

;--------------------------------
; Function to set up process environment for DevPort
; This is called at runtime, not during installation
; The actual PATH injection happens when DevPort launches processes

; Environment variables that DevPort sets for child processes:
; PATH = C:\DevPort\runtime\nodejs;C:\DevPort\runtime\php;C:\DevPort\runtime\git\bin;%PATH%
; DEVPORT_HOME = C:\DevPort
; DEVPORT_RUNTIME = C:\DevPort\runtime
; NODE_PATH = C:\DevPort\runtime\nodejs
; PHP_INI_SCAN_DIR = C:\DevPort\runtime\php

;--------------------------------
; Macro to check if string contains substring
!macro _StrContains ResultVar String SubString
    Push "${String}"
    Push "${SubString}"
    Call StrContains
    Pop "${ResultVar}"
!macroend
!define StrContains '!insertmacro "_StrContains"'

Function StrContains
    Exch $1 ; SubString
    Exch
    Exch $2 ; String
    Push $3
    Push $4
    Push $5

    StrLen $3 $1
    StrLen $4 $2
    StrCpy $5 0

    loop:
        IntCmp $5 $4 done done 0
        StrCpy $0 $2 $3 $5
        StrCmp $0 $1 found
        IntOp $5 $5 + 1
        Goto loop

    found:
        StrCpy $0 $1
        Goto exit

    done:
        StrCpy $0 ""

    exit:
        Pop $5
        Pop $4
        Pop $3
        Pop $2
        Pop $1
        Exch $0
FunctionEnd

;--------------------------------
; Registry keys for DevPort environment tracking
; These help identify what DevPort has modified

!define DEVPORT_ENV_KEY "Software\DevPort\Environment"

Function SaveEnvironmentState
    ; Save original PATH state before modification
    ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
    WriteRegStr HKLM "${DEVPORT_ENV_KEY}" "OriginalPath" $0
    WriteRegStr HKLM "${DEVPORT_ENV_KEY}" "PathModified" "false"
FunctionEnd

Function un.CleanEnvironmentRegistry
    ; Remove DevPort environment registry entries
    DeleteRegKey HKLM "${DEVPORT_ENV_KEY}"
FunctionEnd
