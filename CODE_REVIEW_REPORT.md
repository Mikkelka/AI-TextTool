# Comprehensive Code Review Report: AI-TextTool (Tauri + Vue 3)

**Date**: 2025-11-12  
**Project**: AI-TextTool - AI-powered text processing desktop application  
**Scope**: Frontend (Vue 3/TypeScript), Backend (Rust), Configuration files

---

## Executive Summary

The codebase is well-structured with clear separation of concerns between frontend and backend. However, there are several optimization opportunities, code duplication patterns, type safety improvements, and potential security concerns that should be addressed. The application would benefit from refactoring to reduce code duplication, improve type safety, and enhance error handling.

**Overall Assessment**: The project is production-ready with good architectural patterns but has room for improvement in maintainability and performance.

---

## Frontend Issues (Vue 3 / TypeScript)

### HIGH PRIORITY

#### Issue 1: Type Safety - `any` Type Proliferation
**File**: `/home/user/AI-TextTool/src/types/index.ts` (Line 28)  
**Severity**: HIGH  
**Category**: Type Safety

**Problem**:
```typescript
export interface Config {
  // ...
  providers: Record<string, any>  // <- ANY TYPE!
}

export interface AppError {
  message: string
  code?: string
  details?: any  // <- ANY TYPE!
}
```

The `any` type defeats TypeScript's type checking benefits and masks potential runtime errors.

**Recommended Fix**:
```typescript
// Define provider configuration type
export interface ProviderSettings {
  api_key?: string
  chat_model_name?: string
  text_model_name?: string
  chat_system_instruction?: string
  [key: string]: string | undefined  // Allow other string properties
}

export interface Config {
  // ...
  providers: Record<string, ProviderSettings>  // Type-safe
}

export interface AppError {
  message: string
  code?: string
  details?: Record<string, string | number | boolean | null>  // Specific type
}
```

**Impact**: Medium effort, High improvement in type safety and IDE support

---

#### Issue 2: Security - XSS Risk in Markdown Rendering
**File**: `/home/user/AI-TextTool/src/utils/markdown.ts` (Lines 65, 88-90)  
**Severity**: HIGH  
**Category**: Security

**Problem**:
```typescript
const sanitized = DOMPurify.sanitize(html, {
  ALLOWED_TAGS: [
    // ...
  ],
  ALLOWED_ATTR: ['href', 'target', 'rel', 'class', 'onclick', 'colspan', 'rowspan'],
  // ^ onclick is an EVENT HANDLER - SECURITY RISK!
})

// Line 88-90:
html = html.replace(/<pre><code([^>]*)>([\s\S]*?)<\/code><\/pre>/g, (_match, attrs, content) => {
  return `<pre class="code-block"><code${attrs}>${content}</code><button class="copy-code-btn" onclick="copyCode(this)">📋</button></pre>`
  // ^ Inline onclick handler - vulnerable to XSS!
})
```

DOMPurify is correctly removing dangerous HTML, but then the code adds `onclick` handlers back in, defeating the purpose. Also, `onclick` attribute allows inline script execution.

**Recommended Fix**:
```typescript
// Remove 'onclick' from ALLOWED_ATTR
const sanitized = DOMPurify.sanitize(html, {
  ALLOWED_TAGS: [...],
  ALLOWED_ATTR: ['href', 'target', 'rel', 'class', 'colspan', 'rowspan'],
  // Remove 'onclick'
})

// Replace onclick with data attributes and event delegation
function addCustomClasses(html: string): string {
  // ... other replacements ...
  
  // Add data-copyable attribute instead of onclick
  html = html.replace(/<pre><code([^>]*)>([\s\S]*?)<\/code><\/pre>/g, (_match, attrs, content) => {
    return `<pre class="code-block"><code${attrs}>${content}</code><button class="copy-code-btn" data-copy-code>📋</button></pre>`
  })
  
  return html
}

// In component setup, add event delegation
export function setupMarkdownCopyFunction() {
  document.addEventListener('click', (e: Event) => {
    const target = e.target as HTMLElement
    if (target.classList.contains('copy-code-btn')) {
      const codeBlock = target.parentElement?.querySelector('code')
      if (codeBlock) {
        navigator.clipboard.writeText(codeBlock.textContent || '')
          .then(() => {
            target.textContent = '✅'
            setTimeout(() => { target.textContent = '📋' }, 1000)
          })
          .catch(err => console.error('Failed to copy:', err))
      }
    }
  })
}
```

**Impact**: Low effort, Critical security improvement

---

#### Issue 3: Code Duplication - Chat Message History Preparation
**File**: `/home/user/AI-TextTool/src/components/ChatWindow.vue`  
**Severity**: MEDIUM  
**Category**: Code Duplication

