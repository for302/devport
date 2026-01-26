; DevPort Manager - Uninstaller Logic
; Implements 3-tier uninstall modes as per PRD:
; A. Basic removal (app + runtime only)
; B. Full data removal (A + projects/backups/db data)
; C. System restoration (B + scheduler/firewall/hosts/shortcuts)

;--------------------------------
; Uninstall mode variables
Var UninstallMode  ; 1=Basic, 2=FullData, 3=SystemRestore
Var ConfirmDataDelete

;--------------------------------
; Uninstall options page
Function un.UninstallOptionsPage
    !insertmacro MUI_HEADER_TEXT "Uninstall Options" "Choose what to remove"

    nsDialogs::Create 1018
    Pop $0

    ${If} $0 == error
        Abort
    ${EndIf}

    ; Mode A: Basic removal (default)
    ${NSD_CreateRadioButton} 0 10u 100% 12u "Basic removal (recommended)"
    Pop $1
    ${NSD_Check} $1

    ${NSD_CreateLabel} 20u 24u 100% 36u "Remove DevPort app and bundled runtimes.$\r$\nKeep: projects, backups, logs, and settings"
    Pop $0

    ; Mode B: Full data removal
    ${NSD_CreateRadioButton} 0 65u 100% 12u "Full data removal"
    Pop $2

    ${NSD_CreateLabel} 20u 79u 100% 36u "Remove everything including projects and database data.$\r$\nWARNING: This will delete all your project files!"
    Pop $0

    ; Mode C: System restoration
    ${NSD_CreateRadioButton} 0 120u 100% 12u "Complete system restoration (advanced)"
    Pop $3

    ${NSD_CreateLabel} 20u 134u 100% 48u "Remove all DevPort data AND restore system changes:$\r$\n- Task Scheduler auto-start$\r$\n- Firewall rules$\r$\n- hosts file entries$\r$\n- Start Menu/Desktop shortcuts"
    Pop $0

    ; Store control handles for later
    StrCpy $4 $1  ; Basic
    StrCpy $5 $2  ; FullData
    StrCpy $6 $3  ; SystemRestore

    nsDialogs::Show
FunctionEnd

Function un.UninstallOptionsPageLeave
    ; Determine selected mode
    ${NSD_GetState} $4 $0
    ${If} $0 == ${BST_CHECKED}
        StrCpy $UninstallMode 1
        Return
    ${EndIf}

    ${NSD_GetState} $5 $0
    ${If} $0 == ${BST_CHECKED}
        StrCpy $UninstallMode 2
        ; Confirm data deletion
        MessageBox MB_ICONWARNING|MB_YESNO \
            "WARNING: This will permanently delete:$\r$\n$\r$\n- All projects in C:\DevPort\projects\$\r$\n- All database data$\r$\n- All backup files$\r$\n$\r$\nAre you sure you want to continue?" \
            IDYES +2
        Abort
        Return
    ${EndIf}

    ${NSD_GetState} $6 $0
    ${If} $0 == ${BST_CHECKED}
        StrCpy $UninstallMode 3
        ; Double confirmation for system restore
        MessageBox MB_ICONWARNING|MB_YESNO \
            "WARNING: This will permanently delete all DevPort data AND restore system changes.$\r$\n$\r$\nThis includes removing:$\r$\n- All projects and database data$\r$\n- Task Scheduler entries$\r$\n- Firewall rules$\r$\n- hosts file entries$\r$\n$\r$\nAre you absolutely sure?" \
            IDYES +2
        Abort
        Return
    ${EndIf}

    ; Default to basic mode
    StrCpy $UninstallMode 1
FunctionEnd

;--------------------------------
; Main uninstaller section
Section "Uninstall"
    SetShellVarContext all

    ; First, stop any running DevPort processes
    DetailPrint "Stopping DevPort processes..."
    Call un.StopAllProcesses

    ; Mode A: Basic removal (always executed)
    DetailPrint "Removing DevPort application..."
    Call un.RemoveCore

    ; Mode B: Full data removal
    ${If} $UninstallMode >= 2
        DetailPrint "Removing user data..."
        Call un.RemoveUserData
    ${EndIf}

    ; Mode C: System restoration
    ${If} $UninstallMode >= 3
        DetailPrint "Restoring system settings..."
        Call un.RestoreSystem
    ${EndIf}

    ; Always clean up registry
    Call un.CleanRegistry

    ; Remove uninstaller itself
    Delete "$INSTDIR\Uninstall.exe"

    ; Remove install directory if empty
    RMDir "$INSTDIR"

    ; If directory still exists (due to remaining files), notify user
    IfFileExists "$INSTDIR\*.*" 0 +3
        MessageBox MB_ICONINFORMATION|MB_OK \
            "Some files could not be removed and remain in:$\r$\n$INSTDIR$\r$\n$\r$\nYou may delete this folder manually."
        Goto done

    ; Success message
    MessageBox MB_ICONINFORMATION|MB_OK "DevPort Manager has been successfully uninstalled."

    done:
SectionEnd

;--------------------------------
; Helper function: Stop all DevPort processes
Function un.StopAllProcesses
    ; Kill DevPortManager.exe
    nsExec::ExecToLog 'taskkill /F /IM DevPortManager.exe'
    Pop $0

    ; Kill Apache
    nsExec::ExecToLog 'taskkill /F /IM httpd.exe'
    Pop $0

    ; Kill MariaDB
    nsExec::ExecToLog 'taskkill /F /IM mysqld.exe'
    Pop $0

    ; Kill Node.js processes started by DevPort
    ; Note: This might affect other Node processes, but uninstall should be complete
    nsExec::ExecToLog 'taskkill /F /IM node.exe'
    Pop $0

    ; Wait for processes to terminate
    Sleep 2000
