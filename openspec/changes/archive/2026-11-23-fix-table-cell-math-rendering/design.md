## Context

See `proposal.md` — Why. The data needed to render already exists; only request collection and three view paths skip it.

**Data flow today**

- Parsing: `Event::InlineMath` / `Event::DisplayMath` inside a table go through `push_preview_math`, which stores `InlineSpan { text: math.authored, math: Some(MathSource { latex, authored, style, delimiter, source_range }) }` in the cell's `RichText`. `source_range` is the absolute authored `$...$` / `$$...$$` range. This lives in the per-version derived `PreviewBlock::Table { rows }`, shared via `Arc`.
- Request collection: the root view's render calls `MarkionApp::ensure_math_renders(preview, visual, zoom, display_scale, foreground, cx)` (`src/app/math_render.rs`). It collects keys from headings, paragraphs, list items, footnote definitions, block-quote prose children, and math blocks, reserves pending keys, and spawns background typeset + raster jobs whose completions call `math_cache.complete` and `cx.notify()`. `PreviewBlock::Table` falls into the `_ => {}` arm, so a cell formula's `math_entry` reports `Pending` forever.
- Split Preview / Read: `preview_block_view` → `PreviewBlock::Table` → per cell, `rich_text_with_math_element` only when the cell has an image, otherwise the math-blind `rich_text_element`.
- Visual Edit: `visual_block_view` → `visual_table_view` → per cell with a proven field, `visual_editor_field_element(.., cell_rich: Some(cell))`. A caret-owning cell reveals source; otherwise a cell with an authored line break, an image, or a super/subscript takes a mixed flex-wrap layout of `VisualEditableText` fragments and image atoms; every other cell becomes one `StyledText` via `table_cell_rendered_projection`. All three flatten math spans to text. Cells without a proven field go straight to `rich_text_element`.
- Pointer handling in cells: the cell `div` owns padding clicks (cell-selection gate, then the caret at the cell start). Text fragments stop propagation to keep their precise source position. The prose math atom's hit target (`visual_math_hit_target`) does not stop propagation, so inside a cell the outer handler would overwrite its caret with the cell-start fallback.

## Goals / Non-Goals

**Goals:**

- Cell formulas render in all three surfaces through the existing math pipeline, with no new document or tab state.
- Layout stays stable across a formula's Pending → Ready transition: only that atom swaps.
- Pointer behavior in a cell stays precise at the formula and preserves rectangular cell selection.

**Non-Goals:**

- A floating payload editor for cell formulas. Tables keep their existing reveal and editor behavior while Keep inline syntax hidden is on (`Stable visual inline editing`).
- Measuring rendered formula widths for column weights.
- HTML-table cells, export writers, and the HTML preview pipeline.

## Decisions

**1. Collect table-cell math in `ensure_math_renders` through a recursive preview-block walk.**
The walk covers table rows (header and body) and block-quote children, in addition to the existing prose arms, and builds the same key as prose: `math_font_size(style)`, zoom, display scale, foreground, renderer version. An identical formula in a paragraph and in a cell therefore shares one entry and one job.

*Alternative:* a separate table pass with the table font size — rejected: it splits the cache for identical formulas and makes the same formula a different size in prose and in cells.

*Caching/versioning:* unchanged. Document edit → per-version derived blocks (`Arc`) → root render → `ensure_math_renders` → `reserve_pending` → background job → `math_cache.complete` → `cx.notify()` → repaint. Nothing is typeset on the typing or derivation path, and a late completion still only updates a key that is still pending.

**2. Split Preview / Read: widen the cell condition to "image or math".**
`rich_text_with_math_element` already handles ready (atom), pending, and error (code-styled authored source) math and falls back to `rich_text_element` when nothing special is present.

*Alternative:* make `rich_text_element` math-aware — rejected: it is the single shaped-text fast path used by every prose block.