**Problem**:
Same code appears in two places (lines 250-257 and 322-328):

```typescript
// Lines 250-257 in sendMessage():
const chatHistory = state.messages
  .filter(m => !m.isProcessing)
  .map(m => ({
    role: m.role,
    content: m.content,
    timestamp: m.timestamp
  }))

// Lines 322-328 in regenerateResponse():
const chatHistory = state.messages
  .filter(m => !m.isProcessing)
  .map(m => ({
    role: m.role,
    content: m.content,
    timestamp: m.timestamp
  }))
```

**Recommended Fix**:
```typescript
// Extract to a helper method
const prepareChatHistory = (): Array<{role: string; content: string; timestamp: string}> => {
  return state.messages
    .filter(m => !m.isProcessing)
    .map(m => ({
      role: m.role,
      content: m.content,
      timestamp: m.timestamp
    }))
}

// Then use in both places:
const chatHistory = prepareChatHistory()
```

**Impact**: Low effort, Improved maintainability

---

#### Issue 4: User Experience - Using `prompt()` for Input
**File**: `/home/user/AI-TextTool/src/components/ChatWindow.vue` (Line 376)  
**Severity**: MEDIUM  
**Category**: User Experience

**Problem**:
```typescript
const title = prompt(`Save conversation as:`, defaultTitle)
```

Browser `prompt()` is outdated and blocks the UI thread. Provides poor UX, especially for long default titles.

**Recommended Fix**:
```typescript
// Create a reusable dialog component or use modern dialog API
const saveConversation = async () => {
  if (state.messages.length === 0) return

  const firstUserMessage = state.messages.find(m => m.role === 'user')
  const defaultTitle = firstUserMessage
    ? firstUserMessage.content.length > 50
      ? firstUserMessage.content.substring(0, 50) + '...'
      : firstUserMessage.content
    : 'Untitled Conversation'

  // Option 1: Use native <dialog> element (modern)
  // Option 2: Create a SaveTitleDialog component for better UX
  const title = await showSaveTitleDialog(defaultTitle)
  
  if (!title) return
  
  // ... rest of save logic
}
```

**Impact**: Medium effort, Significant UX improvement

---

#### Issue 5: Type Casting Without Safety
**File**: `/home/user/AI-TextTool/src/components/ChatWindow.vue` (Line 504)  
**Severity**: MEDIUM  
**Category**: Type Safety

**Problem**:
```typescript
state.messages = conversation.messages.map(msg => ({
  role: msg.role as 'user' | 'assistant',  // <- Unsafe assertion
  content: msg.content,
  timestamp: msg.timestamp,
  isProcessing: false,
  thoughts: msg.thoughts
}))
```

The `as` type assertion assumes the backend always returns valid roles. If API changes, this could cause runtime errors.

**Recommended Fix**:
```typescript
// Create a validation helper
function isValidRole(role: unknown): role is 'user' | 'assistant' {
  return role === 'user' || role === 'assistant'
}

// Then use it safely:
state.messages = conversation.messages
  .map(msg => {
    if (!isValidRole(msg.role)) {
      console.warn(`Invalid role: ${msg.role}, defaulting to assistant`)
      return {
        role: 'assistant' as const,
        content: msg.content,
        timestamp: msg.timestamp,
        isProcessing: false,
        thoughts: msg.thoughts
      }
    }
    return {
      role: msg.role,
      content: msg.content,
      timestamp: msg.timestamp,
      isProcessing: false,
      thoughts: msg.thoughts
    }
  })
```

**Impact**: Low effort, Improved robustness

---

### MEDIUM PRIORITY

#### Issue 6: Global State Pollution
**File**: `/home/user/AI-TextTool/src/components/PopupWindow.vue` (Line 99)  
**Severity**: MEDIUM  
**Category**: Code Quality

**Problem**:
```typescript
const clipboardText = ref(props.selectedText || (window as any).clipboardText || '')

// And later in cleanup:
;(window as any).clipboardText = ''
```

Using `window as any` pollutes the global namespace and bypasses type safety.

**Recommended Fix**:
```typescript
// Instead of (window as any), use a proper type declaration
declare global {
  interface Window {
    clipboardText?: string
  }
}

// Then use without 'any':
const clipboardText = ref(props.selectedText || window.clipboardText || '')

// Cleanup:
delete window.clipboardText
```

**Impact**: Low effort, Better type safety and code clarity

---

#### Issue 7: Hardcoded Layout Values
**File**: `/home/user/AI-TextTool/src/components/PopupWindow.vue` (Lines 244, 253)  
**Severity**: MEDIUM  
**Category**: Maintainability

