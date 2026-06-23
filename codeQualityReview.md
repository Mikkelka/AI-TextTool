# Code Quality Review

## Status (2026-06-23)

| Metric | Count |
|--------|-------|
| Open | 38 |
| Fixed since last | 0 |
| New | 38 |

---

## 2026-06-23

First review. Findings are ordered by severity (high → medium → low). Each entry includes a concrete suggestion. No code was modified.

### New findings

#### High (bug risk)

- `src/components/OnboardingWindow.vue:496` — `if (!config.providers.Gemini)` accesses `config.providers` without first guarding that it exists. `SettingsWindow.vue:147` does guard (`if (!config.providers) { config.providers = {} }`), but OnboardingWindow does not, so a config payload where Rust omits `providers` will throw `Cannot read properties of undefined`. **Suggestion:** mirror the defensive `if (!config.providers) config.providers = {}` guard used in SettingsWindow, or (better) extract a shared `ensureDefaultConfig()` helper and reuse it in both files.

- `src-tauri/src/data_manager/types.rs:52` — `Config::active_provider()` uses `.expect("no providers configured")` and is called transitively by `api_key()`, `chat_model()`, `text_model()`, `chat_system_instruction()`, all of which are called inside Tauri command handlers (`ai_commands.rs:87, 160, 231`). A corrupted/empty `app_data.json` panics the command thread instead of returning a clean error. **Suggestion:** return `Option<&ProviderConfig>` (or a `Result`) and have command handlers emit a user-friendly `Err("No provider configured")`.

#### Medium (maintainability)

- `src-tauri/src/ai_provider/gemini.rs:187` and `:347` — `generate_content_with_retry` and `generate_chat_content_with_retry` are ~140 and ~160 lines and share ~80% identical structure (rate-limit lock, request build, status match arms 401/404/429/500–599/_, retry recursion, error mapping). Keeping two copies in sync is a drift hazard. **Suggestion:** unify into one generic `generate_with_retry<F: FnOnce(GeminiResponse) -> R>` that takes a response parser, or extract the shared status-handling arm into a helper `fn classify_status(resp, status, retry_count) -> ControlFlow`.

- `src-tauri/src/ai_provider/gemini.rs:207` — manual `Box::pin` + recursive async pattern for retry. Verbose and hard to read. **Suggestion:** replace with a `loop` that increments `retry_count` and `sleep`s on retryable status, returning when done or retries exhausted.

- `src/components/ChatWindow.vue:390` and `:468` — `sendMessage` and `regenerateResponse` duplicate the bulk of their bodies: building the `instruction` with model note, calling `invoke('chat_with_ai', ...)`, finding the processing message, replacing it, and the error-path splice. **Suggestion:** extract a `runAssistantTurn(history, userContent)` helper that both call.

- `src/components/SettingsWindow.vue:137` and `src/components/OnboardingWindow.vue:486` — the default `Config` object (`provider: 'Gemini'`, `shortcut: 'CmdOrCtrl+Space'`, `locale: 'en'`, `streaming: false`, `providers: {}`) is hardcoded in two places. **Suggestion:** add a `DEFAULT_CONFIG` constant in `src/types/index.ts` (or a `createDefaultConfig()` factory) and import it in both components, plus mirror in Rust `Config::default()` where possible.

- `src/components/ChatWindow.vue:261` and `src-tauri/src/utils/validation.rs:13` — `MAX_MESSAGE_LENGTH = 10000` is defined independently in TS and Rust. If either drifts, frontend validation and backend validation disagree silently. **Suggestion:** treat Rust's `validation.rs` as source of truth and have the frontend fetch the limit via a command, or at minimum add a comment on both sides referencing the other.

- `src/components/ChatWindow.vue:228`, `src/components/ChatHistoryWindow.vue:288`, `src/components/OperationEditWindow.vue:234`, `src/components/OnboardingWindow.vue:302` — promise-dialog pattern (`*DialogResolver` + `requestX` + `handleXConfirm` + `handleXCancel`) is reimplemented four times with small variations (OperationEditWindow even uses a different `pendingConfirmAction: (() => void) => void` shape). **Suggestion:** create a `usePromiseDialog` composable returning `{ visible, open, confirm, cancel }` and reuse across all four windows.

- `src/components/ChatWindow.vue:307`, `src/components/ChatHistoryWindow.vue:351`, `src/components/OperationEditWindow.vue:482` — `showToast` (state + timer + clearTimeout + 3200ms timeout) is duplicated in three components. **Suggestion:** extract a `useToast()` composable.

