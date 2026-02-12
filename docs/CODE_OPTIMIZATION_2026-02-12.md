# Teknisk Gennemgang og Optimeringsrapport

**Projekt:** AI-TextTool  
**Dato:** 12. februar 2026  
**Type:** Prioriteret roadmap (ikke fuld audit)  
**Scope:** Dokumentation af forbedringer og kodeoptimeringer, ingen kodeaendringer i denne leverance

## 1. Scope og metode

Denne rapport er baseret paa aktuel kodeinspektion og verificerede kommandoer i repository'et:

- `npm run check`
- `npx eslint src --ext .vue,.js,.ts -f json`
- `npm run lint:rust`
- `cd src-tauri && cargo test`
- `npm run build:web`
- maalrettede `rg`-soegninger i `src/`, `src-tauri/`, `windows/` og `docs/`

Maalet er at reducere build/lint-stoej, forbedre robusthed og forberede en mere vedligeholdbar arkitektur uden at aendre funktionel adfaerd i denne omgang.

## 2. Baseline-maalinger (verificeret)

1. `npm run check` fejler med **1504 fejl** og **86 warnings**.
2. ESLint-regelfordeling:
- `1504` x `prettier/prettier`
- `81` x `no-console`
- `3` x `vue/no-v-html`
- `2` x `@typescript-eslint/no-explicit-any`
3. `prettier/prettier` er koncentreret i:
- `src/components/OnboardingWindow.vue` (1356)
- `src/utils/markdown.ts` (148)
4. `npm run lint:rust` fejler paa Clippy doc comment-regel i:
- `src-tauri/src/utils/time.rs`
- `src-tauri/src/utils/validation.rs`
5. `cd src-tauri && cargo test`:
- unit tests passerer
- doctest fejler i `src-tauri/src/utils/time.rs`
- `test_rate_limiter` tager ca. 60 sekunder
6. `npm run build:web` passerer.

## 3. Prioriteret roadmap

## P0 - Stabilisering af kvalitetspipeline

### P0.1 Normaliser line endings/format
- **Problem:** Stor CI-stoej fra formatfejl.
- **Paavirkning:** Skjuler reelle fejl og giver langsommere review-loop.
- **Filreferencer:** `src/components/OnboardingWindow.vue`, `src/utils/markdown.ts`
- **Anbefalet losning:** Koer kontrolleret format-normalisering (LF + Prettier) paa disse hotspots foerst.
- **Estimeret indsats:** 0.5 dag
- **Test/acceptkriterie:** `prettier/prettier` for disse filer er 0; samlet `npm run check` fejlvolumen falder markant.

### P0.2 Ret Clippy doc comment-fejl
- **Problem:** Rust lint blokeres af doc comment-layout.
- **Paavirkning:** `lint:rust` er ikke groen.
- **Filreferencer:** `src-tauri/src/utils/time.rs`, `src-tauri/src/utils/validation.rs`
- **Anbefalet losning:** Brug module-level docs (`//!`) eller fjern tom linje efter doc comments i henhold til Clippy.
- **Estimeret indsats:** 0.5 dag
- **Test/acceptkriterie:** `npm run lint:rust` passerer.

### P0.3 Ret doctest i tidsmodul
- **Problem:** `cargo test` fejler paa doctest-eksempel.
- **Paavirkning:** Testsuite er ikke groen end-to-end.
- **Filreferencer:** `src-tauri/src/utils/time.rs`
- **Anbefalet losning:** Giv korrekt namespace/import i doctest eller marker eksempel som `ignore` hvis runtime-kontekst kraeves.
- **Estimeret indsats:** 0.25 dag
- **Test/acceptkriterie:** `cd src-tauri && cargo test` passerer inkl. doctests.

### P0.4 Indfoer samlet logging-strategi
- **Problem:** Massiv ad-hoc logging i frontend/backend.
- **Paavirkning:** Stoej i logs, svaerere fejlsogning, risiko for utilsigtet dataeksponering.
- **Filreferencer (hoejeste koncentration):**
- `src/components/PopupWindow.vue`
- `src/components/ChatWindow.vue`
- `src/components/OperationEditWindow.vue`
- `src/components/ChatHistoryWindow.vue`
- `src/components/SettingsWindow.vue`
- `src-tauri/src/shortcut_manager.rs`
- `src-tauri/src/window_manager.rs`
- `src-tauri/src/commands/ai_commands.rs`
- **Anbefalet losning:** Indfoer central logger i frontend og standardiser Rust logging-kontrakt (`log`/`tracing`), med miljoestyret log-level.
- **Estimeret indsats:** 1-2 dage
- **Test/acceptkriterie:** `no-console` reduceres til aftalt niveau; ingen direkte `println!/eprintln!` i forretningskritiske moduler.