**Problem**:
```typescript
case 'ArrowLeft': {
  event.preventDefault()
  const cols = 2  // <- Hardcoded!
  selectedIndex.value = Math.max(0, selectedIndex.value - cols)
  ...
}

case 'ArrowRight': {
  event.preventDefault()
  const columns = 2  // <- Hardcoded again!
  selectedIndex.value = Math.min(operationCount - 1, selectedIndex.value + columns)
  ...
}
```

Hardcoded values make the component inflexible and error-prone if layout changes.

**Recommended Fix**:
```typescript
// Define constant at top of component
const GRID_COLUMNS = 2

const handleKeydown = async (event: KeyboardEvent) => {
  const operationCount = operations.value.length

  switch (event.key) {
    case 'ArrowLeft': {
      event.preventDefault()
      selectedIndex.value = Math.max(0, selectedIndex.value - GRID_COLUMNS)
      void scrollToSelected()
      break
    }
    case 'ArrowRight': {
      event.preventDefault()
      selectedIndex.value = Math.min(operationCount - 1, selectedIndex.value + GRID_COLUMNS)
      void scrollToSelected()
      break
    }
  }
}
```

**Impact**: Very low effort, Improved maintainability

---

#### Issue 8: Unnecessary Reactive State in InputArea
**File**: `/home/user/AI-TextTool/src/components/InputArea.vue` (Lines 77-82)  
**Severity**: MEDIUM  
**Category**: Performance

**Problem**:
```typescript
const handleInputKeydown = (event: KeyboardEvent) => {
  // ... handle Enter key ...

  // Auto-resize textarea
  void nextTick(() => {  // <- First nextTick
    if (messageInput.value) {
      messageInput.value.style.height = 'auto'
      messageInput.value.style.height = Math.min(messageInput.value.scrollHeight, 120) + 'px'
    }
  })
}

const focusInput = async () => {
  await nextTick()  // <- Repeated nextTick pattern
  messageInput.value?.focus()
}

const clearInput = () => {
  currentMessage.value = ''
  void nextTick(() => {  // <- Third nextTick
    if (messageInput.value) {
      messageInput.value.style.height = 'auto'
    }
  })
}
```

Multiple similar `nextTick` patterns for textarea management.

**Recommended Fix**:
```typescript
// Create a dedicated method for textarea operations
const resizeTextarea = async () => {
  await nextTick()
  if (messageInput.value) {
    messageInput.value.style.height = 'auto'
    messageInput.value.style.height = Math.min(messageInput.value.scrollHeight, 120) + 'px'
  }
}

const resetTextarea = async () => {
  await nextTick()
  if (messageInput.value) {
    messageInput.value.style.height = 'auto'
  }
}

const handleInputKeydown = (event: KeyboardEvent) => {
  // ... handle Enter key ...
  void resizeTextarea()
}

const clearInput = () => {
  currentMessage.value = ''
  void resetTextarea()
}
```

**Impact**: Low effort, Improved code organization

---

#### Issue 9: No Input Validation on Message Content
**File**: `/home/user/AI-TextTool/src/components/ChatWindow.vue`  
**Severity**: MEDIUM  
**Category**: Data Integrity

**Problem**:
No validation of message content before sending to backend:
```typescript
const handleSendMessage = async () => {
  if (!inputArea.value) return

  const userMessage = inputArea.value.getCurrentMessage()
  if (!userMessage) return  // Only checks if empty

  inputArea.value.clearInput()
  await sendMessage(userMessage)  // No content validation
}
```

**Recommended Fix**:
```typescript
const MAX_MESSAGE_LENGTH = 10000
const MIN_MESSAGE_LENGTH = 1

const isValidMessage = (message: string): boolean => {
  const trimmed = message.trim()
  return trimmed.length >= MIN_MESSAGE_LENGTH && trimmed.length <= MAX_MESSAGE_LENGTH
}

const handleSendMessage = async () => {
  if (!inputArea.value) return

  const userMessage = inputArea.value.getCurrentMessage()
  if (!isValidMessage(userMessage)) {
    state.error = `Message must be between ${MIN_MESSAGE_LENGTH} and ${MAX_MESSAGE_LENGTH} characters`
    return
  }

  inputArea.value.clearInput()
  await sendMessage(userMessage)
}
```

**Impact**: Low effort, Improved data integrity

---

### LOW PRIORITY

#### Issue 10: Accessibility - Missing ARIA Labels
**Files**: Multiple Vue components  
**Severity**: LOW  
**Category**: Accessibility

**Problem**:
Buttons lack proper ARIA labels for screen readers:
```vue
<button
  class="action-btn save-btn"
  :disabled="state.messages.length === 0"
  title="Save conversation to history"
  @click="saveConversation"
>
  💾
</button>
```