- `src/components/ChatHistoryWindow.vue:266` — `interface ChatEntry` is declared locally with snake_case fields mirroring `data_manager::types::ChatEntry` in Rust, but no shared TS type exists in `src/types/index.ts`. **Suggestion:** add `ChatEntry` to `src/types/index.ts` and import it.

- `src/components/MessageBubble.vue:99` — inline `Props.message` object type restates the shape of `ChatMessage` from `src/types/index.ts` (with subtly different optional-field syntax). **Suggestion:** `import type { ChatMessage } from '../types'` and use `message: ChatMessage`.

- `src-tauri/src/commands/ai_commands.rs:128` and `:219` — `process_text_with_ai` returns `Err(format!("AI processing failed: {}", e))` and `chat_with_ai` returns `Err(format!("Chat failed: {}", e))`, both stringifying `GeminiError` via `Display`. Yet `gemini_error_to_user_message` (defined at line 14 of the same file) exists to produce user-friendly messages and is only used for `GeminiProvider::new` failures (lines 97, 168). **Suggestion:** route all `GeminiError` through `gemini_error_to_user_message` for consistent UX.

- `src-tauri/src/commands/ai_commands.rs:82` (and `:156`, `:227`) — every command repeats `let state = app.state::<SharedDataManager>(); let manager = state.0.lock().await; let config = manager.get_config().clone(); let rate_limiter = get_rate_limiter(&app); let http_client = get_http_client(&app); let provider = GeminiProvider::new(...)`. **Suggestion:** add `fn create_provider(app: &AppHandle) -> Result<GeminiProvider, String>` that bundles config-load + api-key check + provider construction.

- `src-tauri/src/window_manager.rs:221`, `:251`, `:281` — `create_direct_chat_window`, `create_fallback_chat_window`, and `create_tray_chat_window` are three ~25-line blocks differing only in `maximizable`/`minimizable` and the window-id prefix. **Suggestion:** add a `chat_window_builder(app, id_prefix, maximizable, minimizable)` helper, or a `ChatWindowKind` enum.

- `src-tauri/src/window_manager.rs:192` — `create_popup_window` hardcodes positioning offsets `150.0` and `300.0` that are coupled to the `width: 300.0` / `height: 290.0` defined below. Changing the size without updating the offsets silently mispositions the window. **Suggestion:** compute offsets from the config (`config.width / 2.0`, `config.height + 10.0`).

- `src-tauri/src/commands/window_commands.rs:9` vs `src-tauri/src/window_manager.rs:221` — chat windows created via `invoke('open_chat_window')` use `create_chat_window_builder` (maximizable+minimizable true), while chat windows created from tray/shortcut via `window_manager` use `maximizable(false)`/`minimizable(false)` (except tray). Same "chat window" with three different capability sets. **Suggestion:** route all chat-window creation through one builder/enum.

- `src-tauri/src/data_manager/manager.rs:97` — `migrate_from_old_files` silently swallows read/parse failures for each legacy file (`if let Ok(content) = ...; if let Ok(config) = ...`). A malformed `config.json` results in default config with no warning, potentially losing the user's prior setup. **Suggestion:** log a `warn!` on each `Err` branch so migration problems are diagnosable.

- `src/components/OperationEditWindow.vue:221` and `:226` — `operations` (Record) and `operationsArray` (Array of tuples) are two sources of truth kept in sync manually (lines 264-267, 359-360, 424, 437-441). Easy to drift. **Suggestion:** keep only `operationsArray` as state and derive `operations` via `computed`, or vice versa.

- `src-tauri/src/shortcut_manager.rs:70` — `process_shortcut_trigger` is ~115 lines with three ORed detection strategies, six boolean locals, and multiple early-return branches. Hard to reason about. **Suggestion:** extract each strategy into a named predicate (`fn changed_meaningfully`, `fn has_substantial_content`, `fn duplicate_but_meaningful`) and compose them.

- `src/utils/markdown.ts:79` — `addCustomClasses` post-processes HTML with a chain of regexes (`/<table>/g`, `/<a([^>]*?)>/g`, `/<pre><code([^>]*)>([\s\S]*?)<\/code><\/pre>/g`, …). Parsing HTML with regex is fragile (breaks on attribute order, self-closing variants, or existing classes). **Suggestion:** parse with `DOMParser`, mutate the DOM, then serialize; or use DOMPurify's `onAfterSanitize` hook to add classes.

- `src-tauri/src/data_manager/types.rs:183` and `src/components/ChatHistoryWindow.vue:517` — the default operation names (`Proofread`, `Rewrite`, `Dansk`, `Concise`, `Friendly`, `Professional`, `Key Points`, `Summary`, `Chat`, `Custom`) are hardcoded in Rust `create_default_operations` and again in Vue `getOperationClass`. Adding/renaming an operation requires editing both. **Suggestion:** have the backend expose operation metadata (including badge class) and let the frontend render from it.

