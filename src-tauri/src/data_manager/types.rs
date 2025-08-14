use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

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
            chat_model_name: "gemini-2.5-flash".to_string(),
            text_model_name: "gemini-2.5-flash-lite".to_string(),
            chat_system_instruction: "You are a friendly, helpful AI assistant.".to_string(),
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub locale: String,
    pub streaming: bool,
    pub provider: String,
    pub api_key: String,
    pub chat_model: String,
    pub text_model: String,
    pub chat_system_instruction: String,
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
            api_key: String::new(),
            chat_model: "gemini-2.5-flash".to_string(),
            text_model: "gemini-2.5-flash-lite".to_string(),
            chat_system_instruction: ProviderConfig::default().chat_system_instruction,
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
    pub order: i32,
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
    /// Create default operations
    pub fn create_default_operations() -> HashMap<String, Operation> {
        let mut operations = HashMap::new();
        
        operations.insert("Proofread".to_string(), Operation {
            prefix: "Proofread the following text:\n\n".to_string(),
            instruction: "You are a professional proofreader. Correct any spelling, grammar, and punctuation errors. Maintain the original tone and style. Only return the corrected text.".to_string(),
            icon: Some("pencil".to_string()),
            open_in_window: false,
            order: 1,
        });
        
        operations.insert("Rewrite".to_string(), Operation {
            prefix: "Rewrite the following text to improve clarity:\n\n".to_string(),
            instruction: "You are a professional editor. Rewrite the text to be clearer and more engaging while maintaining the original meaning. Only return the rewritten text.".to_string(),
            icon: Some("rewrite".to_string()),
            open_in_window: false,
            order: 2,
        });
        
        operations.insert("Dansk".to_string(), Operation {
            prefix: "Oversæt følgende tekst til dansk:\n\n".to_string(),
            instruction: "Du er en professionel oversætter med speciale i dansk. Oversæt teksten til naturligt og korrekt dansk. Bevar den oprindelige tone og stil. Returner kun den oversatte tekst.".to_string(),
            icon: Some("translate".to_string()),
            open_in_window: false,
            order: 3,
        });
        
        operations.insert("Concise".to_string(), Operation {
            prefix: "Make the following text more concise:\n\n".to_string(),
            instruction: "You are a professional editor. Make the text shorter and more direct while keeping all important information. Only return the concise version.".to_string(),
            icon: Some("concise".to_string()),
            open_in_window: false,
            order: 4,
        });
        
        operations.insert("Friendly".to_string(), Operation {
            prefix: "Make the following text sound more friendly:\n\n".to_string(),
            instruction: "You are a communication expert. Rewrite the text to sound warmer and more approachable while maintaining professionalism. Only return the friendly version.".to_string(),
            icon: Some("smiley-face".to_string()),
            open_in_window: false,
            order: 5,
        });
        
        operations.insert("Professional".to_string(), Operation {
            prefix: "Make the following text more professional:\n\n".to_string(),
            instruction: "You are a business communication expert. Rewrite the text to be more formal and professional. Only return the professional version.".to_string(),
            icon: Some("briefcase".to_string()),
            open_in_window: false,
            order: 6,
        });
        
        operations.insert("Key Points".to_string(), Operation {
            prefix: "Extract key points from the following text:\n\n".to_string(),
            instruction: "You are an expert at analyzing and summarizing information. Extract the main points and present them as a clear, bulleted list in markdown format.".to_string(),
            icon: Some("list".to_string()),
            open_in_window: true,
            order: 7,
        });
        
        operations.insert("Summary".to_string(), Operation {
            prefix: "Summarize the following text:\n\n".to_string(),
            instruction: "You are an expert at creating comprehensive summaries. Create a well-structured summary that captures all important information. Use markdown formatting.".to_string(),
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
        
        operations.insert("Custom".to_string(), Operation {
            prefix: "".to_string(),
            instruction: "You are a helpful AI assistant. Follow the user's instructions precisely.".to_string(),
            icon: Some("wand".to_string()),
            open_in_window: true,
            order: 10,
        });
        
        operations
    }
}