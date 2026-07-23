## Why

Table cells currently render as plain text in both Preview and Visual Edit modes. In Preview, pulldown-cmark already parses inline markup inside cells, but `push_preview_rich` discards the resolved `style`/`link` for the table branch and concatenates raw `String` text—so `**bold**` shows as unstyled "bold" and `[link](url)` shows as plain "link". In Visual Edit, each cell projects its authored source verbatim (only un-escaping `\|`), so `**bold**` shows as the literal string `**bold**`. Inline formatting inside table cells should render in both surfaces, matching how every other block already renders.

## What Changes

- Store per-cell inline styling in the table data model. `PreviewBlock::Table` and `VisualBlockKind::Table` move from `rows: Vec<Vec<String>>` to `rows: Vec<Vec<RichText>>`, where each cell carries its `text` (concatenation of spans, unchanged for plain-text consumers) plus `spans: Vec<InlineSpan>` (bold, italic, strikethrough, code, highlight, superscript, subscript, link).
- **Preview / Read / Split-Preview modes**: render each cell with the existing `rich_text_element` (bold weight, link color + underline, code background, etc.) instead of `selectable_plain_text`.
- **Visual Edit mode**: render each cell with inline formatting applied while unfocused, and reveal the authored source markup (e.g. `**bold**`) when the cell is focused for editing—mirroring how non-table visual blocks reveal inline constructs via `reveal_groups`. Editing continues to target the exact source range and produce one deterministic table replacement through the existing history path.
- Table cell editing (add/delete/move row/column, direct text edits, Tab traversal) continues to operate on the canonical GFM source table parsed by `parse_markdown_table`, which remains `Vec<Vec<String>>`; the rich spans are a render-time projection over that source, not an editing data structure.
- DOCX and LaTeX exporters keep using the cell's plain `text` (the span concatenation), so their output is unchanged.

## Non-goals

- Changing GFM table parsing, source-range lookup, alignment handling, or the `parse_markdown_table` / `format_markdown_table` round-trip used by table edits.
- Adding rich inline-formatting *controls* (toolbar buttons, keyboard shortcuts) inside table cells—cells remain plain-text editors whose authored source is the source of truth.
- Changing HTML export fidelity (it already renders table inline formatting via the `crates/markdown` AST path).
- Changing the cached-per-version derivation invariant: rich cell spans are computed once per document version alongside the existing `PreviewBlock` derivation and shared via `Arc`, not recomputed per keystroke.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `tables-outline`: The "GFM table rendering with row/column toolbar editing" requirement currently states "inline styles inside table cells are not required to render in the Visual Edit grid." This is upgraded: inline formatting (bold, italic, strikethrough, code, links, etc.) SHALL render in both Preview and Visual Edit table cells, and Visual Edit cells SHALL reveal source markup when focused for editing.

## Impact

- **`src/model.rs`**: `PreviewBlock::Table` and `VisualBlockKind::Table` change `rows` from `Vec<Vec<String>>` to `Vec<Vec<RichText>>`. `RichText` already exists (`model.rs:533`).
- **`src/parse.rs`**: `TableDraft.current_cell` becomes `Vec<InlineSpan>` (or equivalent) so `push_preview_rich`/`push_preview_math` route styled spans into the cell instead of raw `String`. Cell finalization calls `finish_rich_text` (or a trim-only variant) instead of `clean_preview_text`.
- **`src/lib.rs`**: `Event::Start/End(Tag::TableCell)` accumulation updated for span-based cells. Table edit helpers (`edit_table_at`, `visual_editor_field_at`) keep using `parse_markdown_table` (`Vec<Vec<String>>`) for source mutation—no change to editing logic. Tests asserting `PreviewBlock::Table { rows: vec![vec!["A".into()...]] }` update to `RichText`-based assertions.
- **`src/app/preview.rs`**: `PreviewBlock::Table` branch (`preview.rs:3270`) switches from `selectable_plain_text` to `rich_text_element`. `visual_table_view` (`preview.rs:2728`) passes inline-run/reveal data to `visual_editor_field_element`; `visual_editor_field_projection` (`preview.rs:2466`) builds `VisualProjectionSpan`s with real `InlineStyle`/`link` and reveals the full cell source range when the cell owns the caret. `visual_editor_field_element` constructs a highlighted `StyledText` from `projection.spans` (mirroring `visual_text_element`).
- **`src/visual.rs`**: `visual_block_editor` (`visual.rs:603`) and the `VisualBlockKind::Table` construction (`visual.rs:510`) build per-cell `VisualInlineRun`/`VisualRevealGroup` data (or an equivalent inline projection) from the authored cell source so the field projection can render/reveal.
- **`src/export.rs`**: `render_docx_table` (`export.rs:299`) and the LaTeX table renderer read `cell.text` (plain concatenation) instead of the `String` directly—minimal change.
- **`src/table.rs`**: `TableDraft` (`table.rs:11`) updated for span-based `current_cell`; `MarkdownTable`/`parse_markdown_table`/`format_markdown_table` stay `String`-based (they operate on source text, not rendered spans).
- **Invariants touched**: the cached-per-version derivation invariant is preserved—rich cell spans are computed during the existing `preview_blocks` derivation and cached in the `Arc`-shared state, not recomputed per frame or keystroke. Editing still goes through the source-text mutation path; undo snapshots skip the derived caches as before.
