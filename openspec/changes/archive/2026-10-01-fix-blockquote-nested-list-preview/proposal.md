## Why

In Split Preview / Read mode, a blockquote that contains a list (ordered, unordered, or task list) renders incorrectly: the list items escape the blockquote container and render as top-level list blocks above/below the quote, while the quote shows only its paragraph text. The root cause is in `MarkdownDocument::derive_preview_and_outline` (`src/lib.rs`): inline routing (`push_preview_rich`, `src/parse.rs`) prioritizes the open list-item draft over the open blockquote, and `flush_list_item` always emits a top-level `PreviewBlock::ListItem`, so list content inside a quote is never attached to the `PreviewBlock::BlockQuote`. This breaks CommonMark semantics for a common authoring pattern (e.g. `> 1. item`) and makes the preview misrepresent the document.

## What Changes

- Change preview derivation so blocks nested inside a blockquote (paragraphs, ordered/unordered/task list items, and other supported nested blocks) remain attached to that blockquote instead of leaking out as top-level preview blocks.
- Extend the preview block model (`PreviewBlock`) so a blockquote can carry nested child blocks (at minimum list items with their level/numbering/check state) in addition to plain text.
- Update the preview pane rendering (`src/app/preview.rs`) so nested list items render inside the blockquote container with correct ordered numbering (honoring the list start index), unordered markers, and task checkboxes, at the quote's typography.
- Keep all consumers of `PreviewBlock::BlockQuote` consistent: outline/stats/text extraction, export, search, preview text selection/copy, and memory accounting.
- Preserve the derived-state caching invariant: the fix lives entirely inside the existing per-version derive pass; no per-keystroke recomputation is introduced.

Non-goals: no changes to Visual Edit (WYSIWYG) block structure editing, no changes to source-editor syntax highlighting, and no new nested constructs beyond what the preview already supports at top level (e.g. tables inside quotes stay out of scope unless they already work).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: the parsing/preview requirement for nested Markdown constructs is refined so that a blockquote containing a list (or other nested block content) keeps that content inside the blockquote in the rendered preview, with correct list markers and numbering.

## Impact

- `src/lib.rs` (`derive_preview_and_outline`): blockquote/list event routing so nested list items (and paragraph breaks) are collected into the quote's children.
- `src/model.rs`: `PreviewBlock::BlockQuote` gains nested child blocks; helper accessors updated.
- `src/parse.rs` (`push_preview_rich`, `flush_list_item`, `ListItemDraft`): routing priority and flush target when a quote is open.
- `src/app/preview.rs`: blockquote rendering draws nested children (list markers, numbering, checkboxes) inside the quote container.
- Secondary consumers: `src/source_mapped.rs`, `src/document_memory.rs`, `src/export.rs`, `src/app/math_render.rs`, preview selection/copy in `src/app/preview.rs`, and any `PreviewBlock::BlockQuote` pattern matches.
- Tests: parser/preview unit tests covering ordered, unordered, task, and nested lists inside blockquotes, plus list start-index numbering inside quotes.
