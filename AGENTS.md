# Repository Guidelines

## Project Structure & Module Organization
- `src/` houses the Vue 3 + TypeScript UI. Components live in `src/components/`, shared types in `src/types/`, and utilities in `src/utils/`.
- `src-tauri/` contains the Rust backend for system integration, AI calls, and Tauri commands.
- `windows/` stores HTML entrypoints for each Tauri window (e.g., `popup.html`, `chat.html`).
- `public/` is for static assets, and `docs/` contains extended project documentation.

## Build, Test, and Development Commands
- `npm run dev`: Run the full Tauri app in development mode.
- `npm run dev:web`: Run the Vite dev server for frontend-only work.
- `npm run build`: Build the desktop application.
- `npm run build:web`: Type-check and build the frontend bundle.
- `npm run lint` / `npm run check`: Lint Vue/TS and Rust (Clippy), plus Prettier checks.
- `npm run fix`: Auto-fix Vue/TS lint issues and format Rust.
- `npm run security-check`: Run `npm audit` with a moderate threshold.

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
- API keys are configured through the app’s onboarding flow; keep them out of the repo and logs.
