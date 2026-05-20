# Code Review: AI-TextTool

**Model:** deepseek-v4-flash-free  
**Dato:** 2026-05-20  
**Reviewer:** opencode AI agent

---

## Oversigt

App'en er velstruktureret med god sikkerhed (DOMPurify, CSP), fornuftig fejlhåndtering og atomisk filskrivning. Men der er en del væsentlige problemer, især dubletter, dead code, en contrast-bug i lys tilstand og en byggefejl med manglende ikonfiler.

---

## 🔴 Kritiske problemer

### 1. Hvid tekst på lys baggrund – ulæseligt i light mode

- **Fil:** `src/components/MessageBubble.vue:231`
- **Problem:** `.markdown-content { color: #ffffff; }` på en baggrund med `rgba(255, 255, 255, 0.95)` (linje 167). Brødtekst bliver hvid-på-næsten-hvid i light mode. Dark mode fungerer fint (linje 501: `color: #e2e8f0`).
- **Fix:** Ændr base color til `#333` i light mode og override i dark mode.

### 2. Manglende ikonfiler til bundling

- **Fil:** `src-tauri/tauri.conf.json:31-36`
- **Problem:** Bundle config refererer til `icons/32x32.png`, `icons/128x128.png`, `icons/128x128@2x.png`, `icons/icon.icns`, `icons/icon.ico` – men disse filer findes ikke. `src-tauri/icons/` indeholder kun Microsoft Store PNGs. Byg fejler ved packaging.
- **Fix:** Generér/skaf de manglende ikonfiler, eller opdater `tauri.conf.json` til at referere til de eksisterende filer.

### 3. Escape key håndteres dobbelt

- **Fil:** `src/components/PopupWindow.vue:2` og `:306`
- **Problem:** Template binder `@keydown="handleKeydown"` (linje 2) OG `onMounted` tilføjer `document.addEventListener('keydown', handleKeydown)` (linje 306). Begge kalder `closeWindow()`.
- **Fix:** Fjern `document.addEventListener` og behold kun template-bindingen, eller omvendt.

### 4. Duplicate Window interface

- **Filer:** `src/vite-env.d.ts:10-14` og `src/types/window.d.ts:5-13`
- **Problem:** Begge erklærer `interface Window { clipboardText?: string }`. Dette er forvirrende og fejlbehæftet, selvom TypeScript merger dem.
- **Fix:** Fjern erklæringen fra én af filerne. `src/types/window.d.ts` er det mest passende sted.

### 5. `xxx:` pseudo-protokol i DOMPurify regex

- **Fil:** `src/utils/markdown.ts:67-68`
- **Problem:** `ALLOWED_URI_REGEXP` tillader `xxx:` – ser ud som en test-typo.
- **Fix:** Fjern `|xxx` fra regex.

---

## 🟡 Arkitektur & design

### API key gemmes dobbelt

- **Filer:** `src-tauri/src/data_manager/types.rs:38-47`, flere frontend-komponenter
- **Problem:** `Config` gemmer `api_key` direkte OG i `providers.Gemini.api_key`. SettingsWindow og OnboardingWindow opdaterer begge. Inkonistensrisiko.
- **Fix:** Gem credentials kun i `providers`-map'et og udled top-level `api_key` fra `providers[current_provider].api_key`.

### DataManager læser disk ved hver kommando

- **Filer:** `src-tauri/src/commands/ai_commands.rs:51-58`, `src-tauri/src/commands/data_manager/commands.rs:11-15`
- **Problem:** Hver kommando kalder `DataManager::new()` + `initialize()`, som læser `app_data.json` fra disk. For en chat med mange beskeder betyder det read/parse ved hver message.
- **Fix:** Brug Tauri managed state eller lazy-static til at cache data i memory.

### `reqwest::Client` oprettes forfra ved hvert kald

- **Fil:** `src-tauri/src/ai_provider/gemini.rs:84-121`
- **Problem:** `reqwest::Client` er designet til genbrug (connection pools). `GeminiProvider::new()` opretter en ny hver gang.
- **Fix:** Gem client i DataManager eller Tauri managed state.

### Massiv kodeduplikering i Gemini provider

