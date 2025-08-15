use serde::{Deserialize, Serialize};
use thiserror::Error;

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