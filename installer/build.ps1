# DevPort Manager - NSIS Installer Build Script
# This script compiles the NSIS installer and prepares staging files

param(
    [switch]$Clean,
    [switch]$SkipBuild,
    [string]$OutputDir = ".\dist"
)

$ErrorActionPreference = "Stop"

# Configuration
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$StagingDir = Join-Path $ScriptDir "staging"
$NsiFile = Join-Path $ScriptDir "devport.nsi"

# Version from tauri.conf.json
$TauriConfig = Get-Content (Join-Path $ProjectRoot "src-tauri\tauri.conf.json") | ConvertFrom-Json
$Version = $TauriConfig.version
$ProductName = $TauriConfig.productName

Write-Host "================================" -ForegroundColor Cyan
Write-Host " DevPort Manager Installer Build" -ForegroundColor Cyan
Write-Host " Version: $Version" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check for NSIS installation
$NsisPath = $null
$NsisPaths = @(
    "C:\Program Files (x86)\NSIS\makensis.exe",
    "C:\Program Files\NSIS\makensis.exe",
    "$env:ProgramFiles\NSIS\makensis.exe",
    "${env:ProgramFiles(x86)}\NSIS\makensis.exe"
)

foreach ($path in $NsisPaths) {
    if (Test-Path $path) {
        $NsisPath = $path
        break
    }
}

# Try to find in PATH
if (-not $NsisPath) {
    $NsisPath = Get-Command makensis.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
}

if (-not $NsisPath) {
    Write-Host "ERROR: NSIS (makensis.exe) not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install NSIS from: https://nsis.sourceforge.io/Download" -ForegroundColor Yellow
    Write-Host "Or install via Chocolatey: choco install nsis" -ForegroundColor Yellow
    exit 1
}

Write-Host "Found NSIS: $NsisPath" -ForegroundColor Green

# Clean staging directory if requested
if ($Clean) {
    Write-Host "Cleaning staging directory..." -ForegroundColor Yellow
    if (Test-Path $StagingDir) {
        Remove-Item -Recurse -Force $StagingDir
    }
    if (Test-Path $OutputDir) {
        Remove-Item -Recurse -Force $OutputDir
    }
}

# Create directories
New-Item -ItemType Directory -Force -Path $StagingDir | Out-Null
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Build Tauri application if not skipped
if (-not $SkipBuild) {
    Write-Host ""
    Write-Host "Building Tauri application..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    try {
        npm run tauri build
        if ($LASTEXITCODE -ne 0) {
            Write-Host "ERROR: Tauri build failed!" -ForegroundColor Red
            exit 1
        }
    }
    finally {
        Pop-Location
    }
}

# Copy built executable to staging
Write-Host ""
Write-Host "Copying files to staging directory..." -ForegroundColor Yellow

$ExePath = Join-Path $ProjectRoot "src-tauri\target\release\DevPortManager.exe"
if (-not (Test-Path $ExePath)) {
    $ExePath = Join-Path $ProjectRoot "src-tauri\target\release\devport-manager.exe"
}

if (Test-Path $ExePath) {
    Copy-Item $ExePath (Join-Path $StagingDir "DevPortManager.exe")
    Write-Host "  Copied: DevPortManager.exe" -ForegroundColor Green
} else {
    Write-Host "WARNING: DevPortManager.exe not found. Run 'npm run tauri build' first." -ForegroundColor Yellow
    Write-Host "         Continuing with placeholder for NSIS script validation..." -ForegroundColor Yellow
}

# Create staging subdirectories for runtime binaries
$RuntimeDirs = @(
    "apache",
    "mariadb",
    "php",
    "nodejs",
    "git"
)

$ToolDirs = @(
    "phpmyadmin",
    "composer"
)

foreach ($dir in $RuntimeDirs) {
    $path = Join-Path $StagingDir $dir
    New-Item -ItemType Directory -Force -Path $path | Out-Null
    Write-Host "  Created: staging\$dir\" -ForegroundColor Gray
}

