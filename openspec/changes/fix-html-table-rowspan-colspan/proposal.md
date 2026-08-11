## Why

Raw HTML `<table>` blocks in a Markdown document do not render as tables in the GPUI preview — every cell's text collapses onto one line, the row/column grid is lost, and `rowspan`/`colspan` attributes are ignored entirely. GFM pipe tables render fine, but the separate code path that flattens raw HTML (`PreviewBlock::Html` → `HtmlPreviewPart`) has no concept of table structure. Users authoring datasheet-style tables (common case: a `12 V` supply cell spanning several peak-current rows via `rowspan="3"`) get garbled output.

## What Changes

- Parse `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<th>`, and `<td>` elements inside a raw HTML block into a real table grid in the preview, instead of flattening their text content inline.
- Read `rowspan` and `colspan` attributes on `<th>`/`<td>` and expand the grid so a spanning cell occupies the correct number of rows/columns (with `rowspan="3"` repeating the cell value down its column).
- Render the resulting grid with the existing visual-table styling (borders, header emphasis) used for GFM pipe tables, so HTML tables and pipe tables look consistent.
- Preserve inline formatting (bold, italic, code, links) inside HTML table cells.
- Extend the same rendered-table view into **Visual Edit** mode, where raw HTML blocks currently collapse to a verbatim source box (`VisualBlockKind::Unsupported` + `VisualSourceIslandKind::Html`). Visual Edit now renders an HTML block via the shared `html_preview_parts` pipeline (so tables, text, and images appear), while staying read-only — no cell editing.
- Non-goals:
  - No editing of HTML table cells in Visual Edit (read-only visual rendering only, same as other HTML blocks).
  - No `<colgroup>`/`<col>` width hints, `scope`/`headers` accessibility attributes, or CSS styling beyond the default theme.
  - No change to the GFM pipe-table path or to the HTML/LaTeX export paths (export already emits raw HTML faithfully).
  - Malformed/unclosed table HTML falls back to the current flattened-text behavior rather than crashing.

## Capabilities

### New Capabilities
- `html-table-rendering`: Rendering raw HTML `<table>` blocks (with `rowspan`/`colspan`) as visual tables in the preview and Read mode.

### Modified Capabilities
<!-- None. The existing tables-outline capability is scoped to GFM pipe tables; HTML table rendering is a separate, additive capability. -->

## Impact

- **`src/parse.rs`** — `HtmlPreviewPart` gains a `Table` variant; `HtmlPreviewBuilder::handle_tag` learns `<table>/<tr>/<td>/<th>` nesting and reads `rowspan`/`colspan` from `ParsedHtmlTag::attr`.
- **`src/app/preview.rs`** — `html_preview_block_view` gets a rendering branch for the new `HtmlPreviewPart::Table`, reusing the visual-table styling. Visual Edit's `visual_block_view` gains a read-only rendering branch for HTML blocks that reuses the same pipeline.
- **`src/visual.rs`** — `visual_block_from_preview` maps `PreviewBlock::Html` to a new `VisualBlockKind::Html` (carrying the html string) instead of `Unsupported`, and clears the source-island flag so the rendered view is used rather than the raw-source box.
- **`src/model.rs` / `src/table.rs`** — likely a small shared table-grid type (rows of cells, each with optional span) that the HTML flattener produces and the preview consumes. A new `VisualBlockKind::Html { html: String }` variant carries the raw HTML into the Visual Edit view layer.
- **`src/export.rs` / `src/lib.rs`** — the HTML-flattening reuse sites must handle the new `Table` part (at minimum, no panics; ideally emit a `<table>` in HTML export and skip/flatten in plain-text export).
- **Architecture invariants** — the new parsing stays inside the per-document-version derived-state cache path (computed once per version, shared via `Arc`), like the existing `PreviewBlock::Html` flattening; no recomputation per keystroke.
- **Localization** — no new user-facing strings expected (tables are content, not chrome).
