use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Custom error types for configuration management
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    
    #[error("Configuration not found: {0}")]
    NotFound(String),
    
    #[error("Invalid configuration: {0}")]
    Invalid(String),
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
            chat_system_instruction: "You are a friendly, helpful, compassionate, and endearing AI conversational assistant. Avoid making assumptions or generating harmful, biased, or inappropriate content. When in doubt, do not make up information. Ask the user for clarification if needed. Try not be unnecessarily repetitive in your response. You can, and should as appropriate, use Markdown formatting to make your response nicely readable.".to_string(),
        }
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub shortcut: String,
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
            shortcut: "ctrl+space".to_string(),
            providers,
            locale: "en".to_string(),
            streaming: false,
            provider: "Gemini".to_string(),
            api_key: String::new(),
            chat_model: "gemini-2.5-flash".to_string(),
            text_model: "gemini-2.5-flash-lite".to_string(),
            chat_system_instruction: "You are a friendly, helpful, compassionate, and endearing AI conversational assistant. Avoid making assumptions or generating harmful, biased, or inappropriate content. When in doubt, do not make up information. Ask the user for clarification if needed. Try not be unnecessarily repetitive in your response. You can, and should as appropriate, use Markdown formatting to make your response nicely readable.".to_string(),
        }
    }
}

/// Text operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub prefix: String,
    pub instruction: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub open_in_window: bool,
    #[serde(default)]
    pub order: i32,
}

impl Default for Operation {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            instruction: "You are a helpful writing assistant. Follow the user's instructions precisely and provide clear, accurate assistance with their text.".to_string(),
            icon: None,
            open_in_window: false,
            order: 999, // High number for new operations to appear at end
        }
    }
}

/// Configuration manager for handling all config operations
#[derive(Debug)]
pub struct ConfigManager {
    app_handle: AppHandle,
    config: Config,
    operations: HashMap<String, Operation>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager instance
    pub fn new(app_handle: AppHandle) -> Result<Self, ConfigError> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        let config_path = app_data_dir.join("config.json");
        
