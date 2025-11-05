# Tauri 2.0 Features & Solutions Documentation

This document provides comprehensive, battle-tested documentation for Tauri 2.0 features used in this project. It includes working code examples, common issues, and solutions that took significant time to discover.

> **Why this exists**: Tauri's official documentation is often incomplete or unclear. This guide documents real-world usage patterns and solutions to common problems.

---

## Table of Contents

1. [Clipboard Manager](#1-clipboard-manager)
2. [Global Shortcut](#2-global-shortcut)
3. [File System](#3-file-system)
4. [Opener](#4-opener)
5. [Process (Executable Path)](#5-process-executable-path)
6. [Tray Icon](#6-tray-icon)
7. [Window Management](#7-window-management)
8. [Commands (Frontend-Backend Communication)](#8-commands-frontend-backend-communication)

---

## 1. Clipboard Manager

**What it does**: Read and write text to the system clipboard.

### Installation

**Cargo.toml:**
```toml
[dependencies]
tauri-plugin-clipboard-manager = "2"
```

**lib.rs:**
```rust
use tauri_plugin_clipboard_manager::ClipboardExt;

tauri::Builder::default()
    .plugin(tauri_plugin_clipboard_manager::init())
    // ... other plugins
```

### Usage

#### Backend (Rust)

```rust
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

// Read clipboard
fn read_clipboard<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| format!("Failed to read clipboard: {}", e))
}

// Write clipboard
fn write_clipboard<R: Runtime>(app: &AppHandle<R>, text: &str) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to write clipboard: {}", e))
}
```

**Real example from our shortcut_manager.rs:**
```rust
// Get current clipboard content
let original_clipboard = app_handle
    .clipboard()
    .read_text()
    .unwrap_or_else(|_| String::new());

// Clear clipboard with a unique marker
let unique_marker = format!(
    "AI_TOOL_MARKER_{}",
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
);

app_handle.clipboard().write_text(&unique_marker)?;
```

### Common Issues & Solutions

#### Issue 1: Popup windows can't access clipboard

**Problem**: Windows created with `decorations(false)` or popup windows can't access the clipboard plugin.

**Solution**: Use `initialization_script()` to inject data before window loads:

```rust
WebviewWindowBuilder::new(app, "popup", tauri::WebviewUrl::App("popup.html".into()))
    .initialization_script(&format!(
        "window.clipboardText = '{}';",
        clipboard_text
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    ))
    .build()?;
```

**Why this works**: Data is injected into the JavaScript context before the window loads, avoiding all plugin permission issues.

#### Issue 2: Clipboard content detection reliability

**Problem**: After simulating Ctrl+C, clipboard content might not be immediately available.

**Solution**: Implement retry logic with delays:

```rust
// First attempt
let new_clipboard = match app_handle.clipboard().read_text() {
    Ok(content) => content,
    Err(_) => {
        // Retry after delay
        std::thread::sleep(std::time::Duration::from_millis(100));
        app_handle.clipboard().read_text().unwrap_or_default()
    }
};
```

---

## 2. Global Shortcut

**What it does**: Register system-wide keyboard shortcuts (e.g., Ctrl+Space) that work even when your app doesn't have focus.

### Installation

**Cargo.toml:**
```toml
[dependencies]
tauri-plugin-global-shortcut = "2"
```

### Usage

#### Method 1: Plugin-based Handler (Recommended)

**lib.rs:**
```rust
use tauri_plugin_global_shortcut::GlobalShortcutExt;

// In setup() function
app.handle().global_shortcut().register("CmdOrCtrl+Space")?;
```

#### Method 2: Custom Handler with Debouncing (Best for complex logic)

**Create a shortcut handler module:**
```rust
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Runtime};

pub fn create_shortcut_handler<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    // Debouncing state
    let last_trigger = Arc::new(Mutex::new(
        Instant::now() - std::time::Duration::from_millis(1000),
    ));

    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, _shortcut, _event| {
            // Debouncing - prevent multiple triggers
            let now = Instant::now();
            {
                let mut last_time = last_trigger.lock().unwrap();
                if now.duration_since(*last_time).as_millis() < 200 {
                    println!("Debouncing - ignoring duplicate trigger");
                    return;
                }
                *last_time = now;
            }

            // Handle the shortcut
            handle_global_shortcut(app.clone())
        })
        .build()
}

fn handle_global_shortcut<R: Runtime>(app: AppHandle<R>) {
    // Your custom logic here
    println!("Global shortcut triggered!");
}
```

**Register in lib.rs:**
```rust
tauri::Builder::default()
    .plugin(shortcut_manager::create_shortcut_handler())
    .setup(|app| {
        // Register the shortcut
        app.handle().global_shortcut().register("CmdOrCtrl+Space")?;
        Ok(())
    })
```

### Common Issues & Solutions

#### Issue 1: Multiple triggers on Windows

**Problem**: On Windows, shortcuts can trigger multiple times from a single key press.

**Solution**: Implement debouncing with `Arc<Mutex<Instant>>`:

```rust
let last_trigger = Arc::new(Mutex::new(Instant::now() - Duration::from_millis(1000)));

// In handler
let now = Instant::now();
{
    let mut last_time = last_trigger.lock().unwrap();
    if now.duration_since(*last_time).as_millis() < 200 {
        return; // Ignore duplicate
    }
    *last_time = now;
}
```

#### Issue 2: Shortcut not working after onboarding

**Problem**: Global shortcut registered before app setup completes doesn't work.

**Solution**: Register shortcut AFTER setup is complete:

```rust
.setup(|app| {
    // Do other setup first
    setup_app(app)?;

    // Register shortcut LAST
    app.handle().global_shortcut().register("CmdOrCtrl+Space")?;
    Ok(())
})
```

#### Issue 3: Cross-platform key codes

**Use `CmdOrCtrl`** for shortcuts that should use Cmd on Mac and Ctrl on Windows/Linux:

```rust
// ✅ Good - works on all platforms
app.global_shortcut().register("CmdOrCtrl+Space")?;

// ❌ Bad - only works on Windows/Linux
app.global_shortcut().register("Ctrl+Space")?;
```

---

## 3. File System

**What it does**: Read and write files to the filesystem using async Tokio operations.

### Installation

**Cargo.toml:**
```toml
[dependencies]
tauri-plugin-fs = "2"
tokio = { version = "1.0", features = ["full"] }
```

**lib.rs:**
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
```

### Usage

#### Backend (Rust)

```rust
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

// Read file
async fn read_json_file<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T, String> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

// Write file
async fn write_json_file<T: serde::Serialize>(path: &PathBuf, data: &T) -> Result<(), String> {
    let json_content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    let mut file = fs::File::create(path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(json_content.as_bytes())
        .await
        .map_err(|e| format!("Failed to write: {}", e))?;

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush: {}", e))?;

    Ok(())
}
```

**Real example from our data_manager/manager.rs:**
```rust
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn save_data(&self) -> Result<(), DataError> {
    // Serialize to JSON
    let json_content = serde_json::to_string_pretty(&self.data)?;

    // Write to file
    let mut file = fs::File::create(&self.file_path).await?;
    file.write_all(json_content.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}
```

### Best Practices

#### 1. Always use async file operations

```rust
// ✅ Good - non-blocking
use tokio::fs;
let content = fs::read_to_string(path).await?;

// ❌ Bad - blocks the thread
use std::fs;
let content = std::fs::read_to_string(path)?;
```

#### 2. Always flush after writing

```rust
let mut file = fs::File::create(path).await?;
file.write_all(data).await?;
file.flush().await?; // ✅ Ensures data is written to disk
```

#### 3. Use proper error handling

```rust
fs::read_to_string(path)
    .await
    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
```

---

## 4. Opener

**What it does**: Open URLs and files in the user's default external application.

### Installation

**Cargo.toml:**
```toml
[dependencies]
tauri-plugin-opener = "2"
```

**lib.rs:**
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
```

**capabilities/default.json:**
```json
{
  "permissions": [
    "opener:default"
  ]
}
```

### Usage

#### Frontend (Vue/TypeScript)

**CRITICAL**: Use the correct import path for Tauri 2.0:

```typescript
// ✅ Correct - Tauri 2.0
import { openUrl } from '@tauri-apps/plugin-opener'

async function openLink(url: string) {
  await openUrl(url)
}
```

```typescript
// ❌ Wrong - Old Tauri 1.x API
import { open } from '@tauri-apps/api/shell' // Don't use this!
```

**Real example from our SettingsWindow.vue:**
```vue
<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'

async function openGeminiDocs() {
  await openUrl('https://aistudio.google.com/app/apikey')
}
</script>

<template>
  <button @click="openGeminiDocs">
    Get API Key
  </button>
</template>
```

### Common Issues & Solutions

#### Issue 1: Buttons not opening URLs

**Problem**: Using wrong import from old Tauri 1.x API.

**Solution**: Use `@tauri-apps/plugin-opener`:

```typescript
// This is the ONLY correct import for Tauri 2.0
import { openUrl } from '@tauri-apps/plugin-opener'
```

#### Issue 2: Permission denied

**Problem**: Missing permissions in capabilities.

**Solution**: Add to `capabilities/default.json`:

```json
{
  "permissions": [
    "opener:default"
  ]
}
```

---

## 5. Process (Executable Path)

**What it does**: Get information about the current process, especially the executable location.

### Usage

No plugin needed - this is built into Rust's standard library.

```rust
use std::path::PathBuf;

// Get executable directory
let exe_dir = std::env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));

if let Some(exe_path) = exe_dir {
    let config_file = exe_path.join("config.json");
    println!("Config at: {:?}", config_file);
}
```

### File Location Strategy

We store user-editable files **next to the executable** (not in hidden AppData), making them easy to find and backup.

**Real example from our data_manager:**
```rust
// Determine file path - prefer exe directory
let file_path = if let Ok(exe_path) = std::env::current_exe() {
    exe_path
        .parent()
        .map(|p| p.join("app_data.json"))
        .unwrap_or_else(|| PathBuf::from("app_data.json"))
} else {
    PathBuf::from("app_data.json")
};
```

**Priority order:**
1. Same directory as executable (easy to find/backup)
2. Current directory (fallback for development)

**Why this is better:**
- ✅ Users can easily find configuration files
- ✅ Easy to backup entire app folder
- ✅ Portable - can move app folder to another location
- ❌ Hidden AppData folders are hard to find

---

## 6. Tray Icon

**What it does**: Create a system tray icon with a context menu.

### Installation

No separate plugin - this is a core Tauri feature.

**Cargo.toml:**
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

### Usage

#### Backend (Rust)

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Runtime,
};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    // Create menu items
    let chat_item = MenuItem::with_id(app, "chat", "Chat", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build menu
    let menu = Menu::with_items(
        app,
        &[&chat_item, &separator, &settings_item, &quit_item],
    )?;

    // Create tray icon
    let _ = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "quit" => app.exit(0),
                "settings" => open_settings_window(app),
                "chat" => open_chat_window(app),
                _ => {}
            }
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                println!("Tray icon clicked");
            }
        })
        .build(app);

    Ok(())
}
```

### Common Issues & Solutions

#### Issue 1: Tray icon duplication

**Problem**: Tray icon appears twice when configured both in `tauri.conf.json` and programmatically.

**Solution**: Use ONLY programmatic creation in `lib.rs`, NOT in `tauri.conf.json`:

```rust
// ✅ In lib.rs - programmatic creation
.setup(|app| {
    tray_manager::create_tray(app.handle())?;
    Ok(())
})
```

```json
// ❌ Don't add this to tauri.conf.json
{
  "app": {
    "trayIcon": { /* ... */ }  // Remove this!
  }
}
```

#### Issue 2: Menu items not responding

**Problem**: Menu event handler not matching item IDs correctly.

**Solution**: Ensure exact ID match between creation and handler:

```rust
// Create with ID
MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;

// Handle with same ID
.on_menu_event(move |app, event| {
    match event.id.as_ref() {
        "settings" => { /* handle */ },  // Must match exactly!
        _ => {}
    }
})
```

#### Issue 3: Tray icon not showing

**Problem**: Creating tray before app is fully initialized.

**Solution**: Create tray in `setup()` AFTER other initialization:

```rust
.setup(|app| {
    // Do other setup first
    initialize_data(app)?;

    // Create tray LAST
    tray_manager::create_tray(app.handle())?;
    Ok(())
})
```

---

## 7. Window Management

**What it does**: Create, configure, and manage application windows.

### Core Concepts

No plugin needed - this is core Tauri functionality.

### Basic Window Creation

```rust
use tauri::{Manager, WebviewWindowBuilder};

WebviewWindowBuilder::new(
    app,
    "window_id",                              // Unique window identifier
    tauri::WebviewUrl::App("page.html".into())  // URL to load
)
.title("Window Title")
.inner_size(800.0, 600.0)
.center()
.build()?;
```

### Advanced Window Configuration

**Real example from our window_manager.rs:**
```rust
WebviewWindowBuilder::new(app, "popup", tauri::WebviewUrl::App("popup.html".into()))
    .title("AI TextTool - Operations")
    .inner_size(300.0, 290.0)
    .position(mouse_x as f64 - 150.0, mouse_y as f64 - 300.0)  // Custom positioning
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .closable(true)
    .always_on_top(true)     // Stay on top of other windows
    .skip_taskbar(true)      // Don't show in taskbar
    .decorations(false)      // Frameless window
    .initialization_script(&format!(
        "window.clipboardText = '{}';",
        clipboard_text.replace('\'', "\\'").replace('\n', "\\n")
    ))
    .build()?;
```

### Generic Window System (Our Optimization)

We created a reusable window configuration system that reduced code by 75%:

```rust
#[derive(Clone)]
pub struct WindowConfig {
    pub window_id: String,
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub position: WindowPosition,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub initialization_script: Option<String>,
    // ... other fields
}

pub enum WindowPosition {
    Center,
    Coordinates { x: f64, y: f64 },
}

pub fn create_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    config: WindowConfig,
) -> tauri::Result<()> {
    let mut builder = WebviewWindowBuilder::new(
        app,
        &config.window_id,
        tauri::WebviewUrl::App(config.url.into()),
    )
    .title(&config.title)
    .inner_size(config.width, config.height)
    .resizable(config.resizable)
    .decorations(config.decorations);

    // Set position
    match config.position {
        WindowPosition::Center => builder = builder.center(),
        WindowPosition::Coordinates { x, y } => builder = builder.position(x, y),
    }

    // Add initialization script if provided
    if let Some(script) = &config.initialization_script {
        builder = builder.initialization_script(script);
    }

    builder.build()?;
    Ok(())
}
```

**Usage:**
```rust
let config = WindowConfig {
    window_id: "settings".to_string(),
    url: "settings.html".to_string(),
    title: "Settings".to_string(),
    width: 600.0,
    height: 700.0,
    position: WindowPosition::Center,
    resizable: true,
    decorations: false,
    always_on_top: false,
    skip_taskbar: false,
    initialization_script: None,
};

create_window(app, config)?;
```

### Common Issues & Solutions

#### Issue 1: Main window shows on startup

**Problem**: Main window appears when app should start minimized to tray.

**Solution**: Hide main window immediately in `setup()`:

```rust
.setup(|app| {
    // Hide main window FIRST
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    }

    // Then setup tray
    tray_manager::create_tray(app.handle())?;
    Ok(())
})
```

#### Issue 2: Popup windows can't be closed

**Problem**: Custom close buttons don't work with `decorations(false)`.

**Solutions:**

**Option 1: Use native decorations**
```rust
.decorations(true)   // Use native window controls
.closable(true)
```

**Option 2: Use custom drag regions and close button**
```rust
.decorations(false)  // Frameless window
```

```vue
<!-- In Vue component -->
<div data-tauri-drag-region class="titlebar">
  <button @click="closeWindow">×</button>
</div>

<script setup lang="ts">
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

const closeWindow = () => {
  getCurrentWebviewWindow().close()
}
</script>

<style>
[data-tauri-drag-region] {
  -webkit-app-region: drag;
}
</style>
```

#### Issue 3: Window position at mouse cursor

**Problem**: Need to position window at exact mouse location.

**Solution**: Use `enigo` to get mouse position, then calculate window position:

```rust
use enigo::{Enigo, Mouse, Settings};

// Get mouse position
let (mouse_x, mouse_y) = if let Ok(enigo_mouse) = Enigo::new(&Settings::default()) {
    enigo_mouse.location().unwrap_or((100, 100))
} else {
    (100, 100)
};

// Position window centered on mouse
.position(
    mouse_x as f64 - (window_width / 2.0),
    mouse_y as f64 - window_height - 10.0
)
```

#### Issue 4: Frontend WebviewWindow creation fails

**Problem**: `new WebviewWindow()` from frontend shows console logs but no window opens.

**Root Cause**: Missing permissions and frontend context limitations in Tauri v2.

**CRITICAL Solution**: Always create windows from backend commands:

```rust
// ✅ Backend command - ALWAYS WORKS
#[tauri::command]
async fn open_chat_window(app: AppHandle) -> Result<(), String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    WebviewWindowBuilder::new(
        &app,
        &format!("chat_{}", timestamp),
        tauri::WebviewUrl::App("chat.html".into())  // Use App URL type
    )
    .title("Chat")
    .inner_size(900.0, 700.0)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;

    Ok(())
}
```

```typescript
// Frontend - call backend command
import { invoke } from '@tauri-apps/api/core'

async function openChat() {
  await invoke('open_chat_window')
}
```

**Why backend is better:**
- Uses `tauri::WebviewUrl::App()` for proper URL handling
- Avoids frontend permission and context issues
- More reliable in Tauri 2.0
- Consistent behavior across platforms

#### Issue 5: Close window vs. hide window

**Problem**: Need different behavior for main window (hide) vs. other windows (close).

**Solution**: Use `on_window_event` to intercept close:

```rust
.on_window_event(|window, event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == "main" {
            // Main window: hide instead of close
            window.hide().unwrap();
            api.prevent_close();
        }
        // Other windows: allow normal close (do nothing)
    }
})
```

---

## 8. Commands (Frontend-Backend Communication)

**What it does**: Expose Rust functions to the frontend via the `invoke()` system.

### Basic Command

**Backend (Rust):**
```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Register in lib.rs
.invoke_handler(tauri::generate_handler![
    greet,
])
```

**Frontend (TypeScript):**
```typescript
import { invoke } from '@tauri-apps/api/core'

