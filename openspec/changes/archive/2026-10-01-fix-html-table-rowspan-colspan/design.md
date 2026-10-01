## Context

Today there are two unrelated table code paths in Markion:

1. **GFM pipe tables** (`| a | b |`) → pulldown-cmark emits `Tag::Table*` → built into a `TableDraft` (`src/table.rs:11`) → pushed as `PreviewBlock::Table` → rendered as a flexbox grid in `src/app/preview.rs`. This works, but the data model is `Vec<Vec<RichText>>` with no span information, and every cell uses `flex_1()` equal width.
2. **Raw HTML `<table>` blocks** → pulldown-cmark emits `Event::Html` (the parser options do not enable raw-HTML-as-elements) → accumulated into a `PreviewBlock::Html { html: String }` → passed through `HtmlPreviewBuilder::handle_tag` (`src/parse.rs:624`), a minimal flattener that only knows about `Text` and `Image` parts. In that flattener, `<table>` and `<tr>` push a single newline, `<td>`/`<th>` fall into the `_ => {}` catch-all, and `rowspan`/`colspan` attributes are never read. The result: every cell's text concatenates into one line and the grid is lost.

The bug report is a `<table>` using `rowspan="3"` to make a `12 V` supply cell span three peak-current rows. That structure cannot be expressed as a GFM pipe table at all, so fixing the HTML path is the only route.

See `proposal.md` for the motivation and `specs/html-table-rendering/spec.md` for the requirements this design satisfies.

## Goals / Non-Goals

**Goals:**
- Render raw HTML `<table>` blocks as visual grids with header emphasis, consistent with GFM pipe-table styling.
- Honor `rowspan` and `colspan` on `<th>`/`<td>`.
- Keep inline formatting (bold/italic/code/links) working inside HTML table cells by reusing the existing inline pipeline.
- Stay inside the per-document-version derived-state cache so no per-keystroke reparse happens.
- Degrade safely: malformed HTML tables fall back to the current flattener rather than panicking.

**Non-Goals:**
- No Visual Edit cell editing for HTML tables (read-only rendering only).
- No `<colgroup>`/`<col>` width hints, `scope`/`headers`, or per-cell CSS styling.
- No change to the GFM pipe-table path, the LaTeX export path, or the faithful raw-HTML export path (`render_html_fragment`).
- No general HTML5 parser — only the table-related tag set is recognized.

## Decisions

### Decision 1: Detect `<table>` at the `PreviewBlock::Html` boundary, not inside the pulldown-cmark option set

**Choice:** Keep the current pulldown-cmark options (do not enable a raw-HTML-as-elements mode). Instead, after accumulating a `PreviewBlock::Html { html }`, detect whether the trimmed HTML begins with `<table` and, if so, route it through a dedicated table parser; otherwise keep the existing flattener.

**Rationale:** pulldown-cmark's table events are GFM-pipe-only; there is no built-in path that turns a raw `<table>` into `Tag::Table*`. Changing global parser options would risk regressing the existing raw-HTML handling. Scoping the new logic to `PreviewBlock::Html` keeps the blast radius small and makes the "fall back to flattener" non-goal trivial (any table we cannot parse just goes through the old path).

**Alternative considered:** Enable pulldown-cmark's experimental HTML parsing. Rejected — it does not produce table events for raw `<table>` and would perturb unrelated HTML handling.

### Decision 2: A small, GPUI-free `HtmlTableGrid` value type shared between parser and renderer

**Choice:** Introduce a pure data type (in `src/parse.rs` next to `HtmlPreviewPart`, or a tiny `src/html_table.rs`) carrying the resolved grid:

```
HtmlTableGrid {
    header_rows: Vec<Vec<HtmlTableCell>>,
    body_rows:   Vec<Vec<HtmlTableCell>>,
}
HtmlTableCell {
    content: RichText,   // already-resolved inline formatting
    colspan: usize,      // >= 1
    rowspan: usize,      // >= 1
    is_header: bool,     // true for <th>
}
```

Span resolution happens at **parse time**, not render time: the grid is built by placing each `<td>`/`<th>` at the next free column in its row (skipping columns already occupied by a pending rowspan from above) and marking the covered `(row, col)` footprint as occupied. This mirrors how browsers lay out spans and keeps the renderer dumb.

**Rationale:** The renderer (`html_preview_block_view`) should not understand HTML; it should just draw a grid of cells. Putting span resolution in the pure parser makes it unit-testable without GPUI and keeps the cache payload self-contained.

**Alternative considered:** Add `rowspan`/`colspan` to the existing `TableDraft` and reuse `PreviewBlock::Table`. Rejected — `PreviewBlock::Table` is tightly coupled to GFM alignment semantics and the Visual Edit source-range machinery; overloading it would pull editing concerns into the read-only HTML path.

### Decision 3: Add `HtmlPreviewPart::Table { grid: HtmlTableGrid }`

**Choice:** Add a `Table` variant to `HtmlPreviewPart`. A single `PreviewBlock::Html` containing a `<table>` produces one `HtmlPreviewPart::Table` (plus leading/trailing `Text`/`Image` parts if the block has text around the table). Non-table HTML still produces only `Text`/`Image` parts.

**Rationale:** `HtmlPreviewPart` is the existing seam between the GPUI-free parser and the GPUI renderer, and it is already reused by the export path. Adding a variant there keeps all consumers in one place and lets export decide how to serialize a table (HTML export emits `<table>`; plain-text export flattens).