## P1 - Robusthed og vedligeholdbarhed

### P1.1 Erstat blokerende browser-dialoger
- **Problem:** `prompt/alert/confirm` giver blokerende og inkonsistent UX.
- **Paavirkning:** Svag brugeroplevelse og svaerere state-kontrol.
- **Filreferencer:** `src/components/ChatWindow.vue`, `src/components/ChatHistoryWindow.vue`, `src/components/OperationEditWindow.vue`
- **Anbefalet losning:** Erstat med ikke-blokerende, komponent-baserede dialoger/toasts.
- **Estimeret indsats:** 1-2 dage
- **Test/acceptkriterie:** Ingen direkte `prompt/alert/confirm` i disse komponenter; flows virker med tastatur/mus.

### P1.2 Fjern polling i onboarding
- **Problem:** `setInterval` bruges til validering.
- **Paavirkning:** Unodig polling og mindre forudsigelig state-flow.
- **Filreferencer:** `src/components/OnboardingWindow.vue`
- **Anbefalet losning:** Brug `watch` paa relevante felter eller event-drevet validering.
- **Estimeret indsats:** 0.5 dag
- **Test/acceptkriterie:** Ingen `setInterval` i onboarding-flow; validering opdateres korrekt ved input.

### P1.3 Undgaa muterende `sort()` i computed
- **Problem:** `sort()` muterer arrays i computed-kontekst.
- **Paavirkning:** Risiko for sideeffekter og svaere bugtracking.
- **Filreferencer:** `src/components/ChatHistoryWindow.vue`
- **Anbefalet losning:** Sorter paa kopi (`[...arr].sort(...)`) i computed.
- **Estimeret indsats:** 0.25 dag
- **Test/acceptkriterie:** Operation-sortering muterer ikke source state utilsigtet.

### P1.4 Fjern resterende `any`
- **Problem:** To kendte `any`-forekomster.
- **Paavirkning:** Svagere typegarantier.
- **Filreferencer:** `src/components/PopupWindow.vue`, `src/vite-env.d.ts`
- **Anbefalet losning:** Brug konkrete types (`Operation`) og strictere Vue component typing.
- **Estimeret indsats:** 0.5 dag
- **Test/acceptkriterie:** `@typescript-eslint/no-explicit-any` = 0.

### P1.5 Reducer `unwrap/expect` i kritiske Rust-moduler
- **Problem:** Panik-risiko i runtime-kritiske paths.
- **Paavirkning:** Potentielle hard crashes.
- **Filreferencer:** `src-tauri/src/lib.rs`, `src-tauri/src/tray_manager.rs`, `src-tauri/src/shortcut_manager.rs`
- **Anbefalet losning:** Konverter til fejlhaandtering med fallback/logging.
- **Estimeret indsats:** 1 dag
- **Test/acceptkriterie:** Farlige `unwrap/expect` i kritiske paths erstattet eller dokumenteret med sikker begrundelse.

### P1.6 Saml DataManager init-moenster
- **Problem:** Gentaget init i mange kommandoer.
- **Paavirkning:** Duplikering og vedligeholdelsesomkostning.
- **Filreferencer:** `src-tauri/src/data_manager/commands.rs`
- **Anbefalet losning:** Indfoer intern helper, analogt med `load_data_manager` i `ai_commands`.
- **Estimeret indsats:** 0.5-1 dag
- **Test/acceptkriterie:** Duplikeret init-kode reduceret; adfaerd uaendret.

## P2 - Arkitektur- og sikkerhedshardening

