; DevPort Manager - Firewall Configuration
; Creates Windows Firewall rules for runtime binaries
; All rules are prefixed with "DevPort-" for easy identification and cleanup

;--------------------------------
; Firewall rule names (DevPort-* naming convention for tracking)
!define FW_RULE_APACHE "DevPort-Apache-HTTP"
!define FW_RULE_MARIADB "DevPort-MariaDB"
!define FW_RULE_NODEJS "DevPort-NodeJS"
!define FW_RULE_PHP "DevPort-PHP"

;--------------------------------
; Function to add all firewall rules
Function AddFirewallRules
    ; Add inbound rules for each runtime binary
    ; Using netsh for firewall management

    ; Apache HTTP Server (httpd.exe)
    DetailPrint "Adding firewall rule for Apache..."
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="${FW_RULE_APACHE}" dir=in action=allow program="${INSTALL_PATH}\runtime\apache\bin\httpd.exe" enable=yes profile=private description="DevPort Apache Web Server"'
    Pop $0

    ; MariaDB Server (mysqld.exe)
    DetailPrint "Adding firewall rule for MariaDB..."
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="${FW_RULE_MARIADB}" dir=in action=allow program="${INSTALL_PATH}\runtime\mariadb\bin\mysqld.exe" enable=yes profile=private description="DevPort MariaDB Database Server"'
    Pop $0

    ; Node.js (node.exe)
    DetailPrint "Adding firewall rule for Node.js..."
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="${FW_RULE_NODEJS}" dir=in action=allow program="${INSTALL_PATH}\runtime\nodejs\node.exe" enable=yes profile=private description="DevPort Node.js Runtime"'
    Pop $0

    ; PHP (php.exe and php-cgi.exe)
    DetailPrint "Adding firewall rule for PHP..."
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="${FW_RULE_PHP}" dir=in action=allow program="${INSTALL_PATH}\runtime\php\php.exe" enable=yes profile=private description="DevPort PHP Runtime"'
    Pop $0
    nsExec::ExecToLog 'netsh advfirewall firewall add rule name="${FW_RULE_PHP}-CGI" dir=in action=allow program="${INSTALL_PATH}\runtime\php\php-cgi.exe" enable=yes profile=private description="DevPort PHP CGI"'
    Pop $0

    DetailPrint "Firewall rules added successfully."
FunctionEnd

;--------------------------------
; Function to remove all DevPort firewall rules
Function un.RemoveFirewallRules
    DetailPrint "Removing DevPort firewall rules..."

    ; Remove Apache rule
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${FW_RULE_APACHE}"'
    Pop $0

    ; Remove MariaDB rule
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${FW_RULE_MARIADB}"'
    Pop $0

    ; Remove Node.js rule
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${FW_RULE_NODEJS}"'
    Pop $0

    ; Remove PHP rules
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${FW_RULE_PHP}"'
    Pop $0
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="${FW_RULE_PHP}-CGI"'
    Pop $0

    ; Remove all rules matching DevPort-* pattern (catch-all cleanup)
    ; This ensures any additional rules added in future versions are also removed
    nsExec::ExecToLog 'powershell -Command "Get-NetFirewallRule -DisplayName \"DevPort-*\" -ErrorAction SilentlyContinue | Remove-NetFirewallRule"'
    Pop $0

    DetailPrint "Firewall rules removed."
FunctionEnd

;--------------------------------
; Function to check if firewall rules exist
Function CheckFirewallRules
    ; Returns 0 if rules exist, 1 if not
    nsExec::ExecToStack 'netsh advfirewall firewall show rule name="${FW_RULE_APACHE}"'
    Pop $0
    Pop $1
    ${If} $0 == 0
        Push 0  ; Rules exist
    ${Else}
        Push 1  ; Rules don't exist
    ${EndIf}
FunctionEnd

;--------------------------------
; Function to display firewall rule status
Function ShowFirewallStatus
    DetailPrint "Checking firewall rule status..."

    nsExec::ExecToStack 'netsh advfirewall firewall show rule name="${FW_RULE_APACHE}" verbose'
    Pop $0
    Pop $1
    ${If} $0 == 0
        DetailPrint "Apache firewall rule: Enabled"
    ${Else}
        DetailPrint "Apache firewall rule: Not found"
    ${EndIf}

    nsExec::ExecToStack 'netsh advfirewall firewall show rule name="${FW_RULE_MARIADB}" verbose'
    Pop $0
    Pop $1
    ${If} $0 == 0
        DetailPrint "MariaDB firewall rule: Enabled"
    ${Else}
        DetailPrint "MariaDB firewall rule: Not found"
    ${EndIf}

    nsExec::ExecToStack 'netsh advfirewall firewall show rule name="${FW_RULE_NODEJS}" verbose'
    Pop $0
    Pop $1
    ${If} $0 == 0
        DetailPrint "Node.js firewall rule: Enabled"
    ${Else}
        DetailPrint "Node.js firewall rule: Not found"
    ${EndIf}

    nsExec::ExecToStack 'netsh advfirewall firewall show rule name="${FW_RULE_PHP}" verbose'
    Pop $0
    Pop $1
    ${If} $0 == 0
        DetailPrint "PHP firewall rule: Enabled"
    ${Else}
        DetailPrint "PHP firewall rule: Not found"
    ${EndIf}
FunctionEnd

;--------------------------------
; Notes on firewall policy:
;
; Default behavior (127.0.0.1 binding):
;   - Firewall rules are created but services bind to localhost only
;   - External access is blocked by default
;
; External access mode:
;   - When user enables external access in DevPort settings,
;   - DevPort will reconfigure services to bind to 0.0.0.0
;   - Firewall rules then allow external connections
;
; Rule properties:
;   - Profile: private (home/work networks only, not public)
;   - Direction: inbound
;   - Action: allow
;   - Enabled: yes
