# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **complete AI-powered text processing desktop application** built with Tauri + Vue 3 + TypeScript. It provides system-wide text processing capabilities using Google Gemini AI, with a focus on Danish language support and professional text operations.

**Key Features:**
- 🎯 **System Tray Application** - Runs minimized, accessible via tray
- ⌨️ **Global Hotkey (Ctrl+Space)** - Instant text processing from anywhere
- 🤖 **Google Gemini AI Integration** - 10 text operations including Danish translation
- 💬 **Chat Windows** - Extended AI conversations with operation-specific instructions
- 📚 **Chat History** - Complete history management with search and filtering
- ⚙️ **Settings & Configuration** - Full configuration management
- 🚀 **Onboarding Experience** - First-time setup wizard

## Architecture

- **Frontend**: Vue 3 + TypeScript with Composition API
  - **Main App**: `src/App.vue` (hidden main window)
  - **Popup Window**: `src/components/PopupWindow.vue` (text operation selector)
  - **Chat Window**: `src/components/ChatWindow.vue` (AI conversations)
  - **Settings Window**: `src/components/SettingsWindow.vue` (configuration)
  - **Chat History**: `src/components/ChatHistoryWindow.vue` (history management)
  - **Onboarding**: `src/components/OnboardingWindow.vue` (first-time setup)

- **Backend**: Rust with async/await - **Modular Architecture**
  - **Entry Point**: `src-tauri/src/lib.rs` (minimal Tauri app setup)
  - **AI Provider Module**: `src-tauri/src/ai_provider/` (Gemini integration with rate limiting)
  - **Data Manager Module**: `src-tauri/src/data_manager/` (configuration & data persistence)
  - **Window Manager**: `src-tauri/src/window_manager.rs` (all window creation & lifecycle)
  - **Tray Manager**: `src-tauri/src/tray_manager.rs` (system tray functionality)
  - **Shortcut Manager**: `src-tauri/src/shortcut_manager.rs` (global shortcuts & clipboard)
  - **Commands Module**: `src-tauri/src/commands/` (organized Tauri commands)

- **Communication**: Tauri command system with `invoke()` calls

## Development Commands

### Frontend Development
- `npm run dev` - Start Vite development server (runs on port 1420)
- `npm run build` - Build frontend for production (includes TypeScript compilation)
- `npm run preview` - Preview production build

### Tauri Development
- `npm run tauri dev` - Start Tauri development mode (launches desktop app)
- `npm run tauri build` - Build desktop application for distribution

### Full Development Workflow
- `npm run tauri dev` automatically runs `npm run dev` as the beforeDevCommand
- `npm run tauri build` automatically runs `npm run build` as the beforeBuildCommand

## Data Files (Located next to executable)

**Development**: `src-tauri/target/debug/`  
**Production**: `src-tauri/target/release/`

- **`options.json`** - Text operations configuration (10 operations including Danish)
- **`chat_history.json`** - Individual operation history (auto-saved, max 100 entries)
- **`saved_conversations.json`** - Complete chat conversations with custom titles
- **`config.json`** - App settings (API keys, models, system instructions)

## Configuration Files

- `vite.config.ts` - Vite configuration optimized for Tauri (fixed port 1420, HMR setup)
- `src-tauri/tauri.conf.json` - Tauri application configuration (window settings, bundle options)
- `src-tauri/Cargo.toml` - Rust dependencies and build configuration
- `tsconfig.json` & `tsconfig.node.json` - TypeScript configuration

## Adding New Features

1. **Frontend components**: Add Vue components in `src/` directory
2. **Rust commands**: Add functions with `#[tauri::command]` in appropriate module:
   - **AI commands**: `src-tauri/src/commands/ai_commands.rs`
   - **Window commands**: `src-tauri/src/commands/window_commands.rs`
   - **Utility commands**: `src-tauri/src/commands/utility_commands.rs`
   - **Data commands**: `src-tauri/src/data_manager/commands.rs`
   - Register new commands in `src-tauri/src/lib.rs` invoke_handler
