use crate::ai_provider::types::{CHAT_MODEL, TEXT_MODEL};
use crate::ai_provider::GroundingSource;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Custom error types for data management
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// AI Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub chat_model_name: String,
    pub text_model_name: String,
    pub chat_system_instruction: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            chat_model_name: CHAT_MODEL.to_string(),
            text_model_name: TEXT_MODEL.to_string(),
            chat_system_instruction: "You are a friendly, helpful AI assistant.".to_string(),
        }
    }
}

/// Application configuration
/// Single source of truth: all provider-specific data lives in `providers` HashMap.
/// Top-level getters delegate to the active provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub locale: String,
    pub streaming: bool,
    pub provider: String,
    pub shortcut: String,
}

impl Config {
    /// Get the active provider's configuration, or None if no providers are configured
    pub fn active_provider(&self) -> Option<&ProviderConfig> {
        self.providers
            .get(&self.provider)
            .or_else(|| self.providers.values().next())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert("Gemini".to_string(), ProviderConfig::default());

        Self {
            providers,
            locale: "en".to_string(),
            streaming: false,
            provider: "Gemini".to_string(),
            shortcut: "CmdOrCtrl+Space".to_string(),
        }
    }
}

/// Text operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub prefix: String,
    pub instruction: String,
    pub icon: Option<String>,
    pub open_in_window: bool,
    /// Display/sort order. `u32` because negative values are meaningless for sorting.
    /// Uses `default` so a missing field on legacy data deserializes as 0 rather than failing.
    #[serde(default)]
    pub order: u32,
}

/// Display metadata for an operation. Exposed to the frontend so it doesn't
/// need to hardcode badge classes per built-in operation name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    pub badge_class: String,
}

/// Chat history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEntry {
    pub timestamp: String,
    pub original_text: String,
    pub ai_option: String,
    pub processed_text: String,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub thoughts: Option<String>,
    #[serde(default)]
    pub sources: Vec<GroundingSource>,
    #[serde(default)]
    pub search_queries: Vec<String>,
}

/// Saved conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConversation {
    pub id: String,
    pub title: String,
    pub operation: String,
    pub messages: Vec<ConversationMessage>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub thinking_mode_enabled: bool,
    #[serde(default)]
    pub grounding_enabled: bool,
}

/// Metadata for the data file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub version: String,
    pub last_updated: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        }
    }
}

/// Complete application data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub config: Config,
    pub operations: HashMap<String, Operation>,
    pub chat_history: Vec<ChatEntry>,
    pub saved_conversations: Vec<SavedConversation>,
    pub metadata: Metadata,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            config: Config::default(),
            operations: Self::create_default_operations(),
            chat_history: Vec::new(),
            saved_conversations: Vec::new(),
            metadata: Metadata::default(),
        }
    }
}

impl AppData {
    /// Default styling metadata for the built-in operations. Lets the frontend
    /// render badges/colors without hardcoding a parallel map.
    pub fn default_operation_metadata() -> HashMap<String, OperationMetadata> {
        let entries = [
            ("Proofread", "operation-proofread"),
            ("Rewrite", "operation-rewrite"),
            ("Dansk", "operation-translate"),
            ("Concise", "operation-concise"),
            ("Friendly", "operation-friendly"),
            ("Professional", "operation-professional"),
            ("Key Points", "operation-keypoints"),
            ("Summary", "operation-summary"),
            ("Chat", "operation-chat"),
            ("Custom", "operation-custom"),
        ];
        entries
            .iter()
            .map(|(name, badge)| {
                (
                    (*name).to_string(),
                    OperationMetadata {
                        badge_class: (*badge).to_string(),
                    },
                )
            })
            .collect()
    }

