# Implementation Plan: Absorb Typune export engine (Typune integration Phase 1)

## Overview

Copy Typune's `markdown` + `export` crates in verbatim as workspace members, then wire an engine-first-with-fallback path for PDF and DOCX export. HTML/LaTeX/image stay on Markion's native renderers (scope correction: Typune's PDF/DOCX/image exporters are pandoc/wkhtmltoimage subprocess wrappers, not native Rust — see proposal).

## Tasks

- [x] 1. Workspace plumbing (root `Cargo.toml`)
  - [x] 1.1 Add `[workspace.package]` (version, edition 2021 for members) and `[workspace.dependencies]` covering every `.workspace = true` reference in the copied crates (pulldown-cmark 0.11/simd, tree-sitter, syntect, serde, serde_json, serde_yaml, anyhow, thiserror, tracing, regex, tokio, proptest).
  - [x] 1.2 Add `[profile.dev.package.markdown]` and `[profile.dev.package.export]` opt-level 2 overrides (crate-architecture spec requirement).
  - [x] 1.3 Add the member path dependencies to the root package, renamed to `typune-markdown`/`typune-export` (markion has a crate-root `mod export`; a dependency also named `export` would make `use export::...` ambiguous, E0659); delete `crates/.gitkeep`.

- [x] 2. Copy crates from Typune @ `0b9e313`
  - [x] 2.1 Copy `crates/markdown` and `crates/export` verbatim (sources + tests); adapt only their `Cargo.toml` workspace-inheritance references.
  - [x] 2.2 `cargo test -p markdown -p export` green inside the Markion workspace (~315 tests).

- [x] 3. Engine adapter (`src/export.rs`, `src/lib.rs`)
  - [x] 3.1 Adapter function: source text → `markdown::Parser` → `Document` → `PdfExporter`/`DocxExporter` → bytes; any `Err` → `None` (fallback signal).
  - [x] 3.2 `export_to` Pdf/Docx arms: try adapter first, write bytes on success, else run the existing built-in writers unchanged.
  - [x] 3.3 Verify end-to-end on this machine (pandoc installed, xelatex not): DOCX via `export_to` is a real pandoc product (10.4 KB deflate package vs 3.9 KB built-in); PDF correctly falls back (no xelatex here) — pandoc's PDF-to-stdout pipe protocol verified separately with `--pdf-engine=pdfroff` (real 7.4 KB PDF); with pandoc removed from PATH both formats fall back to built-ins. Residual: the exporter hardcodes `--pdf-engine=xelatex`, so the engine PDF path with xelatex present is exercised by the copied crate's own tests, not end-to-end here.

- [x] 4. Verify and finalize
  - [x] 4.1 Full workspace `cargo test` green (Markion 108 + copied ~315).
  - [x] 4.2 `cargo check` / `cargo build` at root unaffected; release pipeline untouched by construction.
  - [x] 4.3 Update `docs/typune-integration-plan.md`: mark Phase 1 implemented; correct the "Typune export engine uniformly stronger" assessment with the pandoc-wrapper discovery; note HTML/LaTeX switch deferred.
