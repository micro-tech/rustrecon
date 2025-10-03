@echo off
echo ========================================
echo Building RustRecon Installer for Windows
echo ========================================
echo.

REM Check if Rust is installed
rustc --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ❌ Rust not found. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo ✅ Rust toolchain found
echo.

REM Build the installer in release mode
echo 🔨 Building installer...
cargo build --release

if %ERRORLEVEL% neq 0 (
    echo ❌ Build failed
    pause
    exit /b 1
)

echo ✅ Build successful!
echo.

REM Copy the installer to the parent directory for easy access
copy target\release\install.exe ..\rustrecon-installer.exe
if %ERRORLEVEL% eq 0 (
    echo ✅ Installer copied to: ..\rustrecon-installer.exe
) else (
    echo ⚠️  Could not copy installer to parent directory
)

echo.
echo ========================================
echo Build Complete!
echo ========================================
echo.
echo You can now run the installer:
echo   ..\rustrecon-installer.exe
echo.
echo Or run with options:
echo   ..\rustrecon-installer.exe --help
echo   ..\rustrecon-installer.exe --silent
echo   ..\rustrecon-installer.exe --uninstall
echo.
pause
