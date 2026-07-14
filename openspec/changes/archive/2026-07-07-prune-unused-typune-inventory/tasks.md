# Implementation Plan: Prune unused Typune inventory (Phase 5)

## Overview

Delete every absorbed-Typune module whose keep/delete verdict is settled (audit P2a/P3 complete) and which is verified zero-live-reference (or coupled to such). Keep `LanguageRegistry` and the live export-parse path. No `src/` edits.

## Tasks

- [x] 1. Export crate — delete unused exporters
  - [x] 1.1 Delete `crates/export/src/html.rs`, `latex.rs`, `image.rs`.
  - [x] 1.2 Remove their `pub mod` + `pub use` lines from `crates/export/src/lib.rs` (keep `ExportFormat::Html/Latex/Image` enum variants in `engine.rs`).

- [x] 2. Markdown crate — delete highlight facade (cascade)
  - [x] 2.1 In `crates/markdown/src/highlight.rs`, remove `SyntaxHighlighter`, `AsyncHighlighter`, `HighlightCache`, `HighlightRequest`, `HighlightResult`, `HighlightStatus`, `BATCH_THRESHOLD_LINES`, `TextStyle`, `HighlightError`. Keep `LanguageRegistry`. (`HighlightedSpan` also removed — it was only referenced by the deleted facade + the View Renderer half; Markion uses its own `model::HighlightedSpan`.)
  - [x] 2.2 Update `crates/markdown/src/lib.rs` `pub use highlight::{...}` to export only `LanguageRegistry`.
  - [x] 2.3 Delete `crates/markdown/tests/highlight_property_test.rs`.

- [x] 3. Markdown crate — truncate renderer.rs View Renderer half
  - [x] 3.1 Truncate `crates/markdown/src/renderer.rs` to lines 1–708 (remove the `View Renderer: AST -> renderable view tree` half, lines 709–2346).
  - [x] 3.2 In `crates/markdown/src/lib.rs`, remove the View Renderer type re-exports (`Color`/`RenderCell`/`RenderInline`/`RenderListItem`/`RenderMode`/`RenderNode`/`RenderStyle`/`RenderTheme`/`RenderedDocument`/`Renderer`/`SourceModeStyle`/`VerticalAlign`/`ViewContext`/`Viewport`/`FocusState`); keep `render_to_markdown`.

- [x] 4. Markdown crate — delete render_cache.rs (coupled to View Renderer)
  - [x] 4.1 Delete `crates/markdown/src/render_cache.rs`.
  - [x] 4.2 Remove `pub mod render_cache;` + `pub use render_cache::{BlockLayout, RenderCache};` from `crates/markdown/src/lib.rs`.

- [x] 5. Markdown crate — delete table_ops.rs (zero Markion reference)
  - [x] 5.1 Delete `crates/markdown/src/table_ops.rs`.
  - [x] 5.2 Remove `pub mod table_ops;` + `pub use table_ops::{...};` from `crates/markdown/src/lib.rs`.
  - [x] 5.3 Delete `crates/markdown/tests/table_property_test.rs`.

- [x] 6. Verification
  - [x] 6.1 `cargo check -p markdown -p export` passes.
  - [x] 6.2 `cargo test -p markdown -p export` green (markdown 372 + export tests, 0 failed); `cargo test -p markion` green (115 tests, 0 failed).
  - [x] 6.3 `openspec validate prune-unused-typune-inventory` passes.
