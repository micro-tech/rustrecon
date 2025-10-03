# RustRecon Quick Install Guide

Get RustRecon up and running in 5 minutes!

## 🚀 One-Command Installation

### For Most Users (Recommended)
```bash
# Download and run - uses user directory, no admin needed
rustrecon-installer.exe --silent
```

### If You Get Permission Errors
```bash
# Force installation with compatibility bypass
rustrecon-installer.exe --force --silent
```

### For System-Wide Installation (Advanced Users)
```bash
# Run as Administrator, then:
rustrecon-installer.exe
# Choose C:\Program Files\RustRecon when prompted
```

## ⚡ Quick Test

After installation:

```bash
# Test installation
rustrecon --help

# Configure API key (required for scanning)
rustrecon init
# Edit the config file and add your Gemini API key

# Test API connection
rustrecon test

# Your first scan
rustrecon scan . --format summary
```

## 🎯 What Gets Installed

**Default Location**: `%LOCALAPPDATA%\RustRecon` (e.g., `C:\Users\YourName\AppData\Local\RustRecon`)

**Files Created**:
- `rustrecon.exe` - Main scanner binary
- `rustrecon_config.toml` - Configuration file
- Desktop shortcut (optional)
- Start Menu entry (optional)

## 🔧 Common Issues & Quick Fixes

### "Windows 11 required" Error
```bash
# Your system is compatible, bypass the check:
rustrecon-installer.exe --force
```

### "Permission denied" or "Access denied"
```bash
# Install to user directory (no admin needed):
rustrecon-installer.exe --silent
```

### "Rust not found"
The installer will offer to install Rust automatically. Choose "Yes" when prompted.

### "API key validation failed"
1. Get a free API key from: https://aistudio.google.com/
2. Edit: `%LOCALAPPDATA%\RustRecon\rustrecon_config.toml`
3. Replace `YOUR_API_KEY_HERE` with your actual key
4. Test: `rustrecon test`

## 🔄 Installation Options

| Command | Description | Admin Required | Best For |
|---------|-------------|----------------|----------|
| `rustrecon-installer.exe` | Interactive setup | No* | First-time users |
| `rustrecon-installer.exe --silent` | Automatic install | No | Quick setup, scripts |
| `rustrecon-installer.exe --force` | Bypass compatibility | No | Older systems |

*Admin required only if installing to `C:\Program Files`

## 📊 Quick Scanning Examples

```bash
# One-line status check
rustrecon scan ./my_project --format summary

# Detailed security report  
rustrecon scan ./my_project --format condensed -o security_report.md

# Full comprehensive analysis
rustrecon scan ./my_project --format markdown -o full_report.md

# JSON for tools/automation
rustrecon scan ./my_project --format json -o results.json

# Code-only scan (skip dependencies)
rustrecon scan ./my_project --skip-dependencies
```

## ❌ Uninstall

```bash
# Complete removal
rustrecon-installer.exe --uninstall

# Or via Windows Settings:
# Settings > Apps > RustRecon Security Scanner > Uninstall
```

## 🆘 Need Help?

**Quick fixes for 90% of issues**:
1. Try `--force` flag for compatibility issues
2. Try `--silent` for permission issues  
3. Run as Administrator for system-wide install
4. Check your Gemini API key at https://aistudio.google.com/

**Still stuck?**
- Check `INSTALLATION.md` for detailed troubleshooting
- Look at `installer\README.md` for technical details
- Report issues at: https://github.com/yourusername/RustRecon/issues

---

**Total install time: ~2-5 minutes** ⏱️

**Happy scanning! 🔍**