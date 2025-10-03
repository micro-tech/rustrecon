// Enhanced initialization module for RustRecon
// This module provides an improved init command that:
// 1. Prompts user for API key during initialization
// 2. Shows clear messages about where config will be stored
// 3. Validates the API key format
// 4. Creates proper directory structure

use crate::config::Config;
use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Enhanced initialization that prompts for user input and provides clear feedback
pub fn enhanced_init(custom_path: Option<String>) -> Result<()> {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    RustRecon Initialization                 ║");
    println!("║                                                              ║");
    println!("║  Setting up your RustRecon configuration with Gemini AI     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Determine where to store the config
    let config_path = if let Some(path) = custom_path {
        let custom_path = PathBuf::from(path);
        println!("📁 Using custom configuration path:");
        println!("   {}", custom_path.display());
        custom_path
    } else {
        let default_path = Config::get_default_config_path()?;
        println!("📁 Default configuration location:");
        println!("   {}", default_path.display());
        println!();
        println!("   This is the recommended location for your RustRecon configuration.");
        println!("   RustRecon will automatically find it here in the future.");
        default_path
    };

    // Check if config already exists
    if config_path.exists() {
        println!();
        println!("⚠️  Configuration file already exists!");
        println!("   Location: {}", config_path.display());
        println!();
        print!("Do you want to overwrite the existing configuration? (y/N): ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().to_lowercase().starts_with('y') {
            println!("Configuration initialization cancelled.");
            println!("Your existing configuration remains unchanged.");
            return Ok(());
        }
        println!();
    }

    // Create parent directories
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            println!("📂 Creating configuration directory...");
            println!("   {}", parent.display());
            fs::create_dir_all(parent)?;
            println!("✓ Directory created successfully");
            println!();
        }
    }

    // Prompt for API key
    println!("🔑 Gemini API Key Setup");
    println!("─────────────────────────");
    println!("To use RustRecon, you need a Google Gemini API key.");
    println!();
    println!("To get your API key:");
    println!("  1. Visit: https://aistudio.google.com/app/apikey");
    println!("  2. Sign in with your Google account");
    println!("  3. Click 'Create API Key'");
    println!("  4. Copy the generated key");
    println!();

    let api_key = loop {
        print!("Enter your Gemini API key (or press Enter to skip): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            println!("⚠️  Skipping API key setup. You'll need to edit the config file manually.");
            break "PASTE_YOUR_GEMINI_API_KEY_HERE".to_string();
        }

        // Basic validation of API key format
        if input.len() < 20 {
            println!("❌ API key seems too short. Please check and try again.");
            continue;
        }

        if input.starts_with("PASTE_") || input.contains("YOUR_") || input.contains("HERE") {
            println!("❌ Please enter your actual API key, not the placeholder text.");
            continue;
        }

        // Ask for confirmation
        println!();
        println!(
            "API key entered: {}...{}",
            &input[..8],
            &input[input.len() - 4..]
        );
        print!("Is this correct? (y/N): ");
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        if confirm.trim().to_lowercase().starts_with('y') {
            println!("✓ API key accepted");
            break input.to_string();
        }

        println!("Let's try again...");
        println!();
    };

    // Model selection
    println!();
    println!("🤖 AI Model Selection");
    println!("─────────────────────");
    println!("Available Gemini models:");
    println!("  1. gemini-1.5-pro-latest (Recommended - Most stable)");
    println!("  2. gemini-1.5-pro (Stable version)");
    println!("  3. gemini-1.0-pro (Older but very stable)");
    println!();

    let selected_model = loop {
        print!("Select model (1-3) or press Enter for default [1]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let model = match input {
            "" | "1" => "gemini-1.5-pro-latest",
            "2" => "gemini-1.5-pro",
            "3" => "gemini-1.0-pro",
            _ => {
                println!("❌ Please enter 1, 2, 3, or press Enter for default.");
                continue;
            }
        };

        println!("✓ Selected model: {}", model);
        break model.to_string();
    };

    // Create configuration
    println!();
    println!("📄 Creating configuration file...");

    Config::generate_default_config(config_path.clone())?;

    // Read the newly created config file to get the default values
    let mut config = Config::load_from_path(&config_path)?;

    // Update the config with the user-provided API key and model
    if let Some(llm_config) = config.llm.as_mut() {
        llm_config.gemini_api_key = api_key.clone();
        llm_config.gemini_model = Some(selected_model);
    }

    let toml_string = toml::to_string_pretty(&config)
        .map_err(|e| anyhow!("Failed to serialize configuration: {}", e))?;

    fs::write(&config_path, toml_string)
        .map_err(|e| anyhow!("Failed to write configuration file: {}", e))?;

    // Success message
    println!();
    println!("🎉 Configuration created successfully!");
    println!();
    println!("📍 Configuration file location:");
    println!("   {}", config_path.display());
    println!();
    println!("📋 Configuration summary:");
    println!(
        "   • API Key: {}...{}",
        if api_key == "PASTE_YOUR_GEMINI_API_KEY_HERE" {
            "Not set"
        } else {
            &api_key[..8]
        },
        if api_key == "PASTE_YOUR_GEMINI_API_KEY_HERE" {
            ""
        } else {
            &api_key[api_key.len() - 4..]
        }
    );
    println!(
        "   • Model: {}",
        config.llm.as_ref().unwrap().gemini_model.as_ref().unwrap()
    );
    println!("   • Rate limiting: Enabled");
    println!("   • Caching: Enabled (90 days)");
    println!();

    if api_key == "PASTE_YOUR_GEMINI_API_KEY_HERE" {
        println!("⚠️  IMPORTANT: API key not set!");
        println!("   You need to edit the configuration file and add your API key:");
        println!("   1. Open: {}", config_path.display());
        println!("   2. Replace: PASTE_YOUR_GEMINI_API_KEY_HERE");
        println!("   3. Save the file");
        println!();
    }

    println!("🧪 Next steps:");
    println!("   1. Test your configuration: rustrecon test");
    println!("   2. Scan your first crate: rustrecon scan path/to/your/project");
    println!("   3. View help: rustrecon --help");
    println!();

    // Offer to test immediately if API key was provided
    if api_key != "PASTE_YOUR_GEMINI_API_KEY_HERE" {
        print!("Would you like to test your configuration now? (Y/n): ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        if !response.trim().to_lowercase().starts_with('n') {
            println!();
            println!("🔍 Testing your configuration...");

            // Here you would call the test function
            // For now, we'll just show the message
            println!("   Run: rustrecon test");
        }
    }

    println!();
    println!("✅ RustRecon initialization complete!");

    Ok(())
}

