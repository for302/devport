; DevPort Manager NSIS Installer Script
; Version: 0.1.0
; https://nsis.sourceforge.io/

;--------------------------------
; Build configuration
!define PRODUCT_NAME "DevPort Manager"
!define PRODUCT_VERSION "0.1.0"
!define PRODUCT_PUBLISHER "DevPort"
!define PRODUCT_WEB_SITE "https://github.com/devport/devport-manager"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\DevPortManager.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

; Fixed install path for MVP
!define INSTALL_PATH "C:\DevPort"

;--------------------------------
; Includes
!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"
!include "WinVer.nsh"
!include "x64.nsh"

; Custom includes
!include "includes\env.nsh"
!include "includes\firewall.nsh"
!include "includes\scheduler.nsh"
!include "includes\uninstaller.nsh"

;--------------------------------
; General Attributes
Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "DevPortManager-${PRODUCT_VERSION}-Setup.exe"
InstallDir "${INSTALL_PATH}"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel admin
BrandingText "${PRODUCT_NAME} Installer"

;--------------------------------
; Variables
Var MariaDBPassword
Var CreateDesktopShortcut
Var RegisterTaskScheduler

;--------------------------------
; Interface Settings
!define MUI_ABORTWARNING
!define MUI_ICON "resources\icon.ico"
!define MUI_UNICON "resources\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "resources\welcome.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "resources\welcome.bmp"

;--------------------------------
; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "resources\license.txt"
!insertmacro MUI_PAGE_COMPONENTS
Page custom MariaDBPasswordPage MariaDBPasswordPageLeave
Page custom OptionsPage OptionsPageLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
UninstPage custom un.UninstallOptionsPage un.UninstallOptionsPageLeave
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
; Languages
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Korean"

;--------------------------------
; Version Information
VIProductVersion "${PRODUCT_VERSION}.0"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductVersion" "${PRODUCT_VERSION}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "CompanyName" "${PRODUCT_PUBLISHER}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${PRODUCT_NAME} Installer"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${PRODUCT_VERSION}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "Copyright (c) ${PRODUCT_PUBLISHER}"

;--------------------------------
; MariaDB Password Page
Function MariaDBPasswordPage
    !insertmacro MUI_HEADER_TEXT "MariaDB Configuration" "Set the root password for MariaDB"

    nsDialogs::Create 1018
    Pop $0

    ${If} $0 == error
        Abort
    ${EndIf}

    ${NSD_CreateLabel} 0 0 100% 24u "Please enter the root password for MariaDB.$\r$\nThis password will be required to access the database."
    Pop $0

    ${NSD_CreateLabel} 0 40u 80u 12u "Password:"
    Pop $0

    ${NSD_CreatePassword} 85u 38u 200u 14u ""
    Pop $1

    ${NSD_CreateLabel} 0 60u 80u 12u "Confirm:"
    Pop $0

    ${NSD_CreatePassword} 85u 58u 200u 14u ""
    Pop $2

    ${NSD_CreateLabel} 0 85u 100% 24u "Note: Password must be at least 8 characters long.$\r$\nThis password is used for DevPort internal operations."
    Pop $0

    nsDialogs::Show
FunctionEnd

Function MariaDBPasswordPageLeave
    ; Get password values
    ${NSD_GetText} $1 $MariaDBPassword
    ${NSD_GetText} $2 $0

    ; Validate password length
    StrLen $3 $MariaDBPassword
    ${If} $3 < 8
        MessageBox MB_ICONEXCLAMATION|MB_OK "Password must be at least 8 characters long."
        Abort
    ${EndIf}

    ; Validate password match
    ${If} $MariaDBPassword != $0
        MessageBox MB_ICONEXCLAMATION|MB_OK "Passwords do not match."
        Abort
    ${EndIf}
FunctionEnd

;--------------------------------
; Options Page
Function OptionsPage
    !insertmacro MUI_HEADER_TEXT "Installation Options" "Configure additional options"

    nsDialogs::Create 1018
    Pop $0

    ${If} $0 == error
        Abort
    ${EndIf}

    ${NSD_CreateCheckbox} 0 10u 100% 12u "Create Desktop shortcut"
    Pop $1
    ${NSD_Check} $1  ; Checked by default

    ${NSD_CreateCheckbox} 0 30u 100% 12u "Register DevPort for automatic startup (Task Scheduler)"
    Pop $2

    ${NSD_CreateLabel} 20u 44u 100% 12u "DevPort will start minimized to system tray on Windows startup"
    Pop $0

    ${NSD_CreateGroupBox} 0 70u 100% 50u "Firewall"
    Pop $0

    ${NSD_CreateLabel} 10u 84u 100% 24u "Firewall rules will be created for:$\r$\nhttpd.exe, mysqld.exe, node.exe, php.exe"
    Pop $0

    nsDialogs::Show