- `src/components/SettingsWindow.vue:153` — `config.providers.Gemini = {} as ProviderSettings` casts an empty object to `ProviderSettings`, then assigns fields one by one. Temporarily violates the type. **Suggestion:** build a complete `ProviderSettings` literal and assign once, or use `Partial<ProviderSettings>` during construction.

- `src/components/ChatWindow.vue:213` and `:210` — `state.availableModels: string[]` and `state.selectedModel: string` (defaulting to `CHAT_MODEL`) are then cast `as ModelName` at lines 253, 257, 298. The `ModelName` type exists in `src/types/index.ts` but is bypassed. **Suggestion:** type both as `ModelName`/`ModelName[]`; validate incoming model strings against `MODEL_NAMES` before assignment.

#### Low (style / minor)

- `src/components/ChatWindow.vue:305` — `import { formatModelName } from '../utils/formatters'` appears mid-script instead of with the other imports at the top. **Suggestion:** move to the import block at line 175-194.

- `src/components/ChatWindow.vue:614` — `handleGlobalKeydown` uses a `switch` for a single `Ctrl+L` case. **Suggestion:** a simple `if` is clearer for one case, or add the other documented shortcut (`Ctrl+S` for save) to justify the switch.

- `src/components/ChatWindow.vue:640` — `loadConversation`'s `map` has two branches that build nearly identical objects; the invalid-role branch duplicates the valid one with `role: 'assistant'`. **Suggestion:** build the object once and override `role` in the invalid branch.

- `src/components/PopupWindow.vue:194` and `:312` — `const { getCurrentWindow } = await import('@tauri-apps/api/window')` is a dynamic import inside two functions, while other components (`ChatWindow`, `SettingsWindow`, `OperationEditWindow`) import `getCurrentWindow` statically at the top. **Suggestion:** use a static top-level import for consistency and to avoid repeated module resolution.

- `src/components/PopupWindow.vue:96` — `Emits` declares `close` and `operation-selected`, but `PopupWindow` is mounted via `mountWindow` with no parent listener; both emits (used at lines 158, 318) are effectively dead. **Suggestion:** either remove the emits, or wire `mountWindow` to forward them.

- `src/components/SettingsWindow.vue:113` and `:118` — `'https://aistudio.google.com/app/apikey'` hardcoded twice. **Suggestion:** hoist to a `const API_KEY_URL` at the top of the script.

- `src/components/SettingsWindow.vue:84` and `:167` — `3000` (message timeout) and `1500` (close delay) are magic numbers. **Suggestion:** name them `MESSAGE_TIMEOUT_MS`, `CLOSE_DELAY_MS`.

- `src/components/ChatHistoryWindow.vue:441` and `:450` — `copyOriginalText` and `copyProcessedText` are identical except the debug log string. **Suggestion:** merge into `copyText(text, label)`.

- `src/components/ChatHistoryWindow.vue:465` — `const result = (await invoke(...)) as string` is assigned and only used for a `logger.debug` call. **Suggestion:** drop the binding or use the value.

- `src/components/ChatHistoryWindow.vue:878` — badge background colors (`#9c27b0`, `#e91e63`, `#795548`, `#00bcd4`, `#3f51b5`, `#ff5722`, `#757575`) are raw hex inline rather than CSS tokens. **Suggestion:** add `--color-badge-*` tokens to `styles/tokens.css`.

- `src/components/ChatHistoryWindow.vue:487` — `formatTimestamp` uses `1000 * 60 * 60` and `1000 * 60 * 60 * 24`. **Suggestion:** name `MS_PER_HOUR` / `MS_PER_DAY`.

- `src/components/OnboardingWindow.vue:204` — empty `Props` interface with `// No props needed for now` and `withDefaults(defineProps<Props>(), {})` is dead. **Suggestion:** remove `Props` and the `withDefaults` call.

- `src/components/OnboardingWindow.vue:279` — `Emits` declares `setup-complete`, `setup-skipped`, `close`, but each is only emitted inside a `catch` fallback that practically never runs (window close succeeds). **Suggestion:** either wire these to the parent or remove.

- `src/components/OnboardingWindow.vue:364` — `validateApiKey` accepts any string containing the substring `"AI"`. Very loose. **Suggestion:** require `startsWith("AIza")` (Gemini key prefix) or drop the prefix check and rely on length + connection test.

