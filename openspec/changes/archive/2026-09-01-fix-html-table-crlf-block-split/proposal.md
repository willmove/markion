## Why

Windows documents store the reported Word cover-sheet HTML with CRLF line endings and no blank lines. pulldown-cmark 0.13 emits one `Event::Html` per line and drops the `\r` from event ranges, so `push_html_block` refuses to join adjacent lines (`end != next.start`). Each `<tr>` becomes its own `PreviewBlock::Html`, the table parser never sees a complete `<table>…</table>`, and the flattener concatenates `文档版本` with `01`. The same markup with LF already parses as one 5×7 grid; unit tests only use `\n`, so this never failed in CI.

## What Changes

- Coalesce consecutive raw-HTML preview events into one `PreviewBlock::Html` when the source bytes between their ranges are only a CRLF “hole” (whitespace that is not a CommonMark blank line).
- Build the merged block’s `html` string from the contiguous document slice (`text[merged_range]`), not by concatenating event payloads that already dropped `\r`.
- Pin the cover-sheet fixture through `MarkdownDocument::preview_blocks()` / Visual Edit mapping on **CRLF** source: one HTML block, one table grid, `文档版本` and `01` in different columns. Keep LF behavior unchanged.
- **Non-goals:** GPUI CSS-grid placement (`fix-html-table-grid-placement`); nested `<table>` abort-to-flatten; rewriting files to LF on disk; merging two complete HTML blocks that CommonMark split on a real blank line.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: consecutive `Event::Html` pieces of one authored HTML block SHALL become a single `PreviewBlock::Html` when only a CRLF range gap separates them; merged `source_range` stays contiguous.
- `html-table-rendering`: a cover-sheet `<table>` whose source uses CRLF and one tag per line SHALL still render as one visual grid (not a card per row and not concatenated cell text). Capability lives in unarchived HTML-table changes until those archive; this change adds the CRLF assembly requirement.

## Impact

- `src/lib.rs` — `push_html_block` / `derive_preview_and_outline` HTML accumulation; tests that feed CRLF cover-sheet source into `preview_blocks()` and visual mapping.
- No new crate, cache surface, or settings. Per-version `Arc` preview/visual caches still recompute only when the document version changes.
- Read, Split Preview, Visual Edit, and export paths that consume `PreviewBlock::Html` all see the reassembled table without per-renderer fixes.
- `crates/*` stay GPUI-free.
