## 1. Model

- [x] 1.1 Extend `PreviewBlock::BlockQuote` in `src/model.rs` with `children: Vec<PreviewBlock>` (nested list items), keeping `text` and `source_range`; update `source_range()` accessors and any exhaustive matches so the crate compiles with the new field
- [x] 1.2 Add a helper (e.g. `PreviewBlock::plain_text()` or quote-specific fold) that returns quote text including child list item text, for use by stats/export/copy consumers

## 2. Derivation

- [x] 2.1 In `MarkdownDocument::derive_preview_and_outline` (`src/lib.rs`), add a per-quote `children` draft; route `flush_list_item` output into it when `quote_depth > 0` (both the `Tag::Item` eager flush and the `TagEnd::Item` flush), and populate `BlockQuote.children` on `TagEnd::BlockQuote`
- [x] 2.2 Verify ordered numbering inside quotes uses the existing `list_stack` so a non-1 start index (e.g. `> 3. x`) is honored, and nested list levels keep relative `level`; adjust `ListItemDraft.level` computation if it assumes top-level lists
- [x] 2.3 Audit `push_preview_rich` / `push_preview_math` routing priority (`src/parse.rs`) so list-item-inside-quote text still lands in the item draft and inline math inside quoted list items is preserved

## 3. Preview Rendering

- [x] 3.1 In `src/app/preview.rs` `BlockQuote` rendering arm, render each child list item inside the quote container using the existing list-item row layout (ordered number from `index`, bullet, task checkbox), with quote typography and relative-level indentation
- [x] 3.2 Ensure preview text selection/copy spanning a quote includes child list item text in document order

## 4. Consumers

- [x] 4.1 Update `src/document_memory.rs`, `src/export.rs`, `src/app/math_render.rs`, and the stats/text-extraction site in `src/lib.rs` so `BlockQuote` children text is included
- [x] 4.2 Audit `src/visual.rs` and `src/source_mapped.rs` `BlockQuote` handling: confirm Visual Edit projection and incremental derivation ignore/pass through `children` without regression, fixing as needed

## 5. Tests

- [x] 5.1 Add parser tests: blockquote containing an ordered list produces one `BlockQuote` block whose children are the `ListItem`s, and zero top-level `ListItem` blocks for that content
- [x] 5.2 Add parser tests for unordered, task list (checked state), nested list levels, and non-1 ordered start index inside a blockquote
- [x] 5.3 Add regression test that a paragraph-only blockquote derives exactly as before (single `BlockQuote`, empty `children`)
- [x] 5.4 Add/adjust consumer tests: stats word count, export output, and preview copy include quoted list item text
- [x] 5.5 Run `cargo test --workspace` and confirm green; manually verify the screenshot scenario (quote with ordered list renders inside the quote container in Split Preview)