### P2.1 Security-konfiguration og capability-scope
- **Problem:** `csp: null` og bred capability-mapping.
- **Paavirkning:** Stoerre angrebsflade end noedvendigt.
- **Filreferencer:** `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- **Anbefalet losning:** Definer minimum-CSP og smal permissions pr. vinduestype.
- **Estimeret indsats:** 1-2 dage (inkl. test)
- **Test/acceptkriterie:** App fungerer med strammere security-profiler uden regression.

### P2.2 Konsolider bootstrap i `windows/*.html`
- **Problem:** Duplikeret inline bootstrap og API-inkonsistens.
- **Paavirkning:** Hoeyere fejlrate og svaerere vedligehold.
- **Filreferencer:** `windows/popup.html`, `windows/onboarding.html`, `windows/settings.html`, `windows/operation-edit.html`, `windows/chat.html`, `windows/history.html`
- **Anbefalet losning:** Flyt delt bootstrap til TS-moduler med ens API-brug.
- **Estimeret indsats:** 1-2 dage
- **Test/acceptkriterie:** Samme vinduesfunktionalitet med mindre duplikering og ens luk/fokus-flow.

## 4. Vigtige interface/type-aendringer (foreslaaet)

1. **Frontend logger API:** ny `src/utils/logger.ts` med `debug/info/warn/error`.
2. **Emit typing i popup:** skift fra `details: any` til `details: Operation` i `src/components/PopupWindow.vue`.
3. **Vue module declaration:** opdater `src/vite-env.d.ts` for at undgaa `any`.
4. **Rust logging-kontrakt:** standardiser paa `log` eller `tracing` i stedet for `println!/eprintln!`.
5. **Capability-scope:** introducer vinduesspecifikke capability-profiler i `src-tauri/capabilities/default.json`.

## 5. Quick wins (foerste 48 timer)

1. Ret format/line endings i `src/components/OnboardingWindow.vue` og `src/utils/markdown.ts`.
2. Ret Clippy doc comments i `src-tauri/src/utils/time.rs` og `src-tauri/src/utils/validation.rs`.
3. Ret doctest i `src-tauri/src/utils/time.rs`.
4. Start logging-oprydning i top-2 frontend hotspots og top-1 backend hotspot.

Forventet effekt: markant reduktion i lint-stoej og groen Rust-pipeline.

## 6. Naeste sprint (strukturelle forbedringer)

1. Erstat blokerende dialog-flow.
2. Fjern polling i onboarding.
3. Udfas `any` og reducer `unwrap/expect`.
4. Saml DataManager-init i kommando-lag.
5. Paabegynd security-hardening (CSP + capability-scope).

## 7. Risiko og kompatibilitet

- **Lav risiko:** format/lint/doctest-oprydning.
- **Mellem risiko:** logger-standardisering (kan paavirke debuggingflows).
- **Mellem-hoej risiko:** CSP/capability-aendringer kan paavirke runtime-permissions.
- **Kompatibilitet:** funktionel adfaerd skal bevares; alle aendringer valideres via eksisterende build/lint/test samt manuel vinduesflow-verifikation.

## 8. Testmatrix og scenarier

1. `npm run check` skal ende uden fejl.
2. `npm run lint:rust` skal vaere groen.
3. `cd src-tauri && cargo test` skal vaere groen inkl. doctests.
4. Verificer markdown-rendering i:
- `src/components/MessageBubble.vue`
- `src/components/ChatHistoryWindow.vue`
5. Verificer vinduesflows via:
- `windows/popup.html`
- `windows/chat.html`
- `windows/onboarding.html`
6. Verificer at operation-sortering i historik ikke muterer source state utilsigtet.

## 9. Definition of done

Rapportens roadmap betragtes som implementeret, naar:

1. Kvalitetspipeline er groen (`check`, `lint:rust`, `cargo test`).
2. `prettier/prettier`-stoej er elimineret.
3. `no-console` er reduceret til aftalt niveau med central logger.
4. Resterende `any` i identificerede filer er fjernet.
5. Kritiske `unwrap/expect` er reduceret eller begrundet.
6. Dialog/polling/muterende sort-problemer er lukket uden regressions.
7. Security-konfiguration er dokumenteret og strammet med valideret funktionalitet.

## 10. Implementeringsraekkefoelge for dokumentationsarbejdet (afsluttet)

1. Baseline og maal skrevet i ny rapport.
2. Prioriteret backlog (`P0` -> `P2`) indsat med acceptance criteria.
3. Afsnit om offentlige interfaces/types tilfoejet.
4. Testmatrix og definition of done tilfoejet.
5. Dokumentationsindex opdateret med link.
