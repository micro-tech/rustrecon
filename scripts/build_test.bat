@echo off
REM Build test script for RustRecon with SQLite cache fix
echo ====================================
echo RustRecon Build Test Script
echo ====================================
echo.

echo Step 1: Cleaning previous builds...
cargo clean
echo.

echo Step 2: Updating dependencies...
cargo update
echo.

echo Step 3: Building in release mode...
cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo ====================================
    echo BUILD FAILED!
    echo ====================================
    echo Please check the error messages above.
    echo Common issues:
    echo - Missing Rust toolchain components
    echo - Dependency conflicts
    echo - Syntax errors in source code
    pause
    exit /b 1
)

echo.
echo ====================================
echo BUILD SUCCESSFUL!
echo ====================================
echo.

echo Step 4: Testing the executable...
if exist "target\release\rustrecon.exe" (
    echo Found executable: target\release\rustrecon.exe
    echo Testing --help command:
    echo.
    target\release\rustrecon.exe --help
    echo.
    echo ====================================
    echo Build and basic test completed successfully!
    echo You can now test the cache functionality.
    echo ====================================
) else (
    echo ERROR: Executable not found at target\release\rustrecon.exe
    echo Build may have failed silently.
)

echo.
pause