    /// Create default operations
    pub fn create_default_operations() -> HashMap<String, Operation> {
        let mut operations = HashMap::new();

        operations.insert("Proofread".to_string(), Operation {
            prefix: "Proofread the following text:\n\n".to_string(),
            instruction: "CRITICAL: Keep the text in its original language. Do NOT translate.\n\nYou are a professional proofreader. Correct any spelling, grammar, and punctuation errors. Maintain the original tone and style. Return ONLY the corrected text without any markdown formatting, headers, explanations, or additional content.".to_string(),
            icon: Some("pencil".to_string()),
            open_in_window: false,
            order: 1,
        });

        operations.insert("Rewrite".to_string(), Operation {
            prefix: "Rewrite the following text to improve clarity:\n\n".to_string(),
            instruction: "CRITICAL: Keep the text in its original language. Do NOT translate.\n\nYou are a professional editor. Rewrite the text to be clearer and more engaging while maintaining the original meaning. Return ONLY the rewritten text without any markdown formatting, headers, explanations, or additional content.".to_string(),
            icon: Some("rewrite".to_string()),
            open_in_window: false,
            order: 2,
        });

        operations.insert("Dansk".to_string(), Operation {
            prefix: "Oversæt følgende tekst til dansk:\n\n".to_string(),
            instruction: "Du er en professionel oversætter med speciale i dansk. Oversæt teksten til naturligt og korrekt dansk. Bevar den oprindelige tone og stil. VIGTIGT: Returner KUN den oversatte tekst uden markdown formatering, overskrifter, forklaringer eller ekstra indhold.".to_string(),
            icon: Some("translate".to_string()),
            open_in_window: false,
            order: 3,
        });

        operations.insert("Concise".to_string(), Operation {
            prefix: "Make the following text more concise:\n\n".to_string(),
            instruction: "CRITICAL: Keep the text in its original language. Do NOT translate.\n\nYou are a professional editor. Make the text shorter and more direct while keeping all important information. Return ONLY the concise text without any markdown formatting, headers, explanations, or additional content.".to_string(),
            icon: Some("concise".to_string()),
            open_in_window: false,
            order: 4,
        });

        operations.insert("Friendly".to_string(), Operation {
            prefix: "Make the following text sound more friendly:\n\n".to_string(),
            instruction: "CRITICAL: Keep the text in its original language. Do NOT translate.\n\nYou are a communication expert. Rewrite the text to sound warmer and more approachable while maintaining professionalism. Return ONLY the friendly text without any markdown formatting, headers, explanations, or additional content.".to_string(),
            icon: Some("smiley-face".to_string()),
            open_in_window: false,
            order: 5,
        });

        operations.insert("Professional".to_string(), Operation {
            prefix: "Make the following text more professional:\n\n".to_string(),
            instruction: "CRITICAL: Keep the text in its original language. Do NOT translate.\n\nYou are a business communication expert. Rewrite the text to be more formal and professional. Return ONLY the professional text without any markdown formatting, headers, explanations, or additional content.".to_string(),
            icon: Some("briefcase".to_string()),
            open_in_window: false,
            order: 6,
        });

        operations.insert("Key Points".to_string(), Operation {
            prefix: "Extract key points from the following text:\n\n".to_string(),
            instruction: "You are an expert at analyzing and summarizing information. Extract the main points and present them as a clear, well-organized response.".to_string(),
            icon: Some("list".to_string()),
            open_in_window: true,
            order: 7,
        });

        operations.insert("Summary".to_string(), Operation {
            prefix: "Summarize the following text:\n\n".to_string(),
            instruction: "You are an expert at creating comprehensive summaries. Create a well-structured summary that captures all important information.".to_string(),
            icon: Some("summary".to_string()),
            open_in_window: true,
            order: 8,
        });

        operations.insert("Chat".to_string(), Operation {
            prefix: "".to_string(),
            instruction: "You are a helpful AI assistant. Engage in natural conversation and help with any questions or tasks.".to_string(),
            icon: Some("chat".to_string()),
            open_in_window: true,
            order: 9,
        });

        operations.insert(
            "Custom".to_string(),
            Operation {
                prefix: "".to_string(),
                instruction:
                    "You are a helpful AI assistant. Follow the user's instructions precisely."
                        .to_string(),
                icon: Some("wand".to_string()),
                open_in_window: true,
                order: 10,
            },
        );

        operations
    }
}
