@echo off
setlocal enabledelayedexpansion

echo.
echo ===============================================================
echo                 RustRecon Configuration Finder
echo ===============================================================
echo.

set CONFIG_NAME=rustrecon_config.toml
set FOUND=0

echo Searching for RustRecon configuration files...
echo.

:: 1. Check Local App Data (Primary location - Windows)
set LOCAL_APPDATA_PATH=%LOCALAPPDATA%\RustRecon\%CONFIG_NAME%
if exist "!LOCAL_APPDATA_PATH!" (
    echo [FOUND] Primary location:
    echo   !LOCAL_APPDATA_PATH!
    echo   Status: Active ^(this is the config being used^)
    echo.
    set FOUND=1
)

:: 2. Check Roaming App Data (Legacy location - Windows)
set ROAMING_APPDATA_PATH=%APPDATA%\RustRecon\%CONFIG_NAME%
if exist "!ROAMING_APPDATA_PATH!" (
    echo [FOUND] Legacy location:
    echo   !ROAMING_APPDATA_PATH!
    if !FOUND!==1 (
        echo   Status: Backup ^(not being used - primary takes precedence^)
    ) else (
        echo   Status: Active ^(this is the config being used^)
        set FOUND=1
    )
    echo.
)

:: 3. Check Home directory fallback
set HOME_PATH=%USERPROFILE%\.rustrecon\%CONFIG_NAME%
if exist "!HOME_PATH!" (
    echo [FOUND] Home directory:
    echo   !HOME_PATH!
    if !FOUND!==1 (
        echo   Status: Backup ^(not being used - higher priority exists^)
    ) else (
        echo   Status: Active ^(this is the config being used^)
        set FOUND=1
    )
    echo.
)

:: 4. Check current directory
if exist "%CONFIG_NAME%" (
    echo [FOUND] Current directory:
    echo   %CD%\%CONFIG_NAME%
    if !FOUND!==1 (
        echo   Status: Backup ^(not being used - higher priority exists^)
    ) else (
        echo   Status: Active ^(this is the config being used^)
        set FOUND=1
    )
    echo.
)

:: 5. Check project root (where we are now)
set PROJECT_CONFIG=%~dp0%CONFIG_NAME%
if exist "!PROJECT_CONFIG!" (
    echo [FOUND] Project root:
    echo   !PROJECT_CONFIG!
    echo   Status: Development config ^(may override others when run from here^)
    echo.
    set FOUND=1
)

:: Summary
echo ===============================================================
if !FOUND!==1 (
    echo                     CONFIGURATION SUMMARY
    echo ===============================================================
    echo.
    echo Configuration file^(s^) found! RustRecon will load config in this order:
    echo.
    echo   1. Local App Data:  %LOCALAPPDATA%\RustRecon\%CONFIG_NAME%
    echo   2. Roaming App Data: %APPDATA%\RustRecon\%CONFIG_NAME%
    echo   3. Home Directory:   %USERPROFILE%\.rustrecon\%CONFIG_NAME%
    echo   4. Current Directory: %CONFIG_NAME%
    echo.
    echo The FIRST existing file in the list above will be used.
    echo.

    :: Show content of active config
    if exist "!LOCAL_APPDATA_PATH!" (
        set ACTIVE_CONFIG=!LOCAL_APPDATA_PATH!
    ) else if exist "!ROAMING_APPDATA_PATH!" (
        set ACTIVE_CONFIG=!ROAMING_APPDATA_PATH!
    ) else if exist "!HOME_PATH!" (
        set ACTIVE_CONFIG=!HOME_PATH!
    ) else if exist "%CONFIG_NAME%" (
        set ACTIVE_CONFIG=%CD%\%CONFIG_NAME%
    ) else if exist "!PROJECT_CONFIG!" (
        set ACTIVE_CONFIG=!PROJECT_CONFIG!
    )

    echo ACTIVE CONFIGURATION CONTENT:
    echo ---------------------------------------------------------------
    type "!ACTIVE_CONFIG!"
    echo ---------------------------------------------------------------
    echo.
    echo To edit the active config: notepad "!ACTIVE_CONFIG!"

) else (
    echo                      NO CONFIGURATION FOUND
    echo ===============================================================
    echo.
    echo No RustRecon configuration files were found in any of the
    echo standard locations.
    echo.
    echo To create a new configuration file, run:
    echo   rustrecon init
    echo.
    echo This will create a default config at:
    echo   %LOCALAPPDATA%\RustRecon\%CONFIG_NAME%
    echo.
    echo You can also specify a custom path:
    echo   rustrecon init --config-path "C:\path\to\your\config.toml"
)

echo.
echo QUICK ACTIONS:
echo ---------------------------------------------------------------
echo ^• Create new config:     rustrecon init
echo ^• Test current config:   rustrecon test
echo ^• Show this help:        %~nx0
echo ^• Edit active config:    notepad "!ACTIVE_CONFIG!"
echo.

pause