**3. Visual Edit: move the cell's mixed layout into a table-specific element and add formula atoms.**
`visual_table_view` receives `display_scale` from `visual_block_view` and calls a new `visual_table_cell_element`, which owns the mixed layout (trigger now also includes math) and otherwise delegates to `visual_editor_field_element`. A ready math span becomes a `visual_math_atom` baseline-compensated for the cell's table font metrics; a pending or failed one falls through to an ordinary authored-source fragment, matching prose Visual Edit. `visual_editor_field_element` keeps its signature, so its eight non-table callers are untouched.

*Alternatives:* thread `display_scale` through `visual_editor_field_element` and every caller — rejected as invasive; read the scale from the app inside the element — rejected because it is implicit and not the scale the frame was rendered with.

**4. A cell-scoped hit target instead of the prose one.**
`visual_table_math_hit_target` stops propagation, then: double-click → select the complete authored formula (`move_to_visual_editor_target`); otherwise the shared cell-selection gate (`structured_pointer_down`, which handles Shift/Alt rectangular selection and the drag anchor); otherwise `move_to(boundary)`. With the caret at the boundary the cell becomes focused and reveals its source, consistent with the whole-cell reveal that table cells already keep under Keep inline syntax hidden.

*Alternative:* no hit target and let the cell fallback put the caret at the cell start — rejected: it loses the boundary-exact hit testing the math capability promises and makes a formula behave worse than the text beside it.

**5. Math atoms carry a debug selector.**
Visual Edit atoms are `visual-math-atom-{source_range.start}` (a document offset), so a test can locate an atom from document text alone, in prose and in cells. Preview atoms are `preview-math-{block_index}-{run_offset}`, mirroring the existing `preview-table-cell-{block}-{row}-{col}-image-{n}` naming, and preview grid cells are `preview-table-cell-bounds-{block}-{row}-{col}`. All are paint-time debug bounds, with no effect outside tests.

**6. Read-only Visual Edit cells follow the preview grid.**
A cell without a proven field that contains math goes through `rich_text_with_math_element`, as in Split Preview / Read, so an exactly unrangeable table stays consistent instead of mixing rendered and raw formulas.

**7. Cell formula atoms narrow with their cell.**
A prose atom is indivisible at its measured width. That is right for a full-width paragraph, but in a narrow column the atom overflows the cell and covers the neighbor's text — seen on the seven-column NoC table in Split Preview. In cells the atom box gets `max-width: 100%` and the raster's aspect ratio, so the layout engine clamps it to the cell's content width and derives the height; the raster fills the box. The hit halves follow the clamped box, so a boundary click still resolves to the authored `$`. Prose and display atoms keep their measured width. One frame constructor (`formula_atom_frame`) serves both surfaces, with a `fit_cell` flag.

*Alternatives:* clip the cell content — hides part of the formula; fall back to authored source when the atom is wider than the cell — the cell width is unknown when the element is built.

## Risks / Trade-offs

- [Row height changes when a formula completes] → the same asynchronous swap prose already performs; completion triggers one repaint and the cell uses the same flex layout as image cells.
- [Formula size follows the math font size, not the table font size] → identical to today's Read / Split behavior for cells that mix images and math, and to prose; left unchanged.
- [Column weights still measure authored LaTeX text] → no worse than today's raw-source display; tracked as a non-goal.
- [A cell containing only math switches from one shaped text to flex-wrap fragments] → line breaking follows the existing image / line-break cell behavior (fragments split at whitespace; CJK inside a fragment still wraps through its shaped text), and an atom wider than the cell scales down (decision 7) instead of overflowing.
- [Background render completion inside tests] → tests locate atoms through `debug_bounds` after `run_until_parked`, like `issue_36_table_cell_image_renders_in_visual_edit_and_read`; if completion is not observable there, the test seeds the entry through the same `math_key` / `complete` path.

## Migration Plan

Presentation-only: no persisted data, configuration, or dependency changes. Rollback is a revert.
