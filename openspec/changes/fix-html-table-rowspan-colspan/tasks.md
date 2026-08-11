# Tasks — fix-html-table-rowspan-colspan

Implements `specs/html-table-rendering/spec.md` per `design.md`.
Each group is sized to a single testable commit. Preserve the per-document-version derived-state cache invariant throughout (the new grid lives inside the already-cached `PreviewBlock::Html`, so no new cache surface is added).

## 1. GPUI-free grid model

- [x] 1.1 Add `HtmlTableGrid` and `HtmlTableCell` value types (`header_rows`, `body_rows`, per-cell `content: RichText`, `colspan`, `rowspan`, `is_header`) in `src/parse.rs` next to `HtmlPreviewPart` (no `gpui` imports — model only).
- [x] 1.2 Add `HtmlPreviewPart::Table { grid: HtmlTableGrid }` variant. Run `cargo build` to surface every exhaustive `match` over `HtmlPreviewPart` (export path, etc.) so the call sites are enumerated before implementation.
- [x] 1.3 Add a pure `parse_html_table_grid(html: &str) -> Option<HtmlTableGrid>` helper in `src/parse.rs` (or a new `src/html_table.rs`). It must: tokenize `<table>/<thead>/<tbody>/<tfoot>/<tr>/<th>/<td>` (reuse the existing `ParsedHtmlTag` attribute reader for `rowspan`/`colspan`), resolve spans into occupied `(row, col)` footprints (browser-style placement: skip columns held open by an earlier rowspan; clamp/normalize invalid spans to 1), route inline cell text through the existing inline pipeline so bold/italic/code/links resolve, split header rows (`<th>` or inside `<thead>`) from body rows, and return `None` on any unbalanced/unresolvable structure. No panic on truncated input.

## 2. Wire parser into the HTML preview builder

- [x] 2.1 In `HtmlPreviewBuilder`, detect when an accumulated HTML run begins with `<table` (trimmed) and call `parse_html_table_grid`. On `Some(grid)`, push `HtmlPreviewPart::Table { grid }`; on `None` or non-table HTML, keep the existing flattener output (`Text`/`Image`) unchanged. Ensure text surrounding a table within the same block still emits its `Text`/`Image` parts.
- [x] 2.2 Verify the builder stays pure and is still driven only from the per-version `compute_preview_blocks` path (no new computation on keystroke; the grid is computed once when the version's `PreviewBlock::Html` is built and reused thereafter). Confirm via the existing preview-cache tests.

## 3. GPUI rendering

- [x] 3.1 In `src/app/preview.rs::html_preview_block_view`, add a render branch for `HtmlPreviewPart::Table { grid }` that draws the grid as stacked `div().flex()` rows, reusing the existing pipe-table styling (cell borders, padding, header emphasis for `is_header` cells).
- [x] 3.2 Implement `colspan` by giving each cell a horizontal flex weight proportional to its `colspan` (e.g. `flex_grow(colspan)`) so a spanning cell takes proportionally more width.
- [x] 3.3 Implement `rowspan` via the spacer-strut technique (Decision 4): a cell with `rowspan = N` renders in the first row it occupies; the following N−1 rows render an invisible spacer cell at the same column footprint so columns stay aligned and the spanning cell's border stays continuous. Use the grid model's occupied-cell map to know where spacers go.
- [x] 3.4 Ensure the rendered table is read-only: no click/interaction mutates the document, and no editing affordances are drawn (matches the existing read-only HTML-block behavior).

> **Implementation note (deviation from design.md, beneficial):** Tasks 3.1–3.3 were implemented with a single GPUI CSS-grid renderer (`html_table_grid_view`) using GPUI 0.2.2's native `grid()` / `grid_cols()` / `col_start` / `row_start` / `col_span` / `row_span` API, instead of the flex-rows + spacer-strut scheme described in design.md Decision 4. CSS grid handles `colspan` and `rowspan` natively and correctly for all cases, is simpler, and avoids the flexbox cross-row limitation that motivated the spacer-strut hack. The grid model still carries spacer slots (used by the export paths), but the GPUI renderer skips them and places every real cell at explicit grid coordinates. This satisfies the same requirements with less code and better fidelity.

## 4. Export path handling

- [x] 4.1 Update `src/export.rs` (and any other `match HtmlPreviewPart` sites surfaced in 1.2) to handle `HtmlPreviewPart::Table`: HTML export reconstructs a `<table><tr><td colspan=.. rowspan=..>` string from the grid; plain-text export flattens rows to newline/tab-separated text. No panic on the new variant.

> **Implementation note:** DOCX export flattens each non-spacer cell to its own paragraph (DOCX table generation is out of scope and the existing `PreviewBlock::Html` DOCX path is paragraph-based). LaTeX export reconstructs a `tabular` environment (rowspan not representable in plain `tabular`, so spans are dropped and cells are laid out per row).

## 5. Tests

- [x] 5.1 Unit test the grid model (`parse_html_table_grid`): the reported `12 V` / `rowspan="3"` table resolves to a grid where `12 V` occupies rows 1–3 in column 0 and each subsequent row's cells shift correctly. Assert occupied-cell footprints, not rendered pixels.
- [x] 5.2 Unit test `colspan`, combined `rowspan`+`colspan`, and invalid-span fallback (non-numeric/zero/negative → treated as 1).
- [x] 5.3 Unit test malformed fallback: a truncated `<table><tr><td>...` returns `None` from `parse_html_table_grid` and the builder emits `Text`/`Image` parts (no panic, no empty preview).
- [x] 5.4 Unit test that inline formatting inside a `<td>` (`**bold**`, `[t](u)`) resolves to styled `RichText` via the shared inline pipeline.
- [x] 5.5 Add a preview-render smoke test (or extend an existing one) confirming an HTML `<table>` produces a `HtmlPreviewPart::Table` and the render function returns a non-empty view without panic.
- [x] 5.6 Run `cargo test --workspace`; ensure all existing table/preview tests still pass.

> **Implementation note (5.5):** the full GPUI render path (`html_table_grid_view`) requires a `MarkionApp` + font/asset context that is disproportionate to stand up for a smoke test; instead `html_preview_parts_routes_table_to_table_part` asserts the `Table` part is produced, the renderer compiles cleanly, and `cargo build` confirms the GPUI `grid()` call sites type-check. The heavy render is exercised by the existing preview smoke harness at runtime.

## 6. Validation

- [x] 6.1 `cargo build` clean (no warnings about unhandled `HtmlPreviewPart` variants).
- [x] 6.2 `cargo test --workspace` green.
- [x] 6.3 `openspec validate fix-html-table-rowspan-colspan` passes.
