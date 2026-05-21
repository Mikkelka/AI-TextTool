# Repository Guidelines

## Project Structure & Module Organization
- `src/` houses the Vue 3 + TypeScript UI. Components live in `src/components/`, shared types in `src/types/`, and utilities in `src/utils/`.
- `src-tauri/` contains the Rust backend for system integration, AI calls, and Tauri commands.
- `windows/` stores HTML entrypoints for each Tauri window (7 total: `index.html`, `popup.html`, `chat.html`, `settings.html`, `history.html`, `onboarding.html`, `operation-edit.html`). Vite builds them as separate rollup inputs — do not add a new window without adding its HTML here and to `vite.config.ts`.
- `public/` is for static assets, and `docs/` contains extended project documentation.

## Build, Test, and Development Commands

### Daily use
- `npm run dev`: Run the full Tauri app in development mode.
- `npm run build`: Build the desktop application.

### Frontend-only
- `npm run dev:web`: Run the Vite dev server for frontend-only work.
- `npm run build:web`: Type-check and build the frontend bundle.

### Quality checks (run before commits)
- `npm run check`: Lint Vue/TS and Rust (Clippy), plus Prettier format check.
- `npm run lint`: Lint Vue/TS (ESLint) and Rust (Clippy).
- `npm run fix`: Auto-fix Vue/TS lint issues + Prettier format + Rust `cargo fmt`.

### Granular (rarely needed)
- `npm run lint:vue` / `npm run lint:rust`: Lint only Vue/TS or Rust.
- `npm run fix:vue` / `npm run fix:rust`: Fix only Vue/TS or Rust.
- `npm run audit`: Check for npm vulnerabilities.
- `npm run audit:fix`: Auto-fix npm vulnerabilities.
- `npm run security-check`: Run `npm audit` with a moderate threshold.

## Critical Tauri Patterns (easy to get wrong)
- **Window creation MUST happen in Rust commands**, never from the frontend via `new WebviewWindow()`. Frontend calls `invoke('open_chat_window')` etc.
- **invoke() parameters use snake_case** to match Rust function signatures. `await invoke('process_text', { input_text: text })` — camelCase will silently fail.
- **Always await invoke()** calls. Unawaited invokes cause silent failures.
- **Clipboard access in popup windows is unreliable**. Use `initialization_script()` to inject data at window creation time instead.
- **Global shortcuts need debouncing on Windows** — single keypress can fire multiple events. See `shortcut_manager.rs` for the `Arc<Mutex<Instant>>` pattern.
- **Main window hides on startup** (app is tray-only). The `setup()` function hides it immediately and only shows onboarding if `app_data.json` is missing.
- **Main window hides instead of closing** — `on_window_event` intercepts `CloseRequested` and calls `api.prevent_close()`. Other windows close normally.
- **Tray icon is created programmatically in `lib.rs`**, NOT in `tauri.conf.json`. Adding it to both causes duplication.

## Architecture Notes
- Tauri v2, Vue 3 Composition API, TypeScript.
- Rust entry point: `src-tauri/src/lib.rs` — registers all plugins, managed state (rate limiter at 15 req, shared HTTP client), and all `invoke_handler` commands.
- Frontend entry point: `src/main.ts`. Vite dev server runs on port 1420 (strict).
- All app data (settings, API keys, operations, chat history, conversations) lives in a single `app_data.json` file next to the executable. Old files are auto-migrated on first launch.
- AI provider: Google Gemini. Chat uses `gemini-3-flash-preview`, text ops use `gemini-3.1-flash-lite`.
- File I/O in Rust uses async `tokio::fs` — always flush after writes.

## Coding Style & Naming Conventions
- Formatting is enforced by Prettier: 2-space indentation, single quotes, no semicolons, 100-char line width.
- Linting uses ESLint for Vue/TS and Clippy for Rust; run `npm run check` before commits.
- Vue components use PascalCase filenames (e.g., `MessageBubble.vue`); keep new components consistent.
- Rust follows `rustfmt` defaults via `npm run fix:rust`.

## Testing Guidelines
- There is no dedicated test runner yet. Treat `npm run check` as the minimum quality gate.
- If you add tests, place them alongside the relevant module (e.g., `src/...`) and document how to run them.

## Commit & Pull Request Guidelines
- Commit messages are short, imperative, and often reference issues (e.g., `Fix Issue #20: ...`, `Add ...`, `Refactor ...`).
- PRs should include a clear description, linked issues when applicable, and screenshots for UI changes.
- Note any data or migration impacts in the PR description.

## Configuration & Data
- User data is stored in `app_data.json` next to the built executable. Avoid committing secrets or local data files.
- API keys are configured through the app's onboarding flow; keep them out of the repo and logs.

## Reference Docs
- `docs/TAURI_REFERENCE.md` — detailed Tauri 2.x patterns, plugin usage, and battle-tested solutions for window management, clipboard, shortcuts, and more.
