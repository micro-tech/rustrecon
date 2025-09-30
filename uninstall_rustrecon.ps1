# RustRecon Uninstaller Script
# This script completely removes RustRecon from the system including all configurations and cached data

param(
    [switch]$Force,
    [switch]$KeepConfig,
    [switch]$Silent,
    [switch]$Help
)

# Color functions for output
function Write-Success { param($Message) Write-Host $Message -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host $Message -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host $Message -ForegroundColor Red }
function Write-Info { param($Message) Write-Host $Message -ForegroundColor Cyan }

# Help text
if ($Help) {
    Write-Host @"
RustRecon Uninstaller

USAGE:
    .\uninstall_rustrecon.ps1 [OPTIONS]

OPTIONS:
    -Force          Skip confirmation prompts and force removal
    -KeepConfig     Keep configuration files and user data
    -Silent         Run without output (except errors)
    -Help           Show this help message

EXAMPLES:
    .\uninstall_rustrecon.ps1
    .\uninstall_rustrecon.ps1 -Force
    .\uninstall_rustrecon.ps1 -KeepConfig
    .\uninstall_rustrecon.ps1 -Force -Silent

"@ -ForegroundColor White
    exit 0
}

# Header
if (-not $Silent) {
    Write-Host @"
╔══════════════════════════════════════════════════════════════╗
║                    RustRecon Uninstaller                     ║
║                                                              ║
║  This script will remove RustRecon and all associated       ║
║  files, configurations, and cached data from your system.   ║
╚══════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan
}

# Check if running as administrator
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Administrator)) {
    Write-Warning "Administrator privileges recommended for complete removal."
    if (-not $Force) {
        $response = Read-Host "Continue anyway? (y/N)"
        if ($response -notmatch '^y|yes$') {
            Write-Info "Uninstallation cancelled."
            exit 0
        }
    }
}

# Define paths to clean
$InstallPath = "H:\GitHub\RustRecon"
$UserConfigPath = "$env:APPDATA\RustRecon"
$LocalConfigPath = "$env:LOCALAPPDATA\RustRecon"
$TempPath = "$env:TEMP\RustRecon"
$DocumentsPath = "$env:USERPROFILE\Documents\RustRecon"

# Define executables and files to remove
$Executables = @(
    "$InstallPath\rustrecon.exe",
    "$InstallPath\rustrecon-installer.exe"
)

$ConfigFiles = @(
    "$InstallPath\rustrecon_config.toml",
    "$UserConfigPath\rustrecon_config.toml",
    "$LocalConfigPath\rustrecon_config.toml"
)

$DatabaseFiles = @(
    "$InstallPath\rustrecon.db",
    "$UserConfigPath\rustrecon.db",
    "$LocalConfigPath\rustrecon.db",
    "$InstallPath\scan_results.db"
)

$LogFiles = @(
    "$InstallPath\rustrecon.log",
    "$UserConfigPath\logs",
    "$LocalConfigPath\logs",
    "$TempPath\rustrecon.log"
)

# Function to safely remove item
function Remove-SafeItem {
    param(
        [string]$Path,
        [string]$Description
    )

    if (Test-Path $Path) {
        try {
            if ((Get-Item $Path -Force) -is [System.IO.DirectoryInfo]) {
                Remove-Item $Path -Recurse -Force -ErrorAction Stop
            } else {
                Remove-Item $Path -Force -ErrorAction Stop
            }
            if (-not $Silent) { Write-Success "✓ Removed: $Description" }
            return $true
        } catch {
            Write-Error "✗ Failed to remove $Description : $($_.Exception.Message)"
            return $false
        }
    } else {
        if (-not $Silent) { Write-Info "- Not found: $Description" }
        return $true
    }
}

# Function to stop processes
function Stop-RustReconProcesses {
    $processes = Get-Process | Where-Object {
        $_.ProcessName -like "*rustrecon*" -or
        $_.ProcessName -eq "rustrecon" -or
        $_.MainWindowTitle -like "*RustRecon*"
    }

    if ($processes) {
        if (-not $Silent) { Write-Info "Stopping RustRecon processes..." }
        foreach ($process in $processes) {
            try {
                $process.Kill()
                if (-not $Silent) { Write-Success "✓ Stopped process: $($process.ProcessName) (PID: $($process.Id))" }
            } catch {
                Write-Error "✗ Failed to stop process $($process.ProcessName): $($_.Exception.Message)"
            }
        }
        Start-Sleep -Seconds 2
    }
}

