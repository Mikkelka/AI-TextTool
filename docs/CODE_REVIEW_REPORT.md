# Code Review Report

## Findings (prioritized)
Note: Ratings assume this is a personal, local-only app used by a single person. If the app is shared or logs are exported, raise the privacy-related severities.

### 1) Sensitive data logged (Low for personal use, 1/5)
**Issue:** User text and clipboard contents are written to logs. For a solo, local app this is mostly a privacy/cleanliness concern rather than a security risk.
- Evidence: `src-tauri/src/commands/ai_commands.rs:65-68`, `src-tauri/src/commands/ai_commands.rs:133`, `src-tauri/src/shortcut_manager.rs:52-55`, `src-tauri/src/shortcut_manager.rs:107-110`
**Improve:** Remove or redact content in logs (log lengths only), and gate any debug logging behind a dev-only flag.
**Status:** Fixed on 2026-01-27 (redacted log output to lengths only).

### 2) Clipboard can be overwritten with marker and not restored (High, 4/5)
**Issue:** The clipboard is set to a marker to detect changes, but if no text is selected or a failure occurs, the original clipboard value may be lost.
- Evidence: `src-tauri/src/shortcut_manager.rs:47-73`, `src-tauri/src/shortcut_manager.rs:146-155`
**Improve:** Always restore `original_clipboard` before returning in the no-text-detected or error paths.
**Status:** Fixed on 2026-01-27 (restore original clipboard on error/no-text paths).

### 3) Rate limiter records pre-wait timestamp (Medium, 3/5)
**Issue:** When the rate limiter sleeps, it still pushes the original `now` timestamp, making the call appear earlier than it really was and allowing extra calls within the window.
- Evidence: `src-tauri/src/ai_provider/gemini.rs:39-63`
**Improve:** Recompute `now` after sleeping (or push `Instant::now()` after the wait).
**Status:** Fixed on 2026-01-27 (record timestamp after waiting).

### 4) Duplicate/competing prompts for text operations (Medium, 3/5)
**Issue:** Text operations combine a prefixed prompt and a hardcoded prompt and a system instruction, which duplicates or conflicts with the user instruction set. This can reduce response quality and make operations inconsistent.
- Evidence: `src-tauri/src/commands/ai_commands.rs:95-109`, `src-tauri/src/ai_provider/gemini.rs:474-499`, `src-tauri/src/data_manager/types.rs:150-193`
**Improve:** Choose a single source of truth (either operation prefix/instruction or hardcoded prompts) and pass only one.
**Status:** Fixed on 2026-01-27 (text operations now use operation config only).

### 5) Non-atomic writes to `app_data.json` (Medium, 3/5)
**Issue:** Writes go directly to the final file, risking corruption on crash or power loss.
- Evidence: `src-tauri/src/data_manager/manager.rs:55-66`
**Improve:** Write to a temp file and atomically rename; consider fsync for durability.
**Status:** Fixed on 2026-01-27 (write temp file, sync, then replace).

### 6) Encoding issues in user-facing strings (Low, 2/5)
**Issue:** Mojibake appears in Danish defaults and UI copy icons, which will display incorrectly.
- Evidence: `src-tauri/src/data_manager/types.rs:166-169`, `src/utils/markdown.ts:89-106`
**Improve:** Ensure files are saved as UTF-8 and replace the garbled strings with proper characters.
**Status:** Fixed on 2026-01-27 (corrected Danish strings and copy button text).

### 7) Link security hardening in markdown render (Low, 2/5)
**Issue:** `target` is allowed but links are not forced to include `rel="noopener noreferrer"` which can enable tabnabbing.
- Evidence: `src/utils/markdown.ts:65-67`
**Improve:** Post-process links to add safe `rel` attributes or configure `marked` to do it.
**Status:** Fixed on 2026-01-27 (ensure rel includes noopener/noreferrer for target=_blank).

## Testing gaps

- No automated tests for clipboard flows, data migration, or AI error handling. Add at least one integration test for clipboard detection and one for data save/load integrity.

## Open questions / assumptions

- Are you ok with removing clipboard content logging entirely, or do you want a debug-only toggle?
- Should text operations be fully driven by the editable operations in `app_data.json` (and remove hardcoded prompts)?

## Change summary (high-level)

- Focus on privacy (logs), clipboard safety, and prompt consistency first. Then fix rate limiting and data-write robustness.
