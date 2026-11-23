## Why

Inline and display math inside GFM table cells is shown as raw LaTeX source (for example `$\lfloor N/2 \rfloor$`) instead of a rendered formula. The `code-and-math` capability already promises typeset formulas in Split Preview, Read, and unfocused Visual Edit, and table cells already parse math into `InlineSpan.math` — but three render paths never consume it: the render-request collector skips `PreviewBlock::Table`, the Read/Split Preview grid only routes image-bearing cells through the math layout, and the Visual Edit cell projection flattens math spans into plain text.

## What Changes

- Request renders for every math span in GFM table cells (header and body) through the existing bounded, coalescing math cache. An identical formula in prose and in a cell shares one cache entry.
- Split Preview and Read mode render table-cell math as baseline-aligned formula atoms, exactly as prose does.
- Visual Edit renders math in an unfocused table cell as formula atoms inside the cell's mixed-prose layout. Pending and Error entries keep the exact authored source text. A focused cell keeps its existing whole-cell source reveal, also while Keep inline syntax hidden is on.
- A formula wider than its cell scales down proportionally to the cell's content width in every surface instead of overflowing into the neighboring cell (found during the real-window smoke on a seven-column table in Split Preview).
- Pointer interaction with a formula atom in an unfocused Visual Edit cell resolves to the authored `$...$` or `$$...$$` source boundary. Shift/Alt clicks keep rectangular cell selection, and a double-click selects the complete authored formula. Rendering and pointer interaction never mutate document text, document version, undo history, or derived Markdown caches.
- Update the Visual Edit WYSIWYG coverage matrix note and add regression tests.
- Non-goals: a floating payload editor for cell formulas, column-width heuristics that measure rendered formulas, export writers, and HTML-table cells.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `tables-outline`: Adds the requirement that native Markdown table cells render math in Split Preview, Read mode, and unfocused Visual Edit, with source reveal on focus, boundary-exact pointer hit testing, and proportional scaling inside a cell narrower than the formula. This brings table cells into compliance with the existing `code-and-math` rendering contract and parallels the existing embedded-images requirement.

## Impact

- `src/app/math_render.rs`: `ensure_math_renders` also walks `PreviewBlock::Table` rows and quote children.
- `src/app/preview.rs`: the Split Preview/Read table branch routes math-bearing cells through `rich_text_with_math_element`; `visual_table_view` threads `display_scale` into the cell element; the unfocused-cell mixed-prose layout gains formula atoms with cell-scoped hit targets.
- `src/app/tests.rs`: regression tests (bin target).
- `docs/visual-editing-quality.md`: coverage-matrix note.
- No data-model, parser, exporter, dependency, or public API changes. Derived Markdown state stays cached per document version, and math rendering stays asynchronous and cache-bounded.
