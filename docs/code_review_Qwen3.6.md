# Code Review — AI TextTool

> Udført: 2026-05-20
> Model: Qwen 3.6 Plus Free

---

## 🔴 Kritiske sikkerhedsproblemer

### 1. XSS i clipboard-tekst injektion (`window_manager.rs:207-212`)

Clipboard-tekst injiceres som et JavaScript-initialiseringsscript via `initialization_script`. Selvom `'`, `\n`, `\r` escapes, er der ingen escaping for `</script>`, backticks, eller Unicode-tegn. En bruger med ondsindet tekst i udklipsholderen kan potentielt udføre XSS.

**Fil:** `src-tauri/src/window_manager.rs:207-212`
**Anbefaling:** Brug `JSON.stringify()` til korrekt escaping eller send data via Tauri events i stedet for inline scripts.

### 2. API key eksponeret i URL query params (`gemini.rs:203`)

API-nøglen sendes som `?key=...` i URL'en. Den vil blive logget af proxies, browser-historik, og server-logs.

**Fil:** `src-tauri/src/ai_provider/gemini.rs:203`
**Anbefaling:** Brug `Authorization: Bearer <key>` headeren i stedet for query parameter.

### 3. Ingen global rate limiting (`gemini.rs:24-68`)

`RateLimiter` er per-instans. Hvis flere vinduer kalder AI samtidig, vil hver have sin egen rate limiter, hvilket kan overskride Gemini's grænser.

**Fil:** `src-tauri/src/ai_provider/gemini.rs:24-68`
**Anbefaling:** Gør rate limiteren global via `Arc<Mutex<RateLimiter>>` i Tauri state (`app.manage()`).

---

## 🟠 Design- og arkitekturproblemer

### 4. DataManager instantieres for hver kommando

Hver Tauri-kommando loader hele `app_data.json` fra disken. Det er ineffektivt og kan give race conditions ved samtidige writes.

**Filer:**
- `src-tauri/src/commands/ai_commands.rs:51-58`
- `src-tauri/src/data_manager/commands.rs:11-15`

**Anbefaling:** Gør `DataManager` til en Tauri state med `app.manage(DataManager)` ved startup. Brug `app.state::<DataManager>()` i kommandoer.

### 5. Redundant config-struktur

`Config` har både flade felter (`api_key`, `chat_model`, `text_model`) OG en `providers` HashMap. Data duplikeres — `api_key` findes både i `config.api_key` og `config.providers["Gemini"].api_key`. `dm_switch_provider` skifter kun `config.provider` men kopierer ikke data fra den nye provider.

**Fil:** `src-tauri/src/data_manager/types.rs:38-65`
**Anbefaling:** Vælg én kilde til sandhed. Enten brug kun `providers` HashMap'en eller fjern den og brug kun de flade felter.

### 6. Hardcodede modelnavne

Modelnavne er hardcodet flere steder. `supports_google_search_grounding` og `supportsThinking` computed properties skal opdateres manuelt når nye modeller tilføjes.

**Filer:**
- `src-tauri/src/ai_provider/gemini.rs:519`
- `src/components/ChatWindow.vue:243-251`

**Anbefaling:** Definer model-metadata ét sted (f.eks. en `ModelInfo` struct) og eksportér til både Rust og frontend.

### 7. Død kode: `greet` og `process_text`

`greet` er en standard Tauri template rest. `process_text` er en placeholder der aldrig bør være i production.

**Fil:** `src-tauri/src/commands/utility_commands.rs:6-31`
**Anbefaling:** Fjern begge kommandoer og fjern dem fra `invoke_handler` listen i `lib.rs`.

---

## 🟡 Kodekvalitetsproblemer

### 8. Duplikeret retry-logik

`generate_content_with_retry` og `generate_chat_content_with_retry` er næsten identiske (~160 linjer hver).

**Fil:** `src-tauri/src/ai_provider/gemini.rs:162-299` vs `322-487`
**Anbefaling:** Lav en generisk `with_retry<F, T>(f: F, max_retries: u32) -> T` funktion.

### 9. `DataManager::new` ignorerer `_app_handle`

Parameteren er unused.

**Fil:** `src-tauri/src/data_manager/manager.rs:42-47`
**Anbefaling:** Fjern parameteren hvis den ikke er nødvendig, ellers brug den.

### 10. `std::process::exit(1)` uden cleanup

Hvis Tauri fejler, kaldes `exit(1)` uden cleanup. Trays, windows, og fil-handles kan blive efterladt i en inkonsistent tilstand.

**Fil:** `src-tauri/src/lib.rs:98-101`
**Anbefaling:** Log fejren og lad Tauri håndtere shutdown naturligt. Undgå `process::exit` medmindre absolut nødvendigt.

### 11. `clear_chat_history` sletter også saved conversations

Funktionen hedder `clear_chat_history` men sletter også `saved_conversations`. Det er uventet adfærd.