- **Fil:** `src-tauri/src/ai_provider/gemini.rs:162-487`
- **Problem:** `generate_content_with_retry` og `generate_chat_content_with_retry` er ~160 linjer hver og næsten identiske. Retry-logik, fejlhåndtering, rate limiting er fuldstændig duplikeret.
- **Fix:** Ekstraher fælles HTTP-request/retry-logik til en private helper parameteriseret med response parser.

### Duplikering i ChatWindow.vue

- **Fil:** `src/components/ChatWindow.vue:393-536`
- **Problem:** `sendMessage` og `regenerateResponse` indeholder næsten identisk logik til chat history, instruktioner, kald til `chat_with_ai` og fejlhåndtering.
- **Fix:** Ekstraher en shared `executeChatRequest` metode.

### Duale datastrukturer i OperationEditWindow

- **Fil:** `src/components/OperationEditWindow.vue:216-221`
- **Problem:** Både `operations` (Record) og `operationsArray` (Array) skal holdes synkront manuelt (linje 257-261, 386-404, 492-500).
- **Fix:** Brug kun sorteret array og udled lookup-map via computed.

### Hardcoded shortcut trods config-felt

- **Fil:** `src-tauri/src/lib.rs:41`
- **Problem:** Global genvej er hardcoded `"CmdOrCtrl+Space"`. `Config` har et `shortcut` felt, og SettingsWindow har default, men ingen kode læser det og registrerer dynamisk.
- **Fix:** Læs `config.shortcut` i `lib.rs` setup og registrér den konfigurerede genvej.

---

## 🟠 Kodekvalitet

### Dead code

- **Fil:** `src-tauri/src/commands/utility_commands.rs:6-31`
- **Problem:** `greet` og `process_text` er aldrig brugt fra frontend. `process_text` har placeholder-implementation.
- **Fix:** Fjern begge funktioner og deres registration i `lib.rs`.

### Encoding-fejl

- **Fil:** `src/components/OperationEditWindow.vue:48`
- **Problem:** `â€¢` – UTF-8 encoding issue. Skal være bullet `•`.
- **Fix:** Erstat med korrekt Unicode bullet (U+2022).

### Identiske computed properties

- **Fil:** `src/components/ChatWindow.vue:242-252`
- **Problem:** `supportsThinking` og `supportsGrounding` tjekker nøjagtig samme betingelse.
- **Fix:** Konsolider til én computed, eller dokumentér fremtidig hensigt.

### Catch-blokke smider ny Error uden original stack

- **Fil:** `src/components/PopupWindow.vue:195` (og flere steder)
- **Problem:** `throw new Error(\`Failed to process text: ${err}\`)` mister original stack trace.
- **Fix:** Brug `throw err` direkte eller wrap med `.cause`.

### .gitignore tastefejl

- **Fil:** `.gitignore:27`
- **Problem:** `pyton program` – matcher intet.
- **Fix:** Ret til `python` eller fjern linjen.

### Cargo.toml placeholders

- **Fil:** `src-tauri/Cargo.toml:4-5`
- **Problem:** `description = "A Tauri App"` og `authors = ["you"]`.
- **Fix:** Opdater med reelle værdier.

### Kodepilkering i window_commands

- **Fil:** `src-tauri/src/commands/window_commands.rs`
- **Problem:** `reopen_chat_conversation` (linje 6-50) og `open_chat_window` (linje 52-108) deler ~25 linjer identisk `WebviewWindowBuilder`-konfiguration.
- **Fix:** Ekstraher shared window builder helper.

### Inkonsistent shortcut-format

| Lokation | Format |
|----------|--------|
| `src-tauri/src/lib.rs:41` | `"CmdOrCtrl+Space"` |
| `src-tauri/src/window_manager.rs:161` | `"CmdOrCtrl+Space"` |
| `src/components/OnboardingWindow.vue:499` | `"ctrl+space"` |
| `src/components/SettingsWindow.vue:146` | `"CommandOrControl+Space"` |

- **Fix:** Brug samme format alle steder (Tauri v2 format).

### Inkonsistent `:deep()` brug i MessageBubble

- **Fil:** `src/components/MessageBubble.vue:247-348`
- **Problem:** De fleste markdown-selectors bruger `:deep()`, men nogle få gør ikke. Da SanitizedMarkdown renderer med `v-html`, er `:deep()` nødvendigt.

---

## 🔵 Type-sikkerhed

### ModelName union type defineret men ikke brugt