const message = await invoke<string>('greet', { name: 'World' })
console.log(message)  // "Hello, World!"
```

### Commands with Error Handling

**Backend:**
```rust
#[tauri::command]
async fn save_config(config: Config) -> Result<(), String> {
    // Do work
    save_to_file(&config)
        .await
        .map_err(|e| format!("Failed to save: {}", e))?;

    Ok(())
}
```

**Frontend:**
```typescript
try {
  await invoke('save_config', { config: myConfig })
  console.log('Success!')
} catch (error) {
  console.error('Failed:', error)
}
```

### Commands with AppHandle

**Backend:**
```rust
use tauri::{AppHandle, State};

#[tauri::command]
async fn process_text(app: AppHandle, text: String) -> Result<String, String> {
    // Can access app resources
    let window = app.get_webview_window("main").ok_or("Window not found")?;

    // Do processing
    Ok(processed_text)
}
```

### Commands with State Management

**Backend:**
```rust
use tauri::State;
use std::sync::Mutex;

struct AppState {
    data: Mutex<Vec<String>>,
}

#[tauri::command]
fn add_item(state: State<AppState>, item: String) -> Result<(), String> {
    state.data
        .lock()
        .map_err(|_| "Lock failed")?
        .push(item);
    Ok(())
}

// Register state in lib.rs
.manage(AppState {
    data: Mutex::new(Vec::new()),
})
```

### Organizing Commands

**Create a commands module:**

**commands/mod.rs:**
```rust
pub mod ai_commands;
pub mod window_commands;
pub mod utility_commands;