3. **Frontend-backend communication**: Use `invoke("command_name", { params })` from the frontend to call Rust commands

## Core Features

### 🎯 System Tray Application
- **Startup**: Starts minimized, no main window shown
- **Tray Menu**: Settings, Chat History, Quit
- **Onboarding**: Shows setup wizard on first run (if no config.json exists)

### ⌨️ Global Hotkey (Ctrl+Space)
- **Text Capture**: Automatically simulates Ctrl+C to copy selected text
- **Popup Window**: Opens compact 4-column operation grid at mouse cursor
- **Debouncing**: Prevents multiple triggers (200ms cooldown)
- **Auto-paste**: Results automatically replace original text after processing

### 🤖 AI Text Operations (10 Operations)

**Direct Processing** (auto-paste results):
1. **Proofread** - Grammar and spelling correction
2. **Rewrite** - Text improvement and rephrasing  
3. **🇩🇰 Dansk** - Professional Danish translation
4. **Concise** - Make text more concise
5. **Friendly** - Adjust tone to be more friendly
6. **Professional** - Make text more professional

**Chat Windows** (open in new window):
7. **Key Points** - Extract key information as markdown list
8. **Summary** - Comprehensive summarization with formatting
9. **Chat** - Open conversation with AI
10. **Custom** - Custom instructions from user

### 💬 Conversation Management
- **Operation-Specific Instructions**: Each operation uses tailored AI instructions
- **Chat History**: All conversations auto-saved with timestamps
- **Markdown Support**: Full markdown rendering in chat windows
- **Message Actions**: Copy, regenerate responses

### ⚙️ Settings & Configuration
- **API Key Management**: Google AI Studio API key setup
- **Model Selection**: Choose between Gemini Flash and Flash-Lite models
- **System Instructions**: Customize AI behavior
- **Shortcuts**: Configurable hotkey (default Ctrl+Space)

### 📚 Chat History Window
- **Dual View System**: Separate tabs for "Conversations" (full saved chats) and "Entries" (individual operations)
- **Conversation Management**: Save complete chat conversations with custom titles
- **Conversation Actions**: Reopen saved conversations, export as markdown, delete conversations
- **Search & Filter**: Search content and filter by operation type
- **Legacy Support**: View individual operation history entries
- **Management**: Clear history, view statistics
- **Responsive**: Mobile-friendly layout

### Technical Implementation Notes

#### Window Management
- **Hidden Main Window**: Main window is hidden, only tray visible
- **Popup Positioning**: Opens at exact mouse cursor position
- **Window Lifecycle**: Proper cleanup and memory management
- **Data Injection**: Uses `initialization_script()` for popup data instead of clipboard API

#### AI Integration
- **Rate Limiting**: Built-in rate limiting per model (10-15 requests/min)
- **Retry Logic**: Automatic retry with exponential backoff
- **Error Handling**: Comprehensive error messages and recovery
- **Custom Instructions**: Per-operation system instructions in Danish and English

#### Data Management
- **File Location**: All data files stored next to executable (not hidden in AppData)
- **Backup Friendly**: Easy to backup entire application folder
- **JSON Format**: Human-readable configuration files
- **Live Reload**: Configuration changes take effect immediately

## Tauri-Specific Solutions & Learnings

### 🚨 Critical Window Management Issues We Solved

#### Problem 1: Main Window Showing on Startup
**Issue**: Main window appeared instead of starting minimized to tray  
**Solution**: Hide main window immediately in `setup()` function
```rust
if let Some(window) = app.get_webview_window("main") {
    window.hide().unwrap();
}
```

