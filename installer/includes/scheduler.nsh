; DevPort Manager - Task Scheduler Configuration
; Handles automatic startup registration via Windows Task Scheduler
; Task name follows DevPort-* naming convention for tracking

;--------------------------------
; Task Scheduler settings
!define TASK_NAME "DevPort-AutoStart"
!define TASK_DESCRIPTION "Start DevPort Manager minimized to system tray on user logon"

;--------------------------------
; Function to register DevPort for automatic startup
Function RegisterAutoStart
    DetailPrint "Registering DevPort for automatic startup..."

    ; Create scheduled task using schtasks command
    ; Task runs at user logon, starts DevPort minimized to tray
    ;
    ; Task properties:
    ; - Trigger: At logon (any user)
    ; - Action: Run DevPortManager.exe --minimized
    ; - Run level: Highest (to handle UAC for hosts file if needed)
    ; - Run whether user is logged on or not: No (interactive session required)

    ; Build the schtasks command
    ; Using XML for more control over task properties
    nsExec::ExecToLog 'schtasks /Create /TN "${TASK_NAME}" /TR "\"${INSTALL_PATH}\DevPortManager.exe\" --minimized" /SC ONLOGON /RL HIGHEST /F /IT'
    Pop $0

    ${If} $0 == 0
        DetailPrint "Automatic startup registered successfully."
        ; Store registration state in registry
        WriteRegStr HKLM "Software\DevPort" "AutoStartRegistered" "true"
    ${Else}
        DetailPrint "Warning: Failed to register automatic startup (error: $0)"
        MessageBox MB_ICONWARNING|MB_OK "Failed to register automatic startup.$\r$\nYou can enable this later from DevPort settings."
    ${EndIf}
FunctionEnd

;--------------------------------
; Function to unregister automatic startup
Function un.UnregisterAutoStart
    DetailPrint "Removing automatic startup registration..."

    ; Delete the scheduled task
    nsExec::ExecToLog 'schtasks /Delete /TN "${TASK_NAME}" /F'
    Pop $0

    ${If} $0 == 0
        DetailPrint "Automatic startup removed successfully."
    ${Else}
        DetailPrint "Note: Automatic startup task was not found or already removed."
    ${EndIf}

    ; Clean up registry
    DeleteRegValue HKLM "Software\DevPort" "AutoStartRegistered"
FunctionEnd

;--------------------------------
; Function to check if auto-start is registered
Function CheckAutoStartStatus
    ; Check if scheduled task exists
    nsExec::ExecToStack 'schtasks /Query /TN "${TASK_NAME}"'
    Pop $0
    Pop $1

    ${If} $0 == 0
        Push 1  ; Task exists
    ${Else}
        Push 0  ; Task doesn't exist
    ${EndIf}
FunctionEnd

;--------------------------------
; Function to enable/disable the task (without removing)
Function EnableAutoStart
    nsExec::ExecToLog 'schtasks /Change /TN "${TASK_NAME}" /ENABLE'
    Pop $0
FunctionEnd

Function DisableAutoStart
    nsExec::ExecToLog 'schtasks /Change /TN "${TASK_NAME}" /DISABLE'
    Pop $0
FunctionEnd

;--------------------------------
; Alternative: XML-based task creation for more control
; This provides finer control over task properties

