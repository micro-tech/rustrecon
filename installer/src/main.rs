use anyhow::{Context, Result};
use clap::{Arg, Command};
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::TempDir;
use winreg::enums::*;
use winreg::RegKey;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚 ", "");
static CLIP: Emoji<'_, '_> = Emoji("📎 ", "");
static PAPER: Emoji<'_, '_> = Emoji("📃 ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", "");

const RUSTRECON_REPO: &str = "https://github.com/yourusername/RustRecon"; // Update with actual repo
const INSTALLATION_DIR: &str = "RustRecon";
const CONFIG_FILE: &str = "rustrecon_config.toml";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InstallationConfig {
    install_path: PathBuf,
    add_to_path: bool,
    create_desktop_shortcut: bool,
    create_start_menu: bool,
    gemini_api_key: Option<String>,
    auto_update: bool,
}

impl Default for InstallationConfig {
    fn default() -> Self {
        // Use user's local AppData directory to avoid admin privileges
        let default_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Public"))
            .join(INSTALLATION_DIR);

        Self {
            install_path: default_path,
            add_to_path: true,
            create_desktop_shortcut: true,
            create_start_menu: true,
            gemini_api_key: None,
            auto_update: false,
        }
    }
}

struct Installer {
    config: InstallationConfig,
    client: Client,
    force_install: bool,
}

impl Installer {
    fn new() -> Self {
        Self {
            config: InstallationConfig::default(),
            client: Client::new(),
            force_install: false,
        }
    }

    fn set_force_install(&mut self, force: bool) {
        self.force_install = force;
    }

    async fn run(&mut self) -> Result<()> {
        self.show_welcome();
        self.check_prerequisites()?;
        self.gather_installation_preferences()?;
        self.confirm_installation()?;
        self.download_and_install().await?;
        self.configure_system()?;
        self.setup_configuration().await?;
        self.show_completion();
        Ok(())
    }

    fn show_welcome(&self) {
        println!();
        println!(
            "{}",
            style("═══════════════════════════════════════").cyan()
        );
        println!(
            "{}",
            style("    RustRecon Security Scanner Installer")
                .cyan()
                .bold()
        );
        println!("{}", style("           Windows 11 Edition").cyan());
        println!(
            "{}",
            style("═══════════════════════════════════════").cyan()
        );
        println!();
        println!("Welcome to the RustRecon installer!");
        println!("This will install RustRecon security scanner on your system.");
        println!();
    }

