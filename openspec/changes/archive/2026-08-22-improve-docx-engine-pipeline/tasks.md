## 1. Engine options and pandoc arguments

- [x] 1.1 Extend `ExportOptions` (`crates/export/src/engine.rs:63-92`) with `reference_doc: Option<PathBuf>`, `resource_path: Option<PathBuf>`, `toc: bool`, and `pandoc_path` already exists on the exporter — confirm and wire
- [x] 1.2 In `DocxExporter` (`crates/export/src/docx.rs:47-146`), add arguments: `--reference-doc=<path>` when set, `--resource-path=<dir>` when set, `--from=markdown+mark+superscript+subscript` (replacing plain `markdown`), `--highlight-style=<choice>`, `--toc` when requested; keep `-V papersize=`
- [x] 1.3 Extend the argument-construction unit tests in `crates/export/src/docx.rs` for each new flag, including extension list and option combinations

## 2. Bundled reference template

- [x] 2.1 Author `assets/templates/reference.docx` with CJK-friendly styles (generate once via pandoc's default reference doc, then restyle: eastAsia fonts, heading sizes, code style); document the regeneration steps in `docs/release-process.md` or a short note beside the template
- [x] 2.2 Wire the template into the packaged app (check `build.rs`/`packager.toml` asset handling) and resolve it at runtime with a dev-mode fallback path; test that the bundled path resolves in dev builds

## 3. Config plumbing and failure disclosure

- [x] 3.1 Add `pandoc_path` and `reference_doc` keys to the `[export]` config section (`src/model.rs:266-278`), parsed alongside `pdf_engine`; thread both plus the document directory into `ExportOptions` in `src/export.rs:35-53` (`DocxExporter::with_pandoc_path` finally gets a caller)
- [x] 3.2 Propagate the engine failure category (missing binary vs. conversion error) out of `engine_docx` instead of flattening to `None`; extend the status message in `src/app/documents.rs:685-693` with the category and add i18n strings (en + zh) in `src/i18n.rs`
- [x] 3.3 Tests: config parsing for the new keys; failure-category mapping; status-message selection

## 4. Verification

- [x] 4.1 `cargo test` (root) and `cargo test --workspace` pass; build warning-free under `-D warnings`
- [x] 4.2 Manual check with pandoc installed: relative images embed, headings use the reference-doc styles, `==mark==`/`^sup^`/`~sub~` survive; rename the pandoc binary to confirm the missing-binary disclosure — verified 2026-08-22 with pandoc 3.7.0.2: engine export embeds relative images (`word/media/`), `==mark==`→`w:highlight`, `^sup^`/`~sub~`→`w:vertAlign`, math→OMML, `--toc` emits a TOC with the bundled reference doc; bogus pandoc path discloses BinaryMissing in the status message. The smoke exposed an invalid hand-rolled theme in `assets/templates/reference.docx` (Word rejected the output as corrupt); the template now embeds pandoc's Office theme verbatim
- [x] 4.3 `openspec validate improve-docx-engine-pipeline` passes
