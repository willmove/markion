## Context

`derive_preview_and_outline` maps pulldown-cmark `Event::Html` into `PreviewBlock::Html` via `push_html_block`. That helper already concatenates consecutive events when `existing_range.end == source_range.start`. LF documents satisfy that: each HTML line event includes the trailing `\n`, so ranges abut and a multi-line `<table>` becomes one block. `html_preview_parts` then builds one `HtmlPreviewPart::Table`.

pulldown-cmark 0.13 (`append_html_line`) special-cases CRLF: it emits the line **without** `\r\n`, then a second `Html` event for `\n` only, and **never assigns `\r` to any event**. The next line’s range therefore starts one byte after the previous event’s end. `push_html_block` does not merge. Probe on the reported cover sheet:

- LF → 1× `PreviewBlock::Html` → `Table` 5×7
- CRLF → 9× `Html` fragments → flattener → `Text("文档版本01")`

Visual Edit maps each fragment to its own bordered HTML row, which matches the stacked-card screenshot. Parser/grid tests never call `preview_blocks()` on CRLF source.

```
Markdown text (CRLF)
        │
        ▼
 pulldown Event::Html per line, \r omitted from ranges
        │
        ▼
 push_html_block          ← this change (gap + source slice)
        │
        ▼
 PreviewBlock::Html { html, source_range }   [cached per version]
        │
        ▼
 html_preview_parts → HtmlTableGrid → existing table renderer
```

No new derived-state surface. Nested list/quote HTML already uses the same `push_html_block`.

## Goals / Non-Goals

**Goals:**

1. CRLF multi-line HTML that CommonMark treats as one HTML block (no blank line) SHALL be one `PreviewBlock::Html` whose `html` equals the authored slice (including `\r`).
2. The cover-sheet table SHALL parse as one grid on CRLF and LF; `文档版本` / `01` stay separate cells.
3. Two HTML blocks separated by a real blank line SHALL stay two preview blocks.
4. Tests go through `MarkdownDocument` derivation, not only `html_preview_parts(lf_string)`.

**Non-Goals:**

- GPUI `col_start`/`col_end` placement (`fix-html-table-grid-placement`).
- Nested `<table>` `failed = true` flatten.
- Forcing LF on save or changing Git `core.autocrlf`.
- Vendor patch to pulldown-cmark.

## Decisions

### D1: Merge on a non-blank whitespace gap, not only exact abutment

When the last block is `PreviewBlock::Html` and the next event is also HTML, merge if `text[prev.end..next.start]` is empty **or** ASCII whitespace that contains **no** `\n`.

pulldown-cmark includes the trailing LF of an HTML line in that line’s event. The CommonMark blank line that ends a type-6 block is then a **single** `\n` sitting in the gap — not `\n\n`. A lone `\r` is the CRLF hole (LF lives on the next event) and MUST merge. LF documents already abut (`end == start`), so their gap is empty.

**Alternatives considered:** (a) patch pulldown so CRLF ranges abut — vendor drift; (b) merge unless the gap contains `\n\n` — incorrectly glues two `<p>` blocks across a blank line; (c) detect `<tr>` continuations only — more brittle than a range-gap rule that also fixes non-table HTML (`<p>\r\n<img>`).

### D2: `html` is the document slice, not concatenated event strings

On merge (and, cheaply, on first push), set `html` from `text[source_range]` (document coordinates, same as `source_range` after `body_offset`). Event payloads omit `\r`; the slice keeps canonical bytes for Visual Edit payload and `html_preview_parts`.

`push_html_block` needs `&str` document text (already in `derive_preview_and_outline`). Nested quote/list targets use the same helper.

**Alternatives considered:** concatenate event strings and insert `\r` by guesswork — rejected; the source is already the truth.

### D3: Regression is derivation-level CRLF, not grid-line math

Add a compact cover-sheet fixture with `\r\n` between tags (same rows as the user’s source). Assert:

- exactly one `PreviewBlock::Html` whose range covers `<table` through `</table>`
- `html_preview_parts` yields one `Table` with 5 columns and `文档版本` / `01` in different columns of the same row
- LF twin of the same fixture still one block (no regression)
- two complete `<p>…</p>` HTML blocks separated by `\n\n` remain two blocks

Optional: Visual Edit `VisualBlockKind::Html` count is 1 for the CRLF table.

Keep existing `html_preview_parts` / `parse_html_table_grid` LF tests.

## Risks / Trade-offs

- **[Risk] Whitespace-only gap merge joins HTML that CommonMark split for a reason other than CRLF** → Mitigation: refuse merge when the gap contains `\n` (the leftover blank line after the previous event already consumed its trailing LF).
- **[Risk] `html` from the slice includes trailing `\r\n` on the last line** → Mitigation: table parser already ignores whitespace between tags; flattener for non-tables already sees newlines.
- **[Risk] Incremental source-mapped parse has a second HTML accumulation path** → Mitigation: grep `push_html_block` / `Event::Html` in `src/source_mapped.rs` and apply the same merge rule, or call the shared helper so incremental output stays equal to full parse (markdown-editing invariant).
- **[Trade-off]** Files that mix LF tables and CRLF tables are handled per event gap; no document-wide newline policy.

## Migration Plan

Presentation-only derived cache. No file format or settings change. Rollback = revert the derive/merge commit. Documents on disk keep their CRLF.

## Open Questions

None — CRLF vs LF probe on the cover sheet is the acceptance bar.
