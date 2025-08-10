# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri application that combines a Vue 3 + TypeScript frontend with a Rust backend. The application creates a desktop app with web technologies, allowing the frontend to invoke Rust functions through Tauri's command system.

## Architecture

- **Frontend**: Vue 3 with TypeScript, built with Vite
  - Entry point: `src/main.ts`
  - Main component: `src/App.vue`
  - Uses Vue 3 Composition API with `<script setup>`

- **Backend**: Rust with Tauri
  - Entry point: `src-tauri/src/main.rs`
  - Core logic: `src-tauri/src/lib.rs`
  - Commands are exposed to frontend via `#[tauri::command]` macro

- **Communication**: Frontend calls backend via `invoke()` from `@tauri-apps/api/core`

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

## Configuration Files

- `vite.config.ts` - Vite configuration optimized for Tauri (fixed port 1420, HMR setup)
- `src-tauri/tauri.conf.json` - Tauri application configuration (window settings, bundle options)
- `src-tauri/Cargo.toml` - Rust dependencies and build configuration
- `tsconfig.json` & `tsconfig.node.json` - TypeScript configuration

## Adding New Features

1. **Frontend components**: Add Vue components in `src/` directory
2. **Rust commands**: Add functions with `#[tauri::command]` in `src-tauri/src/lib.rs` and register them in the `invoke_handler`
3. **Frontend-backend communication**: Use `invoke("command_name", { params })` from the frontend to call Rust commands

## Implemented Features

### System Tray
- System tray icon with menu functionality
- App minimizes to tray instead of closing when X is pressed
- Uses programmatic tray creation (not config-based) to avoid duplication issues

### Global Hotkey (Ctrl+Space)
- Captures selected text from anywhere on the system
- Automatically simulates Ctrl+C to copy selected text
- Shows popup window at mouse cursor position
- Uses debouncing to prevent multiple triggers on Windows

### Popup Windows
- **IMPORTANT**: When passing data to popup windows, use `initialization_script()` instead of clipboard API
- Popup windows don't have proper access to Tauri plugins like clipboard-manager
- Solution: Inject data via initialization script when creating the window:
  ```rust
  .initialization_script(&format!("window.clipboardText = '{}';", escaped_text))
  ```
- This avoids all permission and plugin access issues in popup contexts

### Configuration Notes
- `withGlobalTauri: true` is set in tauri.conf.json to enable global Tauri object access
- Tray icon configured programmatically in lib.rs, not in tauri.conf.json

## Project Structure Notes

- The application uses a lib/main split in Rust (`lib.rs` contains core logic, `main.rs` is minimal entry point)
- Vite is configured to ignore `src-tauri` directory during watch mode
- The app uses Tauri's plugin system (includes `tauri-plugin-clipboard-manager`, `tauri-plugin-global-shortcut`)
- Uses `enigo` crate for keyboard/mouse simulation