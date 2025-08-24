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
│   ├── components/          # Vue components (PopupWindow, ChatWindow, etc.)
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
│   ├── target/debug/        # 📁 Development build + data files
│   ├── target/release/      # 📁 Production build + data files
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

### Data Files
Configuration and history files are stored next to the executable:
- `config.json` - App settings and API keys
- `options.json` - Text operations configuration
- `chat_history.json` - Individual operation history
- `saved_conversations.json` - Complete chat conversations

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
