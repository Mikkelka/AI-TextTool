use tauri::AppHandle;
use chrono::Utc;
use std::collections::HashMap;

use super::types::*;
use super::manager::DataManager;

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
    messages: Vec<ConversationMessage>,
    thinking_mode_enabled: Option<bool>
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
        thinking_mode_enabled: thinking_mode_enabled.unwrap_or(false),
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