**Alternative considered:** A brand-new `PreviewBlock::HtmlTable`. Rejected — `PreviewBlock::Html` already carries the raw HTML string that export needs, and splitting it would force the export path to handle two block types.

### Decision 4: Renderer reuses pipe-table styling, draws spans by letting a cell's `flex_grow`/`flex_basis` reflect `colspan` and its row height follow `rowspan`

**Choice:** In `html_preview_block_view` (`src/app/preview.rs`), render the grid as stacked `div().flex()` rows (one per row), each cell a bordered `div` whose horizontal flex weight is proportional to its `colspan` and whose content is vertically centered. For `rowspan > 1`, the cell is rendered in the **first** row it occupies and given a negative bottom margin / explicit height equal to the spanned rows (equivalently: render the grid with CSS-grid-like absolute spans by giving each cell a `row_span` to a shared vertical layout). Concretely, the simplest correct approach that matches the existing flex idiom:

- Compute a column count = max occupied column across the grid.
- Each row is `div().flex()`. Each cell's width slot is `colspan` columns; give it `flex_grow(colspan)` so it takes proportionally more width.
- For `rowspan`, rather than fighting flexbox row independence, lay the table out as a single vertical `div` of rows but, for a cell with `rowspan=N`, draw it in the first row and let its height be `N × row_height` by reserving a vertical strut in the following N−1 rows at the same column slot (render an invisible spacer cell of the same width). This keeps rows aligned without a real CSS grid.

**Rationale:** GPUI's flexbox does not have native grid-span semantics, so true rowspan requires either a spacer-strut hack or an absolute-positioned overlay. The spacer-strut approach reuses the existing row/column flex code, keeps borders continuous, and is the same technique already implicit in how pipe tables are drawn. It is good enough for datasheet tables and avoids a layout rewrite.

**Alternative considered:** Rewrite the whole table as an absolute-positioned grid. Rejected — too invasive for a read-only rendering fix and risks regressing the working pipe-table styling.

### Decision 5: Fall back on any parse failure

**Choice:** The dedicated table parser returns `Option<HtmlTableGrid>`. On `None` (unbalanced tags, unresolvable overlaps, no `<table>` start), the code emits the existing `Text`/`Image` parts from the flattener as if the new path did not exist. This is enforced by a unit test asserting a truncated `<table><tr><td>...` still yields a `Text` part rather than panicking.

**Rationale:** Makes the "malformed HTML falls back safely" requirement mechanical rather than aspirational.

## Data flow and caching

```
document text (version V)
  └─ compute_preview_blocks(version V)        [cached, Arc-shared]
       └─ for each Event::Html run → PreviewBlock::Html { html }
            └─ HtmlPreviewBuilder::build()    [pure, runs once per version]
                 ├─ if html starts with <table → try parse_html_table_grid → Option<HtmlTableGrid>
                 │      ├─ Some(grid) → push HtmlPreviewPart::Table { grid }
                 │      └─ None       → fall back to flattener → Text/Image parts
                 └─ else → existing flattener → Text/Image parts
  └─ html_preview_block_view(PreviewBlock::Html)   [GPUI, per render]
       └─ match HtmlPreviewPart::Table { grid } → render grid view
```

Caching impact: `PreviewBlock::Html` is already part of the per-version `Vec<PreviewBlock>` cached on `MarkdownDocument`. The new `HtmlTableGrid` lives inside `HtmlPreviewPart`, which already lives inside that cached vector, so **no new cache surface is introduced** — the grid is computed once when the version's preview blocks are built and reused until the version changes. This preserves the "derived state cached per version, shared via `Arc`" invariant. The GPUI render view is stateless and recomputed each frame from the cached grid, same as today.

## Risks / Trade-offs

- **[Rowspan layout fidelity under flexbox]** → The spacer-strut hack renders correctly for the common datasheet case but may show a hairline border gap when spanned cells have very different content heights. Mitigation: unit-test the grid *model* (occupancy) rigorously; visually spot-check the reported `12 V / rowspan=3` table; accept minor cosmetic drift as a known limitation since the bar today is "completely broken."
- **[Inline formatting inside cells]** → Cells must reuse the existing inline span pipeline (bold/italic/code/links), not just take raw text. Mitigation: route cell text through the same `finish_rich_text` / inline-state path the flattener already uses for paragraphs, so inline markup in a cell resolves identically.
- **[Export path breakage]** → `src/export.rs:281-295` and `src/lib.rs:1512-1520` iterate `HtmlPreviewPart`; adding a `Table` variant will exhaustiveness-error those matches. Mitigation: handle the variant explicitly — HTML export emits a `<table>` string reconstructed from the grid; plain-text export falls back to tab/newline flattening.
- **[Nested tables or tables inside other HTML blocks]** → Out of scope; a `<table>` nested inside a non-table HTML block is rare and the detector keys off the block's leading tag, so nested tables inside a cell will be treated as text. Mitigation: documented as a non-goal; can be revisited if real documents need it.
- **[Performance on huge tables]** → Span resolution is O(rows × cols) and runs once per version. Mitigation: acceptable; no per-keystroke cost because of caching.

## Migration Plan

- Pure additive change; no persistence format, file, or settings impact.
- No user-visible setting to toggle — HTML tables simply start rendering.
- Rollback = revert the commit; the fall-back path means a partial implementation degrades to today's behavior rather than crashing.

## Open Questions

- None that would change the specs, approach, or task breakdown. (Whether to later promote `HtmlTableGrid` into a shared type also used by GFM pipe tables is a future refactor, explicitly deferred.)
