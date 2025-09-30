# RustRecon Deployment Readiness Guide

## Overview
This document provides a comprehensive checklist to ensure RustRecon is properly prepared for installer deployment. The installer pulls from the main branch and builds the application, so it's critical that all changes are properly merged and tested.

## Pre-Deployment Checklist

### 🔄 Git Repository Status

#### Branch Management
- [ ] **Check current branch**: Ensure you're on the main branch
- [ ] **Merge all feature branches**: All development branches should be merged into main
- [ ] **Delete merged branches**: Clean up old branches to avoid confusion
- [ ] **Sync with remote**: Pull latest changes and push local commits

#### Commands to Run:
```bash
# Check current status
git status
git branch -a

# Switch to main and update
git checkout main
git pull origin main

# Check for unmerged branches
git branch --no-merged main

# If unmerged branches exist, merge them:
git merge [branch-name] --no-ff -m "Merge [branch-name] into main"

# Push all changes
git push origin main
```

### ⚙️ Configuration Files

#### Critical Configuration Check
- [ ] **rustrecon_config.toml**: Verify correct Gemini model is specified
- [ ] **Cargo.toml**: Ensure version numbers are correct
- [ ] **Dependencies**: All dependencies are properly specified

#### Current Configuration Status:
```toml
[llm]
gemini_api_key = "AIzaSyCjIsIuhjbxXY9HwlbSWCS8uUjD68aFEVQ"
gemini_api_endpoint = "https://generativelanguage.googleapis.com"
gemini_model = "gemini-1.5-pro-latest"  # ✅ Fixed from problematic 1.5-flash
temperature = 0.699999988079071
max_tokens = 1024
```

### 🏗️ Build Verification

#### Local Build Test
- [ ] **Clean build**: `cargo clean && cargo build --release`
- [ ] **Run tests**: `cargo test`
- [ ] **Functionality test**: Test core features locally
- [ ] **Configuration loading**: Verify config file is read correctly

#### Build Commands:
```bash
# Clean previous builds
cargo clean

# Build in release mode (same as installer)
cargo build --release

# Run tests
cargo test

# Test the built executable
.\target\release\rustrecon.exe --help
```

### 🧪 Testing Requirements

#### Functional Testing
- [ ] **API connectivity**: Test Gemini API connection
- [ ] **Configuration loading**: Verify settings are loaded correctly
- [ ] **Core functionality**: Test main features work as expected
- [ ] **Error handling**: Verify graceful error handling

#### Test Commands:
```bash
# Test API connectivity (if you have test endpoint)
.\target\release\rustrecon.exe --test-api

# Test configuration loading
.\target\release\rustrecon.exe --show-config

# Run with debug output to verify functionality
.\target\release\rustrecon.exe --debug [other-args]
```

### 📁 File Structure Verification

