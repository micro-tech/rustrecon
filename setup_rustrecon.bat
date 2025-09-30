@echo off
setlocal enabledelayedexpansion

:: RustRecon Setup Script
:: This script helps you properly initialize RustRecon with correct configuration paths

echo.
echo ===============================================================
echo                     RustRecon Setup Script
echo ===============================================================
echo.
echo This script will help you set up RustRecon properly with:
echo   • Correct configuration location
echo   • API key setup
echo   • Initial testing
echo.

:: Check if rustrecon.exe exists
if not exist "rustrecon.exe" (
    if not exist "target\release\rustrecon.exe" (
        echo ERROR: RustRecon executable not found!
        echo.
        echo Please build RustRecon first:
        echo   cargo build --release
        echo.
        echo Or download the pre-built executable.
        pause
        exit /b 1
    ) else (
        set RUSTRECON_EXE=target\release\rustrecon.exe
        echo Using built executable: !RUSTRECON_EXE!
    )
) else (
    set RUSTRECON_EXE=rustrecon.exe
    echo Using executable: !RUSTRECON_EXE!
)

echo.

:: Show where config will be stored
echo Configuration will be stored in:
echo   Primary:   %LOCALAPPDATA%\RustRecon\rustrecon_config.toml
echo   Fallback:  %APPDATA%\RustRecon\rustrecon_config.toml
echo.

:: Check if config already exists
set CONFIG_EXISTS=0
if exist "%LOCALAPPDATA%\RustRecon\rustrecon_config.toml" set CONFIG_EXISTS=1
if exist "%APPDATA%\RustRecon\rustrecon_config.toml" set CONFIG_EXISTS=1

if !CONFIG_EXISTS!==1 (
    echo WARNING: Configuration already exists!
    echo.
    set /p OVERWRITE="Do you want to overwrite existing configuration? (y/N): "
    if /i not "!OVERWRITE!"=="y" if /i not "!OVERWRITE!"=="yes" (
        echo.
        echo Setup cancelled. Your existing configuration remains unchanged.
        echo.
        echo To test your current configuration: !RUSTRECON_EXE! test
        pause
        exit /b 0
    )
    echo.
)

:: Run the initialization
echo Starting RustRecon initialization...
echo.
echo The init process will:
echo   1. Create the configuration directory if needed
echo   2. Prompt you for your Gemini API key
echo   3. Let you select the AI model to use
echo   4. Create the configuration file in the correct location
echo   5. Offer to test the configuration
echo.
pause

echo.
echo Running: !RUSTRECON_EXE! init
echo.
!RUSTRECON_EXE! init

if errorlevel 1 (
    echo.
    echo ERROR: Initialization failed!
    echo.
    echo Common issues:
    echo   • Missing permissions to create directories
    echo   • Invalid API key format
    echo   • Disk space issues
    echo.
    echo Try running as administrator or check the error message above.
    pause
    exit /b 1
)

echo.
echo ===============================================================
echo                    Setup Complete!
echo ===============================================================
echo.

:: Show final status
echo Configuration file locations checked:
if exist "%LOCALAPPDATA%\RustRecon\rustrecon_config.toml" (
    echo   [FOUND] %LOCALAPPDATA%\RustRecon\rustrecon_config.toml
    set FINAL_CONFIG=%LOCALAPPDATA%\RustRecon\rustrecon_config.toml
) else if exist "%APPDATA%\RustRecon\rustrecon_config.toml" (
    echo   [FOUND] %APPDATA%\RustRecon\rustrecon_config.toml
    set FINAL_CONFIG=%APPDATA%\RustRecon\rustrecon_config.toml
) else (
    echo   [NOT FOUND] No config file created - something went wrong!
    set FINAL_CONFIG=
)

if defined FINAL_CONFIG (
    echo.
    echo Active configuration: !FINAL_CONFIG!
    echo.
    echo Quick commands:
    echo   • Test setup:           !RUSTRECON_EXE! test
    echo   • Scan a project:       !RUSTRECON_EXE! scan "path\to\project"
    echo   • View configuration:   notepad "!FINAL_CONFIG!"
    echo   • Show help:            !RUSTRECON_EXE! --help
    echo.

    set /p TEST_NOW="Would you like to test your configuration now? (Y/n): "
    if /i not "!TEST_NOW!"=="n" if /i not "!TEST_NOW!"=="no" (
        echo.
        echo Testing configuration...
        echo.
        !RUSTRECON_EXE! test
    )
) else (
    echo.
    echo Setup may have failed. Please check the output above for errors.
    echo.
    echo To retry setup:
    echo   !RUSTRECON_EXE! init
)

echo.
echo Setup script complete!
pause
