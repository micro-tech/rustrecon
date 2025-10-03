@echo off
echo ========================================
echo   RustRecon Installer Demo Script
echo ========================================
echo.
echo This script demonstrates the RustRecon installer features.
echo.

REM Check if installer exists
if not exist "rustrecon-installer.exe" (
    echo ❌ Installer not found: rustrecon-installer.exe
    echo.
    echo Please build the installer first:
    echo   cd installer
    echo   cargo build --release
    echo   copy target\release\install.exe ..\rustrecon-installer.exe
    echo.
    pause
    exit /b 1
)

echo ✅ Installer found: rustrecon-installer.exe
echo.

:MENU
echo ========================================
echo         Installer Demo Menu
echo ========================================
echo.
echo 1. Show installer help
echo 2. Show installer version
echo 3. Run interactive installation (DEMO - will not install)
echo 4. Run silent installation (DEMO - will not install)
echo 5. Show uninstall option
echo 6. Exit
echo.
set /p choice="Select an option (1-6): "

if "%choice%"=="1" goto HELP
if "%choice%"=="2" goto VERSION
if "%choice%"=="3" goto INTERACTIVE
if "%choice%"=="4" goto SILENT
if "%choice%"=="5" goto UNINSTALL
if "%choice%"=="6" goto EXIT

echo Invalid choice. Please select 1-6.
echo.
goto MENU

:HELP
echo.
echo ========================================
echo         Installer Help Output
echo ========================================
echo.
rustrecon-installer.exe --help
echo.
pause
goto MENU

:VERSION
echo.
echo ========================================
echo        Installer Version Info
echo ========================================
echo.
rustrecon-installer.exe --version
echo.
pause
goto MENU

:INTERACTIVE
echo.
echo ========================================
echo      Interactive Installation Demo
echo ========================================
echo.
echo This would run the interactive installer with these features:
echo.
echo 🔍 System Requirements Check:
echo   - Windows 11 compatibility verification
echo   - Rust toolchain detection
echo   - Available disk space check
echo   - Git availability check
echo.
echo ⚙️ Installation Configuration:
echo   - Choose installation directory
echo   - Select system integration options
echo   - Configure shortcuts and PATH
echo   - Optional Gemini API key setup
echo.
echo 📦 Installation Process:
echo   - Download/compile RustRecon
echo   - System integration setup
echo   - Configuration file creation
echo   - API key validation
echo.
echo 🎉 Completion Summary:
echo   - Installation success confirmation
echo   - Quick start instructions
echo   - Next steps guidance
echo.
echo NOTE: This is a demo - actual installation disabled for safety
echo.
echo To run actual installation:
echo   rustrecon-installer.exe
echo.
pause
goto MENU

:SILENT
echo.
echo ========================================
echo       Silent Installation Demo
echo ========================================
echo.
echo This would run silent installation with default settings:
echo.
echo 📁 Default Installation Path: C:\Program Files\RustRecon
echo ✅ Add to PATH: Yes
echo ✅ Desktop Shortcut: Yes
echo ✅ Start Menu Entry: Yes
echo ❌ API Key: Not configured (can be done later)
echo ❌ Auto-updates: Disabled
echo.
echo Silent installation is perfect for:
echo   - Deployment scripts
echo   - Corporate environments
echo   - CI/CD pipelines
echo   - Batch installations
echo.
echo NOTE: This is a demo - actual installation disabled for safety
echo.
echo To run actual silent installation:
echo   rustrecon-installer.exe --silent
echo.
pause
goto MENU

:UNINSTALL
echo.
echo ========================================
echo         Uninstall Demo
echo ========================================
echo.
echo The uninstaller would perform these cleanup tasks:
echo.
echo 🗑️ File Removal:
echo   - Remove installation directory
echo   - Delete configuration files
echo   - Clean temporary files
echo.
echo 🔧 System Cleanup:
echo   - Remove from system PATH
echo   - Delete desktop shortcuts
echo   - Remove Start Menu entries
echo   - Clean registry entries
echo.
echo 📋 Uninstall Process:
echo   1. Locate installation via registry
echo   2. Remove system integration
echo   3. Delete installation files
echo   4. Clean registry entries
echo   5. Confirm successful removal
echo.
echo NOTE: This is a demo - actual uninstall disabled for safety
echo.
echo To run actual uninstallation:
echo   rustrecon-installer.exe --uninstall
echo.
pause
goto MENU

:EXIT
echo.
echo ========================================
echo            Demo Complete
echo ========================================
echo.
echo Thank you for trying the RustRecon installer demo!
echo.
echo 📚 Next Steps:
echo   - Read installer\README.md for full documentation
echo   - Check installer\build.bat to build from source
echo   - Run actual installation when ready
echo.
echo 🚀 Installation Commands:
echo   Interactive:  rustrecon-installer.exe
echo   Silent:       rustrecon-installer.exe --silent
echo   Uninstall:    rustrecon-installer.exe --uninstall
echo.
echo 📁 Key Files:
echo   - rustrecon-installer.exe (main installer)
echo   - installer\README.md (documentation)
echo   - installer\build.bat (build script)
echo.
echo Happy installing! 🔧
echo.
pause
