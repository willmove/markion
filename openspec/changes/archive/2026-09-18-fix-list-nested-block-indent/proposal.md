## Why

Blocks nested inside list items (indented code blocks, fenced code blocks, fenced math, tables, standalone HTML blocks) are emitted as flat top-level rows, so both Visual Edit and Reading mode render them at document indent, detached from the list that contains them. The user-visible defect: for `1. Item 2` / `    1.     Sub item 1` (five spaces after the nested marker, which CommonMark parses as an indented code block inside the nested item), the nested `1.` renders as an empty marker and `Sub item 1` escapes the list as a floating code block — visibly broken in both view modes, while GitHub renders the code block nested inside the list item.

## What Changes

- The Markdown parse pass (`MarkdownDocument::derive_preview_and_outline_inner`) records the enclosing list depth (`list_stack.len()` at emit time) on `PreviewBlock::CodeBlock`, `PreviewBlock::MathBlock`, `PreviewBlock::Table`, and `PreviewBlock::Html` as a new `list_depth: usize` field (0 = not nested in a list).
- `VisualBlock` carries the same `list_depth` value, plumbed through `visual_block_from_preview`; structurally synthesized rows (gaps, whitespace, callout titles, front matter, source islands) keep `list_depth: 0`.
- Reading mode (`preview_block_view`) and Visual Edit row rendering indent every block with `list_depth > 0` to its owning list item's content column (the list indent step plus the marker column already used by list rows), so nested blocks stay visually inside their list.
- Non-goals: blockquotes and rules nested in list items keep today's flat presentation; export/publish re-serialization is unchanged (nesting is a render-time inset, not a document-model change); no change to how list item text or markers themselves are laid out.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: the existing requirements "Document-ordered block stream for list-nested blocks" and "Visual Edit list items with nested fenced code blocks" gain the missing indentation contract — nested blocks SHALL render indented under their owning list item's content column in Reading mode and Visual Edit.

## Impact

- `src/model.rs`: `PreviewBlock::{CodeBlock, MathBlock, Table, Html}` gain `list_depth`; `PreviewBlock::list_depth()` accessor. `VisualBlock` gains `list_depth`.
- `src/lib.rs`: parse arms for code blocks (fenced/indented/math), tables, and standalone HTML capture `list_stack.len()`; `push_html_block` forwards the depth; standalone display-math arm stays depth 0 by construction.
- `src/visual.rs`: `visual_block_from_preview` and the row-synthesis constructors set the new field.
- `src/app/preview.rs`: reading-mode block view and Visual Edit row view apply the list-content-column inset for `list_depth > 0`.
- Invariants preserved: derived state stays cached per document version (the field is computed in the single existing parse pass); member crates under `crates/*` are untouched (no `gpui` dependency added); every source byte keeps exactly one visual owner (the inset is presentational and does not alter source ranges or caret mapping, which remain row-internal).
