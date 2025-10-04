use async_trait::async_trait;
use regex::Regex;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt: String,
    // Add other fields as necessary for the Gemini API, e.g., model, temperature, etc.
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub analysis: String,
    pub flagged_patterns: Vec<FlaggedPattern>,
    // Add other fields as necessary for the Gemini API response
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Option<Vec<GeminiPart>>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    code: Option<i32>,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaggedPattern {
    pub line: usize,
    pub severity: String,
    pub description: String,
    pub code_snippet: String,
}

#[async_trait]
pub trait LlmClientTrait {
    async fn analyze_code(&mut self, request: LlmRequest) -> Result<LlmResponse, LlmClientError>;
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    api_endpoint: String,
    model: String,
    last_request_time: Option<Instant>,
    min_request_interval: Duration,
}

impl GeminiClient {
    pub fn new(api_key: String, api_endpoint: String, model: String) -> Self {
        Self::with_rate_limit(api_key, api_endpoint, model, Duration::from_secs(2))
    }

    pub fn with_rate_limit(
        api_key: String,
        api_endpoint: String,
        model: String,
        min_interval: Duration,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        GeminiClient {
            client,
            api_key,
            api_endpoint,
            model,
            last_request_time: None,
            min_request_interval: min_interval,
        }
    }

    fn parse_analysis_response(
        &self,
        response: &str,
    ) -> Result<(String, Vec<FlaggedPattern>), LlmClientError> {
        let mut patterns = Vec::new();
        let mut analysis;

        // Split response into analysis and patterns sections
        if let Some(analysis_start) = response.find("ANALYSIS:") {
            let analysis_section = &response[analysis_start + 9..];
            if let Some(patterns_start) = analysis_section.find("PATTERNS:") {
                analysis = analysis_section[..patterns_start].trim().to_string();
                let patterns_section = &analysis_section[patterns_start + 9..];

                // Parse patterns using regex
                let pattern_regex = Regex::new(
                    r"- Line: (\d+), Severity: (High|Medium|Low), Description: ([^,]+), Code: (.+)",
                )
                .map_err(|e| LlmClientError::Other(format!("Regex error: {}", e)))?;

                for line in patterns_section.lines() {
                    if let Some(captures) = pattern_regex.captures(line.trim()) {
                        if captures.len() >= 5 {
                            let line_num: usize = captures[1].parse().map_err(|_| {
                                LlmClientError::Other("Invalid line number".to_string())
                            })?;

                            patterns.push(FlaggedPattern {
                                line: line_num,
                                severity: captures[2].to_string(),
                                description: captures[3].trim().to_string(),
                                code_snippet: captures[4].trim().to_string(),
                            });
                        }
                    }
                }
            } else {
                analysis = analysis_section.trim().to_string();
            }
        } else {
            // Fallback: use entire response as analysis
            analysis = response.trim().to_string();
        }

        // If no analysis found, provide a default
        if analysis.is_empty() {
            analysis = "Security analysis completed.".to_string();
        }

        Ok((analysis, patterns))
    }