// Re-export all commands
pub use ai_commands::*;
pub use window_commands::*;
pub use utility_commands::*;
```

**commands/ai_commands.rs:**
```rust
#[tauri::command]
pub async fn process_text_with_ai(text: String) -> Result<String, String> {
    // AI processing logic
    Ok(result)
}

#[tauri::command]
pub async fn chat_with_ai(message: String) -> Result<String, String> {
    // Chat logic
    Ok(response)
}
```

**lib.rs:**
```rust
mod commands;

.invoke_handler(tauri::generate_handler![
    commands::process_text_with_ai,
    commands::chat_with_ai,
    commands::open_window,
    // ... all other commands
])
```

### Common Issues & Solutions

#### Issue 1: Unawaited invoke() calls

**Problem**: Calling `invoke()` without awaiting causes silent failures.

**Solution**: Always await invoke calls:

```typescript
// ✅ Good
await invoke('my_command', { param: value })

// ❌ Bad - will not work properly
invoke('my_command', { param: value })  // Missing await!
```

**ESLint rule to catch this:**
```javascript
// eslint.config.js
rules: {
  '@typescript-eslint/no-floating-promises': 'error',  // Catches unawaited invokes
}
```

#### Issue 2: Parameter naming mismatch

**Problem**: Parameter names must match exactly between Rust and TypeScript (snake_case vs camelCase).

**Solution**: Use snake_case in both:

```rust
#[tauri::command]
fn process_text(input_text: String) -> String {
    // ...
}
```

```typescript
// ✅ Good - exact match
await invoke('process_text', { input_text: text })

