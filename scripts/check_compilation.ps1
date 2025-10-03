# RustRecon Compilation Check Script
# This script performs a comprehensive build test and diagnostics

param(
    [switch]$Clean,
    [switch]$Verbose,
    [switch]$TestCache
)

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "RustRecon Compilation Check Script" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host

# Function to check prerequisites
function Test-Prerequisites {
    Write-Host "🔍 Checking Prerequisites..." -ForegroundColor Yellow

    # Check Rust installation
    try {
        $rustVersion = & rustc --version 2>$null
        Write-Host "✅ Rust: $rustVersion" -ForegroundColor Green
    } catch {
        Write-Host "❌ Rust not found. Please install from https://rustup.rs/" -ForegroundColor Red
        return $false
    }

    # Check Cargo
    try {
        $cargoVersion = & cargo --version 2>$null
        Write-Host "✅ Cargo: $cargoVersion" -ForegroundColor Green
    } catch {
        Write-Host "❌ Cargo not found" -ForegroundColor Red
        return $false
    }

    # Check if we're in the right directory
    if (!(Test-Path "Cargo.toml")) {
        Write-Host "❌ Cargo.toml not found. Please run from the rustrecon directory." -ForegroundColor Red
        return $false
    }

    Write-Host "✅ All prerequisites satisfied" -ForegroundColor Green
    return $true
}

# Function to analyze dependencies
function Test-Dependencies {
    Write-Host "`n🔗 Analyzing Dependencies..." -ForegroundColor Yellow

    try {
        # Check for dependency conflicts
        Write-Host "   Checking for dependency conflicts..." -ForegroundColor Gray
        $result = & cargo tree --duplicates 2>&1
        if ($LASTEXITCODE -eq 0 -and $result.Count -eq 0) {
            Write-Host "✅ No duplicate dependencies found" -ForegroundColor Green
        } elseif ($result.Count -gt 0) {
            Write-Host "⚠️  Duplicate dependencies detected:" -ForegroundColor Yellow
            $result | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
        }

        # Verify SQLx configuration
        Write-Host "   Verifying SQLx configuration..." -ForegroundColor Gray
        $cargoToml = Get-Content "Cargo.toml" -Raw
        if ($cargoToml -match 'sqlx.*=.*"0\.8') {
            Write-Host "✅ SQLx 0.8.x configured" -ForegroundColor Green
        } else {
            Write-Host "⚠️  SQLx version may be outdated" -ForegroundColor Yellow
        }

        if ($cargoToml -match 'libsqlite3-sys.*bundled') {
            Write-Host "✅ Bundled SQLite configured" -ForegroundColor Green
        } else {
            Write-Host "❌ Bundled SQLite not configured" -ForegroundColor Red
            Write-Host "     Add: libsqlite3-sys = { version = `"0.30`", features = [`"bundled`"] }" -ForegroundColor Gray
        }

    } catch {
        Write-Host "⚠️  Could not analyze dependencies: $_" -ForegroundColor Yellow
    }
}

# Function to perform clean build
function Invoke-CleanBuild {
    Write-Host "`n🧹 Performing Clean Build..." -ForegroundColor Yellow

    if ($Clean) {
        Write-Host "   Cleaning previous builds..." -ForegroundColor Gray
        & cargo clean
        if ($LASTEXITCODE -ne 0) {
            Write-Host "⚠️  Clean command failed" -ForegroundColor Yellow
        }
    }

    Write-Host "   Updating dependencies..." -ForegroundColor Gray
    & cargo update
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠️  Update command failed" -ForegroundColor Yellow
    }

    Write-Host "   Building in release mode..." -ForegroundColor Gray
    $buildOutput = & cargo build --release 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Build successful!" -ForegroundColor Green
        return $true
    } else {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        Write-Host "`n🔍 Build Error Details:" -ForegroundColor Yellow
        $buildOutput | ForEach-Object {
            if ($_ -match "error\[|warning:|note:") {
                Write-Host "     $_" -ForegroundColor Red
            } elseif ($_ -match "help:") {
                Write-Host "     $_" -ForegroundColor Cyan
            } else {
                Write-Host "     $_" -ForegroundColor Gray
            }
        }
        return $false
    }
}

# Function to run tests
function Invoke-Tests {
    Write-Host "`n🧪 Running Tests..." -ForegroundColor Yellow

    Write-Host "   Running unit tests..." -ForegroundColor Gray
    $testOutput = & cargo test 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ All tests passed!" -ForegroundColor Green

        # Show test summary
        $testSummary = $testOutput | Where-Object { $_ -match "test result:" }
        if ($testSummary) {
            Write-Host "   $testSummary" -ForegroundColor Gray
        }
        return $true
    } else {
        Write-Host "❌ Some tests failed!" -ForegroundColor Red
        if ($Verbose) {
            $testOutput | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
        }
        return $false
    }
}

# Function to test executable
function Test-Executable {
    Write-Host "`n🎯 Testing Executable..." -ForegroundColor Yellow

    $exePath = "target\release\rustrecon.exe"
    if (!(Test-Path $exePath)) {
        Write-Host "❌ Executable not found at $exePath" -ForegroundColor Red
        return $false
    }

    $exeInfo = Get-Item $exePath
    Write-Host "✅ Executable found: $($exeInfo.Length) bytes" -ForegroundColor Green
    Write-Host "   Created: $($exeInfo.CreationTime)" -ForegroundColor Gray
    Write-Host "   Modified: $($exeInfo.LastWriteTime)" -ForegroundColor Gray

    # Test help command
    Write-Host "   Testing --help command..." -ForegroundColor Gray
    try {
        $helpOutput = & $exePath --help 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Help command works" -ForegroundColor Green
        } else {
            Write-Host "⚠️  Help command failed" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "⚠️  Could not test help command: $_" -ForegroundColor Yellow
    }

    return $true
}

