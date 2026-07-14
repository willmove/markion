## Why

Phase 1 of `docs/typune-integration-plan.md`: absorb Typune's `export` crate. Markion's PDF export is a deliberately limited single-page ASCII text dump (`src/export.rs`), and its DOCX is a hand-written minimal ZIP. Typune's export crate (~4000 lines, 219 tests, verified green) provides substantially better output for these formats.

**Scope correction discovered during implementation research:** the integration plan assumed Typune's export engine was uniformly stronger. Close reading shows its PDF/DOCX/Image exporters are **subprocess wrappers** — pandoc (+xelatex) for PDF/DOCX, wkhtmltoimage for images — while only its HTML/LaTeX exporters are native Rust. External tools cannot be assumed on end-user machines, so wholesale replacement would regress users without pandoc from "poor export" to "no export". The scope is therefore:

- **PDF and DOCX: engine-first with silent fallback.** Try the Typune exporter (pandoc); on any error (pandoc missing, xelatex missing, conversion failure) fall back to Markion's existing built-in implementation. Users with pandoc get dramatically better output; users without lose nothing.
- **HTML, LaTeX, Markdown, PNG/JPEG: unchanged.** Markion's native HTML/LaTeX renderers are full-fidelity, deeply integrated (math metadata, front matter, highlight CSS) and heavily tested; Typune's native equivalents produce different output for no clear gain — switching them is deferred to a later evaluation (plan §Phase 5 theme/HTML work). Typune's image exporter needs wkhtmltoimage (abandoned upstream) and is not wired.

## What Changes

- **`crates/markdown`, `crates/export` (new).** Copied verbatim from Typune @ `0b9e313` (including their test suites, ~315 tests) — only the crate manifests change, switching `.workspace = true` references to Markion's root workspace tables. `export` depends on `markdown` (its input type is `markdown::Document`), so both crates arrive together. The `markdown` crate also carries the syntect highlighting and incremental parsing that later phases (2 and 4) will wire up; nothing in Markion consumes them yet.
- **Root `Cargo.toml`.** Add `[workspace.package]` and `[workspace.dependencies]` tables to satisfy the copied crates' workspace inheritance (pulldown-cmark 0.11 for members coexists with the root package's 0.13 — intended, see plan §6). Add `[profile.dev.package.markdown]`/`[profile.dev.package.export]` opt-level overrides per the crate-architecture spec. Add `markdown`/`export` as root-package dependencies. Remove `crates/.gitkeep` (the glob now matches real members).
- **`src/export.rs`.** New adapter: parse the document source with `markdown::Parser`, run the Typune `PdfExporter`/`DocxExporter`, return bytes; callers fall back to the existing `write_pdf`/`write_docx` on any engine error. No UI, keybinding, prompt-flow, or i18n changes — same menu actions, same output paths, byte-level output differs only when pandoc is available.
- **Non-goals:** no HTML/LaTeX/image switch; no use of `markdown` crate types anywhere outside the export adapter (Markion's preview model stays authoritative); no removal of the built-in PDF/DOCX writers (they are the fallback); no new user-facing settings.

## Capabilities

### Modified Capabilities
- `export`: PDF and DOCX exports become engine-first (pandoc via the absorbed Typune export crate) with automatic silent fallback to the built-in simple implementations when the external tool is unavailable or fails.

## Impact

- **New dirs:** `crates/markdown/`, `crates/export/` (from Typune @ `0b9e313`, manifests adapted). **Edited:** `Cargo.toml` (workspace tables, member deps, profile overrides), `src/export.rs` (adapter), `src/lib.rs` (`export_to` Pdf/Docx arms), `docs/typune-integration-plan.md` (Phase 1 status + pandoc correction). **Deleted:** `crates/.gitkeep`.
- **Tests:** workspace `cargo test` now includes the copied crates' suites (~315 tests) alongside Markion's 108. Export behavior tests must pass both with and without pandoc on PATH (the fallback keeps `saves_and_exports_all_formats` green either way).
- **Build cost:** first build compiles syntect/tree-sitter/tokio for the new members; release binary grows accordingly. Typing-path invariants untouched (export runs on demand, not per keystroke; dev-profile overrides added per crate-architecture spec).
- **Release pipeline:** root package still the packaging target; new path deps build into the same binary. pandoc is a soft runtime dependency — never required, only used when present.
