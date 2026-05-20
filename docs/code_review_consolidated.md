# Code Review — AI TextTool (Konsolideret)

> Udført: 2026-05-20
> Kilder: Qwen 3.6 Plus Free + DeepSeek V4 Flash
> Status: P0 delvist fikset (2/3), P1 delvist fikset (2/6), P2-P3 pending. Branch: `fix/code-review-cleanup`

---

## 🔴 P0 — Kritiske sikkerhedsproblemer

### 1. XSS i clipboard-tekst injektion ✅ LØST
- **Fil:** `src-tauri/src/window_manager.rs:206-212`
- **Problem:** Clipboard-tekst injiceres via `initialization_script` med kun `'` escaping. Mangler escaping for `</script>`, backticks, `\` og Unicode. Ondskabsfuld tekst kan bryde ud af strengen.
- **Fix:** Brugt `serde_json::to_string()` til korrekt escaping.
- **Commit:** `2b8e924`

### 2. API key eksponeret i URL query params ⚠️ WONTFIX
- **Fil:** `src-tauri/src/ai_provider/gemini.rs:203`
- **Problem:** `.query(&[("key", &self.api_key)])` sender API-nøglen som URL parameter. Lækkes til proxies, server-logs og browser-historik.
- **Fix:** Brug `Authorization: Bearer <key>` header.
- **Status:** Google AI Studio API understøtter **kun** `?key=` query parameter. `Authorization: Bearer` er kun til OAuth2 tokens. Ikke muligt at ændre uden at skifte til Google Cloud Vertex AI.

### 3. Ingen global rate limiting ✅ LØST
- **Fil:** `src-tauri/src/ai_provider/gemini.rs:24-68, 76`
- **Problem:** `RateLimiter` er et struct-felt i `GeminiProvider`. Hver `new()` giver en ny limiter. Samtidige vinduer overskrider Gemini's grænser.
- **Fix:** `GlobalRateLimiter` oprettes ved startup og gemmes som Tauri managed state. Alle `GeminiProvider` instanser deler samme limiter via `Arc<Mutex<RateLimiter>>`.
- **Commit:** `2b8e924`

---

## 🟠 P1 — Design- og arkitekturproblemer

### 4. Hvid tekst på lys baggrund — ulæseligt i light mode ✅ LØST
- **Fil:** `src/components/MessageBubble.vue:231, 167`
- **Problem:** `.markdown-content { color: #ffffff; }` på baggrund `rgba(255, 255, 255, 0.95)`. Brødtekst er hvid-på-næsten-hvid i light mode. Dark mode fungerer fint (`color: #e2e8f0` via `prefers-color-scheme: dark`).
- **Fix:** Ændret base color til `#333`, headings til `#1a1a2e`, tables/blockquotes/hr til light-mode farver.
- **Commit:** `adf4596`

### 5. Manglende ikonfiler til bundling ⚠️ INVALID
- **Fil:** `src-tauri/tauri.conf.json:31-36`
- **Problem:** Bundle config refererer til `icons/32x32.png`, `icons/128x128.png`, `icons/128x128@2x.png`, `icons/icon.icns`, `icons/icon.ico` — men disse filer findes ikke. `src-tauri/icons/` indeholder kun Microsoft Store PNGs. Byg fejler ved packaging.
- **Status:** Alle ikonfiler eksisterer allerede i `src-tauri/icons/`. Fundet er baseret på en fejltagelse.

### 6. DataManager instantieres for hver kommando
- **Filer:** `src-tauri/src/commands/ai_commands.rs:51-58`, `src-tauri/src/data_manager/commands.rs:11-15`
- **Problem:** `load_data_manager()` kalder `DataManager::new()` + `initialize()` ved hver Tauri-kommando. Hele `app_data.json` loades og parses hver gang.
- **Fix:** Gør `DataManager` til Tauri managed state med `app.manage(DataManager)` ved startup.

### 7. Redundant config-struktur med duplikerede felter
- **Fil:** `src-tauri/src/data_manager/types.rs:18-23, 38-65`
- **Problem:** `Config` har både flade felter (`api_key`, `chat_model`, `text_model`) OG en `providers` HashMap med samme data. `api_key` findes begge steder. `dm_switch_provider` kopierer ikke data fra ny provider.
- **Fix:** Vælg én kilde til sandhed. Brug kun `providers` HashMap og udled top-level værdier derfra.

### 8. Hardcodede modelnavne flere steder
- **Filer:** `src-tauri/src/ai_provider/gemini.rs:519`, `src/components/ChatWindow.vue:242-252`
- **Problem:** Modelnavne hardcodet i `get_available_models()` og i `supportsThinking`/`supportsGrounding` computed properties. Skal opdateres manuelt ved nye modeller.
- **Fix:** Definer model-metadata ét sted (f.eks. `ModelInfo` struct) og eksportér til både Rust og frontend.

### 9. `reqwest::Client` oprettes forfra ved hvert kald
- **Fil:** `src-tauri/src/ai_provider/gemini.rs:84-121`
- **Problem:** `GeminiProvider::new()` opretter en ny `reqwest::Client` hver gang. Clienten er designet til genbrug med connection pooling.
- **Fix:** Gem client i DataManager eller Tauri managed state.

### 10. Hardcoded shortcut trods config-felt
- **Fil:** `src-tauri/src/lib.rs:41`, `src-tauri/src/window_manager.rs:161`
- **Problem:** Global genvej `"CmdOrCtrl+Space"` er hardcoded. Ingen kode læser en konfigureret værdi og registrerer dynamisk.
- **Fix:** Læs shortcut fra config ved startup og registrér dynamisk.

### 11. Inkonsistent shortcut-format på tværs af kodebasen
- **Filer:** `lib.rs:41`, `window_manager.rs:161`, `OnboardingWindow.vue:499`, `SettingsWindow.vue:146`
- **Problem:** Fire forskellige formater: `"CmdOrCtrl+Space"`, `"ctrl+space"`, `"CommandOrControl+Space"`.
- **Fix:** Standardisér til Tauri v2 format ét sted.

### 12. `clear_chat_history` sletter også saved conversations ✅ LØST
- **Fil:** `src-tauri/src/data_manager/manager.rs:244-248`
- **Problem:** Funktionen kaldte både `chat_history.clear()` og `saved_conversations.clear()`. Uventet sideeffekt.
- **Fix:** Splittet i `clear_chat_history()` og `clear_saved_conversations()` med separat Tauri command.
- **Commit:** `adf4596`

---

## 🟡 P2 — Kodekvalitetsproblemer

### 13. Duplikeret retry-logik i Gemini provider
- **Fil:** `src-tauri/src/ai_provider/gemini.rs:162-299` vs `322-487`
- **Problem:** `generate_content_with_retry` og `generate_chat_content_with_retry` er ~160 linjer hver og næsten identiske (rate limit, URL construction, error handling, exponential backoff).
- **Fix:** Lav generisk `with_retry<F, T>(f: F, max_retries: u32) -> T` helper.

### 14. Duplikering i ChatWindow.vue
- **Fil:** `src/components/ChatWindow.vue:393-536`
- **Problem:** `sendMessage` og `regenerateResponse` indeholder næsten identisk logik til chat history, instruktioner, AI-kald og fejlhåndtering.
- **Fix:** Ekstraher shared `execute_chat_request` metode.

### 15. Død kode: `greet` og `process_text`
- **Fil:** `src-tauri/src/commands/utility_commands.rs:6-31`
- **Problem:** `greet` er Tauri template-rest. `process_text` er placeholder der aldrig bør være i production. Registreret i `lib.rs:65-66`.
- **Fix:** Fjern begge kommandoer og deres registration i `invoke_handler`.

### 16. Escape key håndteres dobbelt
- **Fil:** `src/components/PopupWindow.vue:2, 306`
- **Problem:** Template binder `@keydown="handleKeydown"` OG `onMounted` tilføjer `document.addEventListener('keydown', handleKeydown)`. Escape fires to gange.
- **Fix:** Fjern `document.addEventListener`, behold kun template-bindingen.

### 17. Duplicate Window interface
- **Filer:** `src/vite-env.d.ts:10-14`, `src/types/window.d.ts:5-13`
- **Problem:** Begge erklærer `interface Window { clipboardText?: string }`. TypeScript merger dem, men det er forvirrende og fejlbehæftet.
- **Fix:** Fjern erklæringen fra `vite-env.d.ts`, behold kun i `src/types/window.d.ts`.

### 18. `xxx:` pseudo-protokol i DOMPurify regex
- **Fil:** `src/utils/markdown.ts:67-68`
- **Problem:** `ALLOWED_URI_REGEXP` tillader `xxx:` — ikke en reel URI-protokol, kan udnyttes.
- **Fix:** Fjern `|xxx` fra regex.

### 19. Identiske computed properties
- **Fil:** `src/components/ChatWindow.vue:242-252`
- **Problem:** `supportsThinking` og `supportsGrounding` er byte-for-byte identiske — samme model-array check.
- **Fix:** Konsolider til én computed eller giv dem forskellige model-lister.

### 20. Duale datastrukturer i OperationEditWindow
- **Fil:** `src/components/OperationEditWindow.vue:216-221, 257-261, 386-404, 492-500`
- **Problem:** Både `operations` (Record) og `operationsArray` (Array) skal holdes synkront manuelt.
- **Fix:** Brug kun sorteret array og udled lookup-map via `computed`.

### 21. Magic numbers i shortcut_manager
- **Fil:** `src-tauri/src/shortcut_manager.rs:35, 51, 77, 92`
- **Problem:** `200` (debounce), `50` (initial delay), `50` (clipboard write delay), `250` (copy-wait) — ingen navngivne konstanter eller dokumentation.
- **Fix:** Definer som navngivne konstanter øverst i filen.

### 22. `enigo` initieres flere gange
- **Fil:** `src-tauri/src/shortcut_manager.rs:173, 193, 234`
- **Problem:** `Enigo::new()` kaldes hver gang. Relativt tung operation.
- **Fix:** Genbrug via `OnceCell` eller Tauri state.

### 23. `std::process::exit(1)` uden cleanup
- **Fil:** `src-tauri/src/lib.rs:98-101`
- **Problem:** Hvis Tauri fejler, kaldes `exit(1)` uden cleanup. Trays, windows og fil-handles efterlades inkonsistent.
- **Fix:** Log fejlen og lad Tauri håndtere shutdown naturligt.

### 24. `use_formatting: false` giver tom system instruction
- **Fil:** `src-tauri/src/ai_provider/gemini.rs:186`
- **Problem:** Når `use_formatting` er false og ingen custom instruction gives, får AI'en ingen kontekst.
- **Fix:** Brug en fallback system instruction.

### 25. DataManager::new ignorerer `_app_handle`
- **Fil:** `src-tauri/src/data_manager/manager.rs:42-47`
- **Problem:** Parameteren er unused.
- **Fix:** Fjern parameteren eller brug den.

### 26. Catch-blokke smider ny Error uden original stack
- **Fil:** `src/components/PopupWindow.vue:195` (og flere steder)
- **Problem:** `throw new Error(\`Failed: ${err}\`)` mister original stack trace.
- **Fix:** Brug `throw err` direkte eller wrap med `.cause`.

### 27. Encoding-fejl: bullet character
- **Fil:** `src/components/OperationEditWindow.vue:48`
- **Problem:** `â€¢` i stedet for korrekt bullet `•` (U+2022). UTF-8 mojibake.
- **Fix:** Erstat med korrekt Unicode-tegn.

### 28. Kodepilkering i window_commands
- **Fil:** `src-tauri/src/commands/window_commands.rs:6-50, 52-108`
- **Problem:** `reopen_chat_conversation` og `open_chat_window` deler ~25 linjer identisk `WebviewWindowBuilder`-konfiguration.
- **Fix:** Ekstraher shared window builder helper.

### 29. Massiv `ChatWindow.vue` (1247 linjer)
- **Fil:** `src/components/ChatWindow.vue`
- **Fix:** Split i `ChatHeader.vue`, `MessageList.vue`, `ModelControls.vue`.

### 30. Massiv `OnboardingWindow.vue` (1391 linjer)
- **Fil:** `src/components/OnboardingWindow.vue`
- **Fix:** Split steps i `ApiKeyStep.vue`, `InstructionStep.vue`, `ConnectionTestStep.vue`.

---

## 🔵 P3 — Type-sikkerhed og mindre problemer

### 31. `window.d.ts` bruger `any`
- **Fil:** `src/types/window.d.ts`
- **Problem:** `window.clipboardText` er typed som `any`.
- **Fix:** Brug `string | undefined`.

### 32. ModelName union type defineret men ikke brugt
- **Fil:** `src/types/index.ts:88` vs `src/components/ChatWindow.vue:200`
- **Problem:** `ModelName` er defineret men `state.selectedModel` er `string`. `as string` assertion smider literal type væk.
- **Fix:** Brug `ModelName` typen direkte, fjern `as string`.

### 33. ProviderSettings index signature udvander type safety
- **Fil:** `src/types/index.ts:27-33`
- **Problem:** `[key: string]: string | undefined` tillader alle string properties.
- **Fix:** Brug `Partial<{...}>` og fjern index signature.

### 34. Inlinet type i stedet for shared type
- **Fil:** `src/components/ChatWindow.vue:640-659`
- **Problem:** Return type for `load_conversation_messages` er inlinet som dybt nested objekt i stedet for at importere `SavedConversation`.
- **Fix:** Importer og brug den eksisterende type.

### 35. `#[allow(dead_code)]` på 15+ Rust felter
- **Fil:** `src-tauri/src/ai_provider/types.rs`
- **Problem:** Mange struct-felter suppresser dead_code warning uden forklaring.
- **Fix:** Tilføj doc comments eller fjern unødvendige felter.

### 36. ChatEntry duplikeres mellem TS og Rust
- **Filer:** `src/types/index.ts:47-52` vs `src-tauri/src/data_manager/types.rs:79-84`
- **Problem:** Samme struct defineres begge steder uden shared schema.
- **Fix:** Brug f.eks. `ts-rs` crate til at generere TS-typer fra Rust.

### 37. Ingen validering af `invoke` resultater
- **Problem:** Koden caster ofte `as string` eller `as AIResponse` uden runtime validering.
- **Fix:** Brug type guards eller zod-schemaer.

### 38. Manglende error boundaries i Vue
- **Problem:** Hvis en komponent fejler under rendering, crasher hele appen.
- **Fix:** Tilføj `onErrorCaptured` i rod-komponenter.

### 39. Toast timer cleanup
- **Fil:** `src/components/ChatWindow.vue:782-796`
- **Problem:** `saveDialogResolver` resolves med `null` og `clearDialogResolver` med `false` ved unmount — kan silently dismiss pending prompts.
- **Fix:** Håndter unresolved dialogs eksplicit ved cleanup.

### 40. Ingen database migration strategi
- **Fil:** `src-tauri/src/data_manager/manager.rs:68-73`
- **Problem:** `app_data.json` er et enkelt JSON-fil. Schema-ændringer kan give parsing errors. `#[serde(default)]` hjælper, men ingen version-migration udover `normalize_config_models`.
- **Fix:** Implementer version-check ved load og migrér data til nyeste schema.

### 41. URL encoding af stor tekst kan fejle
- **Fil:** `src-tauri/src/commands/window_commands.rs:70-77`
- **Problem:** Hele teksten URL-encodes som query parameter. URL'er har typisk grænse på ~2000-8000 tegn.
- **Fix:** Send tekst via Tauri events eller state.

### 42. `Cargo.toml` mangler `[lints]` sektion og har placeholders
- **Fil:** `src-tauri/Cargo.toml:4-5`
- **Problem:** `description = "A Tauri App"`, `authors = ["you"]`. Mangler `[lints.clippy]` sektion til CI.
- **Fix:** Opdater med reelle værdier og tilføj lint-konfiguration.

### 43. `.gitignore` tastefejl
- **Fil:** `.gitignore:27`
- **Problem:** `pyton program` — "pyton" skal være "python". Mellemrummet gør det til to separate patterns.
- **Fix:** Ret til `python` eller fjern linjen.

### 44. Inkonsistent `:deep()` brug i MessageBubble
- **Fil:** `src/components/MessageBubble.vue:247-348`
- **Problem:** De fleste markdown-selectors bruger `:deep()`, men nogle gør ikke. Da SanitizedMarkdown renderer med `v-html`, er `:deep()` nødvendigt.
- **Fix:** Tilføj `:deep()` på alle selectors der rammer `v-html` indhold.

---

## ⚡ Performance-problemer

### 45. DataManager skriver hele filen ved hver ændring
- **Fil:** `src-tauri/src/data_manager/manager.rs:76-274`
- **Problem:** Hver config-update, operation-ændring eller history-save kalder `save_data()`, som serialiserer HELE `AppData` til JSON.
- **Fix:** Brug debounced writes eller delvis opdatering.

### 46. `get_operations_sorted` kloner alle operations
- **Fil:** `src-tauri/src/data_manager/manager.rs:276-294`
- **Problem:** Kloner alle operations (inkl. `instruction` strings) til ny Vec hver gang popup åbnes.
- **Fix:** Returner references eller brug en cache.

### 47. `clipboardText` som både prop og ref
- **Fil:** `src/components/PopupWindow.vue:80-105`
- **Problem:** Oprettes som `ref` initialiseret fra props i stedet for `computed`.
- **Fix:** Brug `computed(() => props.selectedText || window.clipboardText || '')`.

### 48. Redundant URL parameter parsing i ChatWindow
- **Fil:** `src/components/ChatWindow.vue:755-762`
- **Problem:** `chat.ts` parser allerede URL parametre og sender som props. `onMounted` re-parser som fallback — dead code.
- **Fix:** Fjern den redundante parsing.

---

## ✅ Positivt bemærket

- **Sikkerhed:** DOMPurify til markdown, CSP headers i tauri.conf.json, password-felter til API keys, nulstilling ved unmount
- **Fejlhåndtering:** `gemini_error_to_user_message` oversætter alle Gemini-fejl til brugbare beskeder
- **Atomisk filskrivning:** `.json.tmp` → rename → forhindrer korruption ved crash
- **Multi-window arkitektur:** Clean pattern med HTML entrypoints + `window-bootstrap.ts`, fælles `WindowConfig` struct og `create_window` pattern
- **Input-validering:** Både frontend og backend (`validation.rs`)
- **Migration:** Automatisk migrering fra 4 gamle filformater til `app_data.json` med `.old` backups
- **Debouncing:** Shortcut manager med `Mutex<Instant>` og 200ms debounce, håndterer poisoned mutexes
- **ARIA attributes:** `AppConfirmDialog`, `AppPromptDialog`, `AppToast` med `role="status"`, `aria-live="polite"`
- **TypeScript practices:** Veltypede props/emits interfaces, `const` assertions, optional chaining

---

## Prioriteret handlingsplan

| Prioritet | # | Issue | Status | Indsats | Effekt |
|-----------|---|-------|--------|---------|--------|
| P0 | 1 | XSS i clipboard injection | ✅ Løst | Medium | Høj — sikkerhed |
| P0 | 2 | API key i URL | ⚠️ Wontfix | Lav | Høj — sikkerhed |
| P0 | 3 | Global rate limiting | ✅ Løst | Medium | Høj — stabilitet |
| P1 | 4 | Hvid tekst i light mode | ✅ Løst | Lav | Høj — UI bug |
| P1 | 5 | Manglende ikonfiler | ⚠️ Invalid | — | — |
| P1 | 6 | DataManager som Tauri state | Pending | Høj | Høj — ydeevne + race conditions |
| P1 | 7 | Redundant config | Pending | Medium | Høj — dataintegritet |
| P1 | 9 | reqwest::Client genbrug | Pending | Lav | Medium — ydeevne |
| P1 | 10 | Hardcoded shortcut | Pending | Medium | Medium — funktionalitet |
| P1 | 12 | clear_chat_history sideeffekt | ✅ Løst | Lav | Medium — UX |
| P2 | 13 | Duplikeret retry-logik | Pending | Lav | Medium — vedligeholdelse |
| P2 | 15 | Død kode | Pending | Lav | Lav — oprydning |
| P2 | 16 | Escape key dobbelt | Pending | Lav | Medium — UX bug |
| P2 | 18 | xxx: i DOMPurify | Pending | Lav | Medium — sikkerhed |
| P2 | 21 | Magic numbers | Pending | Lav | Lav — læsbarhed |
| P2 | 29-30 | Store Vue-komponenter | Pending | Høj | Medium — vedligeholdelse |
| P3 | 31-37 | TypeScript type-problemer | Pending | Lav | Lav — typesikkerhed |
| P3 | 42-43 | Cargo.toml / .gitignore | Pending | Lav | Lav — oprydning |
| ⚡ | 45 | DataManager skriver hele filen | Pending | Medium | Høj — ydeevne |
| ⚡ | 47 | clipboardText som computed | Pending | Lav | Lav — ydeevne |
