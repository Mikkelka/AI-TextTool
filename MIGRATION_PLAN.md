# Migration Plan: Python Writing Tool → Tauri Application

## Overview
This document outlines the complete migration strategy for converting the Python-based Writing Tools application (using PySide6/Qt) to a modern Tauri application with Vue 3 frontend and Rust backend.

## Current Python Application Features

### Core Functionality
- **Global Hotkey System**: Ctrl+Space for text capture and processing
- **AI Integration**: Google Gemini API with multiple models (Flash/Flash Lite)
- **Text Operations**: Proofread, Rewrite, Translate, Summarize, Custom operations
- **Chat System**: Full conversation support with follow-up questions
- **Configuration Management**: Settings persistence and user preferences
- **Chat History**: Save, load, and manage conversation history

### UI Components
- Custom popup window for operation selection
- Response window with markdown rendering
- Settings window for configuration
- Chat history browser
- Button editor for custom operations
- Onboarding wizard for first-time setup

## Migration Strategy

## Phase 1: Core Infrastructure (Week 1)

### 1.1 Configuration System
**Rust Backend (`src-tauri/src/config.rs`):**
```rust
- ConfigManager struct for handling config.json
- Options loader for operations.json
- Settings persistence with proper error handling
```

**Data Structures:**
```rust
#[derive(Serialize, Deserialize)]
struct Config {
    api_key: String,
    provider: String,
    shortcut: String,
    chat_model: String,
    text_model: String,
    chat_system_instruction: String,
}

#[derive(Serialize, Deserialize)]
struct Operation {
    prefix: String,
    instruction: String,
    icon: Option<String>,
    open_in_window: bool,
}
```

### 1.2 Gemini AI Provider
**Rust Implementation (`src-tauri/src/ai_provider.rs`):**
```rust
- HTTP client using reqwest
- Async API calls with retry logic
- Rate limiting with exponential backoff
- Response streaming for real-time chat
```

**Key Features:**
- Support for Gemini 2.5 Flash and Flash Lite models
- Thinking mode support
- Conversation context handling
- Error handling and fallback mechanisms

## Phase 2: UI Components (Week 2)

### 2.1 Main Popup Window
**Vue Component (`src/components/PopupWindow.vue`):**
```vue
- Grid layout for operation buttons
- Dynamic positioning at cursor location
- Icon support with theme awareness
- Hover effects and visual feedback
- Chat operation indicators (blue tint)
```

**Features:**
- Keyboard navigation support
- ESC to close
- Auto-hide on focus loss
- Smooth animations

### 2.2 Response/Chat Window
**Vue Component (`src/components/ChatWindow.vue`):**
```vue
- Message bubble interface
- Markdown rendering with syntax highlighting
- User/AI message distinction
- Input field with auto-focus
- Zoom controls (Ctrl +/-)
```

**Sub-components:**
- `MessageBubble.vue` - Individual message display
- `MarkdownRenderer.vue` - Rich text formatting
- `ChatInput.vue` - Input with submit handling

### 2.3 Settings Window
**Vue Component (`src/components/SettingsWindow.vue`):**
```vue
- API key configuration
- Model selection (separate for chat/text)
- Hotkey configuration
- System instruction customization
- Theme preferences
```

## Phase 3: Text Processing Pipeline (Week 3)

### 3.1 Text Capture Enhancement
**Rust Backend (`src-tauri/src/text_operations.rs`):**
```rust
- Improved clipboard handling
- Multi-attempt capture with delays
- Clipboard backup/restore
- Error recovery mechanisms
```

### 3.2 Operation Processing
**Implementation Strategy:**
```rust
#[tauri::command]
async fn process_text_operation(
    text: String,
    operation: String,
    config: State<Config>
) -> Result<ProcessedText, Error> {
    // Load operation definition
    // Call AI provider with appropriate prompt
    // Handle response based on operation type
    // Return processed text or open in window
}
```

### 3.3 Direct Replacement vs Window Display
- Operations with `open_in_window: false` → Direct clipboard replacement
- Operations with `open_in_window: true` → Open chat window
- Automatic paste simulation after replacement

## Phase 4: Advanced Features (Week 4)

### 4.1 Button Customization
**Vue Component (`src/components/ButtonEditor.vue`):**
```vue
- Drag-and-drop interface
- Custom operation creation
- Icon selection
- Prompt configuration
- Save/load custom layouts
```

### 4.2 Chat History Management
**Features:**
```rust
- Automatic conversation saving
- Title generation from content
- Search and filtering
- Conversation continuation
- Export functionality
```

