## 1. Options model and persistence

- [x] 1.1 Add a `DocxExportOptions` model (page size, toc, image policy) to `src/model.rs`; parse/persist a `[export.docx]` config section alongside the existing `[export]` keys (`src/storage/`)
- [x] 1.2 Map the options onto `ExportOptions` fields (`crates/export/src/engine.rs`); extend `PageSize::Custom` handling or remove it if unused
- [x] 1.3 Tests: config round-trip, defaults, mapping to `ExportOptions`

## 2. Options UI

- [x] 2.1 Extend the DOCX export flow (`src/app/documents.rs:622-693`) with an options step following existing in-app dialog patterns: page-size choice, TOC toggle (engine path only, disabled otherwise), image policy toggle
- [x] 2.2 i18n strings (en + zh) in `src/i18n.rs`; persist last-used options after a successful export
- [x] 2.3 Thread the options into both backends (`src/export.rs`): fallback writer uses page-size/margin constants; image policy gates the embedding step

## 3. Test hardening and docs

- [x] 3.1 Add a package-structure validation test helper (part inventory + style-reference resolution) shared by the fallback tests
- [x] 3.2 Add an engine/fallback consistency smoke test: one fixture document, both paths, assert openable packages (PK magic + required parts); engine side gated on pandoc availability like the existing `#[ignore]` test
- [x] 3.3 Update `docs/faq.md` DOCX section with the new options and the reference-doc/pandoc config keys
- [x] 3.4 `cargo test` (root) and `cargo test --workspace` pass; build warning-free under `-D warnings`
- [x] 3.5 `openspec validate add-docx-export-options` passes
