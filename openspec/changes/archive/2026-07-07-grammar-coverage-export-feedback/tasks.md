# Implementation Plan: Grammar coverage + export feedback (audit P1a/P1b/P2a/P2b)

## Overview

Four audit backlog items in one cohesive change: extend the syntect grammar set via two-face (P1a), disclose the export backend in the status bar and make the pandoc PDF engine configurable (P1b), make the advertised language list honest (P2b), and run + record the HTML/LaTeX exporter comparison (P2a, docs-only outcome).

## Tasks

- [x] 1. P1a — extended grammar set
  - [x] 1.1 `LanguageRegistry::with_syntax_set(SyntaxSet)` in `crates/markdown` (additive; `new()` delegates; test).
  - [x] 1.2 Add `two-face` (workspace + root); Markion registry built from `two_face::syntax::extra_newlines()`.
  - [x] 1.3 Probe actual coverage after the swap; retarget the fallback test if its probe language became covered; add an extended-coverage test (typescript/toml via syntect path).
  - [x] 1.4 Re-measure registry warm-up time (existing debug event).

- [x] 2. P1b — export backend feedback + configurable PDF engine
  - [x] 2.1 `PdfExporter::with_pdf_engine` in `crates/export` (additive; test).
  - [x] 2.2 `ExportBackend` + `ExportSettings { pdf_engine }` in model; `export_to` returns the backend; `export_to_with` accepts settings.
  - [x] 2.3 `[export] pdf_engine` in `config.toml` (DTO + AppPreferences.export + main.rs round-trip, config-file only).
  - [x] 2.4 i18n: engine/built-in status variants (en + zh) wired at the export call site in main.rs.

- [x] 3. P2b — honest advertised list
  - [x] 3.1 `supported_highlight_languages()` = sorted deduped union of legacy identifiers + registry names (lowercased), lazily built, signature unchanged.

- [x] 4. P2a — HTML/LaTeX comparison (docs-only)
  - [x] 4.1 Render a representative fixture (front matter, tables, task list, footnote, highlight, math, code) through Markion native and absorbed `html.rs`/`latex.rs`; compare fidelity/self-containment.
  - [x] 4.2 Record decision in `docs/typune-integration-audit.md` and the integration plan.

- [x] 5. Verification
  - [x] 5.1 `cargo test --workspace` fully green.
  - [x] 5.2 Update audit doc P-item statuses; plan doc note.
