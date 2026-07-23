## Context

Markion derives `PreviewBlock`s once per document version (cached, `Arc`-shared) from pulldown-cmark events in `MarkdownDocument::preview_blocks` (`src/lib.rs`). For most blocks, inline formatting is resolved into `RichText { text, spans: Vec<InlineSpan> }` during that pass and rendered later by `rich_text_element`. Tables are the exception: `PreviewBlock::Table` and `VisualBlockKind::Table` store `rows: Vec<Vec<String>>`, and `push_preview_rich` (`src/parse.rs:118-121`) explicitly drops `style`/`link` for the table branch, concatenating raw text into `TableDraft.current_cell: String`.

In Visual Edit, non-table blocks carry `editable_runs: Vec<VisualInlineRun>` and `reveal_groups: Vec<VisualRevealGroup>` (built by `inline_runs` at `src/visual.rs:794`), which `build_visual_projection_with_marked_range` (`src/visual.rs:202`) consumes to render styled text while unfocused and reveal source markup when the caret enters a construct. Table cells bypass this entirely: `visual_editor_field_projection` (`src/app/preview.rs:2466`) walks each cell's authored source character-by-character (only un-escaping `\|`), returning `spans: Vec::new()`, and `VisualEditableText` paints highlights solely from the `StyledText` passed in-which for table cells has no highlights.

## Goals / Non-Goals

**Goals:**
- Render inline formatting (bold, italic, strikethrough, code, highlight, super/subscript, links) inside table cells in Preview, Read, and Split-Preview modes.
- Render the same inline formatting in Visual Edit table cells while unfocused, and reveal the authored source markup (e.g. `**bold**`) when a cell is focused for editing-consistent with non-table visual blocks.
- Preserve all existing table editing semantics: direct text edits, add/delete/move row/column, Tab traversal, alignment preservation, deterministic single-replacement source mutation.
- Preserve the cached-per-version derivation invariant.

**Non-Goals:**
- Per-cell inline-formatting toolbar controls or keyboard shortcuts.
- Re-architecting table cell editing away from the source-text mutation path.
- Changing HTML export (already full-fidelity via `crates/markdown` AST).

## Decisions

### D1: Store `RichText` per cell in the preview/visual data models

Change `PreviewBlock::Table.rows` and `VisualBlockKind::Table.rows` from `Vec<Vec<String>>` to `Vec<Vec<RichText>>`. `RichText` already exists (`src/model.rs:533`) with `text: String` (span concatenation) and `spans: Vec<InlineSpan>`, so plain-text consumers (DOCX, LaTeX, tests) read `cell.text` instead of the `String` directly.

**Rationale:** This is the minimal representation that carries inline styling through the cached derivation into both renderers without introducing a parallel table-specific type. `RichText` is already the currency of every other inline-bearing block.

**Alternative considered:** Keep `Vec<Vec<String>>` and re-parse inline formatting at render time. Rejected-re-rendering happens per frame in GPUI, and re-parsing on every paint would violate the cached-per-version invariant and duplicate work already done by pulldown-cmark during derivation.

### D2: Accumulate spans, not strings, in `TableDraft`

`TableDraft.current_cell` changes from `String` to `Vec<InlineSpan>`. `push_preview_rich` / `push_preview_math` (`src/parse.rs`) route into the cell's span list (calling `append_span` / `append_extended_text`, the same helpers headings/paragraphs use) instead of `push_str`. On `TagEnd::TableCell`, finalize with a trim-and-merge pass (the existing `finish_rich_text` already does per-line trim + style-merge; for single-line cells it reduces to a trim + merge).

**Rationale:** Reuses the exact span-building path every other block uses, so nested formatting (bold link, italic code) and extended syntax (`==highlight==`, emoji) come for free.

### D3: Preview rendering reuses `rich_text_element`

The `PreviewBlock::Table` branch (`src/app/preview.rs:3270`) replaces `selectable_plain_text(...)` with `rich_text_element(...)` for each cell. `rich_text_element` (`preview.rs:1103`) already produces a `SelectablePreviewText` with bold/italic/code/link highlights and click-to-open links.

**Rationale:** Zero new rendering code; the cell becomes a first-class rich-text run.

### D4: Visual Edit builds per-cell inline runs + reveal groups

The current `inline_runs` (`src/visual.rs:794`) is called once over the whole table `source_range`, producing runs that span cell boundaries-unsuitable for per-cell projection. Instead, the table block editor construction (`visual_block_editor`, `src/visual.rs:603`) calls a per-cell inline parse for each `TableCellSourceRange`, producing a `Vec<VisualInlineRun>` + `Vec<VisualRevealGroup>` scoped to that cell's `source_range`. These are stored on `VisualTableCell` (new fields) or on a parallel map keyed by `(row, column)`.

`visual_editor_field_projection` (`src/app/preview.rs:2466`) gains the cell's runs/reveal groups as input. When the cell does **not** own the caret, it builds `VisualProjectionSpan`s with real `InlineStyle`/`link` from the rendered runs (visible text = pulldown-cmark's resolved text, e.g. "bold"). When the cell **does** own the caret, the entire cell `source_range` is treated as one revealed `Source` piece (visible text = authored `**bold**`), mirroring `build_visual_projection_with_marked_range`'s `ProjectionPiece::Source` path (`src/visual.rs:314-328`).

