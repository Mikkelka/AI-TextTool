# Code Review Findings

## Critical

- [ ] **#1 – Race condition on concurrent data access**  
  `data_manager/commands.rs:11-15` — Every command creates a fresh `DataManager` via `load_data_manager()`, reading `app_data.json` from disk each time. Concurrent writes (e.g. saving a chat entry + loading config) read stale data, modify independently, and the last write wins. Replace with a single `Arc<Mutex<DataManager>>` in Tauri managed state.

- [ ] **#2 – Save can lose all data**  
  `data_manager/manager.rs:70-73` — `save_data()` removes the original file, then renames the temp over it. If `rename` fails (cross-volume, permissions), the original is already gone. Reverse the order: rename first, then clean up the temp file on failure.

- [ ] **#3 – `clearAllHistory` doesn't clear conversations**  
  `ChatHistoryWindow.vue:429-433` — Calls only `clear_chat_history`, not `clear_saved_conversations`. Then sets both lists to empty arrays locally. On refresh, conversations reappear. Add the `clear_saved_conversations` call.

- [ ] **#4 – Blocking call in async context**  
  `shortcut_manager.rs:218,227` — `simulate_copy()` uses `std::thread::sleep` inside an async task spawned from `process_shortcut_trigger`. Blocks a tokio runtime thread. Use `tokio::time::sleep` or `spawn_blocking`.

## High

- [ ] **#5 – Type mismatch: TS `ProviderSettings` vs Rust `ProviderConfig`**  
  `SettingsWindow.vue:160-166` — TS sends `api_key` as `string | undefined`, but Rust `ProviderConfig` expects `String` (not `Option`). If a field is `undefined`, serde will fail on deserialization. Either use `Option<String>` in Rust or ensure TS always sends concrete values.

- [ ] **#6 – System instruction dropped on retry**  
  `gemini.rs:284-293, 478-493` — On 429/5xx errors, retry re-calls the method with raw `request.contents`, but passes `system_instruction` and `generation_config` from the outer scope, not from the current request. If `system_instruction` is `None` and `use_formatting` is `true`, the instruction is recomputed without the custom instruction. Pass `request.system_instruction` text directly.

- [ ] **#7 – Dead code: unreachable else branch**  
  `gemini.rs:394` — `use_formatting` is hardcoded to `true` in `generate_chat_content_with_retry`, making the else branch unreachable. Remove the dead code or simplify the logic.

## Medium

- [ ] **#8 – `void` instead of `await` in fire-and-forget async**  
  `ChatWindow.vue:471,597,689`, `PopupWindow.vue:186` — Using `void` on functions that `await nextTick()` means the next Vue tick may not have completed. Edge cases could miss focus/scroll. Use `await` or restructure.

- [ ] **#9 – Duplicated `formatModelName`**  
  `ChatWindow.vue:305-312` and `SettingsWindow.vue:88-95` — Identical function in two components. Extract to `src/utils/formatters.ts`.

- [ ] **#10 – Synchronous sleep in async window creation**  
  `window_manager.rs:72` — `std::thread::sleep(Duration::from_millis(10))` called synchronously from `create_window` when creating a popup. Use an async delay instead.

- [ ] **#11 – Floating promise in `setTimeout`**  
  `PopupWindow.vue:189-196` — `setTimeout(async () => { await invoke('simulate_paste') })` creates an unhandled promise chain. Use structured async error handling.

- [ ] **#12 – `supports_thinking_mode` gated behind `#[cfg(test)]`**  
  `gemini.rs:578-581` — Method only compiles in test builds but is needed at runtime if the frontend ever checks thinking support. Remove the `#[cfg(test)]` gate or add a non-gated variant.

## Low

- [ ] **#13 – `register_global_shortcut` visibility**  
  `lib.rs:18` — Declared `pub` but only used crate-internally. Change to `pub(crate)`.

- [ ] **#14 – Dead function `create_chat_window`**  
  `window_manager.rs:219` — Private function never called (replaced by `window_commands.rs`). Remove it.

- [ ] **#15 – `InputArea` resizes on every keydown**  
  `InputArea.vue:95,61` — `resizeTextarea()` fires on every keystroke. Debounce or use `requestAnimationFrame`.

- [ ] **#16 – Rate limiter test is weak**  
  `gemini.rs:638-647` — `test_rate_limiter` never verifies that the 3rd+ call actually waits. Add timing assertions.

- [ ] **#17 – Duplicate CSS `.restart-notice`**  
  `SettingsWindow.vue:330-335` and `:418-427` — Same class defined twice with different values. Remove one.

- [ ] **#18 – Types duplicated between TS and Rust**  
  `src/types/index.ts` vs `src-tauri/src/data_manager/types.rs` — `Config`, `Operation`, `ConversationMessage` defined in both. Consider generating TS types from Rust.

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