# Function to remove from PATH
function Remove-FromPath {
    param([string]$PathToRemove)

    if (-not (Test-Path $PathToRemove)) { return }

    try {
        # Get current PATH
        $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        $machinePath = [Environment]::GetEnvironmentVariable("PATH", "Machine")

        # Remove from user PATH
        if ($userPath -and $userPath.Contains($PathToRemove)) {
            $newUserPath = ($userPath.Split(';') | Where-Object { $_ -ne $PathToRemove }) -join ';'
            [Environment]::SetEnvironmentVariable("PATH", $newUserPath, "User")
            if (-not $Silent) { Write-Success "✓ Removed from user PATH" }
        }

        # Remove from machine PATH (requires admin)
        if ((Test-Administrator) -and $machinePath -and $machinePath.Contains($PathToRemove)) {
            $newMachinePath = ($machinePath.Split(';') | Where-Object { $_ -ne $PathToRemove }) -join ';'
            [Environment]::SetEnvironmentVariable("PATH", $newMachinePath, "Machine")
            if (-not $Silent) { Write-Success "✓ Removed from system PATH" }
        }
    } catch {
        Write-Error "✗ Failed to remove from PATH: $($_.Exception.Message)"
    }
}

# Function to remove registry entries
function Remove-RegistryEntries {
    $registryPaths = @(
        "HKCU:\Software\RustRecon",
        "HKLM:\Software\RustRecon",
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon"
    )

    foreach ($regPath in $registryPaths) {
        if (Test-Path $regPath) {
            try {
                Remove-Item $regPath -Recurse -Force -ErrorAction Stop
                if (-not $Silent) { Write-Success "✓ Removed registry entry: $regPath" }
            } catch {
                Write-Error "✗ Failed to remove registry entry $regPath : $($_.Exception.Message)"
            }
        }
    }
}

# Function to remove Windows services (if any)
function Remove-Services {
    $services = Get-Service | Where-Object { $_.Name -like "*RustRecon*" }
    foreach ($service in $services) {
        try {
            Stop-Service $service.Name -Force -ErrorAction SilentlyContinue
            sc.exe delete $service.Name | Out-Null
            if (-not $Silent) { Write-Success "✓ Removed service: $($service.Name)" }
        } catch {
            Write-Error "✗ Failed to remove service $($service.Name): $($_.Exception.Message)"
        }
    }
}