**Recommended Fix**:
```vue
<button
  class="action-btn save-btn"
  :disabled="state.messages.length === 0"
  title="Save conversation to history"
  aria-label="Save conversation to history"
  @click="saveConversation"
>
  💾
</button>
```

**Impact**: Low effort, Improved accessibility compliance

---

#### Issue 11: Console Logs in Production
**File**: Multiple Vue components  
**Severity**: LOW  
**Category**: Code Quality

**Problem**:
Many console.log statements throughout components:
```typescript
console.log('ChatWindow mounted with props:', { ... })
console.log('Loading existing configuration...')
console.log('Settings saved successfully')
```

These should be managed through a proper logging system.

**Recommended Fix**:
```typescript
// Create a logging utility
// src/utils/logger.ts
export const logger = {
  debug: (message: string, data?: unknown) => {
    if (import.meta.env.DEV) {
      console.log(`[DEBUG] ${message}`, data)
    }
  },
  error: (message: string, error?: unknown) => {
    console.error(`[ERROR] ${message}`, error)
  },
  warn: (message: string, data?: unknown) => {
    console.warn(`[WARN] ${message}`, data)
  }
}

// Use in components:
logger.debug('ChatWindow mounted with props:', { operation, title })
```

**Impact**: Low effort, Better production readiness

---

## Backend Issues (Rust)

### HIGH PRIORITY

#### Issue 12: Code Duplication in Command Functions
**File**: `/home/user/AI-TextTool/src-tauri/src/commands/ai_commands.rs`  
**Severity**: HIGH  
**Category**: Code Duplication

**Problem**:
DataManager initialization is repeated in multiple functions (lines 20-25, 87-92):

```rust
// In process_text_with_ai():
let mut manager = DataManager::new(app.clone());
manager
    .initialize()
    .await
    .map_err(|e| format!("Failed to initialize data: {}", e))?;
let config = manager.get_config().clone();

// Later in chat_with_ai():
let mut manager = DataManager::new(app);
manager
    .initialize()
    .await
    .map_err(|e| format!("Failed to initialize data: {}", e))?;
let config = manager.get_config().clone();

// And again in test_ai_connection():
let mut manager = DataManager::new(app);
manager
    .initialize()
    .await
    .map_err(|e| format!("Failed to initialize data: {}", e))?;
let config = manager.get_config().clone();
```

**Recommended Fix**:
```rust
// Create a helper function in the module
async fn load_config(app: tauri::AppHandle) -> Result<Config, String> {
    let mut manager = DataManager::new(app);
    manager
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize data: {}", e))?;
    Ok(manager.get_config().clone())
}

// Then use in commands:
#[tauri::command]
pub async fn process_text_with_ai(
    text: String,
    operation: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let config = load_config(app.clone()).await?;
    // ... rest of function
}

#[tauri::command]
pub async fn chat_with_ai(
    message: String,
    history: Vec<ChatMessage>,
    custom_instruction: Option<String>,
    enable_thinking: Option<bool>,
    app: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    let config = load_config(app).await?;
    // ... rest of function
}
```

**Impact**: Low effort, Significant code reduction

---

#### Issue 13: Thread Sleep in Async Context
**File**: `/home/user/AI-TextTool/src-tauri/src/shortcut_manager.rs` (Lines 44, 74, 88)  
**Severity**: HIGH  
**Category**: Performance/Correctness

**Problem**:
Using `std::thread::sleep()` blocks the entire async runtime:

```rust
async fn process_shortcut_trigger<R: Runtime>(app_handle: AppHandle<R>) {
    // Small initial delay to let any ongoing operations (like Ctrl+A) complete
    std::thread::sleep(std::time::Duration::from_millis(50));  // <- BLOCKS!

    // ... later ...
    std::thread::sleep(std::time::Duration::from_millis(50));  // <- BLOCKS!

    // ... and again ...
    std::thread::sleep(std::time::Duration::from_millis(250));  // <- BLOCKS!

    // ... first retry ...
    std::thread::sleep(std::time::Duration::from_millis(100));  // <- BLOCKS!
}
```

`thread::sleep()` blocks the entire async runtime, preventing other tasks from executing.

**Recommended Fix**:
```rust
use tokio::time::sleep;

async fn process_shortcut_trigger<R: Runtime>(app_handle: AppHandle<R>) {
    // Small initial delay to let any ongoing operations (like Ctrl+A) complete
    sleep(Duration::from_millis(50)).await;  // <- Non-blocking!

    // ... later ...
    sleep(Duration::from_millis(50)).await;  // <- Non-blocking!

    // ... and again ...
    sleep(Duration::from_millis(250)).await;  // <- Non-blocking!

    // ... first retry ...
    sleep(Duration::from_millis(100)).await;  // <- Non-blocking!
}
```

