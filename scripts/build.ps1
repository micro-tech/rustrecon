# RustRecon Installer Build Script for Windows
# PowerShell version

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Building RustRecon Installer for Windows" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host

# Check if Rust is installed
try {
    $rustVersion = rustc --version
    Write-Host "✅ Rust toolchain found: $rustVersion" -ForegroundColor Green
}
catch {
    Write-Host "❌ Rust not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host

# Build the installer in release mode
Write-Host "🔨 Building installer..." -ForegroundColor Yellow
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed with exit code $LASTEXITCODE"
    }
    Write-Host "✅ Build successful!" -ForegroundColor Green
}
catch {
    Write-Host "❌ Build failed: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host

# Copy the installer to the parent directory for easy access
try {
    $sourceFile = "target\release\install.exe"
    $destFile = "..\rustrecon-installer.exe"

    if (Test-Path $sourceFile) {
        Copy-Item $sourceFile $destFile -Force
        Write-Host "✅ Installer copied to: $destFile" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Source file not found: $sourceFile" -ForegroundColor Yellow
    }
}
catch {
    Write-Host "⚠️  Could not copy installer to parent directory: $_" -ForegroundColor Yellow
}

Write-Host
Write-Host "========================================" -ForegroundColor Green
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host

Write-Host "You can now run the installer:" -ForegroundColor White
Write-Host "  ..\rustrecon-installer.exe" -ForegroundColor Cyan
Write-Host

Write-Host "Or run with options:" -ForegroundColor White
Write-Host "  ..\rustrecon-installer.exe --help" -ForegroundColor Cyan
Write-Host "  ..\rustrecon-installer.exe --silent" -ForegroundColor Cyan
Write-Host "  ..\rustrecon-installer.exe --uninstall" -ForegroundColor Cyan
Write-Host

# Optional: Create a desktop shortcut to the installer
$createShortcut = Read-Host "Create desktop shortcut to installer? (y/N)"
if ($createShortcut -eq "y" -or $createShortcut -eq "Y") {
    try {
        $desktopPath = [System.Environment]::GetFolderPath('Desktop')
        $shortcutPath = Join-Path $desktopPath "RustRecon Installer.lnk"
        $installerPath = Resolve-Path "..\rustrecon-installer.exe"

        $WshShell = New-Object -comObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut($shortcutPath)
        $Shortcut.TargetPath = $installerPath
        $Shortcut.Description = "RustRecon Security Scanner Installer"
        $Shortcut.IconLocation = $installerPath + ",0"
        $Shortcut.Save()

        Write-Host "✅ Desktop shortcut created: $shortcutPath" -ForegroundColor Green
    }
    catch {
        Write-Host "⚠️  Could not create desktop shortcut: $_" -ForegroundColor Yellow
    }
}

Write-Host
Read-Host "Press Enter to exit"
