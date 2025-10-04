use async_trait::async_trait;
use regex::Regex;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
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
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: String,
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
}

#[async_trait]
impl LlmClientTrait for GeminiClient {
    async fn analyze_code(&mut self, request: LlmRequest) -> Result<LlmResponse, LlmClientError> {
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

        // Enhanced prompt for better security analysis
        let enhanced_prompt = format!(
            "Analyze this Rust code for security vulnerabilities, malicious behavior, backdoors, and unsafe patterns.

            Please provide:
            1. A brief security analysis summary
            2. List any suspicious patterns found with:
               - Line number (estimate if exact line unknown)
               - Severity: High/Medium/Low
               - Description of the issue
               - Code snippet of the problematic code

            Code to analyze:
            ```rust
            {}
            ```

            Format your response as:
            ANALYSIS: [Your analysis summary]

            PATTERNS:
            - Line: [number], Severity: [High/Medium/Low], Description: [description], Code: [snippet]
            - Line: [number], Severity: [High/Medium/Low], Description: [description], Code: [snippet]

            If no security issues found, respond with:
            ANALYSIS: No significant security issues detected.
            PATTERNS: None",
            request.prompt.replace("Analyze the following Rust code for malicious behavior, backdoors, or unsafe patterns. Provide a summary of findings and specific flagged lines with severity (High, Medium, Low) and a brief description:\n\n", "")
        );

        let gemini_request_body = serde_json::json!({
            "contents": [
                {
                    "parts": [
                        {"text": enhanced_prompt}
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 0.7,
                "maxOutputTokens": 2048
            }
        });

        let response = self
            .client
            .post(&url)
            .json(&gemini_request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmClientError::ApiError(format!(
                "API request failed: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;

        // Parse Gemini response
        let gemini_response: GeminiResponse =
            serde_json::from_str(&response_text).map_err(|e| LlmClientError::JsonError(e))?;

        if gemini_response.candidates.is_empty() {
            return Err(LlmClientError::ApiError(
                "No response candidates received".to_string(),
            ));
        }

        let response_content = &gemini_response.candidates[0].content.parts[0].text;

        // Parse the structured response
        let (analysis, flagged_patterns) = self.parse_analysis_response(response_content)?;

        Ok(LlmResponse {
            analysis,
            flagged_patterns,
        })
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