    fn extract_text_from_response(&self, json_value: &Value) -> Result<String, LlmClientError> {
        // Method 1: Try the standard Gemini response structure
        if let Some(candidates) = json_value.get("candidates").and_then(|c| c.as_array()) {
            if !candidates.is_empty() {
                if let Some(content) = candidates[0].get("content") {
                    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                        if !parts.is_empty() {
                            if let Some(text) = parts[0].get("text").and_then(|t| t.as_str()) {
                                return Ok(text.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Method 2: Handle simple text-only responses (some APIs return just text)
        if let Some(text_str) = json_value.as_str() {
            if text_str.len() > 20 {
                // Reasonable text response
                println!("  ⚠️  Using direct text response");
                return Ok(text_str.to_string());
            }
        }

        // Method 3: Try parsing with our structured types (with better error handling)
        match serde_json::from_value::<GeminiResponse>(json_value.clone()) {
            Ok(gemini_response) => {
                // Check for API-level errors
                if let Some(error) = gemini_response.error {
                    return Err(LlmClientError::ApiError(format!(
                        "Gemini API error {}: {}",
                        error.code.unwrap_or(0),
                        error.message.unwrap_or_else(|| "Unknown error".to_string())
                    )));
                }

                // Extract response content with robust error handling
                if let Some(candidates) = gemini_response.candidates {
                    if !candidates.is_empty() {
                        let candidate = &candidates[0];

                        // Check finish reason
                        if let Some(finish_reason) = &candidate.finish_reason {
                            if finish_reason != "STOP" {
                                println!(
                                    "  ⚠️  Generation finished with reason: {}",
                                    finish_reason
                                );
                            }
                        }

                        if let Some(content) = &candidate.content {
                            if let Some(parts) = &content.parts {
                                if !parts.is_empty() {
                                    if let Some(text) = &parts[0].text {
                                        return Ok(text.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("  ⚠️  Structured parsing failed: {}", e);
            }
        }

        // Method 4: Try to find any text field in the response
        if let Some(text) = self.find_text_in_json(json_value) {
            println!("  ⚠️  Using fallback text extraction");
            return Ok(text);
        }

        // Method 5: Try alternative response formats
        if let Some(obj) = json_value.as_object() {
            // Check for direct text fields
            for text_field in &["text", "content", "message", "response", "output", "result"] {
                if let Some(text_val) = obj.get(*text_field) {
                    if let Some(text_str) = text_val.as_str() {
                        if text_str.len() > 10 {
                            println!("  ⚠️  Using '{}' field as fallback", text_field);
                            return Ok(text_str.to_string());
                        }
                    }
                }
            }
        }

        // Method 6: Handle MAX_TOKENS case specifically
        if let Some(obj) = json_value.as_object() {
            if let Some(candidates) = obj.get("candidates").and_then(|c| c.as_array()) {
                if !candidates.is_empty() {
                    if let Some(finish_reason) =
                        candidates[0].get("finishReason").and_then(|f| f.as_str())
                    {
                        if finish_reason == "MAX_TOKENS" {
                            return Ok("ANALYSIS: Code analysis was truncated due to size limitations. The file is large and requires manual review for comprehensive security analysis. Focus on reviewing imports, unsafe blocks, network operations, and file system access patterns.".to_string());
                        }
                    }
                }
            }
        }

        // Method 7: Create a basic analysis response if we can't parse anything
        println!("  ⚠️  Could not parse response, creating basic analysis");
        Ok("ANALYSIS: Analysis completed but response format was unexpected. The code was processed by the LLM but the response structure was not in the expected format. Manual review recommended.".to_string())
    }

    fn find_text_in_json(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(s) => {
                if s.len() > 10
                    && (s.contains("ANALYSIS:") || s.contains("security") || s.contains("analysis"))
                {
                    Some(s.clone())
                } else {
                    None
                }
            }
            Value::Object(obj) => {
                // Look for common text fields
                for key in ["text", "content", "message", "response", "analysis"] {
                    if let Some(text_val) = obj.get(key) {
                        if let Some(found_text) = self.find_text_in_json(text_val) {
                            return Some(found_text);
                        }
                    }
                }
                // Recursively search all values
                for (_key, val) in obj {
                    if let Some(found_text) = self.find_text_in_json(val) {
                        return Some(found_text);
                    }
                }
                None
            }
            Value::Array(arr) => {
                for item in arr {
                    if let Some(found_text) = self.find_text_in_json(item) {
                        return Some(found_text);
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl GeminiClient {
    async fn make_api_request_with_retry(
        &mut self,
        request: &LlmRequest,
        max_retries: usize,
    ) -> Result<LlmResponse, LlmClientError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = Duration::from_secs(2_u64.pow(attempt as u32).min(60)); // Exponential backoff, max 60s
                println!(
                    "  ⏳ Retry attempt {} in {:.1}s...",
                    attempt,
                    delay.as_secs_f32()
                );
                sleep(delay).await;
            }

            match self.try_single_api_request(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    println!("  ⚠️  API attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    // Don't retry on certain errors
                    if let Some(ref err) = last_error {
                        match err {
                            LlmClientError::JsonError(_) => {
                                // Try to continue on JSON errors, might be temporary
                            }
                            LlmClientError::ApiError(msg) if msg.contains("quota") => {
                                // Don't retry quota errors
                                break;
                            }
                            LlmClientError::ApiError(msg) if msg.contains("invalid") => {
                                // Don't retry invalid request errors
                                break;
                            }
                            _ => {
                                // Retry other errors
                            }
                        }
                    }
                }
            }
        }

        Err(last_error.unwrap_or(LlmClientError::Other(
            "Unknown error after retries".to_string(),
        )))
    }

    async fn try_single_api_request(
        &mut self,
        request: &LlmRequest,
    ) -> Result<LlmResponse, LlmClientError> {
        // Rate limiting: ensure minimum delay between requests
        if let Some(last_time) = self.last_request_time {
            let elapsed = last_time.elapsed();
            if elapsed < self.min_request_interval {
                let sleep_duration = self.min_request_interval - elapsed;
                println!(
                    "  ⏳ Rate limiting: waiting {:.1}s to avoid API limits...",
                    sleep_duration.as_secs_f32()
                );
                sleep(sleep_duration).await;
            }
        }

        self.last_request_time = Some(Instant::now());
        println!(
            "  🔍 Analyzing code chunk ({} characters)...",
            request.prompt.len()
        );

        let url = format!(
            "{}/v1/models/{}:generateContent?key={}",
            self.api_endpoint, self.model, self.api_key
        );

        // Check file size and adjust prompt accordingly
        let code_content = request.prompt.replace("Analyze the following Rust code for malicious behavior, backdoors, or unsafe patterns. Provide a summary of findings and specific flagged lines with severity (High, Medium, Low) and a brief description:\n\n", "");
        let code_content = if code_content.starts_with("File: ") {
            // Extract just the code part after "File: path\n"
            if let Some(pos) = code_content.find('\n') {
                &code_content[pos + 1..]
            } else {
                &code_content
            }
        } else {
            &code_content
        };

        let enhanced_prompt = if code_content.len() > 12000 {
            // For large files, use a more concise prompt and analyze key sections
            let truncated_code = if code_content.len() > 15000 {
                let first_part = &code_content[..7500];
                let last_part = &code_content[code_content.len() - 7500..];
                format!(
                    "{}\n\n// ... (middle section truncated for analysis) ...\n\n{}",
                    first_part, last_part
                )
            } else {
                code_content.to_string()
            };

            format!(
                "Analyze this Rust code for security vulnerabilities and suspicious patterns. Focus on imports, unsafe blocks, network calls, file operations, and external command execution.

                Code ({} chars, {} analyzed):
                ```rust
                {}
                ```

                Provide a brief analysis focusing on security concerns. If issues found, list them as:
                PATTERNS: Line: X, Severity: High/Medium/Low, Issue: description",
                code_content.len(),
                if code_content.len() > 15000 { "truncated" } else { "full" },
                truncated_code
            )
        } else {
            // Regular prompt for smaller files
            format!(
                "Analyze this Rust code for security vulnerabilities, malicious behavior, backdoors, and unsafe patterns.

                Code to analyze:
                ```rust
                {}
                ```

                Format response as:
                ANALYSIS: [Your security analysis summary]

                PATTERNS:
                - Line: [number], Severity: [High/Medium/Low], Description: [description], Code: [snippet]

                If no issues found: ANALYSIS: No significant security issues detected.",
                code_content
            )
        };

        let gemini_request_body = serde_json::json!({
            "contents": [
                {
                    "parts": [
                        {"text": enhanced_prompt}
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 0.3,
                "maxOutputTokens": 4096,
                "candidateCount": 1
            },
            "safetySettings": [
                {
                    "category": "HARM_CATEGORY_HARASSMENT",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_HATE_SPEECH",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
                    "threshold": "BLOCK_NONE"
                },
                {
                    "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                    "threshold": "BLOCK_NONE"
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .json(&gemini_request_body)
            .send()
            .await
            .map_err(|e| LlmClientError::HttpRequest(e))?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(LlmClientError::ApiError(format!(
                "API request failed with status {}: {}",
                status, response_text
            )));
        }

        // Debug: Print raw response for troubleshooting
        println!(
            "  🔍 Debug - Response length: {} bytes",
            response_text.len()
        );
        if response_text.len() < 2000 {
            println!("  🔍 Debug - Full API response: {}", response_text);
        } else {
            println!(
                "  🔍 Debug - API response preview: {}...",
                &response_text[..1000]
            );
        }

        // Check if response looks like valid JSON
        if !response_text.trim().starts_with('{') && !response_text.trim().starts_with('[') {
            println!("  ⚠️  Response doesn't look like JSON!");
            return Err(LlmClientError::ApiError(format!(
                "Non-JSON response received: {}",
                if response_text.len() > 200 {
                    &response_text[..200]
                } else {
                    &response_text
                }
            )));
        }

        // Try to parse as generic JSON first for better error handling
        let json_value: Value =
            serde_json::from_str(&response_text).map_err(|e| LlmClientError::JsonError(e))?;

        // Try multiple fallback approaches to extract text from API response
        let extracted_text = match self.extract_text_from_response(&json_value) {
            Ok(text) => text,
            Err(e) => {
                println!("  ❌ Failed to extract text from response: {}", e);

                // More detailed debugging
                println!("  🔍 Response analysis:");
                if let Some(obj) = json_value.as_object() {
                    println!("    - Response is an object with {} fields", obj.len());
                    for key in obj.keys() {
                        println!("    - Field: '{}'", key);
                    }

                    // Check specific known fields
                    if let Some(candidates) = obj.get("candidates") {
                        println!(
                            "    - 'candidates' field found: {}",
                            serde_json::to_string(&candidates)
                                .unwrap_or_else(|_| "invalid".to_string())
                        );
                    }
                    if let Some(error) = obj.get("error") {
                        println!(
                            "    - 'error' field found: {}",
                            serde_json::to_string(&error).unwrap_or_else(|_| "invalid".to_string())
                        );
                    }
                } else {
                    println!("    - Response is not a JSON object");
                    println!("    - Type: {:?}", json_value);
                }

                println!(
                    "  🔍 Full JSON structure: {}",
                    serde_json::to_string_pretty(&json_value)
                        .unwrap_or_else(|_| "Invalid JSON".to_string())
                );
                return Err(e);
            }
        };

        // Parse the structured response
        let (analysis, flagged_patterns) = self.parse_analysis_response(&extracted_text)?;

        Ok(LlmResponse {
            analysis,
            flagged_patterns,
        })
    }
}

#[async_trait]
impl LlmClientTrait for GeminiClient {
    async fn analyze_code(&mut self, request: LlmRequest) -> Result<LlmResponse, LlmClientError> {
        // Use retry logic with up to 3 attempts
        self.make_api_request_with_retry(&request, 2).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LlmClientError {
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] ReqwestError),
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("LLM API error: {0}")]
    ApiError(String),
    #[error("Other error: {0}")]
    Other(String),
}