#### Required Files Present
- [ ] **rustrecon.exe** (built executable)
- [ ] **rustrecon_config.toml** (configuration)
- [ ] **Cargo.toml** and **Cargo.lock** (dependencies)
- [ ] **src/** directory with all source code
- [ ] **README.md** and documentation files

#### File Structure Should Look Like:
```
RustRecon/
├── Cargo.toml
├── Cargo.lock
├── rustrecon_config.toml
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── database.rs
│   └── ...
├── target/release/
│   └── rustrecon.exe
├── README.md
├── installer/
└── ...
```

### 🔒 Security Checks

#### Sensitive Information
- [ ] **API keys**: Ensure no hardcoded keys in source code
- [ ] **Credentials**: No credentials committed to repository
- [ ] **Configuration**: Default config uses placeholder values
- [ ] **.gitignore**: Sensitive files are properly ignored

#### Security Verification:
```bash
# Check for potential secrets in codebase
grep -r "api_key\|password\|secret\|token" src/ --exclude-dir=.git
git log --oneline -p | grep -i "api_key\|password\|secret"
```

## Common Issues and Solutions

### Issue 1: Gemini Model Not Found Error
**Problem**: `models/gemini-1.5-flash is not found for API version v1beta`

**Solution**:
1. Update `rustrecon_config.toml` to use stable model:
   ```toml
   gemini_model = "gemini-1.5-pro-latest"
   ```
2. Rebuild application: `cargo build --release`
3. Test the fix locally before deploying

### Issue 2: Unmerged Branches
**Problem**: Feature branches not merged into main

**Solution**:
1. List unmerged branches: `git branch --no-merged main`
2. Merge each branch: `git merge [branch-name]`
3. Resolve conflicts if any
4. Push changes: `git push origin main`

### Issue 3: Build Failures
**Problem**: Compilation errors during build

**Solution**:
1. Clean build cache: `cargo clean`
2. Update dependencies: `cargo update`
3. Fix any compilation errors in source code
4. Rebuild: `cargo build --release`

### Issue 4: Configuration Not Loading
**Problem**: Application not reading config file

**Solution**:
1. Verify config file path and permissions
2. Check config file syntax (valid TOML)
3. Add debug logging to config loading code
4. Test with verbose output

## Deployment Scripts

### Automated Check Script
Use the provided scripts to automate checks:
- **PowerShell**: `.\check_and_merge_branches.ps1`
- **Batch**: `.\check_branches.bat`

### Script Usage:
```powershell
# Dry run to see what would be done
.\check_and_merge_branches.ps1 -DryRun

# Full check and merge with push
.\check_and_merge_branches.ps1 -Force -Push

# Interactive mode (recommended)
.\check_and_merge_branches.ps1
```

## Installer Integration

### How the Installer Works
1. **Pulls from main branch**: Installer always uses latest main
2. **Builds with cargo**: Uses `cargo build --release`
3. **Copies configuration**: Installs default config file
4. **Sets up environment**: Configures PATH and shortcuts

### Installer Requirements
- [ ] **Main branch is up-to-date**: All changes merged and pushed
- [ ] **Builds successfully**: `cargo build --release` works
- [ ] **Configuration is valid**: Config file format is correct
- [ ] **Dependencies resolved**: All external dependencies available

## Final Verification Steps

### Pre-Deployment Verification
1. **Run the check script**: `.\check_and_merge_branches.ps1`
2. **Build locally**: `cargo build --release`
3. **Test executable**: `.\target\release\rustrecon.exe --help`
4. **Verify configuration**: Check config loading works
5. **Push to main**: `git push origin main`

### Post-Deployment Testing
1. **Run installer**: Test the installer downloads and builds correctly
2. **Test installed version**: Verify installed application works
3. **Configuration test**: Ensure config is properly installed
4. **Feature test**: Test core functionality works

## Deployment Readiness Checklist

### ✅ Ready for Deployment When:
- [ ] All branches merged into main
- [ ] Main branch pushed to remote
- [ ] Configuration uses stable Gemini model
- [ ] Application builds successfully
- [ ] Local testing passes
- [ ] No hardcoded secrets in code
- [ ] Documentation is up to date

### ❌ NOT Ready if:
- [ ] Unmerged branches exist
- [ ] Build fails locally
- [ ] Configuration errors present
- [ ] API connectivity issues
- [ ] Secrets exposed in repository

## Support Information

### If Issues Persist:
1. **Check logs**: Look for error messages in build output
2. **Verify environment**: Ensure Rust toolchain is up to date
3. **Test dependencies**: Verify all external dependencies available
4. **Review recent changes**: Check what changed since last working version

### Rollback Strategy:
If deployment fails:
1. **Identify last working commit**: `git log --oneline`
2. **Revert problematic changes**: `git revert [commit-hash]`
3. **Push fix**: `git push origin main`
4. **Rebuild installer**: Let installer pull fixed version

---

**Document Version**: 1.0  
**Last Updated**: 2024-12-28  
**Status**: Ready for use  
**Next Review**: When making significant changes to deployment process