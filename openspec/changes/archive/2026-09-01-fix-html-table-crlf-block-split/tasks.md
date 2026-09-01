## 1. Failing derivation tests

- [x] 1.1 Add a compact cover-sheet HTML fixture (the reported `<table>` / `文档版本` / `01` shape) in `src/lib.rs` tests, once with `\n` and once with `\r\n` between tags. Do not check in the 70k Word dump.
- [x] 1.2 Assert on `MarkdownDocument::preview_blocks()`: CRLF fixture yields exactly one `PreviewBlock::Html` whose `source_range` covers `<table` through `</table>` and whose `html` equals that document slice; `html_preview_parts` on that `html` is one `Table` with 5 columns and `文档版本` / `01` in different columns of the same row. LF twin stays one HTML block with the same grid.
- [x] 1.3 Assert two complete HTML blocks (e.g. two `<p>…</p>`) separated by a blank line remain two `PreviewBlock::Html` entries.
- [x] 1.4 Assert Visual Edit maps the CRLF cover-sheet document to a single `VisualBlockKind::Html` (not one Html visual row per `<tr>`).

## 2. Merge helper and `push_html_block`

- [x] 2.1 Add a GPUI-free helper (next to `push_html_block`) that returns whether the source gap between two HTML event ranges should merge: empty, or ASCII whitespace with no `\n` (a lone `\r` is the pulldown CRLF hole; a `\n` in the gap is a CommonMark block boundary).
- [x] 2.2 Change `push_html_block` to take the document `&str`. Merge when the last block is `Html` and the gap rule passes (not only `end == start`). Set `html` from `text[merged_source_range]` and extend `source_range` through the new end. First insert also uses the document slice for that event’s range.
- [x] 2.3 Pass `text` from `derive_preview_and_outline` (and `emit_finished_paragraph`) into every `push_html_block` call, including nested list/quote targets.

## 3. Incremental path and validation

- [x] 3.1 Confirm `src/source_mapped.rs` (and any other `Event::Html` → `PreviewBlock::Html` walk) uses the same helper or full `derive_preview_and_outline` so incremental output stays equal to a full parse. Adjust only if a duplicate accumulator exists.
- [x] 3.2 Run the new tests plus existing HTML-table / `html_block_maps_to_rendered_visual_block` coverage (`cargo test --lib` with those filters). Existing LF table tests SHALL still pass.
- [x] 3.3 `openspec validate fix-html-table-crlf-block-split`
