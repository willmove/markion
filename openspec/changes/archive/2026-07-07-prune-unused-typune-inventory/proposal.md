## Why

With the Phase 4 / audit-P3 evaluation (`openspec/changes/archive/2026-07-07-unify-pulldown-cmark/design.md`) complete and the AST-adoption gate recorded as *deferred*, the "去留判定点" defined in `docs/typune-integration-audit.md` §5 is now unblocked: the absorbed Typune crates contain modules whose fate (keep vs delete) is settled but which still compile as permanent inventory. This change deletes everything confirmed unused, and only that.

The audit (`docs/typune-integration-audit.md`) inventoried ~1.76 万 lines absorbed; the live wiring from Markion is just three import lines (`src/export.rs:15-16`, `src/highlight.rs:14`). The rest is "tested inventory". Keeping dead modules has a real cost: every `syntect`/`two-face` upgrade forces re-checking code that never runs, the public API surface of the absorbed crates is misleading, and future contributors are nudged toward treating dormant code as active. The high-performance/large-document positioning (`docs/typune-integration-plan-glm-zcode.md` §7.2) does **not** rescue the highlight facade: the measured bottleneck for large docs is the parsing layer (gate recorded in `unify-pulldown-cmark` design.md), not highlighting, and the `AsyncHighlighter` solves a non-existent bottleneck (current 128-entry LRU + two-face 220 grammars keeps highlighting sub-millisecond per `de43695` measurements).

## What Changes

All deletions were verified by whole-repo reference check; each target is either zero-live-reference or coupled to such a target and deleted in the same cascade.

**Export crate — three self-contained exporters, each zero live reference:**
- `crates/export/src/html.rs` (Markion keeps its native HTML path per P2a)
- `crates/export/src/latex.rs` (P2c ported its strengths into Markion's native LaTeX path)
- `crates/export/src/image.rs` (depends on abandoned wkhtmltoimage)
- Plus their `pub mod` / `pub use` lines in `crates/export/src/lib.rs`. The `ExportFormat::Html/Latex/Image` enum variants in `engine.rs` stay (referenced by `export_with_fallback`, independent of the files).

**Markdown crate — the highlight facade, deleted in full (cascade-coupled):**
- `SyntaxHighlighter`, `AsyncHighlighter`, `HighlightCache`, `HighlightRequest`, `HighlightResult`, `HighlightStatus`, `BATCH_THRESHOLD_LINES`, `TextStyle`, `HighlightError` — removed from `crates/markdown/src/highlight.rs`.
- `AsyncHighlighter` is coupled to `SyntaxHighlighter` (its worker thread constructs and calls `SyntaxHighlighter`), so both go together; keeping a dead `AsyncHighlighter` as a "future spare" costs more than it saves.
- `LanguageRegistry` is **kept** — it is the sole entry Markion uses (`src/highlight.rs:14,23,28`).
- `crates/markdown/tests/highlight_property_test.rs` removed (tests construct `SyntaxHighlighter` directly).

**Markdown crate — `renderer.rs` View Renderer half:**
- `crates/markdown/src/renderer.rs` truncated to lines 1–708 (keep `render_to_markdown` + its inline tests; `pdf.rs`/`docx.rs` consume this function).
- Delete lines 709–2346 (the `View Renderer: AST -> renderable view tree` half), which is the sole site of `use crate::highlight::{..., SyntaxHighlighter, ...}` and `use crate::math::{...}` — cutting it severs those couplings.
- Remove the View Renderer re-exports from `crates/markdown/src/lib.rs` (`Color`/`RenderCell`/`RenderInline`/`RenderListItem`/`RenderMode`/`RenderNode`/`RenderStyle`/`RenderTheme`/`RenderedDocument`/`Renderer`/`SourceModeStyle`/`VerticalAlign`/`ViewContext`/`Viewport`/`FocusState`); keep only `render_to_markdown`.

**Markdown crate — `render_cache.rs` (coupled to View Renderer):**
- Deleted wholesale — its only non-test references are `use crate::renderer::{RenderNode, RenderTheme, RenderStyle, Color, Viewport}`, all of which disappear with the View Renderer half.

**Markdown crate — `table_ops.rs` (zero Markion reference):**
- Deleted; Markion uses its own `src/table.rs` + `apply_table_edit`. Remove `pub mod` + `pub use` from `lib.rs`; remove `crates/markdown/tests/table_property_test.rs` (imports `table_ops` directly).

**Kept (with reason):** `incremental.rs` (Phase 4 gate inventory), `math.rs`/`emoji.rs`/`extended_inline.rs` (live on the export parse path: `parser.rs:736 → extended_inline → emoji`), `parser.rs`/`ast.rs` (export input types), `LanguageRegistry`.

## Capabilities

### Modified Capabilities
- `crate-architecture`: the absorbed crates carry a de-inventory policy — once the keep/delete decision point (audit P2a/P3) lands a "delete" verdict, the module is removed in that same change rather than accumulating as permanent dead code.

## Impact

- Deleted files: `crates/export/src/{html,latex,image}.rs`; `crates/markdown/src/render_cache.rs`; `crates/markdown/src/table_ops.rs`; `crates/markdown/tests/{highlight_property_test,table_property_test}.rs`.
- Truncated: `crates/markdown/src/renderer.rs` (709–2346 removed).
- Edited: `crates/export/src/lib.rs`, `crates/markdown/src/lib.rs`, `crates/markdown/src/highlight.rs`.
- No `src/` changes (the Markion wiring at `src/highlight.rs:14` and `src/export.rs:15-16` references types that remain).
- Net deletion ~5000–6000 lines of dead code; test count drops by the two removed property-test files (~190 tests) with zero failures.