- `src/components/OperationEditWindow.vue:204` — same empty `Props` pattern as OnboardingWindow. **Suggestion:** remove.

- `src/components/OperationEditWindow.vue:414` — `saveOperation` sets `icon: undefined`, ignoring `editForm.value.icon`; the `EditForm.icon` field and its initialization are therefore dead. **Suggestion:** either add an icon picker or remove `icon` from `EditForm`.

- `src/components/OperationEditWindow.vue:496` — `showMessage(title, message)` wraps `showToast` and is called once. **Suggestion:** inline the single call site.

- `src/components/InputArea.vue:86` — the `Ctrl+Enter` branch `return`s early, skipping the `resizeTextarea()` call at the end of `handleInputKeydown`. Inserting a newline via Ctrl+Enter won't auto-resize the textarea. **Suggestion:** call `void resizeTextarea()` before returning, or move the resize to an `@input` handler.

- `src/components/MessageBubble.vue:124` — `formatTime` catch returns the *current* time on parse failure, masking the error. **Suggestion:** return a fallback like `'--:--'` or the raw timestamp.

- `src/components/AppConfirmDialog.vue` and `src/components/AppPromptDialog.vue` — share near-identical CSS (`.dialog-overlay`, `.dialog-title`, `.dialog-buttons`, `.dialog-button`, `.cancel-button`, `.confirm-button`). **Suggestion:** extract shared dialog styles to a common scss/css module or `styles/dialogs.css`.

- `src-tauri/src/lib.rs:33` — `get_configured_shortcut` does five-level nested `if let`/`if` to read one config field. **Suggestion:** deserialize into a `struct ShortcutConfig { config: { shortcut: String } }` or use `serde_json::Value::get` chaining with `and_then`.

- `src-tauri/src/shortcut_manager.rs:144` — `substantial_text_threshold = 10` and the `5` at line 151 are inline magic numbers (other delays are named constants at the top). **Suggestion:** add `const SUBSTANTIAL_TEXT_THRESHOLD: usize = 10;` and `const DUPLICATE_MIN_LEN: usize = 5;`.

- `src-tauri/src/shortcut_manager.rs:209` and `:243` — `simulate_copy` and `simulate_paste` duplicate the Ctrl press / key click / Ctrl release sequence. **Suggestion:** `fn simulate_ctrl_key(key: Key) -> Result<(), String>`.

- `src-tauri/src/ai_provider/gemini.rs:569` — `supports_thinking_mode` is `#[allow(dead_code)]`. **Suggestion:** either use it in `chat_with_ai` to gate thinking, or remove it.

- `src-tauri/src/ai_provider/gemini.rs:628` — `test_rate_limiter` asserts the third call `is_ok()` but does not verify that rate limiting actually blocks/waits. **Suggestion:** add a test with `max_calls_per_minute = 1` and assert the second call waits (or use a mock clock).

- `src-tauri/src/data_manager/manager.rs:56` — `save_data` clones the entire `AppData` on every save to update `metadata.last_updated`. **Suggestion:** update `last_updated` in-place on `&mut self.data`, or write metadata separately.

- `src-tauri/src/data_manager/types.rs:103` — `Operation.order: i32` admits negative values, which are meaningless for sorting. **Suggestion:** `order: u32`.

- `src-tauri/src/utils/validation.rs:33` — `validate_text_input` checks `is_empty()` then `len() < MIN_TEXT_LENGTH` (MIN=1); the second check is unreachable after the empty check. **Suggestion:** drop the redundant length check or raise `MIN_TEXT_LENGTH` if a higher floor is intended.

- `src-tauri/src/data_manager/commands.rs:111` — `load_conversation_messages` scans all saved conversations linearly to find by id. Acceptable at `MAX_HISTORY_ENTRIES = 100`, but worth a comment. **Suggestion:** add a `// O(n); fine for ≤100 entries` note, or index by id if the cap rises.

- `src-tauri/src/data_manager/commands.rs:70` — `format!("conv_{}", Utc::now().timestamp_millis())` can collide if two conversations are saved in the same millisecond. **Suggestion:** append a random suffix or a counter.

- `src/utils/markdown.ts:25` — `marked.parse(markdown) as string` assumes sync output. Currently safe (no `async: true` option), but the cast hides the `string | Promise<string>` union. **Suggestion:** use `marked.parseInline`/sync form explicitly, or assert via `marked.parse(markdown, { async: false })`.

- `src/utils/formatters.ts:1` — `formatModelName` is generic but only handles the `gemini-` prefix; non-Gemini model names pass through the same pipeline with no documentation of intent. **Suggestion:** add a short doc comment stating it is Gemini-specific.