// ❌ Bad - camelCase won't work
await invoke('process_text', { inputText: text })
```

#### Issue 3: Complex types serialization

**Problem**: Passing complex Rust structs to/from frontend.

**Solution**: Use serde with proper derives:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[tauri::command]
async fn send_message(message: ChatMessage) -> Result<ChatMessage, String> {
    Ok(message)
}
```

```typescript
interface ChatMessage {
  role: string
  content: string
  timestamp: string
}

const message: ChatMessage = {
  role: 'user',
  content: 'Hello',
  timestamp: new Date().toISOString()
}

const response = await invoke<ChatMessage>('send_message', { message })
```

#### Issue 4: Async commands with long operations

**Problem**: Long-running operations block the UI.

**Solution**: Use async commands with proper async handling:

```rust
#[tauri::command]
async fn long_operation() -> Result<String, String> {
    // Use tokio for async operations
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    Ok("Done".to_string())
}
```

**Frontend with loading state:**
```typescript
const isLoading = ref(false)

async function runOperation() {
  isLoading.value = true
  try {
    const result = await invoke('long_operation')
    console.log(result)
  } finally {
    isLoading.value = false
  }
}
```

---

## Additional Best Practices

### 1. Error Handling Pattern

**Always use `Result<T, String>` for commands:**
```rust
#[tauri::command]
async fn my_command() -> Result<String, String> {
    do_something()
        .map_err(|e| format!("Operation failed: {}", e))?;
    Ok("Success".to_string())
}
```