# Function to test cache functionality
function Test-CacheSystem {
    Write-Host "`n💾 Testing Cache System..." -ForegroundColor Yellow

    $exePath = "target\release\rustrecon.exe"
    if (!(Test-Path $exePath)) {
        Write-Host "❌ Executable not found, skipping cache test" -ForegroundColor Red
        return $false
    }

    # Test cache stats command
    Write-Host "   Testing cache stats..." -ForegroundColor Gray
    try {
        $cacheOutput = & $exePath cache --stats 2>&1

        if ($cacheOutput -match "Cache database initialized" -or
            $cacheOutput -match "Total cached entries:") {
            Write-Host "✅ Cache system is working" -ForegroundColor Green
        } elseif ($cacheOutput -match "No cache database found") {
            Write-Host "⚠️  Cache database not created yet (normal for first run)" -ForegroundColor Yellow
        } else {
            Write-Host "❌ Cache system issues detected" -ForegroundColor Red
            if ($Verbose) {
                $cacheOutput | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
            }
        }

        # Check if database was created
        $dbPath = "$env:LOCALAPPDATA\RustRecon\scan_cache.db"
        if (Test-Path $dbPath) {
            $dbInfo = Get-Item $dbPath
            Write-Host "✅ Cache database exists: $($dbInfo.Length) bytes" -ForegroundColor Green
        } else {
            Write-Host "⚠️  Cache database not yet created at $dbPath" -ForegroundColor Yellow
        }

    } catch {
        Write-Host "❌ Cache test failed: $_" -ForegroundColor Red
        return $false
    }

    return $true
}

# Function to check code quality
function Test-CodeQuality {
    Write-Host "`n📋 Code Quality Checks..." -ForegroundColor Yellow

    # Check for clippy if available
    try {
        Write-Host "   Running clippy..." -ForegroundColor Gray
        $clippyOutput = & cargo clippy --release -- -D warnings 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Clippy checks passed" -ForegroundColor Green
        } else {
            Write-Host "⚠️  Clippy found issues" -ForegroundColor Yellow
            if ($Verbose) {
                $clippyOutput | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
            }
        }
    } catch {
        Write-Host "⚠️  Clippy not available" -ForegroundColor Yellow
    }

    # Check for formatting
    try {
        Write-Host "   Checking code formatting..." -ForegroundColor Gray
        $fmtOutput = & cargo fmt -- --check 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Code formatting is correct" -ForegroundColor Green
        } else {
            Write-Host "⚠️  Code formatting issues found" -ForegroundColor Yellow
            if ($Verbose) {
                Write-Host "     Run 'cargo fmt' to fix formatting" -ForegroundColor Cyan
            }
        }
    } catch {
        Write-Host "⚠️  Formatter not available" -ForegroundColor Yellow
    }
}

# Function to generate summary report
function Write-Summary {
    param($results)

    Write-Host "`n=====================================" -ForegroundColor Cyan
    Write-Host "COMPILATION CHECK SUMMARY" -ForegroundColor Cyan
    Write-Host "=====================================" -ForegroundColor Cyan

    $passed = 0
    $total = 0

    $results.GetEnumerator() | ForEach-Object {
        $total++
        $status = if ($_.Value) { "✅ PASS"; $passed++ } else { "❌ FAIL" }
        $color = if ($_.Value) { "Green" } else { "Red" }
        Write-Host "$status $($_.Key)" -ForegroundColor $color
    }

    Write-Host "`nOverall Result: $passed/$total checks passed" -ForegroundColor $(if ($passed -eq $total) { "Green" } else { "Yellow" })

    if ($passed -eq $total) {
        Write-Host "`n🎉 All checks passed! RustRecon is ready for use." -ForegroundColor Green
    } else {
        Write-Host "`n⚠️  Some checks failed. Please review the issues above." -ForegroundColor Yellow
    }

    # Additional recommendations
    Write-Host "`n💡 Recommendations:" -ForegroundColor Cyan
    if ($TestCache -and $results["Cache System"]) {
        Write-Host "   - Cache system is working, enjoy faster repeated scans!" -ForegroundColor Gray
    }
    if (!$results["Build Success"]) {
        Write-Host "   - Fix compilation errors before proceeding" -ForegroundColor Gray
    }
    if (!$results["Tests"]) {
        Write-Host "   - Run tests individually to identify specific failures" -ForegroundColor Gray
    }
}

# Main execution
try {
    $results = @{}

    # Run checks
    $results["Prerequisites"] = Test-Prerequisites
    if (!$results["Prerequisites"]) {
        Write-Host "`n❌ Prerequisites not met. Aborting." -ForegroundColor Red
        exit 1
    }

    Test-Dependencies
    $results["Build Success"] = Invoke-CleanBuild

    if ($results["Build Success"]) {
        $results["Tests"] = Invoke-Tests
        $results["Executable"] = Test-Executable

        if ($TestCache) {
            $results["Cache System"] = Test-CacheSystem
        }

        Test-CodeQuality
    } else {
        $results["Tests"] = $false
        $results["Executable"] = $false
        $results["Cache System"] = $false
    }

    # Generate summary
    Write-Summary $results

} catch {
    Write-Host "`n❌ Unexpected error during compilation check: $_" -ForegroundColor Red
    Write-Host "Stack trace:" -ForegroundColor Gray
    Write-Host $_.ScriptStackTrace -ForegroundColor Gray
    exit 1
}

Write-Host "`nScript completed. Press any key to exit..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