        Ok(Self {
            app_handle,
            config: Config::default(),
            operations: HashMap::new(),
            config_path,
        })
    }

    /// Load configuration from config.json
    pub async fn load_config(&mut self) -> Result<(), ConfigError> {
        // Ensure the app data directory exists
        let app_data_dir = self.app_handle.path().app_data_dir()?;
        fs::create_dir_all(&app_data_dir).await?;

        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).await?;
            self.config = serde_json::from_str(&content)?;
            
            // Update flat fields from provider config if they exist
            if let Some(provider_config) = self.config.providers.get(&self.config.provider) {
                if self.config.api_key.is_empty() {
                    self.config.api_key = provider_config.api_key.clone();
                }
                if self.config.chat_model == Config::default().chat_model {
                    self.config.chat_model = provider_config.chat_model_name.clone();
                }
                if self.config.text_model == Config::default().text_model {
                    self.config.text_model = provider_config.text_model_name.clone();
                }
                if self.config.chat_system_instruction == Config::default().chat_system_instruction {
                    self.config.chat_system_instruction = provider_config.chat_system_instruction.clone();
                }
            }
        } else {
            // Create default config file
            self.save_config().await?;
        }
        
        Ok(())
    }

    /// Save configuration to config.json
    pub async fn save_config(&self) -> Result<(), ConfigError> {
        // Ensure the app data directory exists
        let app_data_dir = self.app_handle.path().app_data_dir()?;
        fs::create_dir_all(&app_data_dir).await?;

        let json_content = serde_json::to_string_pretty(&self.config)?;
        let mut file = fs::File::create(&self.config_path).await?;
        file.write_all(json_content.as_bytes()).await?;
        file.flush().await?;
        
        Ok(())
    }

    /// Load operations from options.json - prioritize file next to exe
    pub async fn load_options(&mut self) -> Result<(), ConfigError> {
        // First priority: same directory as executable 
        let exe_dir_path = if let Ok(exe_path) = std::env::current_exe() {
            exe_path.parent()
                .map(|parent| parent.join("options.json"))
        } else {
            None
        };
        
        // Second priority: app data directory
        let app_data_path = self.app_handle.path().app_data_dir()
            .ok()
            .map(|dir| dir.join("options.json"));
            
        let options_path = if let Some(exe_path) = exe_dir_path.as_ref() {
            if exe_path.exists() {
                println!("Loading options.json from exe directory: {:?}", exe_path);
                exe_path.clone()
            } else if let Some(data_path) = app_data_path.as_ref() {
                if data_path.exists() {
                    println!("Loading options.json from app data: {:?}", data_path);
                    data_path.clone()
                } else {
                    // Create default options file next to exe if possible, otherwise in app data
                    let create_path = exe_dir_path.unwrap_or_else(|| data_path.clone());
                    println!("Creating default options.json at: {:?}", create_path);
                    self.create_default_options(&create_path).await?;
                    create_path
                }
            } else {
                return Err(ConfigError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound, 
                    "Unable to determine options.json path"
                )));
            }
        } else if let Some(data_path) = app_data_path.as_ref() {
            if !data_path.exists() {
                println!("Creating default options.json at: {:?}", data_path);
                self.create_default_options(data_path).await?;
            }
            data_path.clone()
        } else {
            return Err(ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                "Unable to determine options.json path"
            )));
        };

        let content = fs::read_to_string(&options_path).await?;
        self.operations = serde_json::from_str(&content)?;
        
        // Fix missing order fields - assign sequential order based on current HashMap iteration
        let mut needs_reordering = false;
        for operation in self.operations.values() {
            if operation.order == 0 {
                needs_reordering = true;
                break;
            }
        }
        
        if needs_reordering {
            println!("Assigning sequential order to operations");
            let mut sorted_names: Vec<String> = self.operations.keys().cloned().collect();
            // Sort by name to get consistent order when order is missing
            sorted_names.sort();
            
            for (index, name) in sorted_names.iter().enumerate() {
                if let Some(operation) = self.operations.get_mut(name) {
                    if operation.order == 0 {
                        operation.order = (index + 1) as i32;
                    }
                }
            }
            
            // Save the updated operations with order
            self.save_operations().await?;
        }
        
        Ok(())
    }

    /// Create default options.json file
    async fn create_default_options(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let mut default_operations = HashMap::new();
        
        default_operations.insert("Proofread".to_string(), Operation {
            prefix: "Proofread this:\n\n".to_string(),
            instruction: "You are a grammar proofreading assistant.\nOutput ONLY the corrected text without any additional comments.\nMaintain the original text structure and writing style.\nRespond in the same language as the input (e.g., English US, French).\nDo not answer or respond to the user's text content.\nIf the text is absolutely incompatible with this (e.g., totally random gibberish), output \"ERROR_TEXT_INCOMPATIBLE_WITH_REQUEST\".".to_string(),
            icon: None,
            open_in_window: false,
            order: 1,
        });

        default_operations.insert("Rewrite".to_string(), Operation {
            prefix: "Rewrite this:\n\n".to_string(),
            instruction: "You are a writing assistant.\nRewrite the text provided by the user to improve phrasing.\nOutput ONLY the rewritten text without additional comments.\nRespond in the same language as the input (e.g., English US, French).\nDo not answer or respond to the user's text content.\nIf the text is absolutely incompatible with proofreading (e.g., totally random gibberish), output \"ERROR_TEXT_INCOMPATIBLE_WITH_REQUEST\".".to_string(),
            icon: None,
            open_in_window: false,
            order: 2,
        });

        default_operations.insert("Concise".to_string(), Operation {
            prefix: "Make this more concise:\n\n".to_string(),
            instruction: "You are a writing assistant focused on brevity.\nRewrite the text to be more concise and to-the-point while preserving all essential information and meaning.\nOutput ONLY the rewritten text without additional comments.\nKeep the same language as the input.\nEliminate redundancy, filler words, and unnecessary elaboration.\nIf the text is incompatible with this request, output \"ERROR_TEXT_INCOMPATIBLE_WITH_REQUEST\".".to_string(),
            icon: None,
            open_in_window: false,
            order: 3,
        });

        default_operations.insert("Summary".to_string(), Operation {
            prefix: "Original text to summarize:\n\n".to_string(),
            instruction: "You are an expert text summarizer.\nProvide a comprehensive summary of the given text that captures the main points, key details, and overall message.\nUse Markdown formatting with appropriate headers, bullet points, and emphasis where helpful.\nMaintain the tone and style appropriate to the source material.\nEnsure the summary is concise yet complete, typically 20-30% of the original length.".to_string(),
            icon: None,
            open_in_window: true,
            order: 4,
        });

        default_operations.insert("Key Points".to_string(), Operation {
            prefix: "Original text to extract key points:\n\n".to_string(),
            instruction: "You are an expert at extracting key information.\nAnalyze the given text and extract the most important key points.\nPresent the key points as a well-organized list using Markdown formatting.\nUse bullet points or numbered lists as appropriate.\nHighlight the most critical information using **bold** text.\nEnsure each point is clear, concise, and captures essential information.".to_string(),
            icon: None,
            open_in_window: true,
            order: 5,
        });

        default_operations.insert("Dansk".to_string(), Operation {
            prefix: "Oversæt følgende tekst til dansk:\n\n".to_string(),
            instruction: "Du er en professionel oversætter. Oversæt teksten til naturligt og flydende dansk.\nBevar den oprindelige betydning og tone.\nOutput KUN den oversatte tekst uden yderligere kommentarer.\nHvis teksten allerede er på dansk, output \"TEKSTEN_ER_ALLEREDE_PÅ_DANSK\".\nSørg for at bruge korrekt dansk grammatik og idiomatiske udtryk.".to_string(),
            icon: Some("🇩🇰".to_string()),
            open_in_window: false,
            order: 6,
        });

        default_operations.insert("Friendly".to_string(), Operation {
            prefix: "Make this sound more friendly:\n\n".to_string(),
            instruction: "You are a writing assistant focused on tone adjustment.\nRewrite the text to sound more friendly, warm, and approachable while maintaining the core message.\nOutput ONLY the rewritten text without additional comments.\nKeep the same language as the input.\nMaintain professionalism while adding warmth.\nIf the text is incompatible with this request, output \"ERROR_TEXT_INCOMPATIBLE_WITH_REQUEST\".".to_string(),
            icon: Some("heart".to_string()),
            open_in_window: false,
            order: 7,
        });

        default_operations.insert("Professional".to_string(), Operation {
            prefix: "Make this sound more professional:\n\n".to_string(),
            instruction: "You are a writing assistant focused on professional tone.\nRewrite the text to sound more professional, formal, and business-appropriate while maintaining the core message.\nOutput ONLY the rewritten text without additional comments.\nKeep the same language as the input.\nUse appropriate professional vocabulary and structure.\nIf the text is incompatible with this request, output \"ERROR_TEXT_INCOMPATIBLE_WITH_REQUEST\".".to_string(),
            icon: Some("briefcase".to_string()),
            open_in_window: false,
            order: 8,
        });

        default_operations.insert("Chat".to_string(), Operation {
            prefix: "".to_string(),
            instruction: "You are a helpful, friendly AI assistant. Have a natural conversation with the user about the text they've selected or any questions they have.".to_string(),
            icon: Some("chat".to_string()),
            open_in_window: true,
            order: 9,
        });

        default_operations.insert("Custom".to_string(), Operation {
            prefix: "".to_string(),
            instruction: "You are a helpful writing assistant. Follow the user's instructions precisely and provide clear, accurate assistance with their text.".to_string(),
            icon: Some("wand".to_string()),
            open_in_window: true,
            order: 10,
        });

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let json_content = serde_json::to_string_pretty(&default_operations)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(json_content.as_bytes()).await?;
        file.flush().await?;
        
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get current operations
    pub fn get_operations(&self) -> &HashMap<String, Operation> {
        &self.operations
    }

    /// Get operations as sorted list by order
    pub fn get_operations_sorted(&self) -> Vec<(String, Operation)> {
        let mut operations_list: Vec<(String, Operation)> = self.operations.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Sort by order field, then by name as fallback
        operations_list.sort_by(|a, b| {
            let order_cmp = a.1.order.cmp(&b.1.order);
            if order_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0) // Sort by name if order is the same
            } else {
                order_cmp
            }
        });
        
        operations_list
    }

    /// Normalize all order values to be sequential (1, 2, 3, 4...)
    pub async fn normalize_order(&mut self) -> Result<(), ConfigError> {
        let sorted_ops = self.get_operations_sorted();
        
        for (index, (name, _)) in sorted_ops.iter().enumerate() {
            if let Some(operation) = self.operations.get_mut(name) {
                operation.order = (index + 1) as i32;
            }
        }
        
        self.save_operations().await?;
        Ok(())
    }

    /// Get a specific operation by name
    pub fn get_operation(&self, name: &str) -> Option<&Operation> {
        self.operations.get(name)
    }

    /// Update configuration and save to file
    pub async fn update_config(&mut self, new_config: Config) -> Result<(), ConfigError> {
        // Validate the configuration
        self.validate_config(&new_config)?;
        
        self.config = new_config;
        self.save_config().await?;
        
        Ok(())
    }

    /// Update a specific operation
    pub async fn update_operation(&mut self, name: String, operation: Operation) -> Result<(), ConfigError> {
        self.operations.insert(name, operation);
        self.save_operations().await?;
        Ok(())
    }

    /// Remove an operation
    pub async fn remove_operation(&mut self, name: &str) -> Result<bool, ConfigError> {
        let removed = self.operations.remove(name).is_some();
        if removed {
            self.save_operations().await?;
        }
        Ok(removed)
    }

    /// Save operations to options.json - prioritize file next to exe
    pub async fn save_operations(&mut self) -> Result<(), ConfigError> {
        // Normalize order values to be sequential before saving
        let sorted_ops = self.get_operations_sorted();
        for (index, (name, _)) in sorted_ops.iter().enumerate() {
            if let Some(operation) = self.operations.get_mut(name) {
                operation.order = (index + 1) as i32;
            }
        }
        
        // First priority: same directory as executable 
        let exe_dir_path = if let Ok(exe_path) = std::env::current_exe() {
            exe_path.parent()
                .map(|parent| parent.join("options.json"))
        } else {
            None
        };
        
        // Second priority: app data directory
        let app_data_path = self.app_handle.path().app_data_dir()
            .ok()
            .map(|dir| dir.join("options.json"));
            
        let options_path = if let Some(exe_path) = exe_dir_path.as_ref() {
            // Try to save next to exe first
            exe_path.clone()
        } else if let Some(data_path) = app_data_path.as_ref() {
            // Fallback to app data directory
            data_path.clone()
        } else {
            return Err(ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                "Unable to determine options.json save path"
            )));
        };

        // Ensure parent directory exists
        if let Some(parent) = options_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let json_content = serde_json::to_string_pretty(&self.operations)?;
        let mut file = fs::File::create(&options_path).await?;
        file.write_all(json_content.as_bytes()).await?;
        file.flush().await?;
        
        println!("Saved operations with normalized order to: {:?}", options_path);
        Ok(())
    }

    /// Update API key for current provider
    pub async fn update_api_key(&mut self, api_key: String) -> Result<(), ConfigError> {
        self.config.api_key = api_key.clone();
        
        // Update the provider-specific config as well
        if let Some(provider_config) = self.config.providers.get_mut(&self.config.provider) {
            provider_config.api_key = api_key;
        }
        
        self.save_config().await?;
        Ok(())
    }

    /// Update chat model for current provider
    pub async fn update_chat_model(&mut self, model: String) -> Result<(), ConfigError> {
        self.config.chat_model = model.clone();
        
        // Update the provider-specific config as well
        if let Some(provider_config) = self.config.providers.get_mut(&self.config.provider) {
            provider_config.chat_model_name = model;
        }
        
        self.save_config().await?;
        Ok(())
    }

    /// Update text model for current provider
    pub async fn update_text_model(&mut self, model: String) -> Result<(), ConfigError> {
        self.config.text_model = model.clone();
        
        // Update the provider-specific config as well
        if let Some(provider_config) = self.config.providers.get_mut(&self.config.provider) {
            provider_config.text_model_name = model;
        }
        
        self.save_config().await?;
        Ok(())
    }

    /// Update global shortcut
    pub async fn update_shortcut(&mut self, shortcut: String) -> Result<(), ConfigError> {
        self.config.shortcut = shortcut;
        self.save_config().await?;
        Ok(())
    }
    

    /// Switch to a different provider
    pub async fn switch_provider(&mut self, provider_name: String) -> Result<(), ConfigError> {
        if !self.config.providers.contains_key(&provider_name) {
            return Err(ConfigError::Invalid(format!("Provider '{}' not found", provider_name)));
        }
        
        self.config.provider = provider_name.clone();
        
        // Update flat fields from the new provider
        if let Some(provider_config) = self.config.providers.get(&provider_name) {
            self.config.api_key = provider_config.api_key.clone();
            self.config.chat_model = provider_config.chat_model_name.clone();
            self.config.text_model = provider_config.text_model_name.clone();
            self.config.chat_system_instruction = provider_config.chat_system_instruction.clone();
        }
        
        self.save_config().await?;
        Ok(())
    }

    /// Add or update a provider configuration
    pub async fn update_provider(&mut self, name: String, provider_config: ProviderConfig) -> Result<(), ConfigError> {
        self.config.providers.insert(name.clone(), provider_config.clone());
        
        // If this is the current provider, update the flat fields as well
        if self.config.provider == name {
            self.config.api_key = provider_config.api_key;
            self.config.chat_model = provider_config.chat_model_name;
            self.config.text_model = provider_config.text_model_name;
            self.config.chat_system_instruction = provider_config.chat_system_instruction;
        }
        
        self.save_config().await?;
        Ok(())
    }

    /// Initialize the configuration manager by loading all config files
    pub async fn initialize(&mut self) -> Result<(), ConfigError> {
        self.load_config().await?;
        self.load_options().await?;
        Ok(())
    }

    /// Validate configuration before saving
    fn validate_config(&self, config: &Config) -> Result<(), ConfigError> {
        if config.shortcut.trim().is_empty() {
            return Err(ConfigError::Invalid("Shortcut cannot be empty".to_string()));
        }
        
        if config.provider.trim().is_empty() {
            return Err(ConfigError::Invalid("Provider cannot be empty".to_string()));
        }
        
        if !config.providers.contains_key(&config.provider) {
            return Err(ConfigError::Invalid(format!("Selected provider '{}' is not configured", config.provider)));
        }
        
        Ok(())
    }

    /// Get configuration file path
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Get options file path
    pub fn get_options_path(&self) -> PathBuf {
        self.app_handle.path().app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("options.json")
    }

    /// Reset configuration to defaults
    pub async fn reset_config(&mut self) -> Result<(), ConfigError> {
        self.config = Config::default();
        self.save_config().await?;
        Ok(())
    }

    /// Reset operations to defaults
    pub async fn reset_operations(&mut self) -> Result<(), ConfigError> {
        self.operations.clear();
        let options_path = self.get_options_path();
        self.create_default_options(&options_path).await?;
        self.load_options().await?;
        Ok(())
    }
}

