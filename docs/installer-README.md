# RustRecon Windows Installer

A comprehensive Rust-based installer for RustRecon Security Scanner on Windows 11.

## Features

- 🚀 **One-click installation** with interactive setup
- 🔧 **Automatic dependency detection** and installation
- 🛠️ **System integration** (PATH, shortcuts, Start Menu)
- 🔑 **API key configuration** during installation
- 🔄 **Auto-update support** (optional)
- ❌ **Clean uninstallation** with registry cleanup
- 🤖 **Silent installation** mode for automation

## Requirements

- **Windows 11** (build 22000 or later)
- **500MB** free disk space
- **Internet connection** for downloading dependencies
- **Administrator privileges** (for system-wide installation)

## Quick Start

### Building the Installer

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/RustRecon.git
   cd RustRecon/installer
   ```

2. **Build the installer:**
   ```bash
   # Using batch script
   build.bat
   
   # Or using PowerShell
   .\build.ps1
   
   # Or manually
   cargo build --release
   ```

3. **Run the installer:**
   ```bash
   ..\rustrecon-installer.exe
   ```

### Installation Options

#### Interactive Installation (Recommended)
```bash
rustrecon-installer.exe
```
- Guided setup with customization options
- Choose installation directory
- Configure system integration
- Set up API keys
- Test configuration

#### Silent Installation
```bash
rustrecon-installer.exe --silent
```
- Installs with default settings
- No user interaction required
- Perfect for deployment scripts

#### Uninstallation
```bash
rustrecon-installer.exe --uninstall
```
- Complete removal of RustRecon
- Cleans registry entries
- Removes shortcuts and PATH entries

## Installation Process

The installer performs the following steps:

1. **System Check**
   - Verifies Windows 11 compatibility
   - Checks available disk space
   - Detects existing Rust installation

2. **Dependency Installation** (if needed)
   - Downloads and installs Rust toolchain
   - Installs Git (optional, for source compilation)

3. **RustRecon Installation**
   - Downloads source code
   - Compiles RustRecon in release mode
   - Copies binary to installation directory

4. **System Integration**
   - Adds RustRecon to system PATH
   - Creates desktop shortcut
   - Adds Start Menu entry
   - Registers uninstaller

5. **Configuration Setup**
   - Generates default configuration file
   - Sets up API keys (if provided)
   - Tests API connectivity

## Configuration Options

During installation, you can customize:

| Option | Description | Default |
|--------|-------------|---------|
| **Installation Path** | Where RustRecon will be installed | `C:\Program Files\RustRecon` |
| **Add to PATH** | Make `rustrecon` command available globally | ✅ Yes |
| **Desktop Shortcut** | Create desktop icon | ✅ Yes |
| **Start Menu Entry** | Add to Start Menu | ✅ Yes |
| **Gemini API Key** | Configure API key during installation | ❌ Optional |
| **Auto-updates** | Enable automatic updates | ❌ No |

## Post-Installation

After successful installation:

### Verify Installation
```bash
rustrecon --help
```

### Quick Test
```bash
rustrecon test
```

### First Scan
```bash
rustrecon scan ./my_project
```

### Configuration
Edit configuration file at:
```
C:\Program Files\RustRecon\rustrecon_config.toml
```

## Troubleshooting

### Common Issues

**"Rust not found" Error:**
- The installer can automatically install Rust
- Or install manually from: https://rustup.rs/

**"Insufficient permissions" Error:**
- Run installer as Administrator
- Right-click → "Run as administrator"

**"API key validation failed":**
- Check your Gemini API key
- Verify internet connectivity
- Check API quotas and billing

**Installation hangs during compilation:**
- Check internet connection
- Ensure sufficient disk space
- Try running installer as Administrator

### Log Files

Installation logs are saved to:
```
%TEMP%\rustrecon-installer.log
```

### Manual Cleanup

If uninstallation fails, manually remove:
- Installation directory: `C:\Program Files\RustRecon`
- Registry entry: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon`
- PATH entry: Remove RustRecon path from system PATH
- Shortcuts: Desktop and Start Menu

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy
```

### Project Structure

```
installer/
├── src/
│   └── main.rs          # Main installer logic
├── Cargo.toml           # Project dependencies
├── build.bat            # Windows batch build script
├── build.ps1            # PowerShell build script
└── README.md            # This file
```

### Dependencies

- **anyhow** - Error handling
- **tokio** - Async runtime
- **reqwest** - HTTP client
- **clap** - Command line parsing
- **indicatif** - Progress bars
- **dialoguer** - Interactive prompts
- **winreg** - Windows registry access
- **dirs** - Standard directory locations

## Security

### Code Signing

For production deployment, sign the installer:

```powershell
# Using signtool (Windows SDK)
signtool sign /f certificate.pfx /p password rustrecon-installer.exe

# Using PowerShell (with certificate in store)
Set-AuthenticodeSignature -FilePath "rustrecon-installer.exe" -Certificate $cert
```

### Verification

Users can verify the installer signature:

```powershell
Get-AuthenticodeSignature rustrecon-installer.exe
```

### Antivirus

Some antivirus software may flag the installer. To minimize false positives:
- Sign the executable with a trusted certificate
- Submit to antivirus vendors for whitelisting
- Use established distribution channels

## Distribution

### Release Packaging

1. Build signed installer
2. Create checksums:
   ```bash
   certutil -hashfile rustrecon-installer.exe SHA256
   ```
3. Upload to GitHub releases
4. Update download links

### Version Management

Update version numbers in:
- `Cargo.toml` - Package version
- `main.rs` - Display version
- Registry entries - Uninstaller version

## Support

- 📚 **Documentation**: [GitHub Wiki](https://github.com/yourusername/RustRecon/wiki)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/yourusername/RustRecon/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/yourusername/RustRecon/discussions)

## License

This installer is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

---

**Happy scanning with RustRecon! 🔍**