# Main uninstallation process
try {
    if (-not $Force -and -not $Silent) {
        Write-Warning "This will permanently remove RustRecon and all associated data."
        Write-Info "Items to be removed:"
        Write-Host "  • Installation directory: $InstallPath"
        Write-Host "  • Configuration files"
        Write-Host "  • Database and cache files"
        Write-Host "  • Log files"
        Write-Host "  • Registry entries"
        Write-Host "  • PATH environment variables"
        Write-Host ""

        $confirmation = Read-Host "Are you sure you want to continue? (y/N)"
        if ($confirmation -notmatch '^y|yes$') {
            Write-Info "Uninstallation cancelled by user."
            exit 0
        }
    }

    if (-not $Silent) { Write-Info "Starting RustRecon uninstallation..." }

    # Stop running processes
    Stop-RustReconProcesses

    # Remove from PATH
    Remove-FromPath $InstallPath

    # Remove executables
    if (-not $Silent) { Write-Info "Removing executables..." }
    foreach ($exe in $Executables) {
        Remove-SafeItem $exe "Executable: $(Split-Path $exe -Leaf)"
    }

    # Remove configuration files (unless KeepConfig is specified)
    if (-not $KeepConfig) {
        if (-not $Silent) { Write-Info "Removing configuration files..." }
        foreach ($config in $ConfigFiles) {
            Remove-SafeItem $config "Configuration: $(Split-Path $config -Leaf)"
        }
    }

    # Remove database files
    if (-not $Silent) { Write-Info "Removing database files..." }
    foreach ($db in $DatabaseFiles) {
        Remove-SafeItem $db "Database: $(Split-Path $db -Leaf)"
    }

    # Remove log files
    if (-not $Silent) { Write-Info "Removing log files..." }
    foreach ($log in $LogFiles) {
        Remove-SafeItem $log "Log: $(Split-Path $log -Leaf)"
    }

    # Remove directories
    if (-not $Silent) { Write-Info "Removing directories..." }
    $directories = @($UserConfigPath, $LocalConfigPath, $TempPath)
    if (-not $KeepConfig) {
        $directories += $DocumentsPath
    }

    foreach ($dir in $directories) {
        if (Test-Path $dir) {
            Remove-SafeItem $dir "Directory: $dir"
        }
    }

    # Remove main installation directory
    if (Test-Path $InstallPath) {
        if (-not $Silent) { Write-Info "Removing installation directory..." }

        # Remove specific files first
        $filesToRemove = @(
            "*.exe", "*.dll", "*.toml", "*.db", "*.log", "*.bat",
            "README.md", "*.md", ".gitignore"
        )

        foreach ($pattern in $filesToRemove) {
            Get-ChildItem "$InstallPath\$pattern" -ErrorAction SilentlyContinue | ForEach-Object {
                Remove-SafeItem $_.FullName "File: $($_.Name)"
            }
        }

        # Remove subdirectories
        $subDirs = @("installer", "rustrecon", ".zed", "target")
        foreach ($subDir in $subDirs) {
            $fullPath = Join-Path $InstallPath $subDir
            if (Test-Path $fullPath) {
                Remove-SafeItem $fullPath "Directory: $subDir"
            }
        }

        # If directory is empty, remove it
        if ((Get-ChildItem $InstallPath -ErrorAction SilentlyContinue | Measure-Object).Count -eq 0) {
            Remove-SafeItem $InstallPath "Installation directory"
        } elseif (-not $Silent) {
            Write-Warning "Installation directory not empty, some files remain"
        }
    }

    # Remove registry entries
    if (-not $Silent) { Write-Info "Removing registry entries..." }
    Remove-RegistryEntries

    # Remove services
    Remove-Services

    # Clean up Cargo cache (if exists)
    $cargoCache = "$env:USERPROFILE\.cargo\registry\cache"
    if (Test-Path $cargoCache) {
        Get-ChildItem $cargoCache -ErrorAction SilentlyContinue | Where-Object {
            $_.Name -like "*rustrecon*"
        } | ForEach-Object {
            Remove-SafeItem $_.FullName "Cargo cache: $($_.Name)"
        }
    }

    # Final cleanup - remove any remaining rustrecon files
    if (-not $Silent) { Write-Info "Performing final cleanup..." }

    $searchPaths = @(
        $env:TEMP,
        $env:LOCALAPPDATA,
        $env:APPDATA
    )

    foreach ($searchPath in $searchPaths) {
        Get-ChildItem $searchPath -Filter "*rustrecon*" -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
            Remove-SafeItem $_.FullName "Remaining file: $($_.Name)"
        }
    }

    # Success message
    if (-not $Silent) {
        Write-Host ""
        Write-Success "╔══════════════════════════════════════════════════════════════╗"
        Write-Success "║                 Uninstallation Complete!                    ║"
        Write-Success "║                                                              ║"
        Write-Success "║  RustRecon has been successfully removed from your system.  ║"
        if ($KeepConfig) {
        Write-Success "║  Configuration files were preserved as requested.           ║"
        }
        Write-Success "╚══════════════════════════════════════════════════════════════╝"
        Write-Host ""
        Write-Info "You may need to restart your terminal or computer for PATH changes to take effect."
    }

} catch {
    Write-Error "An error occurred during uninstallation: $($_.Exception.Message)"
    Write-Error "Stack trace: $($_.ScriptStackTrace)"
    exit 1
}

# Optional: Offer to remove the uninstaller itself
if (-not $Silent -and -not $Force) {
    Write-Host ""
    $removeSelf = Read-Host "Remove this uninstaller script as well? (y/N)"
    if ($removeSelf -match '^y|yes$') {
        Start-Process -FilePath "cmd.exe" -ArgumentList "/c timeout 2 > nul && del `"$($MyInvocation.MyCommand.Path)`"" -WindowStyle Hidden
        Write-Success "Uninstaller will be removed in 2 seconds..."
    }
}

exit 0