- **Fil:** `src/types/index.ts:88` vs `src/components/ChatWindow.vue:200`
- **Problem:** `export type ModelName = 'gemini-3-flash-preview' | 'gemini-3.1-flash-lite-preview'` er defineret, men `state.selectedModel` er typed som `string`.
- **Fix:** Brug `ModelName` typen i stedet for `string`.

### ProviderSettings index signature udvander type safety

- **Fil:** `src/types/index.ts:27-33`
- **Problem:** `[key: string]: string | undefined` tillader alle string properties.
- **Fix:** Brug `Partial<{...}>` og fjern index signature.

### Unødvendig `as string` assertion

- **Fil:** `src/components/ChatWindow.vue:200`
- **Problem:** `state.selectedModel = 'gemini-3-flash-preview' as string` – smider literal typen væk.
- **Fix:** Fjern `as string` og brug `ModelName` typen.

### Inlinet type i stedet for genbrug af shared type

- **Fil:** `src/components/ChatWindow.vue:640-659`
- **Problem:** Return type for `invoke('load_conversation_messages')` er inlinet som dybt nested objekt i stedet for at importere `SavedConversation`.
- **Fix:** Importer og brug `SavedConversation`.

### `#[allow(dead_code)]` på 15+ Rust felter

- **Fil:** `src-tauri/src/ai_provider/types.rs`
- **Problem:** Mange struct-felter suppresser dead_code warning. Nogle er API response-felter der ikke bruges.
- **Fix:** Tilføj doc comments der forklarer hvorfor de eksisterer, eller fjern unødvendige felter.

### ChatEntry duplikeres mellem TS og Rust

- **Filer:** `src/types/index.ts:47-52` vs `src-tauri/src/data_manager/types.rs:79-84`
- **Problem:** Samme struct defineres begge steder uden shared schema til at holde dem synkroniserede.

---

## ⚡ Performance

### DataManager skriver hele filen ved hver ændring

- **Fil:** `src-tauri/src/data_manager/manager.rs:76-274`
- **Problem:** Hver config-update, operation-ændring eller history-save kalder `save_data()`, som serialiserer HELE `AppData` til JSON og skriver til disk.

### get_operations_sorted kloner alle operations

- **Fil:** `src-tauri/src/data_manager/manager.rs:276-294`
- **Problem:** Kloner alle operations (inkl. `instruction` strings) til ny Vec hver gang popup åbnes.

### clipboardText som både prop og ref

- **Fil:** `src/components/PopupWindow.vue:80-105`
- **Problem:** `clipboardText` oprettes som `ref` initialiseret fra props, i stedet for at være `computed`.
- **Fix:** Brug `computed(() => props.selectedText || window.clipboardText || '')`.

### Redundant URL parameter parsing i ChatWindow onMounted

- **Fil:** `src/components/ChatWindow.vue:755-762`
- **Problem:** `chat.ts` parser allerede URL parametre og sender som props. `onMounted` re-parser som fallback – dead code.

---

## ✅ Positivt

- **Sikkerhed:** DOMPurify til markdown, CSP headers i tauri.conf.json, password-felter til API keys, nulstilling ved unmount.
- **Fejlhåndtering:** `gemini_error_to_user_message` oversætter alle Gemini-fejl til brugbare beskeder.
- **Atomisk filskrivning:** `.json.tmp` → rename → forhindrer korruption ved crash.
- **Multi-window arkitektur:** Clean pattern med HTML entrypoints + `window-bootstrap.ts`.
- **Input-validering:** Både frontend (`ChatWindow.vue:254-275`) og backend (`validation.rs`).
- **Migration:** Automatisk migrering fra 4 gamle filformater (`config.json`, `options.json`, `chat_history.json`, `saved_conversations.json`) til `app_data.json`, med `.old` backups.
- **Debouncing:** Shortcut manager med `Mutex<Instant>` og 200ms debounce. Håndterer poisoned mutexes.
- **ARIA attributes:** `AppConfirmDialog`, `AppPromptDialog`, `AppToast` med `role="status"`, `aria-live="polite"`, `aria-hidden`.
- **TypeScript practices:** Veltypede props/emits interfaces, `const` assertions, optional chaining.
- **Asynkron window management:** Fælles `WindowConfig` struct og `create_window` pattern på tværs af alle vinduer.
