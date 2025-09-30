@echo off
setlocal enabledelayedexpansion

REM RustRecon Batch Uninstaller
REM This batch file removes RustRecon from the system

title RustRecon Uninstaller

echo.
echo ╔══════════════════════════════════════════════════════════════╗
echo ║                    RustRecon Uninstaller                     ║
echo ║                                                              ║
echo ║  This script will remove RustRecon and all associated       ║
echo ║  files, configurations, and cached data from your system.   ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

REM Check for admin privileges
net session >nul 2>&1
if %errorLevel% == 0 (
    echo [INFO] Running with administrator privileges
) else (
    echo [WARN] Administrator privileges recommended for complete removal
    echo [WARN] Some items may not be removed without admin rights
)

echo.
echo Items to be removed:
echo   • Installation directory: H:\GitHub\RustRecon
echo   • Configuration files
echo   • Database and cache files
echo   • Log files
echo   • PATH environment variables
echo.

set /p confirm="Are you sure you want to continue? (y/N): "
if /i not "%confirm%"=="y" if /i not "%confirm%"=="yes" (
    echo [INFO] Uninstallation cancelled by user.
    pause
    exit /b 0
)

echo.
echo [INFO] Starting RustRecon uninstallation...

REM Stop any running RustRecon processes
echo [INFO] Stopping RustRecon processes...
taskkill /f /im rustrecon.exe >nul 2>&1
taskkill /f /im rustrecon-installer.exe >nul 2>&1
taskkill /f /t /fi "WINDOWTITLE eq *RustRecon*" >nul 2>&1
timeout /t 2 /nobreak >nul

REM Define paths
set INSTALL_PATH=H:\GitHub\RustRecon
set USER_CONFIG=%APPDATA%\RustRecon
set LOCAL_CONFIG=%LOCALAPPDATA%\RustRecon
set TEMP_PATH=%TEMP%\RustRecon
set DOCS_PATH=%USERPROFILE%\Documents\RustRecon

REM Remove executables
echo [INFO] Removing executables...
if exist "%INSTALL_PATH%\rustrecon.exe" (
    del /f /q "%INSTALL_PATH%\rustrecon.exe" >nul 2>&1
    if !errorlevel! equ 0 (
        echo [OK] Removed: rustrecon.exe
    ) else (
        echo [ERROR] Failed to remove rustrecon.exe
    )
)

if exist "%INSTALL_PATH%\rustrecon-installer.exe" (
    del /f /q "%INSTALL_PATH%\rustrecon-installer.exe" >nul 2>&1
    if !errorlevel! equ 0 (
        echo [OK] Removed: rustrecon-installer.exe
    ) else (
        echo [ERROR] Failed to remove rustrecon-installer.exe
    )
)

REM Remove configuration files
echo [INFO] Removing configuration files...
for %%f in (
    "%INSTALL_PATH%\rustrecon_config.toml"
    "%USER_CONFIG%\rustrecon_config.toml"
    "%LOCAL_CONFIG%\rustrecon_config.toml"
) do (
    if exist "%%f" (
        del /f /q "%%f" >nul 2>&1
        if !errorlevel! equ 0 (
            for %%i in ("%%f") do echo [OK] Removed: %%~nxi
        ) else (
            for %%i in ("%%f") do echo [ERROR] Failed to remove %%~nxi
        )
    )
)

REM Remove database files
echo [INFO] Removing database files...
for %%f in (
    "%INSTALL_PATH%\rustrecon.db"
    "%INSTALL_PATH%\scan_results.db"
    "%USER_CONFIG%\rustrecon.db"
    "%LOCAL_CONFIG%\rustrecon.db"
) do (
    if exist "%%f" (
        del /f /q "%%f" >nul 2>&1
        if !errorlevel! equ 0 (
            for %%i in ("%%f") do echo [OK] Removed: %%~nxi
        ) else (
            for %%i in ("%%f") do echo [ERROR] Failed to remove %%~nxi
        )
    )
)

REM Remove log files
echo [INFO] Removing log files...
for %%f in (
    "%INSTALL_PATH%\rustrecon.log"
    "%TEMP_PATH%\rustrecon.log"
) do (
    if exist "%%f" (
        del /f /q "%%f" >nul 2>&1
        if !errorlevel! equ 0 (
            for %%i in ("%%f") do echo [OK] Removed: %%~nxi
        )
    )
)

REM Remove log directories
for %%d in (
    "%USER_CONFIG%\logs"
    "%LOCAL_CONFIG%\logs"
) do (
    if exist "%%d" (
        rmdir /s /q "%%d" >nul 2>&1
        if !errorlevel! equ 0 (
            echo [OK] Removed: logs directory
        )
    )
)

REM Remove directories
echo [INFO] Removing directories...
for %%d in (
    "%USER_CONFIG%"
    "%LOCAL_CONFIG%"
    "%TEMP_PATH%"
    "%DOCS_PATH%"
) do (
    if exist "%%d" (
        rmdir /s /q "