#### Problem 2: Popup Windows Couldn't Be Closed
**Issue**: Custom close buttons didn't work, ESC made window white  
**Root Cause**: `decorations(false)` disabled native window controls  
**Solution**: Use `decorations(true)` and rely on native close button
```rust
.decorations(true)  // Enable native window controls
.closable(true)     // Explicitly enable closing
```

#### Problem 3: Data Injection in Popup Windows
**Issue**: Popup windows can't access Tauri plugins like clipboard-manager  
**Problem**: Permission and context isolation issues  
**Critical Solution**: Use `initialization_script()` instead of clipboard API
```rust
.initialization_script(&format!(
    "window.clipboardText = '{}';", 
    escaped_text.replace('\'', "\\'").replace('\n', "\\n")
))
```
**Why This Works**: Injects data before window loads, avoiding all plugin access issues

#### Problem 4: Settings Window Buttons Not Working
**Issue**: Buttons in settings didn't open URLs or save data  
**Root Cause**: Wrong import path for Tauri opener plugin  
**Solution**: Use correct plugin import
```typescript
import { openUrl } from '@tauri-apps/plugin-opener'  // Correct
// Not: import { open } from '@tauri-apps/api/shell'  // Old API
```

#### Problem 5: System Tray Configuration
**Issue**: Tray icon duplication when using both config and programmatic creation  
**Solution**: Use ONLY programmatic tray creation in `lib.rs`, not in `tauri.conf.json`
```rust
let _ = TrayIconBuilder::with_id("main-tray")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .build(app);
```

#### Problem 6: Global Shortcut Debouncing on Windows
**Issue**: Multiple triggers on Windows systems  
**Solution**: Implement debouncing with Arc<Mutex<Instant>>
```rust
let last_trigger = Arc::new(Mutex::new(Instant::now() - Duration::from_millis(1000)));
if now.duration_since(*last_time).as_millis() < 200 {
    return; // Ignore duplicate
}
```

#### Problem 7: Frontend WebviewWindow Creation Fails in Tauri v2
**Issue**: Frontend `new WebviewWindow()` shows console logs but no window actually opens  
**Root Cause**: Missing permissions in `capabilities/default.json` and frontend context limitations  
**Critical Solution**: Always use backend commands for window creation
```rust
#[tauri::command]
async fn reopen_chat_conversation(app: AppHandle, conversation_id: String, operation: String, title: String) -> Result<(), String> {
    let chat_url = format!(
        "chat.html?operation={}&title={}&conversationId={}&t={}",
        urlencoding::encode(&operation),
        urlencoding::encode(&title),
        urlencoding::encode(&conversation_id),
        timestamp
    );
    
    WebviewWindowBuilder::new(&app, &window_id, tauri::WebviewUrl::App(chat_url.into()))
        .title(&title)
        .inner_size(900.0, 700.0)
        .build()
}
```

**Frontend Usage**:
```typescript
// Instead of: new WebviewWindow() - this often fails
// Use: Backend command
await invoke('reopen_chat_conversation', {
    conversationId: conversation.id,
    operation: conversation.operation,
    title: conversation.title
})
```

**Why Backend Works Better**:
- Uses `tauri::WebviewUrl::App()` instead of string URLs
- Avoids frontend permission and context issues
- Consistent with proven tray chat pattern
- More reliable window creation in Tauri v2

### 📁 File Location Strategy We Learned

#### Problem: Hidden AppData Files
**Issue**: Users couldn't find configuration files in hidden AppData folder  
**Better Solution**: Store user-editable files next to executable
```rust
let exe_dir = std::env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));

let history_file = if let Some(exe_path) = exe_dir {
    exe_path.join("chat_history.json")  // Next to .exe
} else {
    app_data_dir.join("chat_history.json")  // Fallback to AppData
}
```

**Priority Order**:
1. Same directory as executable (easy to find/backup)
2. AppData directory (fallback for permissions)

### 🔧 Configuration Management Pattern