Function RegisterAutoStartXML
    ; Create XML file for task definition
    FileOpen $0 "$TEMP\devport-task.xml" w
    FileWrite $0 '<?xml version="1.0" encoding="UTF-16"?>$\r$\n'
    FileWrite $0 '<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">$\r$\n'
    FileWrite $0 '  <RegistrationInfo>$\r$\n'
    FileWrite $0 '    <Description>${TASK_DESCRIPTION}</Description>$\r$\n'
    FileWrite $0 '    <Author>DevPort</Author>$\r$\n'
    FileWrite $0 '  </RegistrationInfo>$\r$\n'
    FileWrite $0 '  <Triggers>$\r$\n'
    FileWrite $0 '    <LogonTrigger>$\r$\n'
    FileWrite $0 '      <Enabled>true</Enabled>$\r$\n'
    FileWrite $0 '    </LogonTrigger>$\r$\n'
    FileWrite $0 '  </Triggers>$\r$\n'
    FileWrite $0 '  <Principals>$\r$\n'
    FileWrite $0 '    <Principal id="Author">$\r$\n'
    FileWrite $0 '      <LogonType>InteractiveToken</LogonType>$\r$\n'
    FileWrite $0 '      <RunLevel>HighestAvailable</RunLevel>$\r$\n'
    FileWrite $0 '    </Principal>$\r$\n'
    FileWrite $0 '  </Principals>$\r$\n'
    FileWrite $0 '  <Settings>$\r$\n'
    FileWrite $0 '    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>$\r$\n'
    FileWrite $0 '    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>$\r$\n'
    FileWrite $0 '    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>$\r$\n'
    FileWrite $0 '    <AllowHardTerminate>true</AllowHardTerminate>$\r$\n'
    FileWrite $0 '    <StartWhenAvailable>false</StartWhenAvailable>$\r$\n'
    FileWrite $0 '    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>$\r$\n'
    FileWrite $0 '    <AllowStartOnDemand>true</AllowStartOnDemand>$\r$\n'
    FileWrite $0 '    <Enabled>true</Enabled>$\r$\n'
    FileWrite $0 '    <Hidden>false</Hidden>$\r$\n'
    FileWrite $0 '    <RunOnlyIfIdle>false</RunOnlyIfIdle>$\r$\n'
    FileWrite $0 '    <DisallowStartOnRemoteAppSession>false</DisallowStartOnRemoteAppSession>$\r$\n'
    FileWrite $0 '    <UseUnifiedSchedulingEngine>true</UseUnifiedSchedulingEngine>$\r$\n'
    FileWrite $0 '    <WakeToRun>false</WakeToRun>$\r$\n'
    FileWrite $0 '    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>$\r$\n'
    FileWrite $0 '    <Priority>7</Priority>$\r$\n'
    FileWrite $0 '  </Settings>$\r$\n'
    FileWrite $0 '  <Actions Context="Author">$\r$\n'
    FileWrite $0 '    <Exec>$\r$\n'
    FileWrite $0 '      <Command>${INSTALL_PATH}\DevPortManager.exe</Command>$\r$\n'
    FileWrite $0 '      <Arguments>--minimized</Arguments>$\r$\n'
    FileWrite $0 '      <WorkingDirectory>${INSTALL_PATH}</WorkingDirectory>$\r$\n'
    FileWrite $0 '    </Exec>$\r$\n'
    FileWrite $0 '  </Actions>$\r$\n'
    FileWrite $0 '</Task>$\r$\n'
    FileClose $0

    ; Import the task from XML
    nsExec::ExecToLog 'schtasks /Create /TN "${TASK_NAME}" /XML "$TEMP\devport-task.xml" /F'
    Pop $0

    ; Clean up XML file
    Delete "$TEMP\devport-task.xml"

    ${If} $0 == 0
        DetailPrint "Automatic startup registered successfully (XML method)."
    ${Else}
        DetailPrint "Warning: Failed to register automatic startup."
    ${EndIf}
FunctionEnd

;--------------------------------
; Notes on Task Scheduler behavior:
;
; When DevPort starts via Task Scheduler:
; - Starts minimized to system tray (--minimized flag)
; - Window is hidden, only tray icon visible
; - User can double-click tray icon to open main window
;
; Task Scheduler vs Registry Run key:
; - Task Scheduler allows "Run with highest privileges"
; - Needed for hosts file modifications without UAC prompts
; - More reliable than HKCU\...\Run for elevated apps
;
; Quit behavior:
; - When user quits DevPort, task remains registered
; - Next logon will start DevPort again
; - User must disable in Settings to prevent auto-start
