use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing Gemini API Connection...\n");

    // Read API key from config or environment
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| {
            // Try to read from config file
            let config_content = std::fs::read_to_string("rustrecon_config.toml")?;
            if let Some(start) = config_content.find("gemini_api_key = \"") {
                let key_start = start + 18;
                if let Some(end) = config_content[key_start..].find("\"") {
                    let key = &config_content[key_start..key_start + end];
                    if key != "PASTE_YOUR_ACTUAL_GEMINI_API_KEY_HERE" && !key.is_empty() {
                        return Ok(key.to_string());
                    }
                }
            }
            Err("API key not found")
        })
        .map_err(|_| "❌ API key not found. Please set GEMINI_API_KEY environment variable or configure rustrecon_config.toml")?;

    if api_key.starts_with("PASTE_") || api_key.len() < 20 {
        println!("❌ Please configure your real Gemini API key in rustrecon_config.toml");
        println!(
            "   Current key looks like a placeholder: {}",
            &api_key[..20.min(api_key.len())]
        );
        println!("\nTo get a free API key:");
        println!("1. Visit: https://aistudio.google.com");
        println!("2. Sign in with your Google account");
        println!("3. Click 'Get API key' -> 'Create API key'");
        println!("4. Copy the key to rustrecon_config.toml");
        return Ok(());
    }

    println!("✓ API key found ({}...)", &api_key[..8]);

    // Create HTTP client
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let test_request = json!({
        "contents": [
            {
                "parts": [
                    {"text": "Hello! Please respond with 'API test successful' to confirm the connection is working."}
                ]
            }
        ],
        "generationConfig": {
            "temperature": 0.1,
            "maxOutputTokens": 50
        }
    });

    println!("🚀 Sending test request to Gemini API...");

    match client.post(&url).json(&test_request).send().await {
        Ok(response) => {
            let status = response.status();
            println!("📡 Response status: {}", status);

            if status.is_success() {
                match response.text().await {
                    Ok(body) => {
                        println!("✅ API connection successful!");
                        println!("📋 Response preview:");

                        // Try to parse and pretty print the response
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(candidates) = json_val["candidates"].as_array() {
                                if let Some(first_candidate) = candidates.first() {
                                    if let Some(parts) =
                                        first_candidate["content"]["parts"].as_array()
                                    {
                                        if let Some(first_part) = parts.first() {
                                            if let Some(text) = first_part["text"].as_str() {
                                                println!("   Gemini says: \"{}\"", text.trim());
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("   Raw response: {}", &body[..200.min(body.len())]);
                        }

                        println!("\n🎉 Your Gemini API is configured correctly!");
                        println!("   You can now run: cargo run -- scan . -o report.md");
                    }
                    Err(e) => {
                        println!("❌ Failed to read response body: {}", e);
                    }
                }
            } else {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                println!("❌ API request failed!");
                println!("   Status: {}", status);
                println!("   Error: {}", error_body);

                if status == 403 {
                    println!("\n💡 This usually means:");
                    println!("   - Invalid API key");
                    println!("   - API key doesn't have proper permissions");
                    println!("   - Billing not enabled (but free tier should work)");
                } else if status == 429 {
                    println!("\n💡 Rate limit exceeded. Wait a moment and try again.");
                }
            }
        }
        Err(e) => {
            println!("❌ Network error: {}", e);
            println!("\n💡 Check your internet connection and try again.");
        }
    }

    Ok(())
}