**Impact**: Medium effort, Critical performance improvement

---

#### Issue 14: Hardcoded Rate Limiting Configuration
**File**: `/home/user/AI-TextTool/src-tauri/src/ai_provider/gemini.rs` (Line 114)  
**Severity**: MEDIUM  
**Category**: Configuration

**Problem**:
```rust
pub fn new(api_key: String) -> Result<Self, GeminiError> {
    // ...
    Ok(Self {
        client,
        api_key,
        base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new(15))),  // <- HARDCODED!
        default_generation_config: GenerationConfig::default(),
        default_safety_settings,
        max_retries: 3,
    })
}
```

Rate limiting should be configurable, not hardcoded. Gemini API rate limits vary by model and can change.

**Recommended Fix**:
```rust
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    default_generation_config: GenerationConfig,
    default_safety_settings: Vec<SafetySetting>,
    max_retries: u32,
    requests_per_minute: usize,  // <- Make configurable
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Result<Self, GeminiError> {
        Self::with_rate_limit(api_key, 15)  // Default
    }

    pub fn with_rate_limit(api_key: String, requests_per_minute: usize) -> Result<Self, GeminiError> {
        // ...
        Ok(Self {
            // ...
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(requests_per_minute))),
            requests_per_minute,
        })
    }

    pub fn get_available_models() -> &'static [&'static str] {
        &["gemini-2.5-flash", "gemini-2.5-flash-lite"]
    }
}
```

**Impact**: Medium effort, Better flexibility and configuration management

---

#### Issue 15: SystemTime Handling Bloat
**File**: `/home/user/AI-TextTool/src-tauri/src/commands/window_commands.rs` (Lines 12-15)  
**Severity**: MEDIUM  
**Category**: Code Quality

**Problem**:
```rust
let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis();

let window_id = format!("chat_reopen_{}", timestamp);
```

And similar code in `open_chat_window()`. This pattern is repeated and could be extracted. Also, `.unwrap()` is risky.

**Recommended Fix**:
```rust
// Add utility function
fn get_current_timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)  // Fallback to 0 instead of panic
}

// Use everywhere:
let timestamp = get_current_timestamp_millis();
let window_id = format!("chat_reopen_{}", timestamp);
```

**Impact**: Low effort, Improved code reusability and robustness

---

#### Issue 16: Data Cloning for Persistence
**File**: `/home/user/AI-TextTool/src-tauri/src/data_manager/manager.rs` (Line 60)  
**Severity**: MEDIUM  
**Category**: Performance

**Problem**:
```rust
pub async fn save_data(&self) -> Result<(), DataError> {
    // Update metadata
    let mut data = self.data.clone();  // <- Cloning entire AppData!
    data.metadata.last_updated = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Serialize to JSON
    let json_content = serde_json::to_string_pretty(&data)?;
    // ...
}
```

Cloning the entire AppData structure (which could be large with chat history) is inefficient.

**Recommended Fix**:
```rust
pub async fn save_data(&self) -> Result<(), DataError> {
    // Create a wrapper that updates metadata on serialization
    #[derive(Serialize)]
    struct AppDataSnapshot<'a> {
        #[serde(flatten)]
        data: &'a AppData,
        #[serde(skip)]
        metadata: Metadata,
    }

    // Or simpler: serialize with custom Serialize impl
    let mut data_copy = self.data.clone();
    data_copy.metadata.last_updated = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let json_content = serde_json::to_string_pretty(&data_copy)?;
    
    let mut file = fs::File::create(&self.file_path).await?;
    file.write_all(json_content.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}

// Or better: use Cow (Copy-On-Write) pattern if appropriate
```

**Impact**: Medium effort, Potential performance improvement for large datasets

---

#### Issue 17: Array Drain Optimization
**File**: `/home/user/AI-TextTool/src-tauri/src/data_manager/manager.rs` (Lines 184-187)  
**Severity**: LOW  
**Category**: Performance

**Problem**:
```rust
// Keep only last 100 entries
if self.data.chat_history.len() > 100 {
    self.data
        .chat_history
        .drain(0..self.data.chat_history.len() - 100);  // <- Inefficient!
}
```

The `drain()` call removes from the beginning of the vec, which is O(n) complexity.