`visual_editor_field_element` (`src/app/preview.rs:2425`) then converts `projection.spans` into `HighlightStyle`s (reusing the conversion in `visual_text_element`, `preview.rs:1704-1719`) and constructs a `StyledText::new(text).with_highlights(highlights)` to pass into `VisualEditableText`, instead of the current highlight-less `StyledText`.

**Rationale:** This mirrors the proven non-table visual path exactly. `VisualEditableText` already supports highlighted `StyledText` (non-table blocks use it); only the table path was passing none. The caret-ownership signal (`caret_active`, already computed at `preview.rs:2438`) drives the reveal toggle.

**Alternative considered:** Reuse the block-level `editable_runs`/`reveal_groups` (whole-table parse) and filter to each cell. Rejected-whole-table runs don't respect cell boundaries (a `|` inside the parse input isn't a cell delimiter to pulldown-cmark's inline parser), so mapping them to individual cells is fragile.

### D5: Table editing stays on the `parse_markdown_table` (String) path

`edit_table_at`, `visual_editor_field_at`, and the Tab-traversal logic (`src/lib.rs:610-675`, `1420-1440`) keep using `parse_markdown_table` / `format_markdown_table` / `table_cell_source_ranges`, which remain `Vec<Vec<String>>` over raw source text. The `RichText` cells are a render-time projection; they are never the source of truth for edits. `visual_block_editor` still calls `table_cell_source_ranges` to get field `source_range`s.

**Rationale:** Editing correctness (alignment preservation, deterministic replacement, exact selection) is already proven on the String path. Coupling rich spans into the mutation logic would risk regressions for no benefit, since the authored source is already canonical.

## Data flow & caching

```
pulldown-cmark events (per version, cached)
  └─ Tag::TableCell Start/End
       └─ TableDraft.current_cell: Vec<InlineSpan>   [D2]
            └─ finish_rich_text → RichText per cell    [D2]
                 └─ PreviewBlock::Table { rows: Vec<Vec<RichText>> }  [D1]
                      ├─ (Arc-shared, cached per version) ──────────────┐
                      │                                                  │
                      ├─► preview_block_view Table branch                │
                      │     └─ rich_text_element(cell)        [D3]       │
                      │                                                  │
                      └─► visual_block (build_visual_blocks)              │
                            ├─ VisualBlockKind::Table { rows: Vec<Vec<RichText>> }  [D1]
                            └─ visual_block_editor → per-cell inline_runs/reveal_groups  [D4]
                                  └─ VisualTableCell { field, runs, reveal_groups }
                                       └─ visual_table_view
                                            └─ visual_editor_field_element
                                                 └─ visual_editor_field_projection (caret-aware)  [D4]
                                                      └─ StyledText + highlights → VisualEditableText

edit path (unchanged):
  user input ─► visual_editor_field_at ─► parse_markdown_table (String)  [D5]
              ─► format_markdown_table ─► replace_range ─► re-derive (next version cache)
```

The rich cell spans are computed during the existing `preview_blocks` derivation (one pulldown-cmark pass that already visits table events) and cached in the `Arc`-shared state. Visual Edit's per-cell inline parse runs during `build_visual_blocks` (already per-version), and its output lives on the `VisualBlock` (also cached per version). No per-frame or per-keystroke recompute. Undo snapshots continue to skip derived caches (they store raw text only).

## Risks / Trade-offs

- **[Risk] `RichText` cells increase memory per table.** Each cell now holds a `String` + `Vec<InlineSpan>` instead of a `String`. Mitigation: most cells are plain text, so `spans` is a single default-style span; `finish_rich_text` already merges equal-style neighbors. The `Arc`-sharing means the cost is paid once per version, not per frame.
- **[Risk] Per-cell inline parse in Visual Edit doubles pulldown-cmark work for tables.** Each cell's authored text is parsed separately. Mitigation: tables are small (bounded cells); parsing happens once per version in `build_visual_blocks`, not per paint. If profiling later shows cost, the per-cell parse output can be memoized alongside the existing `editable_runs`.
- **[Risk] Reveal-on-focus changes the cell's display width.** When `**bold**` (7 chars) replaces "bold" (4 chars), the caret/selection mapping must track the new display↔source offsets. Mitigation: `VisualProjection.segments` already provides per-character display↔source mapping (used for caret painting and selection); the reveal path just produces different segments (`Source` vs `Rendered`), and the existing `display_for_source` / `boundary_candidates` machinery handles the offset translation.
- **[Risk] Tests asserting `rows: vec![vec!["A".into()...]]` break.** These are `String`-to-`RichText` construction changes. Mitigation: `RichText` can have an ergonomic `From<&str>`/`From<String>` impl so `.into()` keeps working, or tests update to `RichText::plain("A")`. The exact approach is a task-level decision.
