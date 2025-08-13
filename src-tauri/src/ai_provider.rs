use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::time::sleep;

/// Custom error types for Gemini API operations
#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("API error: {status} - {message}")]
    ApiError {
        status: u16,
        message: String,
    },
    
    #[error("Rate limit exceeded. Retry after {retry_after_seconds} seconds")]
    RateLimitExceeded {
        retry_after_seconds: u64,
    },
    
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("Model not found: {model}")]
    ModelNotFound { model: String },
    
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    
    #[error("Network timeout")]
    Timeout,
    
    #[error("Service unavailable")]
    ServiceUnavailable,
}

/// Part of a content message (text or other media types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<bool>,
}

/// Content structure for Gemini API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl Content {
    /// Create new content from text with optional role
    pub fn new(text: impl Into<String>, role: Option<String>) -> Self {
        Self {
            parts: vec![Part { text: text.into(), thought: None }],
            role,
        }
    }
    
    /// Create user content
    pub fn user(text: impl Into<String>) -> Self {
        Self::new(text, Some("user".to_string()))
    }
    
}

/// Chat message for conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
}

impl ChatMessage {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
    
}

/// Safety settings for content filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

impl Default for SafetySetting {
    fn default() -> Self {
        Self {
            category: "HARM_CATEGORY_HARASSMENT".to_string(),
            threshold: "BLOCK_ONLY_HIGH".to_string(),
        }
    }
}

/// Thinking configuration for Gemini models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    pub thinking_budget: i32,
    #[serde(rename = "includeThoughts")]
    pub include_thoughts: bool,
}

impl ThinkingConfig {
    /// Create dynamic thinking config with thought summaries
    pub fn dynamic_with_thoughts() -> Self {
        Self {
            thinking_budget: -1,
            include_thoughts: true,
        }
    }
    
    /// Create dynamic thinking config (model decides complexity)
    pub fn dynamic() -> Self {
        Self {
            thinking_budget: -1,
            include_thoughts: false,
        }
    }
    
    /// Create no thinking config (fast responses)
    pub fn disabled() -> Self {
        Self {
            thinking_budget: 0,
            include_thoughts: false,
        }
    }
}

/// Generation configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "thinkingConfig")]
    pub thinking_config: Option<ThinkingConfig>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.8),
            top_k: Some(40),
            max_output_tokens: Some(8192),
            candidate_count: Some(1),
            thinking_config: None, // No thinking by default (fast for text operations)
        }
    }
}

/// Request structure for Gemini API
#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
}

/// Candidate response from Gemini API
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub safety_ratings: Option<Vec<serde_json::Value>>,
}

/// Usage metadata from Gemini API
#[derive(Debug, Deserialize)]
pub struct UsageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub prompt_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub candidates_token_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub total_token_count: Option<i32>,
}

/// Response structure from Gemini API
#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub usage_metadata: Option<UsageMetadata>,
}

/// Chat response with separated answer and thought summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub answer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts: Option<String>,
}

/// Error response from Gemini API
#[derive(Debug, Deserialize)]
pub struct GeminiErrorResponse {
    pub error: GeminiErrorDetails,
}

#[derive(Debug, Deserialize)]
pub struct GeminiErrorDetails {
    #[allow(dead_code)]
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub status: Option<String>,
}

/// Rate limiting tracker
#[derive(Debug)]
struct RateLimiter {
    calls: VecDeque<Instant>,
    max_calls_per_minute: usize,
}

impl RateLimiter {
    fn new(max_calls_per_minute: usize) -> Self {
        Self {
            calls: VecDeque::new(),
            max_calls_per_minute,
        }
    }
    
    /// Check if we can make a request, and wait if necessary
    async fn check_rate_limit(&mut self) -> Result<(), GeminiError> {
        let now = Instant::now();
        
        // Remove calls older than 1 minute
        while let Some(&front_time) = self.calls.front() {
            if now.duration_since(front_time) > Duration::from_secs(60) {
                self.calls.pop_front();
            } else {
                break;
            }
        }
        
        // Check if we're at the rate limit
        if self.calls.len() >= self.max_calls_per_minute {
            // Calculate how long to wait
            if let Some(&oldest_call) = self.calls.front() {
                let wait_time = Duration::from_secs(60) - now.duration_since(oldest_call);
                if !wait_time.is_zero() {
                    println!("Rate limit reached, waiting {:?}", wait_time);
                    sleep(wait_time).await;
                }
            }
        }
        
        // Record this call
        self.calls.push_back(now);
        Ok(())
    }
}

