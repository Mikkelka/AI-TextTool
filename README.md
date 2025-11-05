# AI TextTool

A powerful desktop application that brings AI-powered text processing to your fingertips with a simple global hotkey. Transform any text, anywhere on your system, using Google Gemini AI.

## 🌟 Features

### ⌨️ **Global Hotkey Access**
- **Ctrl+Space** to instantly process selected text from any application
- Automatic text capture and result replacement
- Works system-wide across all applications

### 🤖 **10 AI-Powered Operations**
**Direct Processing** (auto-paste results):
- **Proofread** - Grammar and spelling correction
- **Rewrite** - Text improvement and rephrasing
- **🇩🇰 Dansk** - Professional Danish translation
- **Concise** - Make text more concise
- **Friendly** - Adjust tone to be more friendly
- **Professional** - Make text more professional

**Chat Windows** (open in new window):
- **Key Points** - Extract key information as markdown list
- **Summary** - Comprehensive summarization with formatting
- **Chat** - Open conversation with AI
- **Custom** - Custom instructions from user

### 💬 **Conversation Management**
- Full chat history with searchable entries
- Save complete conversations with custom titles
- Export conversations as markdown
- Operation-specific AI instructions

### ⚙️ **Professional Configuration**
- Google AI Studio API key integration
- Choose between Gemini Flash and Flash-Lite models
- Customizable system instructions
- Configurable global shortcuts

## 🚀 Quick Start

1. **Download** the latest release from the [Releases](../../releases) page
2. **Install** and launch AI TextTool
3. **Setup** your Google AI Studio API key in the onboarding wizard
4. **Use** Ctrl+Space to process text from any application

## 📖 How It Works

1. **Select text** in any application (Word, browser, email, etc.)
2. **Press Ctrl+Space** to open the operation selector
3. **Choose an operation** from the compact popup menu
4. **Get instant results** - either auto-pasted or in a new chat window

## 🛠️ Technology Stack

- **Frontend**: Vue 3 + TypeScript with Composition API
- **Backend**: Rust with Tauri framework
- **AI Integration**: Google Gemini API with rate limiting
- **Platform**: Cross-platform desktop application

## 📁 Project Structure

AI TextTool uses **Tauri's hybrid architecture** - a Vue.js frontend communicates with a Rust backend:

```
AI-TextTool/
├── src/                     # 🌐 Vue 3 Frontend (runs in webview)
│   ├── components/          # Vue components
│   │   ├── PopupWindow.vue      # Operation selector (Ctrl+Space popup)
│   │   ├── ChatWindow.vue       # AI conversation windows
│   │   ├── MessageBubble.vue    # Reusable message display
│   │   ├── InputArea.vue        # Reusable chat input
│   │   ├── SettingsWindow.vue   # Settings configuration
│   │   ├── ChatHistoryWindow.vue # History management
│   │   └── OnboardingWindow.vue # First-time setup
│   ├── types/               # Shared TypeScript interfaces
│   ├── utils/               # Utility functions (markdown rendering)
│   ├── assets/              # CSS, images, static files
│   └── main.ts              # Vue app entry point
├── src-tauri/              # 🦀 Rust Backend (native desktop app)
│   ├── src/
│   │   ├── lib.rs           # Tauri app setup & command registration
│   │   ├── main.rs          # Binary entry point
│   │   ├── ai_provider/     # Google Gemini AI integration
│   │   ├── data_manager/    # Configuration & file I/O
│   │   ├── commands/        # Tauri commands (called from frontend)
│   │   ├── window_manager.rs # Window creation & management
│   │   ├── tray_manager.rs  # System tray functionality
│   │   └── shortcut_manager.rs # Global hotkeys & clipboard
│   ├── target/debug/        # 📁 Development build + app_data.json
│   ├── target/release/      # 📁 Production build + app_data.json
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── *.html                   # 📄 Separate window templates
│   ├── index.html           # Main app window (hidden, tray only)
│   ├── popup.html           # Operation selector (Ctrl+Space popup)
│   ├── chat.html            # Chat windows
│   ├── settings.html        # Settings window
│   ├── history.html         # Chat history window
│   └── onboarding.html      # First-time setup wizard
├── public/                  # 📄 Static assets (icons, etc.)
└── dist/                    # 📦 Built Vue frontend (served to webview)
```

### Why This Structure?

**`src/` (Frontend)**: Vue.js code that runs in Tauri's webview (like a browser)
- Handles UI, user interactions, and visual components
- Calls Rust functions using `invoke("command_name")`

**`src-tauri/` (Backend)**: Native Rust code that runs as the desktop application
- System integration (global shortcuts, clipboard, file I/O)
- AI API calls and data processing
- Window management and system tray
- Exposes commands that frontend can call

**Communication**: Frontend calls backend using Tauri's `invoke()` system - this is how Vue talks to Rust!

## 🔧 Development

### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+
- Git

### Setup
```bash
git clone https://github.com/yourusername/AI-TextTool.git
cd AI-TextTool
npm install
```

### Development Commands
```bash
# Start development environment
npm run tauri dev

# Build for production
npm run tauri build

# Frontend only development
npm run dev
```

### Data Storage
All application data is stored in a **single JSON file** next to the executable:
- **`app_data.json`** - Unified data file containing:
  - App settings and API keys
  - Text operations configuration
  - Individual operation history
  - Complete chat conversations
  - Version metadata

**Migration**: If you're upgrading from an older version, the app automatically migrates your old files (`config.json`, `options.json`, `chat_history.json`, `saved_conversations.json`) to the new format on first launch. Old files are safely renamed to `.old`.

### Code Quality
```bash
# Lint and format checks
npm run check              # Run all linting (ESLint + Prettier + Clippy)
npm run lint               # Check ESLint errors/warnings
npm run lint:fix           # Auto-fix ESLint issues
npm run format             # Format code with Prettier
npm run lint:rust          # Run Clippy on Rust code
npm run format:rust        # Format Rust code with rustfmt
```

**Important**: Always run `npm run check` before committing to ensure code quality!

## 📚 Documentation

- **[CLAUDE.md](CLAUDE.md)** - Complete project documentation for developers
- **[Tauri2.0-docs.md](Tauri2.0-docs.md)** - Comprehensive Tauri 2.0 plugin usage guide with real-world solutions

The Tauri 2.0 documentation covers:
- Clipboard Manager, Global Shortcut, File System, Opener plugins
- Window Management patterns and solutions
- Common issues and their fixes
- All code examples are battle-tested from this project

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋‍♂️ Support

If you encounter any issues or have questions:
- Open an issue on [GitHub Issues](../../issues)
- Check the [Discussions](../../discussions) for community help

## 🌟 Show Your Support

If you find AI TextTool useful, please consider:
- ⭐ Starring this repository
- 🐛 Reporting bugs
- 💡 Suggesting new features
- 🤝 Contributing code

---

**Made with ❤️ using Tauri + Vue 3 + Rust**
