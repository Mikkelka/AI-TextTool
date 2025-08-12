# Plan: Simplify Data Storage to Single JSON File

## Current State (4 separate files):
1. `config.json` - API keys, models, system instructions
2. `options.json` - Text operations definitions
3. `chat_history.json` - Individual operation history (max 100 entries)
4. `saved_conversations.json` - Complete saved conversations

## Proposed: Single `app_data.json`
```json
{
  "config": {
    "api_key": "",
    "chat_model": "gemini-2.5-flash",
    "text_model": "gemini-2.5-flash-lite",
    "chat_system_instruction": "...",
    "providers": {...}
  },
  "operations": {
    "Proofread": {...},
    "Rewrite": {...},
    "Dansk": {...},
    // etc...
  },
  "chat_history": [
    // Last 100 entries
  ],
  "saved_conversations": [
    // All saved conversations
  ],
  "metadata": {
    "version": "1.0.0",
    "last_updated": "2024-12-12T10:00:00Z"
  }
}
```

## Benefits:
✅ **Simpler** - Only one file to manage
✅ **Easier backup** - Users just copy one file
✅ **Atomic updates** - No risk of partial updates across files
✅ **Easier migration** - Single file to move when updating
✅ **No sync issues** - All data always consistent

## Performance Considerations:
- Current total size: ~50-100KB maximum
- JSON parsing time: <1ms for 100KB
- Memory usage: Negligible
- **Conclusion**: No performance impact ✓

## Migration Strategy:
1. On app startup, check for `app_data.json`
2. If not exists, check for old files
3. If old files exist, merge them into `app_data.json`
4. Delete old files after successful migration
5. Always use `app_data.json` going forward

## Implementation Changes Needed:

### 1. Create new DataManager in `src-tauri/src/data_manager.rs`:
- Single struct to manage all data
- Load/save methods for entire data structure
- Getters/setters for each section
- Auto-save on changes

### 2. Update all existing functions:
- `config.rs` - Use DataManager instead of ConfigManager
- `lib.rs` - Update all save/load calls
- Remove separate file operations

### 3. Migration function:
```rust
async fn migrate_to_single_file() {
    if !app_data.json exists {
        let config = load_if_exists("config.json");
        let operations = load_if_exists("options.json");
        let history = load_if_exists("chat_history.json");
        let conversations = load_if_exists("saved_conversations.json");
        
        let app_data = AppData {
            config,
            operations,
            chat_history: history,
            saved_conversations: conversations,
            metadata: Metadata::new()
        };
        
        save_app_data(app_data);
        
        // Optionally delete old files or keep as backup
    }
}
```

## File Location:
Keep same strategy - next to .exe:
- `C:\Users\mikke\Desktop\AI-Tool\src-tauri\target\debug\app_data.json`
- `C:\Users\mikke\Desktop\AI-Tool\src-tauri\target\release\app_data.json`

## Backwards Compatibility:
- Keep migration code for 2-3 versions
- Warn users if using old file structure
- Auto-migrate silently on first run

## Estimated Work:
- 2-3 hours to implement
- Low risk - can keep old code as fallback
- Easy to test - just check file operations

## Future Extensions:
- Could add compression if file gets large
- Could add encryption for sensitive data
- Could add cloud sync support
- Could add import/export features