# RustRecon Configuration Finder (PowerShell)
# Locates RustRecon configuration files in all standard locations

param(
    [switch]$ShowContent,
    [switch]$Edit,
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
RustRecon Configuration Finder

USAGE:
    .\find_config.ps1 [OPTIONS]

OPTIONS:
    -ShowContent    Display the content of the active configuration file
    -Edit           Open the active configuration file in default editor
    -Help           Show this help message

EXAMPLES:
    .\find_config.ps1
    .\find_config.ps1 -ShowContent
    .\find_config.ps1 -Edit

"@ -ForegroundColor White
    exit 0
}

# Header
Write-Host @"
╔══════════════════════════════════════════════════════════════╗
║                RustRecon Configuration Finder               ║
║                                                              ║
║  Locating RustRecon configuration files in all standard     ║
║  locations and showing which one is currently active.       ║
╚══════════════════════════════════════════════════════════════╝
"@ -ForegroundColor Cyan

$ConfigName = "rustrecon_config.toml"
$FoundConfigs = @()

Write-Info "Searching for RustRecon configuration files..."
Write-Host ""

# Define search locations in priority order
$SearchLocations = @(
    @{
        Name = "Local App Data (Primary)"
        Path = Join-Path $env:LOCALAPPDATA "RustRecon\$ConfigName"
        Description = "Windows standard location for application data"
    },
    @{
        Name = "Roaming App Data (Legacy)"
        Path = Join-Path $env:APPDATA "RustRecon\$ConfigName"
        Description = "Older Windows location, kept for compatibility"
    },
    @{
        Name = "Home Directory (.rustrecon)"
        Path = Join-Path $env:USERPROFILE ".rustrecon\$ConfigName"
        Description = "Unix-style hidden directory in user home"
    },
    @{
        Name = "Current Directory"
        Path = Join-Path (Get-Location) $ConfigName
        Description = "Local config in current working directory"
    },
    @{
        Name = "Project Root"
        Path = Join-Path $PSScriptRoot $ConfigName
        Description = "Config in the RustRecon project directory"
    }
)

# Search for configurations
$ActiveConfig = $null
$Priority = 1

foreach ($Location in $SearchLocations) {
    if (Test-Path $Location.Path) {
        $Status = if ($null -eq $ActiveConfig) {
            $ActiveConfig = $Location.Path
            "ACTIVE (this config is being used)"
        } else {
            "BACKUP (not used - higher priority exists)"
        }

        Write-Success "[FOUND] $($Location.Name):"
        Write-Host "  Path: $($Location.Path)"
        Write-Host "  Status: $Status" -ForegroundColor $(if ($Status.StartsWith("ACTIVE")) { "Green" } else { "Yellow" })
        Write-Host "  Description: $($Location.Description)"
        Write-Host ""

        $FoundConfigs += @{
            Priority = $Priority
            Name = $Location.Name
            Path = $Location.Path
            Status = $Status
            Description = $Location.Description
        }
    }
    $Priority++
}

# Summary
Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan

if ($FoundConfigs.Count -gt 0) {
    Write-Success "                    CONFIGURATION SUMMARY"
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host ""
    Write-Success "Found $($FoundConfigs.Count) configuration file(s)!"
    Write-Host ""
    Write-Info "RustRecon loads configuration in this priority order:"

    for ($i = 0; $i -lt $SearchLocations.Count; $i++) {
        $location = $SearchLocations[$i]
        $exists = Test-Path $location.Path
        $marker = if ($exists) { "✓" } else { "○" }
        $color = if ($exists) { "Green" } else { "DarkGray" }

        Write-Host "  $($i + 1). $marker $($location.Name)" -ForegroundColor $color
        Write-Host "     $($location.Path)" -ForegroundColor $color
        Write-Host ""
    }

    Write-Success "ACTIVE CONFIGURATION:"
    Write-Host "Path: $ActiveConfig" -ForegroundColor Green
    Write-Host ""

    # Show content if requested
    if ($ShowContent -or $FoundConfigs.Count -eq 1) {
        Write-Info "CONFIGURATION CONTENT:"
        Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Gray
        try {
            $content = Get-Content $ActiveConfig -Raw
            Write-Host $content
        } catch {
            Write-Error "Failed to read configuration file: $($_.Exception.Message)"
        }
        Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Gray
        Write-Host ""
    }

    # Edit if requested
    if ($Edit) {
        Write-Info "Opening configuration file in default editor..."
        try {
            Start-Process $ActiveConfig
            Write-Success "Configuration file opened: $ActiveConfig"
        } catch {
            Write-Error "Failed to open configuration file: $($_.Exception.Message)"
            Write-Info "You can manually edit: $ActiveConfig"
        }
    }

} else {
    Write-Warning "                     NO CONFIGURATION FOUND"
    Write-Host "═══════════════════════════════════════════════════════════════" -ForegroundColor Yellow
    Write-Host ""
    Write-Warning "No RustRecon configuration files were found in any standard locations."
    Write-Host ""
    Write-Info "To create a new configuration file:"
    Write-Host "  rustrecon init" -ForegroundColor White
    Write-Host ""
    Write-Info "This will create a default config at:"
    Write-Host "  $env:LOCALAPPDATA\RustRecon\$ConfigName" -ForegroundColor White
    Write-Host ""
    Write-Info "You can also specify a custom path:"
    Write-Host "  rustrecon init --config-path `"C:\path\to\your\config.toml`"" -ForegroundColor White
}

Write-Host ""
Write-Info "QUICK ACTIONS:"
Write-Host "───────────────────────────────────────────────────────────────" -ForegroundColor Gray
Write-Host "• Create new config:      " -NoNewline; Write-Host "rustrecon init" -ForegroundColor White
Write-Host "• Test current config:    " -NoNewline; Write-Host "rustrecon test" -ForegroundColor White
Write-Host "• Show config content:    " -NoNewline; Write-Host ".\find_config.ps1 -ShowContent" -ForegroundColor White
Write-Host "• Edit active config:     " -NoNewline; Write-Host ".\find_config.ps1 -Edit" -ForegroundColor White

if ($ActiveConfig) {
    Write-Host "• Direct edit command:    " -NoNewline; Write-Host "notepad `"$ActiveConfig`"" -ForegroundColor White
}

Write-Host ""
