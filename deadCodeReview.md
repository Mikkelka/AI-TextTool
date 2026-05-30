# Dead Code Review

**Date:** 2026-05-30
**Scope:** `src/` (Vue/TS), `src-tauri/src/` (Rust), `windows/` (HTML), `package.json`
**Branch:** `cleanup/dead-code`

## Summary

| Category | Before | Fixed | Remaining |
|----------|--------|-------|-----------|
| Unused imports | 1 | 0 | 1 (style-only) |
| Dead functions | 2 | 0 | 2 (test-only, annotated) |
| Unused variables | 4 | 3 | 1 (annotated with `_`) |
| Unused emits | 1 | 1 | 0 |
| Unused props | 1 | 1 | 0 |
| Unused type exports | 2 | 2 | 0 |
| Dead CSS selectors | 12 | 11 | 1 (possibly used) |
| Dead Rust struct fields | ~14 | 0 | ~14 (annotated, API completeness) |
| Misplaced imports | 1 | 0 | 1 (style-only) |
| **Total** | **38** | **18** | **20** |

## Fixed in this cleanup

### Unused variables — removed
- `src/components/InputArea.vue:83` — removed unused `const message` in `handleInputKeydown`, inlined the check
- `src/components/InputArea.vue:100` — removed unused `const message` in `handleSendClick`, inlined the check
- `src-tauri/src/ai_provider/gemini.rs:377` — removed dead `let _use_formatting = true` leftover

### Unused emits — removed
- `src/components/OperationEditWindow.vue:212–216` — removed unused `close` emit definition (component uses `getCurrentWindow().close()` directly)

### Unused props — removed
- `src/components/OnboardingWindow.vue:279` — removed unused `visible` prop and its `withDefaults`

### Unused type exports — removed
- `src/types/index.ts:42` — removed `ChatHistoryEntry` interface (never imported, `ChatHistoryWindow.vue` uses its own local `ChatEntry`)
- `src/types/index.ts:102` — removed `AppError` interface (never imported or referenced)

### Dead CSS selectors — removed
- `src/components/SettingsWindow.vue` — removed 7 selectors: `.form-input.error`, `.error-message`, `.success-message`, `.suggestions-title`, `.suggestion-buttons`, `.suggestion-btn`, `.suggestion-btn:hover`, `.restart-notice`
- `src/components/ChatWindow.vue:949` — removed `.zoom-btn`
- `src/components/ChatHistoryWindow.vue:970–984` — removed `.processed-text .code-block` and `.processed-text .inline-code` (scoped CSS can't reach child component internals)

## Remaining (not addressed)

### Low priority — annotated / intentional

- `src-tauri/src/ai_provider/types.rs` — ~14 dead struct fields (`Candidate::finish_reason`, `Candidate::safety_ratings`, `UsageMetadata`, `GeminiResponse::usage_metadata`, `GroundingMetadata::grounding_supports`, `GroundingSupport`, `GroundingSegment`, `GeminiErrorDetails::code`, `GeminiErrorDetails::status`). All annotated with `#[allow(dead_code)]`. Intentional API completeness pattern for deserialization.
- `src-tauri/src/ai_provider/gemini.rs:570` — `supports_thinking_mode()` annotated `#[allow(dead_code)]`, only called from `#[cfg(test)]` block.
- `src-tauri/src/utils/time.rs:33` — `get_current_timestamp_secs()` annotated `#[allow(dead_code)]`, only called from `#[cfg(test)]` block.
- `src/components/PopupWindow.vue:169` — `_operation` parameter prefixed with `_`, never read. Low noise.
- `src/components/ChatWindow.vue:305` — `import { formatModelName }` placed mid-file instead of with other imports. Style issue, not dead code.
- `src/components/ChatHistoryWindow.vue:559` — `_index` param in `forEach` callback. Prefixed with `_`, conventional ignore.

### Possibly unused type exports (transitively used)

- `src/types/index.ts:27` — `ProviderSettings` — 0 direct imports, only used internally by `Config`. Likely intentional.
- `src/types/index.ts:60` — `ConversationMessage` — 0 direct imports, only used internally by `SavedConversation`. Likely intentional.
- `src/types/index.ts:13` — `GroundingSource` — 0 direct imports, only used internally by `ChatMessage`, `AIResponse`, etc. Likely intentional.

## No issues found

- **Unreachable code** — none detected
- **Commented-out code blocks** — none detected
- **Unused files** — all `.vue`/`.ts` files are imported somewhere
- **Dead modules (Rust)** — all modules declared in `lib.rs`/`mod.rs` are used
- **Unused Rust imports** — all `use` statements are referenced
- **Unregistered Tauri commands** — all 26 `#[tauri::command]` functions are registered in `lib.rs:104-135`
- **Unused script references in `windows/`** — all 7 HTML files correctly reference their entry scripts
