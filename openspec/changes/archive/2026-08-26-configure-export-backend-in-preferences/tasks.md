## Tasks

### 1. Model and persistence

- [x] 1.1 Add `ExportBackendPreference` (`BuiltIn` default, `Pandoc`) with `config_value`/`from_config` tokens `builtin`/`pandoc` in `src/model.rs`, and add `backend: ExportBackendPreference` to `ExportPreferences` defaulting to `BuiltIn`
- [x] 1.2 Persist `backend` as the `[export] backend` string in `src/storage/preferences.rs` (serde default `builtin`, unknown tokens fall back to `builtin`) with save/parse round-trip coverage

### 2. Export orchestration

- [x] 2.1 Branch `export_to_with` in `src/lib.rs` on the backend preference: `builtin` writes PDF/DOCX through the built-in writers directly with a neutral built-in outcome; `pandoc` keeps the engine-first flow with the silent built-in fallback
- [x] 2.2 Simplify `backend_status_msg` in `src/export.rs` to `(backend, engine_failure)`, route explicit-built-in exports to the neutral message, and remove the now-unreachable `Msg::StatusExportedDocxBuiltin` (update its unit test)
- [x] 2.3 Add model/export tests: default backend is built-in and no engine failure is reported; `pandoc` preference with a missing binary still exports via the fallback with `BinaryMissing`

### 3. Preferences panel Export tab

- [x] 3.1 Add `PreferencesTab::Export`, wire the tab strip button and `select_preferences_tab`, and render a scrollable Export body with the four sections
- [x] 3.2 Implement the backend choice row and the background pandoc-availability probe cached in app state (refresh on tab open, backend switch to Pandoc, and pandoc-path change)
- [x] 3.3 Implement the pandoc-only rows: binary path and reference template with native *Browse…* pickers (rfd, parented like the save dialogs) plus *Reset* actions, and the PDF-engine choice buttons
- [x] 3.4 Implement the Word (DOCX) section (page size, engine-path TOC, image policy) and the PDF section (page size, margin stepper, TOC, page numbers) as immediate-apply persisted setters on `MarkionApp`

### 4. Dialog removal

- [x] 4.1 Replace the `export_docx` options-dialog flow with a direct `export_with_prompt` call; delete `page_size_label`, the menu-path `pandoc_available` call, and the persist-after-success special case in `export_with_prompt`
- [x] 4.2 Remove the `DialogDocx*` message variants from all seven language functions and from the exhaustive i18n test list; prune now-unused imports in `src/app/documents.rs`

### 5. Localization

- [x] 5.1 Add Msg variants and translations for every new Export-tab string in En/Ja/Fr/De/Es/ZhHans/ZhHant, and extend the `every_message_returns_non_empty_text_for_every_language` list

### 6. Verification

- [x] 6.1 Run `cargo test --workspace` and fix all failures
- [x] 6.2 Run `openspec validate configure-export-backend-in-preferences` and confirm the change is spec-consistent
