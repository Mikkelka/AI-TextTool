use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::types::*;
use crate::utils::file_paths;

/// Maximum number of entries to keep in history collections
const MAX_HISTORY_ENTRIES: usize = 100;

/// Data manager for handling all application data
pub struct DataManager {
    data: AppData,
    file_path: PathBuf,
}

impl DataManager {

    /// Create a new DataManager instance
    pub fn new() -> Self {
        Self {
            data: AppData::default(),
            file_path: PathBuf::new(),
        }
    }

    /// Initialize the data manager (load or migrate data)
    pub async fn initialize(&mut self) -> Result<(), DataError> {
        // Determine file path (next to exe)
        self.file_path = file_paths::get_app_data_path();

        // Try to load existing app_data.json
        if self.file_path.exists() {
            log::info!("Loading app_data.json from: {:?}", self.file_path);
            self.load_data().await?;
        } else {
            log::info!("No app_data.json found, attempting migration from old files");
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

        // Write to temp file, then replace the original (best-effort atomic)
        let temp_path = self.file_path.with_extension("json.tmp");
        let mut file = fs::File::create(&temp_path).await?;
        file.write_all(json_content.as_bytes()).await?;
        file.flush().await?;
        file.sync_all().await?;

        match fs::rename(&temp_path, &self.file_path).await {
            Ok(()) => {}
            Err(e) => {
                log::error!("Failed to atomically save data file, falling back to direct write: {e}");
                let mut file = fs::File::create(&self.file_path).await?;
                file.write_all(json_content.as_bytes()).await?;
                file.flush().await?;
                file.sync_all().await?;
                let _ = fs::remove_file(&temp_path).await;
            }
        }

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
            log::info!("Migrating config.json");
            if let Ok(content) = fs::read_to_string(&config_path).await {
                if let Ok(config) = serde_json::from_str(&content) {
                    self.data.config = config;
                }
            }
        }

        // Load options.json if exists
        let options_path = exe_dir.join("options.json");
        if options_path.exists() {
            log::info!("Migrating options.json");
            if let Ok(content) = fs::read_to_string(&options_path).await {
                if let Ok(operations) = serde_json::from_str(&content) {
                    self.data.operations = operations;
                }
            }
        }

        // Load chat_history.json if exists
        let history_path = exe_dir.join("chat_history.json");
        if history_path.exists() {
            log::info!("Migrating chat_history.json");
            if let Ok(content) = fs::read_to_string(&history_path).await {
                if let Ok(history) = serde_json::from_str(&content) {
                    self.data.chat_history = history;
                }
            }
        }

        // Load saved_conversations.json if exists
        let conversations_path = exe_dir.join("saved_conversations.json");
        if conversations_path.exists() {
            log::info!("Migrating saved_conversations.json");
            if let Ok(content) = fs::read_to_string(&conversations_path).await {
                if let Ok(conversations) = serde_json::from_str(&content) {
                    self.data.saved_conversations = conversations;
                }
            }
        }

        log::info!("Migration complete - data consolidated into app_data.json");

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
            let _ = fs::rename(
                &conversations_path,
                exe_dir.join("saved_conversations.json.old"),
            )
            .await;
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

    pub async fn update_operations(
        &mut self,
        operations: HashMap<String, Operation>,
    ) -> Result<(), DataError> {
        self.data.operations = operations;
        self.save_data().await
    }

    pub async fn add_chat_entry(&mut self, entry: ChatEntry) -> Result<(), DataError> {
        self.data.chat_history.push(entry);

        // Keep only last MAX_HISTORY_ENTRIES entries
        if self.data.chat_history.len() > MAX_HISTORY_ENTRIES {
            let excess = self.data.chat_history.len() - MAX_HISTORY_ENTRIES;
            self.data.chat_history.drain(0..excess);
        }

        self.save_data().await
    }

    pub async fn add_saved_conversation(
        &mut self,
        conversation: SavedConversation,
    ) -> Result<(), DataError> {
        self.data.saved_conversations.push(conversation);

        // Keep only last MAX_HISTORY_ENTRIES conversations
        if self.data.saved_conversations.len() > MAX_HISTORY_ENTRIES {
            let excess = self.data.saved_conversations.len() - MAX_HISTORY_ENTRIES;
            self.data.saved_conversations.drain(0..excess);
        }

        self.save_data().await
    }

    pub async fn delete_saved_conversation(
        &mut self,
        conversation_id: &str,
    ) -> Result<(), DataError> {
        self.data
            .saved_conversations
            .retain(|c| c.id != conversation_id);
        self.save_data().await
    }

    pub async fn clear_chat_history(&mut self) -> Result<(), DataError> {
        self.data.chat_history.clear();
        self.save_data().await
    }

    pub async fn clear_saved_conversations(&mut self) -> Result<(), DataError> {
        self.data.saved_conversations.clear();
        self.save_data().await
    }

    pub fn get_operation(&self, name: &str) -> Option<&Operation> {
        self.data.operations.get(name)
    }

    pub async fn update_operation(
        &mut self,
        name: String,
        operation: Operation,
    ) -> Result<(), DataError> {
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
        let mut operations_list: Vec<(&String, &Operation)> =
            self.data.operations.iter().collect();

        operations_list.sort_by(|a, b| {
            let order_cmp = a.1.order.cmp(&b.1.order);
            if order_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(b.0)
            } else {
                order_cmp
            }
        });

        operations_list
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
