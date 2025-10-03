@echo off
REM Test script to verify cache database functionality
REM This script tests the cache system after the bundled SQLite fix

echo ====================================
echo RustRecon Cache Test Script
echo ====================================
echo.

echo Step 1: Building with SQLx 0.8.6 and bundled SQLite...
echo.
cargo update
cargo build --release
if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo Step 2: Testing cache stats (should create database)...
echo.
target\release\rustrecon.exe cache --stats

echo.
echo Step 3: Checking if database was created...
echo.
if exist "C:\Users\%USERNAME%\AppData\Local\RustRecon\scan_cache.db" (
    echo SUCCESS: Cache database found at C:\Users\%USERNAME%\AppData\Local\RustRecon\scan_cache.db
    dir "C:\Users\%USERNAME%\AppData\Local\RustRecon\scan_cache.db"
) else (
    echo WARNING: Cache database not found in expected location
    echo Checking current directory...
    if exist "scan_cache.db" (
        echo Found database in current directory: scan_cache.db
        dir scan_cache.db
    ) else (
        echo No database found
    )
)

echo.
echo Step 4: Testing a simple scan to trigger cache creation...
echo.
echo Creating a test Rust file with security patterns...
echo use std::process::Command; > test_temp.rs
echo. >> test_temp.rs
echo fn main() { >> test_temp.rs
echo     let user_input = "test"; >> test_temp.rs
echo     let cmd = format!("echo {}", user_input); >> test_temp.rs
echo     Command::new("sh").arg("-c").arg(^&cmd).output().unwrap(); >> test_temp.rs
echo     println!("Hello, cache test!"); >> test_temp.rs
echo } >> test_temp.rs

target\release\rustrecon.exe scan test_temp.rs --format summary

echo.
echo Step 5: Checking cache stats again...
echo.
target\release\rustrecon.exe cache --stats

echo.
echo Step 6: Cleanup...
del test_temp.rs >nul 2>&1

echo.
echo ====================================
echo Cache test completed!
echo ====================================
echo.
echo If you see "Cache database initialized" messages above,
echo the SQLx 0.8.6 upgrade with bundled SQLite was successful!
echo.
echo Technical details:
echo - SQLx version: 0.8.6
echo - libsqlite3-sys: bundled feature enabled
echo - This eliminates system SQLite dependency requirements
echo.
pause