#### Hardcoded Defaults + Runtime Files
**Problem**: Missing operations when only runtime files exist  
**Solution**: Always create hardcoded defaults first, then load runtime overrides
```rust
// In config.rs - create_default_operations()
default_operations.insert("Dansk".to_string(), Operation {
    prefix: "Oversæt følgende tekst til dansk:\n\n".to_string(),
    instruction: "Du er en professionel oversætter...",
    // ... other fields
});
```

**File Priority**: `exe_dir/options.json` > `app_data/options.json` > hardcoded defaults

### 🎨 UI/UX Lessons

#### Compact Popup Design
**User Feedback**: "det vindue skal fylde så lidt så muligt"  
**Final Solution**: 420x280px with 4-column grid, text-only buttons, no icons
```vue
<style>
.operations-grid {
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
.operation-button {
  min-height: 45px;
  max-height: 45px;
}
</style>
```

#### Window Close Behavior
**Pattern**: Different close behaviors for different window types
- **Main window**: Hide instead of close (minimize to tray)
- **Popup windows**: Normal close behavior
- **Chat/Settings**: Normal close with cleanup

```rust
if window.label() == "main" {
    window.hide().unwrap();
    api.prevent_close();
}
```

### 🔌 Plugin Integration Patterns

#### Required Plugins for This App
```toml
[dependencies]
tauri-plugin-opener = "2.0"           # Opening URLs
tauri-plugin-clipboard-manager = "2.0" # Clipboard operations  
tauri-plugin-global-shortcut = "2.0"  # System-wide hotkeys
tauri-plugin-fs = "2.0"              # File system access
urlencoding = "2.1"                  # URL encoding for window creation
```

#### Plugin Usage Pattern
1. **Backend**: Use plugins in Rust commands
2. **Frontend**: Call Rust commands via `invoke()`
3. **Avoid**: Direct plugin access in popup windows (context issues)

## Project Structure Notes

### 🗂️ Rust Backend File Structure

```
src-tauri/src/
├── lib.rs                           # Minimal entry point (~80 lines)
├── main.rs                          # Binary entry point (calls lib::run)
├── ai_provider/
│   ├── mod.rs                       # Module exports & re-exports
│   ├── types.rs                     # Data structures (Content, ChatMessage, etc.)
│   └── gemini.rs                    # GeminiProvider implementation & rate limiting
├── data_manager/
│   ├── mod.rs                       # Module exports & re-exports
│   ├── types.rs                     # Data structures (Config, Operation, etc.)
│   ├── manager.rs                   # DataManager core logic & file I/O
│   └── commands.rs                  # Tauri commands for data operations
├── commands/
│   ├── mod.rs                       # Command module exports
│   ├── ai_commands.rs              # AI-related commands (process_text_with_ai, chat_with_ai)
│   ├── window_commands.rs          # Window management commands (reopen_chat_conversation)
│   └── utility_commands.rs         # Utility commands (greet, simulate_paste)
├── window_manager.rs               # All window creation & management functions
├── tray_manager.rs                 # System tray creation & menu handling
└── shortcut_manager.rs             # Global shortcuts, clipboard & debouncing
```

### 📁 Module Organization Benefits

- **Better Maintainability**: Each file has a clear, focused responsibility
- **Easier Navigation**: Find specific functionality quickly
- **Cleaner Separation**: Window, tray, shortcuts, AI, data all separated
- **Rust Best Practices**: Proper module structure with mod.rs files
- **Reusability**: Modules can be easily tested and reused

### 🔧 Development Notes

- The application uses a lib/main split in Rust (`lib.rs` contains core logic, `main.rs` is minimal entry point)
- Vite is configured to ignore `src-tauri` directory during watch mode  
- Configuration: `withGlobalTauri: true` in tauri.conf.json enables global Tauri object access
- Uses `enigo` crate for keyboard/mouse simulation (Ctrl+C, Ctrl+V)
- All Tauri commands are organized by functionality and re-exported through modules