**Fil:** `src-tauri/src/data_manager/manager.rs:244-248`
**Anbefaling:** Split i to separate funktioner: `clear_chat_history` og `clear_saved_conversations`.

### 12. Magic numbers i shortcut_manager

`200ms` debounce, `50ms` delay, `250ms` copy-wait, `100ms` retry — ingen af disse er dokumenteret eller konfigurerbare.

**Fil:** `src-tauri/src/shortcut_manager.rs:35, 51, 77, 92`
**Anbefaling:** Definer som navngivne konstanter øverst i filen med dokumentation.

### 13. `use_formatting: false` giver tom system instruction

Når `use_formatting` er false og ingen custom instruction gives, får AI'en ingen kontekst.

**Fil:** `src-tauri/src/ai_provider/gemini.rs:186`
**Anbefaling:** Brug en fallback system instruction når ingen er givet.

---

## 🟢 Mindre problemer og forbedringer

### 14. `window.d.ts` bruger `any`

`window.clipboardText` er typed som `any`.

**Fil:** `src/types/window.d.ts`
**Anbefaling:** Brug `string | undefined`.

### 15. Manglende error boundaries i Vue

Hvis en komponent fejler under rendering, crasher hele appen uden graceful fallback.

**Anbefaling:** Tilføj Vue error boundaries eller brug `onErrorCaptured` i rod-komponenter.

### 16. `ChatWindow.vue` er for stor (1247 linjer)

**Fil:** `src/components/ChatWindow.vue`
**Anbefaling:** Split op i: `ChatHeader.vue`, `MessageList.vue`, `ModelControls.vue`.

### 17. `OnboardingWindow.vue` er for stor (1391 linjer)

**Fil:** `src/components/OnboardingWindow.vue`
**Anbefaling:** Split steps i separate komponenter: `ApiKeyStep.vue`, `InstructionStep.vue`, `ConnectionTestStep.vue`.

### 18. Ingen validering af `invoke` resultater

Koden caster ofte `as string` eller `as AIResponse` uden validering af at svaret faktisk har den forventede form.

**Anbefaling:** Brug type guards eller zod-schemaer til runtime validering af Tauri-svar.

### 19. URL encoding af stor tekst kan fejle

Hele teksten URL-encodes og sendes som query parameter. URL'er har typisk en grænse på ~2000-8000 tegn.

**Fil:** `src-tauri/src/commands/window_commands.rs:70-77`
**Anbefaling:** Send tekst via Tauri events eller state i stedet for URL-parametre.

### 20. `Cargo.toml` mangler `[lints]` sektion

Clippy kører via `npm run fix` men er ikke konfigureret i Cargo.toml til at fejle i CI.

**Fil:** `src-tauri/Cargo.toml`
**Anbefaling:** Tilføj `[lints.clippy]` sektion med ønskede regler.

### 21. Ingen database migration strategi

`app_data.json` er et enkelt JSON-file. Når schema ændres, kan gamle filer give parsing errors. `#[serde(default)]` hjælper, men der er ingen version-migration udover `normalize_config_models`.

**Fil:** `src-tauri/src/data_manager/manager.rs:68-73`
**Anbefaling:** Implementer en version-check ved load og migrer data til nyeste schema.

### 22. `enigo` initieres flere gange

`Enigo::new()` kaldes hver gang. Det er en relativt tung operation.

**Fil:** `src-tauri/src/shortcut_manager.rs:173, 193, 234`
**Anbefaling:** Caches eller genbruges via `OnceCell` eller Tauri state.

### 23. Toast timer cleanup

`toastTimer` cleanuppes i `onUnmounted`, men hvis komponenten unmountes mens toast er synlig, vil `toastVisible` forblive `true`.

**Fil:** `src/components/ChatWindow.vue:782-796`
**Anbefaling:** Sæt `toastVisible.value = false` i cleanup.

---

## Prioriteret handlingsplan

| Prioritet | Issue | Indsats | Effekt |
|-----------|-------|---------|--------|
| P0 | #1 XSS i clipboard injection | Medium | Høj — sikkerhed |
| P0 | #2 API key i URL | Lav | Høj — sikkerhed |
| P0 | #3 Global rate limiting | Medium | Høj — stabilitet |
| P1 | #4 DataManager som Tauri state | Høj | Høj — ydeevne + race conditions |
| P1 | #5 Redundant config | Medium | Medium — dataintegritet |
| P1 | #19 URL encoding af stor tekst | Medium | Høj — funktionalitet |
| P2 | #8 Duplikeret retry-logik | Lav | Medium — vedligeholdelse |
| P2 | #11 clear_chat_history adfærd | Lav | Medium — UX |
| P2 | #16, #17 Store Vue-komponenter | Høj | Medium — vedligeholdelse |
| P3 | #7 Død kode | Lav | Lav — oprydning |
| P3 | #12 Magic numbers | Lav | Lav — læsbarhed |
| P3 | #14, #18 TypeScript typer | Lav | Lav — typesikkerhed |
