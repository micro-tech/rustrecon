@echo off
echo ========================================
echo   RustRecon Installer Test Script
echo ========================================
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

echo ========================================
echo Testing Silent Installation
echo ========================================
echo.
echo This will test the installer with default settings:
echo - Install to: %LOCALAPPDATA%\RustRecon
echo - Add to PATH: Yes
echo - Create shortcuts: Yes
echo - No API key configuration
echo.

echo Starting silent installation...
echo.

REM Run silent installation
rustrecon-installer.exe --silent

if %ERRORLEVEL% eq 0 (
    echo.
    echo ✅ Installation completed successfully!
    echo.

    REM Test if RustRecon was installed
    if exist "%LOCALAPPDATA%\RustRecon\rustrecon.exe" (
        echo ✅ RustRecon binary found at: %LOCALAPPDATA%\RustRecon\rustrecon.exe
        echo.

        REM Test if it runs
        echo Testing RustRecon execution...
        "%LOCALAPPDATA%\RustRecon\rustrecon.exe" --help > nul 2>&1
        if %ERRORLEVEL% eq 0 (
            echo ✅ RustRecon executes successfully
        ) else (
            echo ⚠️  RustRecon binary found but may not execute properly
        )
    ) else (
        echo ❌ RustRecon binary not found in expected location
    )

    echo.
    echo Installation Summary:
    echo - Installation path: %LOCALAPPDATA%\RustRecon
    echo - Binary: rustrecon.exe
    echo - Config: rustrecon_config.toml
    echo.
    echo Next steps:
    echo 1. Configure your Gemini API key
    echo 2. Test with: rustrecon test
    echo 3. Run first scan: rustrecon scan . --format summary
    echo.

) else (
    echo.
    echo ❌ Installation failed with error code: %ERRORLEVEL%
    echo.
    echo Common issues:
    echo - Insufficient permissions
    echo - Antivirus interference
    echo - Missing dependencies
    echo.
    echo Try running as Administrator or use --force flag
)

echo.
echo ========================================
echo Testing Uninstall
echo ========================================
echo.
set /p uninstall="Do you want to test uninstallation? (y/N): "

if /i "%uninstall%"=="y" (
    echo.
    echo Running uninstaller...
    rustrecon-installer.exe --uninstall

    if %ERRORLEVEL% eq 0 (
        echo ✅ Uninstallation completed successfully!
    ) else (
        echo ❌ Uninstallation failed with error code: %ERRORLEVEL%
    )
) else (
    echo.
    echo Skipping uninstall test.
    echo.
    echo To manually uninstall later:
    echo   rustrecon-installer.exe --uninstall
)

echo.
echo ========================================
echo Test Complete
echo ========================================
echo.
echo Installer test finished!
echo.
pause
