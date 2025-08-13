use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::AppHandle;
use tokio::fs;
use tokio::io::AsyncWriteExt;
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
    fn create_default_operations() -> HashMap<String, Operation> {
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

/// Data manager for handling all application data
pub struct DataManager {
    data: AppData,
    file_path: PathBuf,
}

impl DataManager {
    /// Create a new DataManager instance
    pub fn new(_app_handle: AppHandle) -> Self {
        Self {
            data: AppData::default(),
            file_path: PathBuf::new(),
        }
    }
    
    /// Initialize the data manager (load or migrate data)
    pub async fn initialize(&mut self) -> Result<(), DataError> {
        // Determine file path (next to exe)
        self.file_path = if let Ok(exe_path) = std::env::current_exe() {
            exe_path.parent()
                .map(|p| p.join("app_data.json"))
                .unwrap_or_else(|| PathBuf::from("app_data.json"))
        } else {
            PathBuf::from("app_data.json")
        };
        
        // Try to load existing app_data.json
        if self.file_path.exists() {
            println!("Loading app_data.json from: {:?}", self.file_path);
            self.load_data().await?;
        } else {
            println!("No app_data.json found, attempting migration from old files");
            self.migrate_from_old_files().await?;
            self.save_data().await?;
        }
        
        Ok(())
    }
    
    /// Load data from app_data.json
    async fn load_data(&mut self) -> Result<(), DataError> {
        let content = fs::read_to_string(&self.file_path).await?;
        self.data = serde_json::from_str(&content)?;
        Ok(())
    }
    
    /// Save data to app_data.json
    pub async fn save_data(&self) -> Result<(), DataError> {
        // Update metadata
        let mut data = self.data.clone();
        data.metadata.last_updated = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        // Serialize to JSON
        let json_content = serde_json::to_string_pretty(&data)?;
        
        // Write to file
        let mut file = fs::File::create(&self.file_path).await?;
        file.write_all(json_content.as_bytes()).await?;
        file.flush().await?;
        
        Ok(())
    }
    
    /// Migrate from old file structure
    async fn migrate_from_old_files(&mut self) -> Result<(), DataError> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        
        // Load config.json if exists
        let config_path = exe_dir.join("config.json");
        if config_path.exists() {
            println!("Migrating config.json");
            if let Ok(content) = fs::read_to_string(&config_path).await {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.data.config = config;
                }
            }
        }
        
        // Load options.json if exists
        let options_path = exe_dir.join("options.json");
        if options_path.exists() {
            println!("Migrating options.json");
            if let Ok(content) = fs::read_to_string(&options_path).await {
                if let Ok(operations) = serde_json::from_str(&content) {
                    self.data.operations = operations;
                }
            }
        }
        
        // Load chat_history.json if exists
        let history_path = exe_dir.join("chat_history.json");
        if history_path.exists() {
            println!("Migrating chat_history.json");
            if let Ok(content) = fs::read_to_string(&history_path).await {
                if let Ok(history) = serde_json::from_str(&content) {
                    self.data.chat_history = history;
                }
            }
        }
        
        // Load saved_conversations.json if exists
        let conversations_path = exe_dir.join("saved_conversations.json");
        if conversations_path.exists() {
            println!("Migrating saved_conversations.json");
            if let Ok(content) = fs::read_to_string(&conversations_path).await {
                if let Ok(conversations) = serde_json::from_str(&content) {
                    self.data.saved_conversations = conversations;
                }
            }
        }
        
        println!("Migration complete - data consolidated into app_data.json");
        
        // Optional: Archive old files instead of deleting
        // This is safer for users
        if config_path.exists() {
            let _ = fs::rename(&config_path, exe_dir.join("config.json.old")).await;
        }
        if options_path.exists() {
            let _ = fs::rename(&options_path, exe_dir.join("options.json.old")).await;
        }
        if history_path.exists() {
            let _ = fs::rename(&history_path, exe_dir.join("chat_history.json.old")).await;
        }
        if conversations_path.exists() {
            let _ = fs::rename(&conversations_path, exe_dir.join("saved_conversations.json.old")).await;
        }
        
        Ok(())
    }
    
    // Getter methods
    pub fn get_config(&self) -> &Config {
        &self.data.config
    }
    
    pub fn get_operations(&self) -> &HashMap<String, Operation> {
        &self.data.operations
    }
    
    pub fn get_chat_history(&self) -> &Vec<ChatEntry> {
        &self.data.chat_history
    }
    
    pub fn get_saved_conversations(&self) -> &Vec<SavedConversation> {
        &self.data.saved_conversations
    }
    
    // Update methods
    pub async fn update_config(&mut self, config: Config) -> Result<(), DataError> {
        self.data.config = config;
        self.save_data().await
    }
    
    pub async fn update_operations(&mut self, operations: HashMap<String, Operation>) -> Result<(), DataError> {
        self.data.operations = operations;
        self.save_data().await
    }
    
    pub async fn add_chat_entry(&mut self, entry: ChatEntry) -> Result<(), DataError> {
        self.data.chat_history.push(entry);
        
        // Keep only last 100 entries
        if self.data.chat_history.len() > 100 {
            self.data.chat_history.drain(0..self.data.chat_history.len()-100);
        }
        
        self.save_data().await
    }
    
    pub async fn add_saved_conversation(&mut self, conversation: SavedConversation) -> Result<(), DataError> {
        self.data.saved_conversations.push(conversation);
        
        // Keep only last 100 conversations
        if self.data.saved_conversations.len() > 100 {
            self.data.saved_conversations.drain(0..self.data.saved_conversations.len()-100);
        }
        
        self.save_data().await
    }
    
    pub async fn delete_saved_conversation(&mut self, conversation_id: &str) -> Result<(), DataError> {
        self.data.saved_conversations.retain(|c| c.id != conversation_id);
        self.save_data().await
    }
    
    pub async fn clear_chat_history(&mut self) -> Result<(), DataError> {
        self.data.chat_history.clear();
        self.data.saved_conversations.clear();
        self.save_data().await
    }
    
    pub fn get_operation(&self, name: &str) -> Option<&Operation> {
        self.data.operations.get(name)
    }
    
    pub async fn update_operation(&mut self, name: String, operation: Operation) -> Result<(), DataError> {
        self.data.operations.insert(name, operation);
        self.save_data().await
    }
    
    pub async fn remove_operation(&mut self, name: &str) -> Result<bool, DataError> {
        let removed = self.data.operations.remove(name).is_some();
        if removed {
            self.save_data().await?;
        }
        Ok(removed)
    }
    
    pub async fn reset_operations(&mut self) -> Result<(), DataError> {
        self.data.operations = AppData::create_default_operations();
        self.save_data().await
    }
    
    pub fn get_operations_sorted(&self) -> Vec<(String, Operation)> {
        let mut operations_list: Vec<(String, Operation)> = self.data.operations.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        operations_list.sort_by(|a, b| {
            let order_cmp = a.1.order.cmp(&b.1.order);
            if order_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                order_cmp
            }
        });
        
        operations_list
    }
}