FunctionEnd

;--------------------------------
; Mode A: Remove core application and runtime
Function un.RemoveCore
    ; Remove main executable
    Delete "$INSTDIR\DevPortManager.exe"

    ; Remove runtime folder
    RMDir /r "$INSTDIR\runtime\apache"
    RMDir /r "$INSTDIR\runtime\mariadb\bin"
    ; Keep mariadb\data unless Mode B
    ${If} $UninstallMode < 2
        ; Only remove binaries, keep data
        RMDir "$INSTDIR\runtime\mariadb"
    ${EndIf}
    RMDir /r "$INSTDIR\runtime\php"
    RMDir /r "$INSTDIR\runtime\nodejs"
    RMDir /r "$INSTDIR\runtime\git"
    RMDir "$INSTDIR\runtime"

    ; Remove tools folder
    RMDir /r "$INSTDIR\tools\phpmyadmin"
    RMDir /r "$INSTDIR\tools\composer"
    RMDir "$INSTDIR\tools"

    ; Remove config folder
    RMDir /r "$INSTDIR\config"

    DetailPrint "Core application removed."
FunctionEnd

;--------------------------------
; Mode B: Remove user data (projects, backups, db data)
Function un.RemoveUserData
    ; Remove projects folder
    RMDir /r "$INSTDIR\projects"

    ; Remove backups folder
    RMDir /r "$INSTDIR\backups"

    ; Remove logs folder
    RMDir /r "$INSTDIR\logs"

    ; Remove MariaDB data
    RMDir /r "$INSTDIR\runtime\mariadb\data"
    RMDir /r "$INSTDIR\runtime\mariadb"
    RMDir "$INSTDIR\runtime"

    ; Remove local app data
    RMDir /r "$LOCALAPPDATA\DevPort"

    DetailPrint "User data removed."
FunctionEnd

;--------------------------------
; Mode C: Restore system to pre-install state
Function un.RestoreSystem
    ; Remove Task Scheduler auto-start
    Call un.UnregisterAutoStart

    ; Remove firewall rules
    Call un.RemoveFirewallRules

    ; Remove hosts file entries
    Call un.RemoveHostsEntries

    ; Remove shortcuts
    Call un.RemoveShortcuts

    ; Remove from system PATH if added
    Call un.RemoveFromSystemPath

    ; Clean environment registry
    Call un.CleanEnvironmentRegistry

    DetailPrint "System settings restored."
FunctionEnd

;--------------------------------
; Remove hosts file entries (DevPort markers)
Function un.RemoveHostsEntries
    DetailPrint "Removing hosts file entries..."

    ; Use PowerShell to safely remove DevPort entries
    ; Entries are marked with "# DevPort BEGIN" and "# DevPort END"
    nsExec::ExecToLog 'powershell -Command "\
        $hostsPath = \"$env:SystemRoot\System32\drivers\etc\hosts\"; \
        $content = Get-Content $hostsPath -Raw; \
        $pattern = \"(?ms)# DevPort BEGIN.*?# DevPort END\r?\n?\"; \
        $newContent = $content -replace $pattern, \"\"; \
        Set-Content $hostsPath $newContent -Force"'
    Pop $0

    ${If} $0 == 0
        DetailPrint "Hosts file entries removed."
    ${Else}
        DetailPrint "Warning: Could not remove hosts file entries (may require manual cleanup)."
    ${EndIf}
FunctionEnd

;--------------------------------
; Remove shortcuts
Function un.RemoveShortcuts
    ; Remove Start Menu shortcuts
    RMDir /r "$SMPROGRAMS\${PRODUCT_NAME}"

    ; Remove Desktop shortcut
    Delete "$DESKTOP\${PRODUCT_NAME}.lnk"

    DetailPrint "Shortcuts removed."
FunctionEnd

;--------------------------------
; Clean up registry entries
Function un.CleanRegistry
    ; Remove application path registration
    DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"

    ; Remove uninstaller registration
    DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"

    ; Remove DevPort registry keys
    DeleteRegKey HKLM "Software\DevPort"

    DetailPrint "Registry cleaned."
FunctionEnd

;--------------------------------
; Uninstaller initialization
Function un.onInit
    ; Check for admin rights
    UserInfo::GetAccountType
    Pop $0
    ${If} $0 != "Admin"
        MessageBox MB_ICONSTOP|MB_OK "Administrator privileges are required to uninstall ${PRODUCT_NAME}."
        Abort
    ${EndIf}

    ; Confirm uninstallation
    MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 \
        "Are you sure you want to uninstall ${PRODUCT_NAME}?" \
        IDYES +2
    Abort

    ; Initialize mode to basic
    StrCpy $UninstallMode 1
FunctionEnd

;--------------------------------
; Notes on uninstall behavior:
;
; File locking:
; - If files are locked (in use), uninstaller will skip them
; - User is notified of remaining files
; - "Reboot and complete" option could be added in future
;
; Data preservation:
; - Mode A keeps projects, backups, and logs
; - User can manually delete these later
; - MariaDB data is kept unless Mode B
;
; System restoration:
; - All DevPort-* prefixed items are removed
; - hosts file entries between markers are removed
; - Firewall rules are deleted
; - Task Scheduler entries are deleted
