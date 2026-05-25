# Code Review Findings

## Critical

- [x] **#1 – Race condition on concurrent data access**  
  `data_manager/commands.rs:11-15` — Refactored `DataManager` into `Arc<Mutex<DataManager>>` managed as Tauri shared state (initialized once at startup). All commands lock the same instance.

- [x] **#2 – Save can lose all data**  
  `data_manager/manager.rs:70-73` — Changed to rename-first pattern. On failure, falls back to direct write. Original file is never removed unless new one is safe.

- [x] **#3 – `clearAllHistory` doesn't clear conversations**  
  `ChatHistoryWindow.vue:429-433` — Now calls both `clear_chat_history` and `clear_saved_conversations` in parallel.

- [x] **#4 – Blocking call in async context**  
  `shortcut_manager.rs:218,227` — `simulate_copy()` uses `std::thread::sleep` for hardware keyboard timing. Inherently sync (enigo). Noted as acceptable — wrapping in `spawn_blocking` would add overhead for 20ms delays.

## High

- [x] **#5 – Type mismatch: TS `ProviderSettings` vs Rust `ProviderConfig`**  
  `SettingsWindow.vue:160-166` — Made TS `ProviderSettings` fields required (`string` not `string | undefined`) to match Rust `ProviderConfig`.

- ~~#6 – System instruction dropped on retry~~  
  `gemini.rs:284-293, 478-493` — False positive. Retry passes the same `system_instruction` parameter; combined instruction is recomputed identically. Not a bug.

- [x] **#7 – Dead code: unreachable else branch**  
  `gemini.rs:394` — Removed the dead else branch from `generate_chat_content_with_retry`. Kept the branch in `generate_content_with_retry` where `use_formatting` can be `false` for text ops.

## Medium

- ~~#8 – `void` instead of `await`~~  
  `ChatWindow.vue:471,597,689` — Idiomatic Vue fire-and-forget pattern for non-critical UI side effects. No behavioral change needed.

- [x] **#9 – Duplicated `formatModelName`**  
  Extracted to `src/utils/formatters.ts`. Updated both `ChatWindow.vue` and `SettingsWindow.vue`.

- ~~#10 – Synchronous sleep in async window creation~~  
  `window_manager.rs:72` — 10ms blocking in `create_window`, called from both sync and async contexts. Trivial; acceptable as-is.

- [x] **#11 – Floating promise in `setTimeout`**  
  `PopupWindow.vue:189-196` — Restructured to hide -> paste -> close, eliminating the context-destroyed-before-paste race.

- [x] **#12 – `supports_thinking_mode` gated behind `#[cfg(test)]`**  
  `gemini.rs:578-581` — Removed `#[cfg(test)]`. Added `#[allow(dead_code)]` since it is not yet called at runtime.

## Low

- [x] **#13 – `register_global_shortcut` visibility**  
  `lib.rs:18` — Changed from `pub` to `pub(crate)`.

- [x] **#14 – Dead function `create_chat_window`**  
  `window_manager.rs:219` — Removed. Inlined configs into the three callers.

- ~~#15 – `InputArea` resizes on every keydown~~  
  `InputArea.vue:95,61` — Micro-optimization. Not causing issues.

- ~~#16 – Rate limiter test is weak~~  
  `gemini.rs:638-647` — Low priority enhancement. Skipped.

- [x] **#17 – Duplicate CSS `.restart-notice`**  
  `SettingsWindow.vue:330-335` — Removed the first (overridden) definition.

- ~~#18 – Types duplicated between TS and Rust~~  
  `src/types/index.ts` vs `src-tauri/src/data_manager/types.rs` — Enhancement suggestion for future. Skipped for now.

---

## Language preservation fix (2026-05-25)

### Before
- Text operations (proofread, rewrite, etc.) used `temperature: 0.7` (default `GenerationConfig`), passed as `None` in `ai_commands.rs:118`.
- Language instruction buried mid-prompt: `"IMPORTANT: Maintain the original language of the text..."`

### Changed
- Text operations now pass `temperature: 0.0` for deterministic output.
- Language instruction moved to **front** of each operation instruction as the first sentence: `"CRITICAL: Keep the text in its original language. Do NOT translate."`

### Files changed
- `src-tauri/src/commands/ai_commands.rs:118` — added low-temp `GenerationConfig`
- `src-tauri/src/data_manager/types.rs:185,193,209,217,225` — strengthened Proofread, Rewrite, Concise, Friendly, Professional instructions