foreach ($dir in $ToolDirs) {
    $path = Join-Path $StagingDir $dir
    New-Item -ItemType Directory -Force -Path $path | Out-Null
    Write-Host "  Created: staging\$dir\" -ForegroundColor Gray
}

# Copy default configuration
$ConfigSrc = Join-Path $ScriptDir "config\devport.json"
$ConfigDest = Join-Path $StagingDir "config"
New-Item -ItemType Directory -Force -Path $ConfigDest | Out-Null
if (Test-Path $ConfigSrc) {
    Copy-Item $ConfigSrc $ConfigDest
    Write-Host "  Copied: config\devport.json" -ForegroundColor Green
}

# Copy resources
$ResourcesSrc = Join-Path $ScriptDir "resources"
if (Test-Path $ResourcesSrc) {
    # Ensure icon exists (create placeholder if not)
    $IconPath = Join-Path $ResourcesSrc "icon.ico"
    if (-not (Test-Path $IconPath)) {
        # Try to copy from Tauri icons
        $TauriIcon = Join-Path $ProjectRoot "src-tauri\icons\icon.ico"
        if (Test-Path $TauriIcon) {
            Copy-Item $TauriIcon $IconPath
            Write-Host "  Copied: icon.ico from Tauri icons" -ForegroundColor Green
        } else {
            Write-Host "  WARNING: icon.ico not found. Create resources\icon.ico" -ForegroundColor Yellow
        }
    }

    # Create welcome bitmap placeholder if not exists
    $WelcomeBmp = Join-Path $ResourcesSrc "welcome.bmp"
    if (-not (Test-Path $WelcomeBmp)) {
        Write-Host "  NOTE: welcome.bmp not found. Using NSIS default." -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "Staging complete. Runtime binaries should be placed in:" -ForegroundColor Yellow
Write-Host "  $StagingDir\apache\     - Apache HTTP Server" -ForegroundColor Gray
Write-Host "  $StagingDir\mariadb\    - MariaDB Server" -ForegroundColor Gray
Write-Host "  $StagingDir\php\        - PHP Runtime" -ForegroundColor Gray
Write-Host "  $StagingDir\nodejs\     - Node.js Runtime" -ForegroundColor Gray
Write-Host "  $StagingDir\git\        - Git for Windows" -ForegroundColor Gray
Write-Host "  $StagingDir\phpmyadmin\ - phpMyAdmin" -ForegroundColor Gray
Write-Host "  $StagingDir\composer\   - Composer" -ForegroundColor Gray
Write-Host ""

# Compile NSIS installer
Write-Host "Compiling NSIS installer..." -ForegroundColor Yellow
Write-Host "  Source: $NsiFile" -ForegroundColor Gray
Write-Host "  Output: $OutputDir" -ForegroundColor Gray
Write-Host ""

# Check if NSI file exists
if (-not (Test-Path $NsiFile)) {
    Write-Host "ERROR: NSIS script not found: $NsiFile" -ForegroundColor Red
    exit 1
}

# Change to installer directory for relative paths
Push-Location $ScriptDir
try {
    # Run makensis
    $OutputFile = "DevPortManager-$Version-Setup.exe"
    & $NsisPath /V3 /DOUTDIR="$OutputDir" $NsiFile

    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "================================" -ForegroundColor Green
        Write-Host " Build Successful!" -ForegroundColor Green
        Write-Host "================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Installer: $OutputDir\$OutputFile" -ForegroundColor Cyan
    } else {
        Write-Host ""
        Write-Host "ERROR: NSIS compilation failed with exit code $LASTEXITCODE" -ForegroundColor Red
        Write-Host ""
        Write-Host "Common issues:" -ForegroundColor Yellow
        Write-Host "  - Missing NSIS plugins (install via NSIS installer)" -ForegroundColor Gray
        Write-Host "  - Syntax errors in .nsi or .nsh files" -ForegroundColor Gray
        Write-Host "  - Missing resource files (icon.ico, etc.)" -ForegroundColor Gray
        exit 1
    }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "NOTE: To create a complete installer, add runtime binaries to staging directory" -ForegroundColor Yellow
Write-Host "and uncomment the File commands in devport.nsi" -ForegroundColor Yellow
