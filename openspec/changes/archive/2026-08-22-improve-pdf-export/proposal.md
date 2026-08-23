# improve-pdf-export

## Why

PDF export is Markion's weakest export path. The pandoc engine path requires a multi-GB XeLaTeX toolchain almost no user has, emits no CJK font configuration (Chinese glyphs go missing), drops relative-path images (no `--resource-path`), and passes a dead `--katex` flag. The built-in fallback (`write_pdf` in `src/export.rs`) replaces every non-ASCII character with `?` (Chinese documents export as literal `???`), silently truncates the document to 52 lines on a single page, and flattens all structure to plain text. The export spec itself still defines PDF as "deliberately limited (a simple single-page text PDF)". Mainstream editors (Typora, Obsidian, MarkText, VS Code Markdown PDF) all ship paginated, font-embedded, WYSIWYG-grade PDF export; Markion needs a built-in path of comparable class that works with zero external tools.

## What Changes

- **New built-in PDF writer: a self-built layout engine** in a new GPUI-free workspace member crate — `cosmic-text` (fontdb system-font discovery, harfrust shaping, UAX#14 line breaking, bidi) for text layout, `krilla` (font subsetting/embedding, images, vector fills, link annotations, document outline, metadata) for PDF emission. Chosen over embedding or bundling Typst: roughly +2–5 MB instead of +20–40 MB of binary, no ~1M-SLoC dependency tree, and better preview consistency (see math/code below). Pagination, block layout, and tables are our code.
- **Preview-consistent rendering**: math formulas are exported through the same GPUI-free SVG math renderer used by native preview (embedded as vector SVG via `krilla-svg`), and code blocks reuse the editor's memoized syntax-highlight colors — exported PDFs look like the preview, not like a separate toolchain's idea of the document.
- **Real document structure**: multi-page pagination, CJK-aware line wrapping, headings/lists/tables/code/quote/alert/rule blocks, inline styling, clickable links, PDF outline bookmarks from headings, PDF metadata from front matter, local image embedding, and page footnotes — rendered from the cached `preview_blocks()` like the built-in DOCX writer.
- **CJK font strategy**: bundle an OFL-licensed Noto Sans SC subset plus a Latin serif/sans/mono set as guaranteed fallbacks, with runtime system-font discovery (Microsoft YaHei / PingFang SC / Noto Sans CJK) ordered ahead of them; krilla subsets and embeds fonts per document.
- **User-facing PDF options**: page size (A4/Letter/Legal), margins, table of contents, and page numbering, persisted via a new `[export.pdf]` config section; honored by the built-in writer (and mapped to pandoc variables on the engine path).
- **Pandoc PDF engine fixes**: emit a CJK-aware font setup (`CJKmainfont` or configurable `mainfont`), pass `--resource-path` so relative images resolve, remove the inert `--katex` flag, and honor the PDF page-size option.
- **Export flow/status updates**: the built-in writer is no longer a degraded last resort, so the "installing pandoc yields richer output" hint is reworked for PDF; the pandoc engine remains an optional alternative backend.
- **Remove** the hand-rolled single-page `write_pdf` writer and the `plain_text_preview()` PDF input path.
- **Non-goals**: a graphical PDF export options dialog (follow-up change; this change is config-driven), PNG/JPEG snapshot fidelity, remote-image fetching, graphical diagram rendering in PDF (fences keep code-block behavior; the SVG-embed mechanism makes this an easy follow-up), Chromium-class HTML-fidelity printing, and advanced CJK book typography (vertical writing, punctuation hanging).

## Capabilities

### New Capabilities

- none

### Modified Capabilities

- `export`: the multi-format requirement changes (PDF is no longer a deliberately-limited single-page text dump; built-in fallback semantics and status-bar disclosure are reworked); new requirements are added for the built-in PDF writer (pagination, fonts/CJK, inline/block fidelity, tables, images, math, footnotes, metadata/bookmarks, user options) and for pandoc PDF engine font/resource handling.

## Impact

- **New workspace member crate** (`crates/pdf`, package `markion-pdf`): depends on `krilla`, `krilla-svg`, `cosmic-text` (pulling `fontdb` + `harfrust`) — all pure Rust, GPUI-free per the workspace invariant. Root `Cargo.toml` gains the member and (because `[profile.dev.package."*"]` does not cover members) an explicit dev-profile override for it. Binary grows by only a few MB; the real cost is engineering effort — pagination, tables, and footnotes are our code (see design risks).
- **Root crate**: `src/export.rs` gains a `preview_blocks()` → layout-IR renderer alongside the DOCX renderer (pure data, no markup-string escaping surface); `write_pdf`, `plain_pdf_text`, `wrap_text` and the PDF use of `plain_text_preview()` are removed; `src/lib.rs` `export_to_with` rewires the PDF branch; `src/model.rs` gains `PdfExportOptions` under `ExportPreferences`; `src/i18n.rs` gains/reworks export status strings (both locales). The memoized syntax highlighter and the GPUI-free math SVG renderer are reused, not duplicated.
- **typune-export crate** (`crates/export/src/pdf.rs`): pandoc args gain CJK font variables and `--resource-path`; `--katex` removed.
- **Config**: additive `[export.pdf]` section; `[export] pdf_engine` unchanged.
- **Assets**: bundled font subsets (Noto Sans SC + Liberation Serif/Sans/Mono, all OFL) embedded in the member crate via `include_bytes!` with license texts — no installer/packager changes needed.
- **Docs**: `docs/faq.md` and release notes describe the new built-in PDF engine and its options.
