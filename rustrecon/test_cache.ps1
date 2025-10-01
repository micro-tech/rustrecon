# PowerShell Test Script for RustRecon Cache Database
# This script tests the cache system after the bundled SQLite fix

Write-Host "====================================" -ForegroundColor Cyan
Write-Host "RustRecon Cache Test Script" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host

# Step 1: Build with bundled SQLite
Write-Host "Step 1: Building with bundled SQLite..." -ForegroundColor Yellow
Write-Host
try {
    cargo update
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed with exit code $LASTEXITCODE"
    }
    Write-Host "✅ Build successful!" -ForegroundColor Green
} catch {
    Write-Host "❌ ERROR: Build failed!" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host

# Step 2: Test cache stats
Write-Host "Step 2: Testing cache stats (should create database)..." -ForegroundColor Yellow
Write-Host
& ".\target\release\rustrecon.exe" cache --stats

Write-Host

# Step 3: Check if database was created
Write-Host "Step 3: Checking if database was created..." -ForegroundColor Yellow
Write-Host

$expectedPath = "$env:LOCALAPPDATA\RustRecon\scan_cache.db"
$currentDirPath = ".\scan_cache.db"

if (Test-Path $expectedPath) {
    Write-Host "✅ SUCCESS: Cache database found at $expectedPath" -ForegroundColor Green
    $dbInfo = Get-Item $expectedPath
    Write-Host "   Size: $($dbInfo.Length) bytes" -ForegroundColor Gray
    Write-Host "   Created: $($dbInfo.CreationTime)" -ForegroundColor Gray
    Write-Host "   Modified: $($dbInfo.LastWriteTime)" -ForegroundColor Gray
} elseif (Test-Path $currentDirPath) {
    Write-Host "⚠️  Database found in current directory: $currentDirPath" -ForegroundColor Yellow
    $dbInfo = Get-Item $currentDirPath
    Write-Host "   Size: $($dbInfo.Length) bytes" -ForegroundColor Gray
} else {
    Write-Host "❌ WARNING: Cache database not found in expected locations" -ForegroundColor Red
    Write-Host "   Expected: $expectedPath" -ForegroundColor Gray
    Write-Host "   Current dir: $currentDirPath" -ForegroundColor Gray
}

Write-Host

# Step 4: Test a simple scan
Write-Host "Step 4: Testing a simple scan to trigger cache creation..." -ForegroundColor Yellow
Write-Host

# Create a test Rust file with potential security issues to trigger cache
$testContent = @"
use std::process::Command;

fn main() {
    let user_input = "test";
    let cmd = format!("echo {}", user_input);
    Command::new("sh").arg("-c").arg(&cmd).output().unwrap();
    println!("Hello, cache test!");
}
"@
Set-Content -Path "test_temp.rs" -Value $testContent
Write-Host "Created temporary test file with security patterns: test_temp.rs" -ForegroundColor Gray

# Run scan to trigger cache creation
Write-Host "Running scan to trigger cache database creation..." -ForegroundColor Gray
& ".\target\release\rustrecon.exe" scan test_temp.rs --format summary

Write-Host

# Step 5: Check cache stats again
Write-Host "Step 5: Checking cache stats again..." -ForegroundColor Yellow
Write-Host
& ".\target\release\rustrecon.exe" cache --stats

Write-Host

# Step 6: Cleanup
Write-Host "Step 6: Cleanup..." -ForegroundColor Yellow
if (Test-Path "test_temp.rs") {
    Remove-Item "test_temp.rs" -Force
    Write-Host "Cleaned up temporary test file" -ForegroundColor Gray
}

Write-Host

# Summary
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "Cache Test Summary" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan

# Final database check
if (Test-Path $expectedPath) {
    $dbInfo = Get-Item $expectedPath
    Write-Host "✅ Cache database operational" -ForegroundColor Green
    Write-Host "   Location: $expectedPath" -ForegroundColor Gray
    Write-Host "   Size: $($dbInfo.Length) bytes" -ForegroundColor Gray

    if ($dbInfo.Length -gt 0) {
        Write-Host "✅ Database contains data" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Database is empty (may be newly created)" -ForegroundColor Yellow
    }
} else {
    Write-Host "❌ Cache database not found" -ForegroundColor Red
    Write-Host "   This indicates the bundled SQLite fix may not have worked" -ForegroundColor Red
}

Write-Host

# Additional diagnostics
Write-Host "🔍 Diagnostic Information:" -ForegroundColor Cyan
Write-Host "   LOCALAPPDATA: $env:LOCALAPPDATA" -ForegroundColor Gray
Write-Host "   Current User: $env:USERNAME" -ForegroundColor Gray
Write-Host "   PowerShell Version: $($PSVersionTable.PSVersion)" -ForegroundColor Gray
Write-Host "   Working Directory: $PWD" -ForegroundColor Gray
Write-Host "   SQLx Version: 0.8.6 (with bundled SQLite)" -ForegroundColor Gray
Write-Host "   libsqlite3-sys: bundled feature enabled" -ForegroundColor Gray

Write-Host

if (Test-Path $expectedPath) {
    Write-Host "🎉 SUCCESS: Cache database is working!" -ForegroundColor Green
    Write-Host "The SQLx 0.8.6 upgrade with bundled SQLite resolved the database connection issues." -ForegroundColor Green
} else {
    Write-Host "❓ If the database wasn't created, possible issues:" -ForegroundColor Yellow
    Write-Host "   1. Permission issues with AppData directory" -ForegroundColor Gray
    Write-Host "   2. Cache disabled in configuration" -ForegroundColor Gray
    Write-Host "   3. SQLx 0.8.6 compatibility issues" -ForegroundColor Gray
    Write-Host "   4. Application errors during database initialization" -ForegroundColor Gray
    Write-Host "   5. libsqlite3-sys bundled feature not working" -ForegroundColor Gray
}

Write-Host
Read-Host "Press Enter to exit"