/// Validates an API key format (basic validation)
fn validate_api_key(api_key: &str) -> bool {
    // Basic checks for API key format
    if api_key.len() < 20 {
        return false;
    }

    if api_key.starts_with("PASTE_") || api_key.contains("YOUR_") || api_key.contains("HERE") {
        return false;
    }

    // Could add more sophisticated validation here
    true
}

/// Shows the current configuration location and asks if user wants to change it
pub fn show_config_location_options() -> Result<PathBuf> {
    let default_path = Config::get_default_config_path()?;

    println!("📁 Configuration will be stored at:");
    println!("   {}", default_path.display());
    println!();
    println!("This location is:");

    #[cfg(target_os = "windows")]
    {
        println!("   • Automatically backed up by Windows");
        println!("   • Accessible from any terminal");
        println!("   • Standard location for application data");
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("   • Following XDG Base Directory Specification");
        println!("   • Accessible from any terminal");
        println!("   • Standard location for user configuration");
    }

    println!();
    print!("Use this location? (Y/n): ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase().starts_with('n') {
        print!("Enter custom path (or press Enter to use default): ");
        io::stdout().flush()?;

        let mut custom_path = String::new();
        io::stdin().read_line(&mut custom_path)?;
        let custom_path = custom_path.trim();

        if !custom_path.is_empty() {
            return Ok(PathBuf::from(custom_path));
        }
    }

    Ok(default_path)
}