**Recommended Fix**:
```rust
// Keep only last 100 entries
const MAX_HISTORY_ENTRIES: usize = 100;
if self.data.chat_history.len() > MAX_HISTORY_ENTRIES {
    let excess = self.data.chat_history.len() - MAX_HISTORY_ENTRIES;
    self.data.chat_history.drain(0..excess);
}

// OR better - use a more idiomatic approach:
if self.data.chat_history.len() > MAX_HISTORY_ENTRIES {
    let drain_count = self.data.chat_history.len() - MAX_HISTORY_ENTRIES;
    let _ = self.data.chat_history.drain(0..drain_count);
}

// OR best - maintain the limit upfront:
pub async fn add_chat_entry(&mut self, entry: ChatEntry) -> Result<(), DataError> {
    self.data.chat_history.push(entry);
    
    // Keep only last 100 entries (more efficient)
    while self.data.chat_history.len() > 100 {
        self.data.chat_history.remove(0);  // Or use swap_remove for better perf
    }
    
    self.save_data().await
}
```

**Impact**: Very low effort, Minor performance improvement

---

### MEDIUM PRIORITY

#### Issue 18: Duplicated File Path Logic
**File**: `/home/user/AI-TextTool/src-tauri/src/lib.rs` (Lines 27-34) and `/home/user/AI-TextTool/src-tauri/src/data_manager/manager.rs` (Lines 28-35)  
**Severity**: MEDIUM  
**Category**: Code Duplication

**Problem**:
The file path determination logic is duplicated:

```rust
// In lib.rs
let config_path = if let Ok(exe_path) = std::env::current_exe() {
    exe_path
        .parent()
        .map(|parent| parent.join("app_data.json"))
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("app_data.json"))
} else {
    std::env::current_dir().unwrap().join("app_data.json")
};

// In manager.rs - SAME LOGIC
self.file_path = if let Ok(exe_path) = std::env::current_exe() {
    exe_path
        .parent()
        .map(|p| p.join("app_data.json"))
        .unwrap_or_else(|| PathBuf::from("app_data.json"))
} else {
    PathBuf::from("app_data.json")
};
```

**Recommended Fix**:
```rust
// Create a utility module: src-tauri/src/utils/mod.rs
pub mod file_paths;

// In src-tauri/src/utils/file_paths.rs
use std::path::PathBuf;

pub fn get_app_data_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("app_data.json"))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|d| d.join("app_data.json"))
                .unwrap_or_else(|_| PathBuf::from("app_data.json"))
        })
}

// Use in both places:
let config_path = get_app_data_path();
```

**Impact**: Low effort, Improved maintainability

---

#### Issue 19: Unsafe Unwrap Usage
**File**: Multiple Rust files  
**Severity**: MEDIUM  
**Category**: Error Handling

**Problem**:
Several `.unwrap()` calls that could panic:

```rust
// shortcut_manager.rs, line 78
if let Err(e) = simulate_copy() {
    // ... but no handling for all paths
}

// window_commands.rs, lines 14-15
.duration_since(std::time::UNIX_EPOCH)
.unwrap()  // <- Could panic!
```

**Recommended Fix**:
```rust
// Better error handling:
fn get_current_timestamp_millis() -> Result<u128, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .map_err(|e| format!("Failed to get current time: {}", e))
}

// Or with fallback:
fn get_current_timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_else(|e| {
            eprintln!("Failed to get current time: {}", e);
            0  // Fallback
        })
}
```

**Impact**: Medium effort, Improved robustness

---

#### Issue 20: Missing Error Context
**File**: `/home/user/AI-TextTool/src-tauri/src/commands/ai_commands.rs` (Lines 36-38)  
**Severity**: MEDIUM  
**Category**: Error Handling

**Problem**:
```rust
let provider = match GeminiProvider::new(config.api_key) {
    Ok(provider) => provider,
    Err(e) => return Err(format!("Failed to create AI provider: {}", e)),
};
```

Error message is generic and doesn't provide actionable information.

**Recommended Fix**:
```rust
let provider = GeminiProvider::new(config.api_key)
    .map_err(|e| {
        let context = match e {
            GeminiError::InvalidApiKey => "API key is invalid. Please check your settings.",
            GeminiError::Timeout => "Connection timed out. Please check your internet connection.",
            GeminiError::ServiceUnavailable => "Gemini service is currently unavailable. Please try again later.",
            other => &format!("Failed to initialize AI provider: {}", other),
        };
        context.to_string()
    })?;
```

**Impact**: Medium effort, Significantly better error messages for users

---

### LOW PRIORITY

#### Issue 21: Println Debugging in Production
**File**: Multiple files (lib.rs, shortcut_manager.rs, window_commands.rs, etc.)  
**Severity**: LOW  
**Category**: Code Quality

**Problem**:
```rust
println!("Main window hidden on startup");
println!("Global shortcut 'Ctrl+Space' registered successfully!");
println!("Failed to register global shortcut: {:?}", e);
```

Using `println!` for logging mixes output with actual logging needs.

**Recommended Fix**:
```rust
// Use the 'tracing' or 'log' crate
use log::{debug, info, warn, error};

debug!("Main window hidden on startup");
info!("Global shortcut 'Ctrl+Space' registered successfully");
warn!("Failed to register global shortcut: {:?}", e);
```