/// Tauri commands for configuration management
#[tauri::command]
pub async fn load_config(app: AppHandle) -> Result<Config, String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_config().clone())
}

#[tauri::command]
pub async fn save_config(app: AppHandle, config: Config) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_config(config).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_operations(app: AppHandle) -> Result<HashMap<String, Operation>, String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_operations().clone())
}

#[tauri::command]
pub async fn load_operations_sorted(app: AppHandle) -> Result<Vec<(String, Operation)>, String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_operations_sorted())
}

#[tauri::command]
pub async fn save_operations(app: AppHandle, operations: HashMap<String, Operation>) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    
    for (name, operation) in operations {
        manager.update_operation(name, operation).await.map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_operation(app: AppHandle, name: String) -> Result<Option<Operation>, String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    Ok(manager.get_operation(&name).cloned())
}

#[tauri::command]
pub async fn update_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_api_key(api_key).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_shortcut(shortcut).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn switch_provider(app: AppHandle, provider_name: String) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.switch_provider(provider_name).await.map_err(|e| e.to_string())?;
    Ok(())
}


#[tauri::command]
pub async fn update_operation(app: AppHandle, name: String, operation: Operation) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_operation(name, operation).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_operation(app: AppHandle, name: String) -> Result<bool, String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.remove_operation(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_chat_model(app: AppHandle, model: String) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_chat_model(model).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_text_model(app: AppHandle, model: String) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.update_text_model(model).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reset_operations(app: AppHandle) -> Result<(), String> {
    let mut manager = ConfigManager::new(app).map_err(|e| e.to_string())?;
    manager.initialize().await.map_err(|e| e.to_string())?;
    manager.reset_operations().await.map_err(|e| e.to_string())?;
    Ok(())
}