FunctionEnd

Function OptionsPageLeave
    ${NSD_GetState} $1 $CreateDesktopShortcut
    ${NSD_GetState} $2 $RegisterTaskScheduler
FunctionEnd

;--------------------------------
; Installer Sections

Section "DevPort Manager (required)" SEC_CORE
    SectionIn RO  ; Read-only, always installed

    SetOutPath "$INSTDIR"
    SetOverwrite on

    ; Create directory structure
    CreateDirectory "$INSTDIR\runtime"
    CreateDirectory "$INSTDIR\runtime\apache"
    CreateDirectory "$INSTDIR\runtime\mariadb"
    CreateDirectory "$INSTDIR\runtime\mariadb\data"
    CreateDirectory "$INSTDIR\runtime\php"
    CreateDirectory "$INSTDIR\runtime\nodejs"
    CreateDirectory "$INSTDIR\runtime\git"
    CreateDirectory "$INSTDIR\tools"
    CreateDirectory "$INSTDIR\tools\phpmyadmin"
    CreateDirectory "$INSTDIR\tools\composer"
    CreateDirectory "$INSTDIR\config"
    CreateDirectory "$INSTDIR\logs"
    CreateDirectory "$INSTDIR\logs\apache"
    CreateDirectory "$INSTDIR\logs\mariadb"
    CreateDirectory "$INSTDIR\logs\projects"
    CreateDirectory "$INSTDIR\projects"
    CreateDirectory "$INSTDIR\backups"

    ; Copy main executable
    File "/oname=$INSTDIR\DevPortManager.exe" "..\src-tauri\target\release\DevPortManager.exe"

    ; Copy default configuration
    SetOutPath "$INSTDIR\config"
    File "config\devport.json"

    ; Store MariaDB password in config (encrypted in real implementation)
    ; This is a placeholder - actual implementation should use secure storage
    WriteINIStr "$INSTDIR\config\credentials.ini" "mariadb" "root_password" "$MariaDBPassword"

    ; Register application path
    WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\DevPortManager.exe"
    WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "Path" "$INSTDIR"

    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Register uninstaller in Windows
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\DevPortManager.exe"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "NoRepair" 1

    ; Calculate installed size
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "EstimatedSize" "$0"
SectionEnd

Section "Apache Web Server" SEC_APACHE
    SetOutPath "$INSTDIR\runtime\apache"
    ; Apache binaries will be copied here during build
    ; File /r "staging\apache\*.*"

    ; Create initial Apache configuration
    ; Actual config files will be generated by build script
SectionEnd

Section "MariaDB Database" SEC_MARIADB
    SetOutPath "$INSTDIR\runtime\mariadb"
    ; MariaDB binaries will be copied here during build
    ; File /r "staging\mariadb\*.*"

    ; Initialize MariaDB data directory
    ; This will be done by build script or first-run setup
SectionEnd

Section "PHP Runtime" SEC_PHP
    SetOutPath "$INSTDIR\runtime\php"
    ; PHP binaries will be copied here during build
    ; File /r "staging\php\*.*"
SectionEnd

Section "Node.js Runtime" SEC_NODEJS
    SetOutPath "$INSTDIR\runtime\nodejs"
    ; Node.js binaries will be copied here during build
    ; File /r "staging\nodejs\*.*"
SectionEnd

Section "Git" SEC_GIT
    SetOutPath "$INSTDIR\runtime\git"
    ; Git binaries will be copied here during build
    ; File /r "staging\git\*.*"
SectionEnd

Section "phpMyAdmin" SEC_PHPMYADMIN
    SetOutPath "$INSTDIR\tools\phpmyadmin"
    ; phpMyAdmin files will be copied here during build
    ; File /r "staging\phpmyadmin\*.*"
SectionEnd

