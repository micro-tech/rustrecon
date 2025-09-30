@echo off
setlocal enabledelayedexpansion

:: RustRecon Git Branch Checker (Batch Version)
:: This script checks branch status and provides merge guidance

echo.
echo ===============================================================
echo                 RustRecon Git Branch Checker
echo ===============================================================
echo.

:: Check if we're in a git repository
if not exist ".git" (
    echo ERROR: Not in a git repository!
    echo Please run this script from the RustRecon root directory.
    pause
    exit /b 1
)

echo Checking git repository status...
echo.

:: Get current branch
for /f "tokens=*" %%i in ('git rev-parse --abbrev-ref HEAD 2^>nul') do set CURRENT_BRANCH=%%i

if "%CURRENT_BRANCH%"=="" (
    echo ERROR: Failed to get current branch
    pause
    exit /b 1
)

echo Current branch: %CURRENT_BRANCH%

:: Check working directory status
git status --porcelain > temp_status.txt 2>nul
for /f %%i in ("temp_status.txt") do set SIZE=%%~zi
if %SIZE% gtr 0 (
    echo.
    echo WARNING: Working directory is not clean!
    echo Uncommitted changes found:
    type temp_status.txt
    echo.
    set /p CONTINUE="Continue anyway? (y/N): "
    if /i not "!CONTINUE!"=="y" if /i not "!CONTINUE!"=="yes" (
        del temp_status.txt
        echo Operation cancelled.
        pause
        exit /b 0
    )
)
del temp_status.txt 2>nul

:: Get branch information
echo.
echo Local branches:
git branch 2>nul
if errorlevel 1 (
    echo ERROR: Failed to get branch information
    pause
    exit /b 1
)

echo.
echo Remote branches:
git branch -r 2>nul

:: Determine main branch
git show-ref --verify --quiet refs/heads/main
if not errorlevel 1 (
    set MAIN_BRANCH=main
) else (
    git show-ref --verify --quiet refs/heads/master
    if not errorlevel 1 (
        set MAIN_BRANCH=master
    ) else (
        echo ERROR: Neither 'main' nor 'master' branch found!
        pause
        exit /b 1
    )
)

echo.
echo Main branch identified as: %MAIN_BRANCH%

:: Switch to main branch if needed
if not "%CURRENT_BRANCH%"=="%MAIN_BRANCH%" (
    echo.
    echo Switching to %MAIN_BRANCH% branch...
    git checkout %MAIN_BRANCH%
    if errorlevel 1 (
        echo ERROR: Failed to switch to %MAIN_BRANCH% branch
        pause
        exit /b 1
    )
    echo Successfully switched to %MAIN_BRANCH%
)

:: Fetch from remote
echo.
echo Fetching latest changes from remote...
git fetch origin
if errorlevel 1 (
    echo WARNING: Failed to fetch from remote, continuing with local branches
) else (
    echo Successfully fetched from remote
)

:: Check if main is behind remote
for /f "tokens=*" %%i in ('git rev-list --count HEAD..origin/%MAIN_BRANCH% 2^>nul') do set BEHIND_COUNT=%%i
if not "%BEHIND_COUNT%"=="" if not "%BEHIND_COUNT%"=="0" (
    echo.
    echo WARNING: %MAIN_BRANCH% is %BEHIND_COUNT% commits behind origin/%MAIN_BRANCH%
    set /p PULL="Pull latest changes? (Y/n): "
    if /i not "!PULL!"=="n" if /i not "!PULL!"=="no" (
        git pull origin %MAIN_BRANCH%
        if errorlevel 1 (
            echo ERROR: Failed to pull from remote
            pause
            exit /b 1
        )
        echo Successfully updated from remote
    )
)

:: Check for branches with unmerged changes
echo.
echo Checking for unmerged branches...

set FOUND_UNMERGED=0
for /f "tokens=2" %%i in ('git branch ^| findstr /v "\*" ^| findstr /v "%MAIN_BRANCH%"') do (
    for /f "tokens=*" %%j in ('git rev-list --count %MAIN_BRANCH%..%%i 2^>nul') do (
        if not "%%j"=="" if not "%%j"=="0" (
            echo   %%i: %%j unmerged commits
            set FOUND_UNMERGED=1
        )
    )
)

if %FOUND_UNMERGED%==0 (
    echo No unmerged branches found. %MAIN_BRANCH% is up to date.
) else (
    echo.
    echo WARNING: Found branches with unmerged changes!
    echo.
    echo MANUAL MERGE REQUIRED:
    echo 1. Review the branches listed above
    echo 2. Merge them manually using: git merge [branch_name]
    echo 3. Resolve any conflicts if they occur
    echo 4. Delete merged branches: git branch -d [branch_name]
    echo 5. Push changes: git push origin %MAIN_BRANCH%
    echo.
    set /p CONTINUE="Press Enter to continue with status check..."
)

:: Final status
echo.
echo ===============================================================
echo                     Final Repository Status
echo ===============================================================

git status --short > temp_final.txt 2>nul
for /f %%i in ("temp_final.txt") do set FINAL_SIZE=%%~zi
if %FINAL_SIZE% gtr 0 (
    echo Uncommitted changes remain:
    type temp_final.txt
) else (
    echo Working directory is clean
)
del temp_final.txt 2>nul

:: Check if ahead of remote
for /f "tokens=*" %%i in ('git rev-list --count origin/%MAIN_BRANCH%..HEAD 2^>nul') do set AHEAD_COUNT=%%i
if not "%AHEAD_COUNT%"=="" if not "%AHEAD_COUNT%"=="0" (
    echo.
    echo INFO: %MAIN_BRANCH% is %AHEAD_COUNT% commits ahead of origin/%MAIN_BRANCH%
    echo Run 'git push origin %MAIN_BRANCH%' to push changes
)

echo.
echo ===============================================================
echo                         SUMMARY
echo ===============================================================
echo.
echo Repository Status: Ready for installer
echo Main Branch: %MAIN_BRANCH%
echo.
echo NEXT STEPS:
echo 1. Verify rustrecon_config.toml has correct model (gemini-1.5-pro-latest)
echo 2. Test the application locally
echo 3. Push any remaining changes: git push origin %MAIN_BRANCH%
echo 4. Run installer to get updated version
echo.
echo The installer will pull from the main branch and build the latest version.
echo.
pause