### 2. Logging Pattern

**Use println! for development, consider proper logging for production:**
```rust
println!("Window created: {}", window_id);
eprintln!("Error occurred: {}", error);
```

### 3. Window Lifecycle

**Proper cleanup when closing windows:**
```rust
.on_window_event(|window, event| {
    match event {
        tauri::WindowEvent::CloseRequested { .. } => {
            // Clean up resources
            cleanup_window_data(window.label());
        },
        tauri::WindowEvent::Destroyed => {
            println!("Window destroyed: {}", window.label());
        },
        _ => {}
    }
})
```

### 4. Capabilities Configuration

**Always specify required permissions in capabilities/default.json:**
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main", "*"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-create",
    "core:webview:allow-create-webview-window",
    "opener:default"
  ]
}
```

---

## Summary of Critical Lessons

1. **Clipboard in popups**: Use `initialization_script()` instead of direct clipboard access
2. **Global shortcuts**: Always implement debouncing on Windows
3. **Window creation**: Create windows from backend commands, not frontend
4. **Opener plugin**: Use `@tauri-apps/plugin-opener`, not old shell API
5. **Tray icon**: Use programmatic creation only, not config file
6. **File locations**: Store user files next to executable, not AppData
7. **Error handling**: Always use `Result<T, String>` in commands
8. **Async operations**: Always await `invoke()` calls on frontend

---

## Resources

- **Tauri 2.0 Official Docs**: https://v2.tauri.app/
- **Plugin Documentation**: https://v2.tauri.app/plugin/
- **API Reference**: https://docs.rs/tauri/2.0/

---

*This documentation was created based on real-world implementation challenges and solutions. All code examples are battle-tested and production-ready.*