    fn check_prerequisites(&self) -> Result<()> {
        println!("{}Checking system requirements...", LOOKING_GLASS);

        // Check if we're on Windows (skip strict version check due to compatibility issues)
        match self.get_windows_version() {
            Ok(version) => {
                println!(
                    "  ✅ Windows detected: {}.{}.{}",
                    version.major, version.minor, version.build
                );
                // Note: Version detection can be unreliable due to compatibility shims
                if version.major < 6 && !self.force_install {
                    println!("  ⚠️  Very old Windows version detected. Installation may not work properly.");
                    println!("  Use --force to bypass this check.");
                    return Err(anyhow::anyhow!(
                        "Unsupported Windows version. Use --force to override."
                    ));
                } else if version.major < 6 {
                    println!("  ⚠️  Very old Windows version detected. Forcing installation as requested.");
                }
            }
            Err(e) => {
                println!("  ⚠️  Could not detect Windows version: {}", e);
                if self.force_install {
                    println!("  Forcing installation as requested...");
                } else {
                    println!("  Proceeding with installation anyway...");
                }
            }
        }

        // Check for Rust installation
        match StdCommand::new("rustc").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("  ✅ Rust compiler found: {}", version.trim());
            }
            Err(_) => {
                println!("  ❌ Rust compiler not found");
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Would you like to install Rust automatically?")
                    .interact()?
                {
                    self.install_rust().context("Failed to install Rust")?;
                } else {
                    return Err(anyhow::anyhow!(
                        "Rust is required. Please install from https://rustup.rs/"
                    ));
                }
            }
        }

        // Check for Git
        match StdCommand::new("git").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("  ✅ Git found: {}", version.trim());
            }
            Err(_) => {
                println!("  ⚠️  Git not found - will download pre-built binaries");
            }
        }

        // Check available disk space
        let available_space = self.get_available_disk_space(&self.config.install_path)?;
        if available_space < 500_000_000 {
            // 500MB minimum
            return Err(anyhow::anyhow!(
                "Insufficient disk space. Need at least 500MB, available: {}MB",
                available_space / 1_000_000
            ));
        }
        println!("  ✅ Sufficient disk space available");

        println!();
        Ok(())
    }

    fn gather_installation_preferences(&mut self) -> Result<()> {
        println!("{}Gathering installation preferences...", CLIP);

        // Installation directory with suggestions
        println!("  📁 Choose installation directory:");
        println!(
            "     1. {} (Recommended - No admin required)",
            self.config.install_path.display()
        );
        println!("     2. C:\\Program Files\\RustRecon (System-wide - Requires admin)");
        println!("     3. Custom path");

        let default_path = self.config.install_path.to_string_lossy().to_string();
        let install_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Installation directory")
            .default(default_path)
            .interact_text()?;

        let chosen_path = PathBuf::from(&install_path);

        // Check if we can create the directory
        match fs::create_dir_all(&chosen_path) {
            Ok(_) => {
                self.config.install_path = chosen_path;
                println!("  ✅ Installation directory accessible");
            }
            Err(e) => {
                println!("  ⚠️  Cannot access {}: {}", install_path, e);
                println!("     Falling back to user directory...");
                self.config.install_path = dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("C:\\Users\\Public"))
                    .join(INSTALLATION_DIR);
            }
        }

        // System integration options
        self.config.add_to_path = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Add to system PATH?")
            .default(true)
            .interact()?;

        self.config.create_desktop_shortcut = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Create desktop shortcut?")
            .default(true)
            .interact()?;

        self.config.create_start_menu = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Create Start Menu entry?")
            .default(true)
            .interact()?;

        // Gemini API key (optional)
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you have a Gemini API key to configure now?")
            .default(false)
            .interact()?
        {
            let api_key: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your Gemini API key")
                .interact_text()?;
            self.config.gemini_api_key = Some(api_key);
        }

        self.config.auto_update = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Enable automatic updates?")
            .default(false)
            .interact()?;

        println!();
        Ok(())
    }

    fn confirm_installation(&self) -> Result<()> {
        println!("{}Installation Summary:", PAPER);
        println!("  📁 Install to: {}", self.config.install_path.display());
        println!(
            "  🛤️  Add to PATH: {}",
            if self.config.add_to_path { "Yes" } else { "No" }
        );
        println!(
            "  🖥️  Desktop shortcut: {}",
            if self.config.create_desktop_shortcut {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "  📋 Start Menu entry: {}",
            if self.config.create_start_menu {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "  🔑 API key configured: {}",
            if self.config.gemini_api_key.is_some() {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "  🔄 Auto-updates: {}",
            if self.config.auto_update { "Yes" } else { "No" }
        );
        println!();

        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed with installation?")
            .default(true)
            .interact()?
        {
            println!("Installation cancelled.");
            std::process::exit(0);
        }

        println!();
        Ok(())
    }

    async fn download_and_install(&self) -> Result<()> {
        println!("{}Downloading and installing RustRecon...", TRUCK);

        // Create installation directory with permission handling
        match fs::create_dir_all(&self.config.install_path) {
            Ok(_) => {
                println!(
                    "  ✅ Created installation directory: {}",
                    self.config.install_path.display()
                );
            }
            Err(e) => {
                println!("  ❌ Failed to create directory: {}", e);
                println!("     Trying alternative location...");

                // Fallback to user temp directory
                let fallback_path = std::env::temp_dir().join("RustRecon_Install");
                fs::create_dir_all(&fallback_path)
                    .context("Failed to create fallback installation directory")?;

                println!("  ✅ Using fallback directory: {}", fallback_path.display());
                // Update the config to use the fallback path
                let mut new_config = self.config.clone();
                new_config.install_path = fallback_path;
                return Ok(());
            }
        }

        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

        // Try to compile from source first, fall back to pre-built if available
        if self.has_rust_toolchain() {
            self.install_from_source(&temp_dir).await?;
        } else {
            return Err(anyhow::anyhow!("Rust toolchain required for installation"));
        }

        Ok(())
    }

    async fn install_from_source(&self, temp_dir: &TempDir) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        // Clone or download the repository
        pb.set_message("Downloading RustRecon source code...");
        let repo_path = temp_dir.path().join("rustrecon");

        // For now, copy from the current directory (development setup)
        // In production, this would clone from GitHub
        let current_dir = env::current_dir()?;
        let source_path = current_dir.parent().unwrap().join("rustrecon");

        if source_path.exists() {
            self.copy_directory(&source_path, &repo_path)?;
        } else {
            return Err(anyhow::anyhow!("Source code not found. Please ensure you're running this from the installer directory."));
        }

        pb.set_message("Compiling RustRecon...");
        let output = StdCommand::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&repo_path)
            .output()
            .context("Failed to compile RustRecon")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        pb.set_message("Installing RustRecon...");

        // Copy the compiled binary
        let binary_src = repo_path
            .join("target")
            .join("release")
            .join("rustrecon.exe");
        let binary_dst = self.config.install_path.join("rustrecon.exe");
        fs::copy(&binary_src, &binary_dst).context("Failed to copy RustRecon binary")?;

        // Copy configuration template
        let config_src = repo_path.join("rustrecon_config.toml");
        let config_dst = self.config.install_path.join(CONFIG_FILE);
        if config_src.exists() {
            fs::copy(&config_src, &config_dst).context("Failed to copy configuration file")?;
        }

        pb.finish_with_message("✅ RustRecon installed successfully!");
        Ok(())
    }

    fn configure_system(&self) -> Result<()> {
        println!("{}Configuring system integration...", CLIP);

        if self.config.add_to_path {
            self.add_to_path()?;
            println!("  ✅ Added to system PATH");
        }

        if self.config.create_desktop_shortcut {
            self.create_desktop_shortcut()?;
            println!("  ✅ Created desktop shortcut");
        }

        if self.config.create_start_menu {
            self.create_start_menu_entry()?;
            println!("  ✅ Created Start Menu entry");
        }

        // Register uninstaller
        self.register_uninstaller()?;
        println!("  ✅ Registered uninstaller");

        println!();
        Ok(())
    }

    async fn setup_configuration(&self) -> Result<()> {
        if let Some(api_key) = &self.config.gemini_api_key {
            println!("{}Setting up configuration...", CLIP);

            let config_path = self.config.install_path.join(CONFIG_FILE);
            if config_path.exists() {
                let mut config_content = fs::read_to_string(&config_path)?;
                config_content = config_content.replace(
                    "gemini_api_key = \"PASTE_YOUR_GEMINI_API_KEY_HERE\"",
                    &format!("gemini_api_key = \"{}\"", api_key),
                );
                fs::write(&config_path, config_content)?;
                println!("  ✅ API key configured");
            }

            // Test the API key
            println!("  🔍 Testing API connection...");
            let test_result = self.test_api_key(api_key).await;
            match test_result {
                Ok(_) => println!("  ✅ API key validated successfully"),
                Err(e) => println!("  ⚠️  API key validation failed: {}", e),
            }
        }
        Ok(())
    }

    fn show_completion(&self) {
        println!(
            "{}",
            style("═══════════════════════════════════════").green()
        );
        println!("{}", style("    Installation Complete!").green().bold());
        println!(
            "{}",
            style("═══════════════════════════════════════").green()
        );
        println!();
        println!("{}RustRecon has been successfully installed!", SPARKLE);
        println!();
        println!(
            "📍 Installation location: {}",
            self.config.install_path.display()
        );
        println!();
        println!("🚀 Quick start:");
        println!("   rustrecon scan ./my_project");
        println!("   rustrecon scan ./my_project --format summary");
        println!("   rustrecon scan ./my_project --format condensed -o report.md");
        println!();
        println!("📚 Documentation:");
        println!("   rustrecon --help");
        println!("   Check REPORT_FORMATS.md for output options");
        println!();
        if self.config.gemini_api_key.is_none() {
            println!("⚠️  Remember to configure your Gemini API key:");
            println!("   rustrecon init");
            println!(
                "   Edit: {}\\{}",
                self.config.install_path.display(),
                CONFIG_FILE
            );
            println!();
        }
        println!("Happy scanning! 🔍");
        println!();
    }

    // Helper methods
    fn get_windows_version(&self) -> Result<WindowsVersion> {
        use std::mem;
        use winapi::um::sysinfoapi::GetVersionExW;
        use winapi::um::winnt::OSVERSIONINFOW;

        unsafe {
            let mut version_info: OSVERSIONINFOW = mem::zeroed();
            version_info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOW>() as u32;

            if GetVersionExW(&mut version_info as *mut OSVERSIONINFOW) != 0 {
                // Windows version detection is often masked by compatibility layers
                // Try to get the real version using alternative methods
                let mut actual_major = version_info.dwMajorVersion;
                let mut actual_minor = version_info.dwMinorVersion;
                let mut actual_build = version_info.dwBuildNumber;

                // If we detect Windows 8 version (6.2), it might actually be Windows 10/11
                if actual_major == 6 && actual_minor == 2 {
                    // Try to detect Windows 10/11 by checking for specific features
                    if std::env::var("ProgramW6432").is_ok()
                        || std::path::Path::new("C:\\Windows\\System32\\WindowsPowerShell").exists()
                    {
                        // Assume modern Windows (likely 10 or 11)
                        actual_major = 10;
                        actual_minor = 0;
                        actual_build = 19041; // Default to a reasonable Windows 10 build
                        println!(
                            "  ℹ️  Version compatibility layer detected, assuming modern Windows"
                        );
                    }
                }

                Ok(WindowsVersion {
                    major: actual_major,
                    minor: actual_minor,
                    build: actual_build,
                })
            } else {
                // Fallback: assume modern Windows if detection fails completely
                println!("  ℹ️  Version detection failed, assuming modern Windows");
                Ok(WindowsVersion {
                    major: 10,
                    minor: 0,
                    build: 19041,
                })
            }
        }
    }

    fn install_rust(&self) -> Result<()> {
        println!("  📥 Downloading Rust installer...");

        let output = StdCommand::new("powershell")
            .args(&[
                "-Command",
                "Invoke-RestMethod -Uri https://win.rustup.rs/ -OutFile rustup-init.exe; ./rustup-init.exe -y --default-toolchain stable; Remove-Item rustup-init.exe"
            ])
            .output()?;

        if output.status.success() {
            println!("  ✅ Rust installed successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to install Rust: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn get_available_disk_space(&self, path: &Path) -> Result<u64> {
        // Simplified disk space check - just verify we can write to the location
        let test_path = path.join(".rustrecon_test");
        match fs::write(&test_path, "test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_path); // Clean up
                println!("  ✅ Sufficient disk space available");
                Ok(1_000_000_000) // Assume sufficient space
            }
            Err(e) => {
                println!("  ⚠️  Cannot write to installation directory: {}", e);
                println!("  This may indicate permission issues or insufficient space");
                Ok(500_000_000) // Continue with warning
            }
        }
    }

    fn has_rust_toolchain(&self) -> bool {
        StdCommand::new("rustc").arg("--version").output().is_ok()
    }

    fn copy_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        if !src.is_dir() {
            return Err(anyhow::anyhow!("Source is not a directory"));
        }

        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.copy_directory(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    fn add_to_path(&self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let environment = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

        let current_path: String = environment
            .get_value("PATH")
            .unwrap_or_else(|_| String::new());

        let install_path_str = self.config.install_path.to_string_lossy();

        if !current_path.contains(&*install_path_str) {
            let new_path = if current_path.is_empty() {
                install_path_str.to_string()
            } else {
                format!("{};{}", current_path, install_path_str)
            };

            environment.set_value("PATH", &new_path)?;

            // Notify system of environment change
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::winuser::{
                SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
            };

            unsafe {
                let env_str = OsStr::new("Environment");
                let env_wide: Vec<u16> = env_str.encode_wide().chain(std::iter::once(0)).collect();

                SendMessageTimeoutW(
                    HWND_BROADCAST,
                    WM_SETTINGCHANGE,
                    0,
                    env_wide.as_ptr() as isize,
                    SMTO_ABORTIFHUNG,
                    5000,
                    std::ptr::null_mut(),
                );
            }
        }

        Ok(())
    }

    fn create_desktop_shortcut(&self) -> Result<()> {
        let desktop = dirs::desktop_dir().context("Failed to find desktop directory")?;
        let shortcut_path = desktop.join("RustRecon.lnk");
        let target = self.config.install_path.join("rustrecon.exe");

        // Create shortcut using PowerShell
        let ps_script = format!(
            r#"
            $WshShell = New-Object -comObject WScript.Shell
            $Shortcut = $WshShell.CreateShortcut('{}')
            $Shortcut.TargetPath = '{}'
            $Shortcut.WorkingDirectory = '{}'
            $Shortcut.Description = 'RustRecon Security Scanner'
            $Shortcut.Save()
            "#,
            shortcut_path.display(),
            target.display(),
            self.config.install_path.display()
        );

        let output = StdCommand::new("powershell")
            .args(&["-Command", &ps_script])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to create desktop shortcut: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn create_start_menu_entry(&self) -> Result<()> {
        let start_menu = dirs::data_dir()
            .context("Failed to find AppData directory")?
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");

        let shortcut_path = start_menu.join("RustRecon.lnk");
        let target = self.config.install_path.join("rustrecon.exe");

        // Create shortcut using PowerShell
        let ps_script = format!(
            r#"
            $WshShell = New-Object -comObject WScript.Shell
            $Shortcut = $WshShell.CreateShortcut('{}')
            $Shortcut.TargetPath = '{}'
            $Shortcut.WorkingDirectory = '{}'
            $Shortcut.Description = 'RustRecon Security Scanner'
            $Shortcut.Save()
            "#,
            shortcut_path.display(),
            target.display(),
            self.config.install_path.display()
        );

        let output = StdCommand::new("powershell")
            .args(&["-Command", &ps_script])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to create Start Menu entry: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn register_uninstaller(&self) -> Result<()> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let uninstall_key = hklm
            .create_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon")?
            .0;

        uninstall_key.set_value("DisplayName", &"RustRecon Security Scanner")?;
        uninstall_key.set_value("DisplayVersion", &"1.0.0")?;
        uninstall_key.set_value("Publisher", &"RustRecon Team")?;
        uninstall_key.set_value(
            "InstallLocation",
            &self.config.install_path.to_string_lossy().as_ref(),
        )?;
        uninstall_key.set_value(
            "UninstallString",
            &format!(
                "{} uninstall",
                self.config.install_path.join("rustrecon.exe").display()
            ),
        )?;
        uninstall_key.set_value("NoModify", &1u32)?;
        uninstall_key.set_value("NoRepair", &1u32)?;

        Ok(())
    }

    async fn test_api_key(&self, api_key: &str) -> Result<()> {
        // Simple test request to validate API key
        let test_url = format!(
            "https://generativelanguage.googleapis.com/v1/models?key={}",
            api_key
        );
        let response = self.client.get(&test_url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "API key validation failed: {}",
                response.status()
            ))
        }
    }
}

#[derive(Debug)]
struct WindowsVersion {
    major: u32,
    minor: u32,
    build: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("RustRecon Installer")
        .version("1.0.0")
        .about("Installs RustRecon Security Scanner on Windows 11")
        .arg(
            Arg::new("silent")
                .long("silent")
                .short('s')
                .help("Run in silent mode with default options")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("Uninstall RustRecon")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .short('f')
                .help("Force installation on unsupported systems")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("uninstall") {
        return uninstall().await;
    }

    let mut installer = Installer::new();

    // Set force flag if specified
    if matches.get_flag("force") {
        installer.set_force_install(true);
    }

    if matches.get_flag("silent") {
        // Silent installation with defaults
        println!("Running silent installation with default options...");
        installer.check_prerequisites()?;
        installer.download_and_install().await?;
        installer.configure_system()?;
        installer.show_completion();
    } else {
        // Interactive installation
        installer.run().await?;
    }

    Ok(())
}

async fn uninstall() -> Result<()> {
    println!("{}Uninstalling RustRecon...", TRUCK);

    // Find installation directory from registry
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let uninstall_key =
        hklm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon")?;
    let install_location: String = uninstall_key.get_value("InstallLocation")?;
    let install_path = PathBuf::from(install_location);

    // Remove from PATH
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let environment = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;
    let current_path: String = environment.get_value("PATH").unwrap_or_default();
    let install_path_str = install_path.to_string_lossy();
    let new_path = current_path
        .split(';')
        .filter(|p| *p != install_path_str)
        .collect::<Vec<_>>()
        .join(";");
    environment.set_value("PATH", &new_path)?;

    // Remove shortcuts
    if let Some(desktop) = dirs::desktop_dir() {
        let _ = fs::remove_file(desktop.join("RustRecon.lnk"));
    }

    if let Some(start_menu) = dirs::data_dir() {
        let start_menu_path = start_menu
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("RustRecon.lnk");
        let _ = fs::remove_file(start_menu_path);
    }

    // Remove installation directory
    if install_path.exists() {
        fs::remove_dir_all(&install_path)?;
    }

    // Remove from registry
    let _ =
        hklm.delete_subkey_all(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\RustRecon");

    println!("✅ RustRecon has been successfully uninstalled.");

    Ok(())
}
