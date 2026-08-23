# Tasks — improve-pdf-export

## 1. Pandoc engine path fixes (quick stop-bleed, lands first)

- [x] 1.1 In `crates/export/src/pdf.rs`, remove the inert `--katex` flag from `build_pandoc_args` and update the affected unit tests.
- [x] 1.2 Add CJK font support to the pandoc invocation: detect CJK content in the prepared input and append `-V CJKmainfont=<platform default>` (Microsoft YaHei / PingFang SC / Noto Sans CJK SC), with optional `[export]` config overrides for the font names; unit-test the arg builder for CJK vs non-CJK input.
- [x] 1.3 Thread the document directory into `engine_pdf` (currently only `engine_docx` receives it) and pass `--resource-path <document dir>`; extend `ExportOptions` usage accordingly and unit-test.
- [x] 1.4 Map the configured PDF page size and TOC flag (from the new `[export.pdf]` options, see 3.1) onto the pandoc geometry variable and `--toc` instead of the hardcoded A4 default in `engine_pdf`.

## 2. New member crate `markion-pdf` (self-built layout engine)

- [x] 2.1 Create `crates/pdf` (`markion-pdf`, GPUI-free per the workspace invariant), register it in the root `Cargo.toml` workspace members, and add `[profile.dev.package.markion-pdf] opt-level = 2` (the wildcard dev profile does not cover members). Pin exact versions of `krilla`, `krilla-svg`, and `cosmic-text`.
- [x] 2.2 API spike: verify krilla's glyph-run emission, link annotations, document outline, and metadata APIs at the pinned versions, and confirm `krilla-svg` renders the preview math renderer's SVG output faithfully; record outcomes against design.md open questions 1–2.
- [x] 2.3 Font subsystem: one `fontdb::Database` behind a `OnceLock` (system scan) plus bundled Liberation Serif/Sans/Mono and a subset Noto Sans SC (OFL, common-use Han + punctuation, ≈1–2 MB) via `include_bytes!`; ship the OFL license texts in the crate; test that Chinese text resolves to a CJK-capable face even when no system CJK font is present.
- [x] 2.4 Text layout: convert IR styled runs into per-paragraph cosmic-text `Buffer`s (attrs for weight/style/family/color, background for highlight, baseline offset for super/subscript, underline+color for links) with UAX#14 wrapping; unit-test spaceless Chinese wrapping and mixed CJK/Latin runs.
- [x] 2.5 Block layout and pagination: page model from options (size, margins, page-number footer), keep-with-next headings, generated list markers and nesting indents, quote/alert accent blocks, code-block keep-together with line-boundary splitting, graphical rules; multi-page fixture tests.
- [x] 2.6 Tables: column widths from separator-row proportions fitted to the text column, bold header row repeated on page continuation, per-column alignment, splitting only between rows.
- [x] 2.7 Footnotes: superscript references linked to a per-page note area at the page bottom with carry-forward overflow; fixture test with multiple notes on one page. If the overflow algorithm proves unstable, fall back to an endnotes section and amend the spec scenario before archive (design risk note).
- [x] 2.8 Emission: shaped glyph runs → krilla, PNG/JPEG images and SVG via krilla-svg (scaled to the text column), vector strokes/fills for rules/borders/alert accents, link annotations per link run, heading outline bookmarks, front-matter metadata into document properties.
- [x] 2.9 Public `render(ir, options) -> Result<Vec<u8>, PdfError>` entry point plus a crate-level integration test asserting `%PDF` bytes for a representative IR.

## 3. Root-crate options model and layout-IR builder

- [x] 3.1 Add `PdfExportOptions { page_size, margin_mm, toc, page_numbers }` to `src/model.rs` under `ExportPreferences`, persisted via a new `[export.pdf]` config section with defaults and unknown-value tolerance (mirroring `DocxExportOptions` parsing), with storage tests.
- [x] 3.2 Implement `build_pdf_ir(document, options)` in `src/export.rs` walking the cached `preview_blocks_shared()` (no re-parse, preserving the per-version caching invariant), covering headings, paragraphs, lists (bullet/ordered/task, nested), quotes and GFM alerts, code blocks, tables and raw-HTML table grids, rules, images, footnotes, and front-matter title.
- [x] 3.3 Convert inline spans into IR styled runs: bold/italic/strikethrough/highlight/superscript/subscript/inline-code composition and links with targets; pure-data unit tests (the IR is structured data — no markup escaping surface).
- [x] 3.4 Implement local image resolution for the IR (PNG/JPEG/SVG, percent-decoded paths resolved against the document directory) with the `alt: url` text fallback for remote/data-URI/missing images — mirroring the DOCX policy.
- [x] 3.5 Route math spans through the same GPUI-free SVG math renderer used by preview/HTML export into IR vector images; on renderer failure emit the byte-identical authored LaTeX as a code-styled block.
- [x] 3.6 Reuse the editor's memoized syntax-highlight colors for code-block IR run colors so exported code matches the preview palette (light-theme variant for print).
- [x] 3.7 Wire the PDF branch of `export_to_with` in `src/lib.rs`: pandoc attempt (fixed args) → built-in writer; delete `write_pdf`, `plain_pdf_text`, `wrap_text` and the PDF use of `plain_text_preview()`; keep the `lib.rs:3411`-style export integration test passing without pandoc.

## 4. Status, i18n, and disclosure

- [x] 4.1 Rework `backend_status_msg` and the `Msg` variants in `src/i18n.rs` (both locales): built-in PDF disclosure no longer hints that pandoc is richer; DOCX keeps its hint; the engine-failure category disclosure is retained for both formats.
- [x] 4.2 Update the status-bar/export UI wiring in `src/app/documents.rs` if message signatures changed; keep the save-dialog PDF flow unchanged.

## 5. Verification, docs, and cleanup

- [x] 5.1 Golden-fixture tests in the root crate: a mixed CJK/Latin fixture with headings, lists, table, code, math, footnotes, and an image exports to a multi-page PDF; assert bytes start with `%PDF`, page count > 1, CJK glyphs render, and no `?`-substitution regression (guarding the deleted `plain_pdf_text` behavior).
- [x] 5.2 Run `cargo test --workspace`; manually verify exported PDFs on Windows (Microsoft YaHei path) and spot-check macOS/Linux font resolution; check outline bookmarks, links, TOC, page numbers, and footnotes in a viewer.
- [x] 5.3 Update `docs/faq.md` and user docs for the new built-in PDF engine and `[export.pdf]` options; add the bundled-font license notes to release-facing docs.
- [x] 5.4 At archive time, also update the `export` capability Purpose sentence that describes PDF as "deliberately limited (a simple single-page text PDF)".