**Vue Component (`src/components/ChatHistory.vue`):**
```vue
- List view with timestamps
- Search bar
- Delete confirmation
- Open in new window
```

### 4.3 Onboarding Flow
**Vue Component (`src/components/Onboarding.vue`):**
```vue
- Welcome screen
- API key setup
- Hotkey configuration
- Test operation
- Success confirmation
```

## Phase 5: Polish & Optimization (Week 5)

### 5.1 Performance Optimizations
- Lazy loading for components
- Virtual scrolling for long chat histories
- Debouncing for hotkey triggers
- Caching for API responses
- Background processing for heavy operations

### 5.2 Error Handling
- User-friendly error messages
- Automatic retry mechanisms
- Offline mode detection
- Graceful degradation
- Error reporting system

### 5.3 UI/UX Enhancements
- Smooth transitions and animations
- Loading states with progress indicators
- Keyboard shortcuts throughout
- Accessibility improvements
- Responsive design for different screen sizes

## Technical Stack

### Backend (Rust)
- **Tauri**: Application framework
- **reqwest**: HTTP client for API calls
- **tokio**: Async runtime
- **serde**: JSON serialization
- **chrono**: Date/time handling
- **log**: Logging framework

### Frontend (Vue 3)
- **Vue 3**: Composition API
- **Pinia**: State management
- **TypeScript**: Type safety
- **Vite**: Build tool
- **Tailwind CSS**: Styling framework
- **markdown-it**: Markdown rendering

## File Structure
```
src/
├── components/
│   ├── PopupWindow.vue
│   ├── ChatWindow.vue
│   ├── SettingsWindow.vue
│   ├── ButtonEditor.vue
│   ├── ChatHistory.vue
│   └── Onboarding.vue
├── stores/
│   ├── config.ts
│   ├── chat.ts
│   └── operations.ts
├── utils/
│   ├── api.ts
│   └── helpers.ts
└── App.vue

src-tauri/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── ai_provider.rs
│   ├── text_operations.rs
│   ├── chat_manager.rs
│   └── commands.rs
└── Cargo.toml
```

## Migration Checklist

### Essential Features (MVP)
- [ ] Global hotkey capture
- [ ] Text selection and processing
- [ ] Basic AI operations (Proofread, Rewrite)
- [ ] Popup window for operation selection
- [ ] Response window with results
- [ ] Settings for API key
- [ ] System tray integration

### Advanced Features
- [ ] Chat with follow-up questions
- [ ] Chat history persistence
- [ ] Custom operations
- [ ] Button editor
- [ ] Multiple AI models
- [ ] Thinking mode
- [ ] Onboarding wizard
- [ ] Theme support

### Polish
- [ ] Animations and transitions
- [ ] Keyboard navigation
- [ ] Error recovery
- [ ] Performance optimization
- [ ] Accessibility
- [ ] Localization support
- [ ] Auto-update system

## Implementation Notes

### Key Differences from Python Version
1. **Threading Model**: Replace Python threading with Rust async/await
2. **UI Framework**: Qt/PySide6 → Vue 3 components
3. **State Management**: Python class attributes → Pinia stores
4. **Deployment**: PyInstaller → Tauri bundler
5. **Resource Management**: Python resource_path → Tauri asset protocol

### Challenges to Address
1. **Clipboard Access**: Ensure proper permissions on all platforms
2. **Global Hotkeys**: Handle platform-specific implementations
3. **Window Positioning**: Accurate cursor position detection
4. **Focus Management**: Proper focus return after operations
5. **API Rate Limiting**: Implement robust retry mechanisms

### Testing Strategy
1. Unit tests for Rust backend functions
2. Component tests for Vue components
3. Integration tests for API communication
4. E2E tests for complete workflows
5. Performance testing for responsiveness
6. Cross-platform testing (Windows/macOS/Linux)

## Timeline

| Week | Phase | Deliverables |
|------|-------|-------------|
| 1 | Core Infrastructure | Config system, AI provider |
| 2 | UI Components | Popup, Chat, Settings windows |
| 3 | Text Processing | Operations pipeline, capture |
| 4 | Advanced Features | Button editor, Chat history |
| 5 | Polish | Performance, Error handling, UX |

## Success Metrics
- All Python features successfully migrated
- Performance improvement (< 100ms hotkey response)
- Reduced memory footprint (< 50MB idle)
- Cross-platform compatibility
- User-friendly error handling
- Smooth UI/UX experience

## Next Steps
1. Set up development environment
2. Create base Tauri project structure
3. Implement configuration system
4. Build AI provider integration
5. Create first UI component (Popup Window)
6. Test basic text capture workflow

---

*This migration plan is a living document and will be updated as the implementation progresses.*