Or configure `println!` output based on environment:

```rust
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}
```

**Impact**: Low effort, Better production readiness

---

#### Issue 22: Missing Documentation Comments
**File**: Multiple Rust files  
**Severity**: LOW  
**Category**: Documentation

**Problem**:
Many public functions lack documentation:

```rust
pub struct GeminiProvider {
    client: Client,
    api_key: String,
    // ...
}

pub async fn generate_content(
    &self,
    model: &str,
    contents: Vec<Content>,
    system_instruction: Option<&str>,
    generation_config: Option<GenerationConfig>,
) -> Result<String, GeminiError> {
    // Missing: What does this do? What are the parameters?
}
```

**Recommended Fix**:
```rust
/// Generates content using the Gemini API with the specified model.
///
/// # Arguments
///
/// * `model` - The model identifier (e.g., "gemini-2.5-flash")
/// * `contents` - The content to send to the model
/// * `system_instruction` - Optional system-level instructions for the model
/// * `generation_config` - Optional generation configuration (temperature, tokens, etc.)
///
/// # Returns
///
/// Returns the generated text content or an error if the request fails.
///
/// # Example
///
/// ```
/// let content = vec![Content::user("Explain quantum computing")];
/// let response = provider.generate_content(
///     "gemini-2.5-flash",
///     content,
///     None,
///     None
/// ).await?;
/// ```
pub async fn generate_content(
    &self,
    model: &str,
    contents: Vec<Content>,
    system_instruction: Option<&str>,
    generation_config: Option<GenerationConfig>,
) -> Result<String, GeminiError> {
    // ...
}
```

**Impact**: Low effort, Significantly improved code maintainability

---

## Configuration & Build Issues

### HIGH PRIORITY

#### Issue 23: Overly Permissive Cargo Dependencies
**File**: `/home/user/AI-TextTool/src-tauri/Cargo.toml` (Line 31)  
**Severity**: MEDIUM  
**Category**: Dependencies

**Problem**:
```toml
tokio = { version = "1.0", features = ["full"] }
```

Using `features = ["full"]` includes many unnecessary features, increasing binary size and build time.

**Recommended Fix**:
```toml
tokio = { version = "1.0", features = [
    "rt",         # Runtime
    "macros",     # #[tokio::main]
    "time",       # sleep, intervals
    "sync",       # Mutex, channels
    "fs",         # File I/O
    "io-util",    # AsyncRead/Write utilities
] }
```

**Impact**: Low effort, Reduced binary size, faster builds

---

#### Issue 24: Missing Security Validation
**File**: `/home/user/AI-TextTool/src-tauri/src/commands/ai_commands.rs`  
**Severity**: MEDIUM  
**Category**: Security

**Problem**:
No validation of API responses before use:

```rust
let result = match provider
    .process_text_operation(
        &full_prompt,
        &operation.to_lowercase(),
        Some(&operation_details.instruction),
        &config.text_model,
    )
    .await
{
    Ok(result) => result,
    Err(e) => return Err(format!("AI processing failed: {}", e)),
};

// Result used directly without validation!
```

**Recommended Fix**:
```rust
fn validate_ai_response(response: &str, max_length: usize) -> Result<(), String> {
    if response.is_empty() {
        return Err("AI response is empty".to_string());
    }
    if response.len() > max_length {
        return Err(format!("Response exceeds maximum length of {} characters", max_length));
    }
    Ok(())
}

let result = provider
    .process_text_operation(...)
    .await?;

validate_ai_response(&result, 50000)?;  // Validate before using
```

**Impact**: Medium effort, Improved security posture

---

### MEDIUM PRIORITY

#### Issue 25: Missing Input Validation at API Boundaries
**File**: Multiple command functions  
**Severity**: MEDIUM  
**Category**: Data Validation

**Problem**:
Commands don't validate input parameters:

```rust
#[tauri::command]
pub async fn process_text_with_ai(
    text: String,
    operation: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // No validation that text or operation are reasonable
    // No length checks
    // No content checks
}
```

**Recommended Fix**:
```rust
const MAX_TEXT_LENGTH: usize = 100_000;
const MIN_TEXT_LENGTH: usize = 1;

fn validate_text_input(text: &str) -> Result<(), String> {
    let trimmed = text.trim();
    
    if trimmed.is_empty() {
        return Err("Text cannot be empty".to_string());
    }
    
    if trimmed.len() < MIN_TEXT_LENGTH {
        return Err(format!("Text must be at least {} characters", MIN_TEXT_LENGTH));
    }
    
    if trimmed.len() > MAX_TEXT_LENGTH {
        return Err(format!("Text cannot exceed {} characters", MAX_TEXT_LENGTH));
    }
    
    Ok(())
}