Section "Composer" SEC_COMPOSER
    SetOutPath "$INSTDIR\tools\composer"
    ; Composer will be copied here during build
    ; File "staging\composer\composer.phar"
SectionEnd

Section "-Shortcuts" SEC_SHORTCUTS
    ; Start Menu shortcuts (always created)
    CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
    CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" "$INSTDIR\DevPortManager.exe"
    CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

    ; Desktop shortcut (optional)
    ${If} $CreateDesktopShortcut == ${BST_CHECKED}
        CreateShortCut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\DevPortManager.exe"
    ${EndIf}
SectionEnd

Section "-Firewall" SEC_FIREWALL
    ; Add firewall rules for runtime binaries
    Call AddFirewallRules
SectionEnd

Section "-TaskScheduler" SEC_TASKSCHEDULER
    ${If} $RegisterTaskScheduler == ${BST_CHECKED}
        Call RegisterAutoStart
    ${EndIf}
SectionEnd

Section "-Post" SEC_POST
    ; Final setup tasks
    ; Create initial devport.json if not exists
    IfFileExists "$INSTDIR\config\devport.json" +2 0
        Call CreateDefaultConfig
SectionEnd

;--------------------------------
; Section Descriptions
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_CORE} "DevPort Manager application (required)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_APACHE} "Apache 2.4 web server (port 8080)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_MARIADB} "MariaDB 10.x database server (port 3306)"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_PHP} "PHP 8.x runtime with common extensions"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_NODEJS} "Node.js 20.x LTS runtime"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_GIT} "Git version control system"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_PHPMYADMIN} "phpMyAdmin database management tool"
    !insertmacro MUI_DESCRIPTION_TEXT ${SEC_COMPOSER} "Composer PHP package manager"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

;--------------------------------
; Helper Functions

Function CreateDefaultConfig
    ; Create default devport.json configuration
    FileOpen $0 "$INSTDIR\config\devport.json" w
    FileWrite $0 '{$\r$\n'
    FileWrite $0 '  "version": "${PRODUCT_VERSION}",$\r$\n'
    FileWrite $0 '  "installPath": "${INSTALL_PATH}",$\r$\n'
    FileWrite $0 '  "services": {$\r$\n'
    FileWrite $0 '    "apache": {$\r$\n'
    FileWrite $0 '      "port": 8080,$\r$\n'
    FileWrite $0 '      "autoStart": false$\r$\n'
    FileWrite $0 '    },$\r$\n'
    FileWrite $0 '    "mariadb": {$\r$\n'
    FileWrite $0 '      "port": 3306,$\r$\n'
    FileWrite $0 '      "autoStart": false$\r$\n'
    FileWrite $0 '    }$\r$\n'
    FileWrite $0 '  },$\r$\n'
    FileWrite $0 '  "autoStartOnBoot": false,$\r$\n'
    FileWrite $0 '  "projects": []$\r$\n'
    FileWrite $0 '}$\r$\n'
    FileClose $0
FunctionEnd

Function .onInit
    ; Check Windows version (Windows 10 or later required)
    ${IfNot} ${AtLeastWin10}
        MessageBox MB_ICONSTOP|MB_OK "Windows 10 or later is required."
        Abort
    ${EndIf}

    ; Check for 64-bit Windows
    ${IfNot} ${RunningX64}
        MessageBox MB_ICONSTOP|MB_OK "64-bit Windows is required."
        Abort
    ${EndIf}

    ; Check if already installed
    ReadRegStr $0 ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString"
    ${If} $0 != ""
        MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 \
            "${PRODUCT_NAME} is already installed.$\r$\n$\r$\nDo you want to uninstall the previous version first?" \
            IDYES uninst IDNO abort
        abort:
            Abort
        uninst:
            ExecWait '$0 _?=$INSTDIR'
    ${EndIf}

    ; Initialize variables
    StrCpy $CreateDesktopShortcut ${BST_CHECKED}
    StrCpy $RegisterTaskScheduler ${BST_UNCHECKED}
FunctionEnd

Function .onInstSuccess
    ; Installation completed successfully
    MessageBox MB_ICONINFORMATION|MB_OK \
        "${PRODUCT_NAME} has been installed successfully.$\r$\n$\r$\nYou can now start DevPort Manager from the Start Menu."
FunctionEnd

;--------------------------------
; Uninstaller Section is in includes/uninstaller.nsh