/// Main Gemini AI provider
#[derive(Debug)]
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    default_generation_config: GenerationConfig,
    default_safety_settings: Vec<SafetySetting>,
    max_retries: u32,
}

impl GeminiProvider {
    /// Create a new Gemini provider instance
    pub fn new(api_key: String) -> Result<Self, GeminiError> {
        if api_key.trim().is_empty() {
            return Err(GeminiError::InvalidApiKey);
        }
        
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;
        
        let default_safety_settings = vec![
            SafetySetting {
                category: "HARM_CATEGORY_HARASSMENT".to_string(),
                threshold: "BLOCK_ONLY_HIGH".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                threshold: "BLOCK_ONLY_HIGH".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                threshold: "BLOCK_ONLY_HIGH".to_string(),
            },
            SafetySetting {
                category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                threshold: "BLOCK_ONLY_HIGH".to_string(),
            },
        ];
        
        Ok(Self {
            client,
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(15))), // Gemini allows ~15 requests per minute
            default_generation_config: GenerationConfig::default(),
            default_safety_settings,
            max_retries: 3,
        })
    }
    
    /// Generate content using the specified model
    pub async fn generate_content(
        &self,
        model: &str,
        contents: Vec<Content>,
        system_instruction: Option<&str>,
        generation_config: Option<GenerationConfig>,
    ) -> Result<String, GeminiError> {
        self.generate_content_with_retry(model, contents, system_instruction, generation_config, 0)
            .await
    }
    
    /// Internal method with retry logic
    fn generate_content_with_retry<'a>(
        &'a self,
        model: &'a str,
        contents: Vec<Content>,
        system_instruction: Option<&'a str>,
        generation_config: Option<GenerationConfig>,
        retry_count: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, GeminiError>> + Send + 'a>> {
        Box::pin(async move {
        // Check rate limits
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.check_rate_limit().await?;
        }
        
        let request = GeminiRequest {
            contents,
            system_instruction: system_instruction.map(|instruction| {
                Content::new(instruction, None)
            }),
            generation_config: generation_config.or_else(|| Some(self.default_generation_config.clone())),
            safety_settings: Some(self.default_safety_settings.clone()),
        };
        
        let url = format!("{}/models/{}:generateContent", self.base_url, model);
        
        let response = self.client
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&request)
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                let status = resp.status();
                
                if status.is_success() {
                    let gemini_response: GeminiResponse = resp.json().await?;
                    
                    if let Some(candidate) = gemini_response.candidates.first() {
                        if let Some(part) = candidate.content.parts.first() {
                            return Ok(part.text.clone());
                        }
                    }
                    
                    Err(GeminiError::InvalidRequest {
                        message: "No content in response".to_string(),
                    })
                } else {
                    // Handle different error status codes
                    match status.as_u16() {
                        401 => Err(GeminiError::InvalidApiKey),
                        404 => Err(GeminiError::ModelNotFound {
                            model: model.to_string(),
                        }),
                        429 => {
                            // Rate limit exceeded - implement exponential backoff
                            if retry_count < self.max_retries {
                                let delay = Duration::from_secs(2_u64.pow(retry_count + 1));
                                println!("Rate limited, retrying after {:?}", delay);
                                sleep(delay).await;
                                return self.generate_content_with_retry(
                                    model,
                                    request.contents,
                                    system_instruction,
                                    request.generation_config,
                                    retry_count + 1,
                                ).await;
                            } else {
                                Err(GeminiError::RateLimitExceeded {
                                    retry_after_seconds: 60,
                                })
                            }
                        }
                        500..=599 => {
                            // Server error - retry with exponential backoff
                            if retry_count < self.max_retries {
                                let delay = Duration::from_secs(2_u64.pow(retry_count + 1));
                                println!("Server error, retrying after {:?}", delay);
                                sleep(delay).await;
                                return self.generate_content_with_retry(
                                    model,
                                    request.contents,
                                    system_instruction,
                                    request.generation_config,
                                    retry_count + 1,
                                ).await;
                            } else {
                                Err(GeminiError::ServiceUnavailable)
                            }
                        }
                        _ => {
                            // Try to parse error response
                            if let Ok(error_resp) = resp.json::<GeminiErrorResponse>().await {
                                Err(GeminiError::ApiError {
                                    status: status.as_u16(),
                                    message: error_resp.error.message,
                                })
                            } else {
                                Err(GeminiError::ApiError {
                                    status: status.as_u16(),
                                    message: "Unknown error".to_string(),
                                })
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    Err(GeminiError::Timeout)
                } else {
                    Err(GeminiError::HttpError(e))
                }
            }
        }
        })
    }
    
    /// Generate chat content with thought summaries support
    pub async fn generate_chat_content(
        &self,
        model: &str,
        contents: Vec<Content>,
        system_instruction: Option<&str>,
        generation_config: Option<GenerationConfig>,
    ) -> Result<ChatResponse, GeminiError> {
        self.generate_chat_content_with_retry(model, contents, system_instruction, generation_config, 0)
            .await
    }
    
    /// Internal method with retry logic for chat responses
    fn generate_chat_content_with_retry<'a>(
        &'a self,
        model: &'a str,
        contents: Vec<Content>,
        system_instruction: Option<&'a str>,
        generation_config: Option<GenerationConfig>,
        retry_count: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ChatResponse, GeminiError>> + Send + 'a>> {
        Box::pin(async move {
        // Check rate limits
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.check_rate_limit().await?;
        }
        
        let request = GeminiRequest {
            contents,
            system_instruction: system_instruction.map(|instruction| {
                Content::new(instruction, None)
            }),
            generation_config: generation_config.or_else(|| Some(self.default_generation_config.clone())),
            safety_settings: Some(self.default_safety_settings.clone()),
        };
        
        let url = format!("{}/models/{}:generateContent", self.base_url, model);
        
        let response = self.client
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&request)
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                let status = resp.status();
                
                if status.is_success() {
                    let gemini_response: GeminiResponse = resp.json().await?;
                    
                    if let Some(candidate) = gemini_response.candidates.first() {
                        // Parse thoughts and answers separately
                        let mut thoughts_parts = Vec::new();
                        let mut answer_parts = Vec::new();
                        
                        for part in &candidate.content.parts {
                            if part.thought.unwrap_or(false) {
                                thoughts_parts.push(part.text.clone());
                            } else {
                                answer_parts.push(part.text.clone());
                            }
                        }
                        
                        let answer = answer_parts.join("");
                        let thoughts = if thoughts_parts.is_empty() {
                            None
                        } else {
                            Some(thoughts_parts.join("\n"))
                        };
                        
                        return Ok(ChatResponse { answer, thoughts });
                    }
                    
                    Err(GeminiError::InvalidRequest {
                        message: "No content in response".to_string(),
                    })
                } else {
                    // Handle different error status codes (same logic as original method)
                    match status.as_u16() {
                        401 => Err(GeminiError::InvalidApiKey),
                        404 => Err(GeminiError::ModelNotFound {
                            model: model.to_string(),
                        }),
                        429 => {
                            if retry_count < self.max_retries {
                                let delay = Duration::from_secs(2_u64.pow(retry_count + 1));
                                sleep(delay).await;
                                return self.generate_chat_content_with_retry(
                                    model,
                                    request.contents,
                                    system_instruction,
                                    request.generation_config,
                                    retry_count + 1,
                                ).await;
                            } else {
                                Err(GeminiError::RateLimitExceeded {
                                    retry_after_seconds: 60,
                                })
                            }
                        }
                        500..=599 => {
                            if retry_count < self.max_retries {
                                let delay = Duration::from_secs(2_u64.pow(retry_count + 1));
                                sleep(delay).await;
                                return self.generate_chat_content_with_retry(
                                    model,
                                    request.contents,
                                    system_instruction,
                                    request.generation_config,
                                    retry_count + 1,
                                ).await;
                            } else {
                                Err(GeminiError::ServiceUnavailable)
                            }
                        }
                        _ => {
                            if let Ok(error_resp) = resp.json::<GeminiErrorResponse>().await {
                                Err(GeminiError::ApiError {
                                    status: status.as_u16(),
                                    message: error_resp.error.message,
                                })
                            } else {
                                Err(GeminiError::ApiError {
                                    status: status.as_u16(),
                                    message: "Unknown error".to_string(),
                                })
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    Err(GeminiError::Timeout)
                } else {
                    Err(GeminiError::HttpError(e))
                }
            }
        }
        })
    }
    
    /// Process text with a specific operation
    pub async fn process_text_operation(
        &self,
        text: &str,
        operation: &str,
        instruction: Option<&str>,
        model: &str,
    ) -> Result<String, GeminiError> {
        let prompt = match operation {
            "proofread" => format!("Please proofread and correct the following text, maintaining its original style and structure:\n\n{}", text),
            "rewrite" => format!("Please rewrite the following text to improve clarity and flow while maintaining the original meaning:\n\n{}", text),
            "summarize" => format!("Please provide a concise summary of the following text:\n\n{}", text),
            "translate" => format!("Please translate the following text to English (or if it's already in English, improve the translation):\n\n{}", text),
            "expand" => format!("Please expand on the following text, adding more detail and context:\n\n{}", text),
            "simplify" => format!("Please simplify the following text to make it easier to understand:\n\n{}", text),
            "custom" => {
                if let Some(custom_instruction) = instruction {
                    format!("{}\n\n{}", custom_instruction, text)
                } else {
                    format!("Please process the following text:\n\n{}", text)
                }
            }
            _ => format!("Please process the following text:\n\n{}", text),
        };
        
        let contents = vec![Content::user(prompt)];
        
        self.generate_content(
            model,
            contents,
            instruction,
            None,
        ).await
    }
    
    /// Handle chat completion with conversation history
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        system_instruction: Option<&str>,
        model: &str,
        generation_config: Option<GenerationConfig>,
    ) -> Result<String, GeminiError> {
        // Convert chat messages to Gemini Content format
        let contents: Vec<Content> = messages
            .into_iter()
            .map(|msg| {
                let role = if msg.role == "user" { "user" } else { "model" };
                Content::new(msg.content, Some(role.to_string()))
            })
            .collect();
        
        self.generate_content(
            model,
            contents,
            system_instruction,
            generation_config,
        ).await
    }
    
    /// Handle chat completion with conversation history and thought summaries
    pub async fn chat_completion_with_thoughts(
        &self,
        messages: Vec<ChatMessage>,
        system_instruction: Option<&str>,
        model: &str,
        generation_config: Option<GenerationConfig>,
    ) -> Result<ChatResponse, GeminiError> {
        // Convert chat messages to Gemini Content format
        let contents: Vec<Content> = messages
            .into_iter()
            .map(|msg| {
                let role = if msg.role == "user" { "user" } else { "model" };
                Content::new(msg.content, Some(role.to_string()))
            })
            .collect();
        
        self.generate_chat_content(
            model,
            contents,
            system_instruction,
            generation_config,
        ).await
    }
    
    /// Get available models (placeholder - would require additional API call)
    pub fn get_available_models() -> Vec<&'static str> {
        vec![
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
        ]
    }
    
    /// Check if a model supports thinking mode (for advanced reasoning)
    #[cfg(test)]
    pub fn supports_thinking_mode(model: &str) -> bool {
        // Currently, thinking mode is supported by certain models
        matches!(model, "gemini-2.5-flash" | "gemini-1.5-pro")
    }
    
    
    
    
    
    
    /// Test the connection to Gemini API
    pub async fn test_connection(&self) -> Result<bool, GeminiError> {
        let test_content = vec![Content::user("Hello")];
        
        match self.generate_content(
            "gemini-2.5-flash-lite", // Use the fastest model for testing
            test_content,
            Some("Please respond with just 'OK' to test the connection."),
            Some(GenerationConfig {
                max_output_tokens: Some(10),
                temperature: Some(0.0),
                ..Default::default()
            }),
        ).await {
            Ok(_) => Ok(true),
            Err(GeminiError::InvalidApiKey) => Ok(false),
            Err(e) => Err(e),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_creation() {
        let content = Content::user("Hello world");
        assert_eq!(content.parts[0].text, "Hello world");
        assert_eq!(content.role, Some("user".to_string()));
    }
    
    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::user("Test message");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Test message");
        assert!(msg.timestamp.is_some());
    }
    
    #[test]
    fn test_model_support() {
        assert!(GeminiProvider::supports_thinking_mode("gemini-2.5-flash"));
        assert!(!GeminiProvider::supports_thinking_mode("gemini-2.5-flash-lite"));
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);
        
        // Should be able to make 2 calls immediately
        assert!(limiter.check_rate_limit().await.is_ok());
        assert!(limiter.check_rate_limit().await.is_ok());
        
        // Third call should still work but might have delay logic
        assert!(limiter.check_rate_limit().await.is_ok());
    }
}