#[tauri::command]
pub async fn process_text_with_ai(
    text: String,
    operation: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    validate_text_input(&text)?;
    
    // ... rest of function
}
```

**Impact**: Medium effort, Improved robustness and security

---

#### Issue 26: Package.json Security Script Missing
**File**: `/home/user/AI-TextTool/package.json`  
**Severity**: LOW  
**Category**: Dependencies

**Problem**:
No npm audit or dependency check scripts:

```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "lint": "npm run lint:vue && npm run lint:rust",
    // Missing: "audit", "security-check"
  }
}
```

**Recommended Fix**:
```json
{
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "lint": "npm run lint:vue && npm run lint:rust",
    "audit": "npm audit",
    "audit:fix": "npm audit fix",
    "security-check": "npm audit --audit-level=moderate"
  }
}
```

**Impact**: Low effort, Better security practices

---

## Summary Table

| Issue # | File | Severity | Category | Effort | Impact |
|---------|------|----------|----------|--------|--------|
| 1 | types/index.ts | HIGH | Type Safety | Low | High |
| 2 | utils/markdown.ts | HIGH | Security | Low | Critical |
| 3 | ChatWindow.vue | MEDIUM | Duplication | Low | Medium |
| 4 | ChatWindow.vue | MEDIUM | UX | Medium | High |
| 5 | ChatWindow.vue | MEDIUM | Type Safety | Low | Medium |
| 6 | PopupWindow.vue | MEDIUM | Quality | Low | Medium |
| 7 | PopupWindow.vue | MEDIUM | Maintainability | Very Low | Medium |
| 8 | InputArea.vue | MEDIUM | Performance | Low | Low |
| 9 | ChatWindow.vue | MEDIUM | Integrity | Low | Medium |
| 10 | Multiple | LOW | Accessibility | Low | Low |
| 11 | Multiple | LOW | Quality | Low | Low |
| 12 | ai_commands.rs | HIGH | Duplication | Low | Medium |
| 13 | shortcut_manager.rs | HIGH | Performance | Medium | Critical |
| 14 | gemini.rs | MEDIUM | Configuration | Medium | Medium |
| 15 | window_commands.rs | MEDIUM | Quality | Low | Low |
| 16 | manager.rs | MEDIUM | Performance | Medium | Low |
| 17 | manager.rs | LOW | Performance | Very Low | Low |
| 18 | Multiple | MEDIUM | Duplication | Low | Medium |
| 19 | Multiple | MEDIUM | Error Handling | Medium | Medium |
| 20 | ai_commands.rs | MEDIUM | UX | Medium | Medium |
| 21 | Multiple | LOW | Quality | Low | Low |
| 22 | Multiple | LOW | Documentation | Low | Low |
| 23 | Cargo.toml | MEDIUM | Dependencies | Low | Medium |
| 24 | ai_commands.rs | MEDIUM | Security | Medium | Medium |
| 25 | Multiple | MEDIUM | Validation | Medium | High |
| 26 | package.json | LOW | Dependencies | Low | Low |

---

## Recommendations for Next Steps

### Immediate Actions (Critical - Do First)
1. **Issue #2**: Remove XSS risk in markdown rendering (event handlers)
2. **Issue #13**: Replace thread::sleep with tokio::time::sleep in shortcut manager
3. **Issue #24**: Add response validation for AI provider

### Short Term (1-2 Weeks)
4. Extract DataManager initialization into helper function (Issue #12)
5. Fix type safety issues with `any` types (Issue #1)
6. Add input validation at API boundaries (Issue #25)
7. Improve error messages and context (Issue #20)

### Medium Term (2-4 Weeks)
8. Refactor duplicate code across components
9. Add comprehensive logging system
10. Implement proper dialog component instead of `prompt()`
11. Add unit tests for critical functions
12. Document public APIs with doc comments

### Long Term (Quality Improvements)
13. Set up continuous security scanning
14. Add performance monitoring
15. Implement proper analytics/telemetry
16. Add end-to-end tests

---

## Conclusion

The AI-TextTool codebase demonstrates solid architectural decisions with clear separation between frontend (Vue 3) and backend (Rust). The main opportunities for improvement are:

1. **Reduce code duplication** through helper functions and utilities
2. **Improve type safety** by eliminating `any` types
3. **Fix security concerns** in markdown rendering
4. **Optimize async operations** by avoiding thread::sleep in async contexts
5. **Add comprehensive validation** at API boundaries

Most issues are straightforward to fix and would significantly improve code quality, maintainability, and security. The estimated effort to address all issues is 2-3 weeks for a team of 2-3 developers.