// Tauri commands for chat history and conversation management

#[tauri::command]
pub async fn save_chat_entry(app: AppHandle, original_text: String, ai_option: String, processed_text: String) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    
    let entry = ChatEntry {
        timestamp: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        original_text,
        ai_option,
        processed_text,
    };
    
    manager.add_chat_entry(entry).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_chat_history(app: AppHandle) -> Result<Vec<ChatEntry>, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_chat_history().clone())
}

#[tauri::command]
pub async fn clear_chat_history(app: AppHandle) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.clear_chat_history().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_conversation(
    app: AppHandle,
    title: String,
    operation: String,
    messages: Vec<ConversationMessage>
) -> Result<String, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    
    let conversation_id = format!("conv_{}", Utc::now().timestamp_millis());
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    let conversation = SavedConversation {
        id: conversation_id.clone(),
        title,
        operation,
        messages,
        created_at: now.clone(),
        updated_at: now,
    };
    
    manager.add_saved_conversation(conversation).await.map_err(|e| e.to_string())?;
    Ok(conversation_id)
}

#[tauri::command]
pub async fn load_saved_conversations(app: AppHandle) -> Result<Vec<SavedConversation>, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_saved_conversations().clone())
}

#[tauri::command]
pub async fn delete_saved_conversation(app: AppHandle, conversation_id: String) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.delete_saved_conversation(&conversation_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_conversation_messages(app: AppHandle, conversation_id: String) -> Result<SavedConversation, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    
    manager.get_saved_conversations()
        .iter()
        .find(|c| c.id == conversation_id)
        .cloned()
        .ok_or_else(|| format!("Conversation '{}' not found", conversation_id))
}

// Tauri commands for config and operations management

#[tauri::command]
pub async fn dm_load_config(app: AppHandle) -> Result<Config, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_config().clone())
}

#[tauri::command]
pub async fn dm_save_config(app: AppHandle, config: Config) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_config(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dm_load_operations(app: AppHandle) -> Result<HashMap<String, Operation>, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_operations().clone())
}

#[tauri::command]
pub async fn dm_load_operations_sorted(app: AppHandle) -> Result<Vec<(String, Operation)>, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_operations_sorted())
}

#[tauri::command]
pub async fn dm_save_operations(app: AppHandle, operations: HashMap<String, Operation>) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_operations(operations).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dm_get_operation(app: AppHandle, name: String) -> Result<Option<Operation>, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_operation(&name).cloned())
}

#[tauri::command]
pub async fn dm_update_operation(app: AppHandle, name: String, operation: Operation) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_operation(name, operation).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dm_remove_operation(app: AppHandle, name: String) -> Result<bool, String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.remove_operation(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dm_reset_operations(app: AppHandle) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.reset_operations().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dm_update_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    
    let mut config = manager.get_config().clone();
    config.api_key = api_key;
    
    manager.update_config(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dm_switch_provider(app: AppHandle, provider_name: String) -> Result<(), String> {
    let mut manager = DataManager::new(app);
    manager.initialize().await.map_err(|e| e.to_string())?;
    
    let mut config = manager.get_config().clone();
    if config.providers.contains_key(&provider_name) {
        config.provider = provider_name;
        manager.update_config(config).await.map_err(|e| e.to_string())
    } else {
        Err(format!("Provider '{}' not found", provider_name))
    }
}


