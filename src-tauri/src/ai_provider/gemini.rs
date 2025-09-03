use reqwest::Client;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

use super::types::*;

/// Central formatting instruction for all AI responses
const FORMATTING_INSTRUCTION: &str = "\
IMPORTANT: Use markdown formatting in your responses to make them clear and well-structured:
- Headers: # ## ### for organizing content
- Lists: - for bullet points, 1. for numbered lists  
- Emphasis: **bold** for important points, *italic* for subtle emphasis
- Code: `inline code` for technical terms, ```language blocks for code
- Tables: | Column | Format | for data presentation
- Quotes: > for important quotes or blockquotes
- Separators: --- for horizontal rules/breaks
Always format your response professionally using these markdown features.";

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
        self.generate_content_with_formatting(
            model,
            contents,
            system_instruction,
            generation_config,
            true,
        )
        .await
    }

    /// Generate content with optional formatting control
    pub async fn generate_content_with_formatting(
        &self,
        model: &str,
        contents: Vec<Content>,
        system_instruction: Option<&str>,
        generation_config: Option<GenerationConfig>,
        use_formatting: bool,
    ) -> Result<String, GeminiError> {
        self.generate_content_with_retry(
            model,
            contents,
            system_instruction,
            generation_config,
            use_formatting,
            0,
        )
        .await
    }

    /// Internal method with retry logic
    fn generate_content_with_retry<'a>(
        &'a self,
        model: &'a str,
        contents: Vec<Content>,
        system_instruction: Option<&'a str>,
        generation_config: Option<GenerationConfig>,
        use_formatting: bool,
        retry_count: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, GeminiError>> + Send + 'a>>
    {
        Box::pin(async move {
            // Check rate limits
            {
                let mut rate_limiter = self.rate_limiter.lock().await;
                rate_limiter.check_rate_limit().await?;
            }

            // Combine formatting instruction with custom system instruction (only if formatting is enabled)
            let combined_instruction = if use_formatting {
                match system_instruction {
                    Some(instruction) => format!("{}\n\n{}", FORMATTING_INSTRUCTION, instruction),
                    None => FORMATTING_INSTRUCTION.to_string(),
                }
            } else {
                system_instruction.unwrap_or("").to_string()
            };

            let request = GeminiRequest {
                contents,
                system_instruction: Some(Content::new(combined_instruction, None)),
                generation_config: generation_config
                    .or_else(|| Some(self.default_generation_config.clone())),
                safety_settings: Some(self.default_safety_settings.clone()),
            };

            let url = format!("{}/models/{}:generateContent", self.base_url, model);

            let response = self
                .client
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
                                    return self
                                        .generate_content_with_retry(
                                            model,
                                            request.contents,
                                            system_instruction,
                                            request.generation_config,
                                            use_formatting,
                                            retry_count + 1,
                                        )
                                        .await;
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
                                    return self
                                        .generate_content_with_retry(
                                            model,
                                            request.contents,
                                            system_instruction,
                                            request.generation_config,
                                            use_formatting,
                                            retry_count + 1,
                                        )
                                        .await;
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
        self.generate_chat_content_with_retry(
            model,
            contents,
            system_instruction,
            generation_config,
            0,
        )
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
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatResponse, GeminiError>> + Send + 'a>,
    > {
        let use_formatting = true; // Chat responses should always use formatting
        Box::pin(async move {
            // Check rate limits
            {
                let mut rate_limiter = self.rate_limiter.lock().await;
                rate_limiter.check_rate_limit().await?;
            }

            // Combine formatting instruction with custom system instruction (only if formatting is enabled)
            let combined_instruction = if use_formatting {
                match system_instruction {
                    Some(instruction) => format!("{}\n\n{}", FORMATTING_INSTRUCTION, instruction),
                    None => FORMATTING_INSTRUCTION.to_string(),
                }
            } else {
                system_instruction.unwrap_or("").to_string()
            };

            let request = GeminiRequest {
                contents,
                system_instruction: Some(Content::new(combined_instruction, None)),
                generation_config: generation_config
                    .or_else(|| Some(self.default_generation_config.clone())),
                safety_settings: Some(self.default_safety_settings.clone()),
            };

            let url = format!("{}/models/{}:generateContent", self.base_url, model);

            let response = self
                .client
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
                                    return self
                                        .generate_chat_content_with_retry(
                                            model,
                                            request.contents,
                                            system_instruction,
                                            request.generation_config,
                                            retry_count + 1,
                                        )
                                        .await;
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
                                    return self
                                        .generate_chat_content_with_retry(
                                            model,
                                            request.contents,
                                            system_instruction,
                                            request.generation_config,
                                            retry_count + 1,
                                        )
                                        .await;
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

        self.generate_content_with_formatting(
            model,
            contents,
            instruction,
            None,
            false, // Disable formatting for direct text operations
        )
        .await
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

        self.generate_chat_content(model, contents, system_instruction, generation_config)
            .await
    }

    /// Get available models (placeholder - would require additional API call)
    pub fn get_available_models() -> Vec<&'static str> {
        vec!["gemini-2.5-flash", "gemini-2.5-flash-lite"]
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

        match self
            .generate_content(
                "gemini-2.5-flash-lite", // Use the fastest model for testing
                test_content,
                Some("Please respond with just 'OK' to test the connection."),
                Some(GenerationConfig {
                    max_output_tokens: Some(10),
                    temperature: Some(0.0),
                    ..Default::default()
                }),
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(GeminiError::InvalidApiKey) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);

        // Should be able to make 2 calls immediately
        assert!(limiter.check_rate_limit().await.is_ok());
        assert!(limiter.check_rate_limit().await.is_ok());

        // Third call should still work but might have delay logic
        assert!(limiter.check_rate_limit().await.is_ok());
    }

    #[test]
    fn test_model_support() {
        assert!(GeminiProvider::supports_thinking_mode("gemini-2.5-flash"));
        assert!(!GeminiProvider::supports_thinking_mode(
            "gemini-2.5-flash-lite"
        ));
    }
}
