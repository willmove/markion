## Why

Visual Edit currently gives an authored empty line a different rendered row height from the one-line paragraph that replaces it when the user clicks and types. The downstream content therefore shifts during an ordinary edit, creating a visible layout jump at the caret.

## What Changes

- Make each contiguous Visual Edit paragraph/whitespace body flow contribute paragraph spacing exactly once, independent of how CommonMark splits or merges its paragraph blocks.
- Preserve stable downstream row positions when a user clicks an authored empty line and enters single-line text between headings or ordinary paragraphs.
- Keep click/caret mapping source-backed and keep typography-only geometry changes out of document history and derived Markdown caches.
- Add a rendered GPUI regression test that measures the whitespace row before input and the replacement paragraph row after input.
- Non-goals: changing Read/Split Preview spacing, rewriting authored newlines, changing multi-line paragraph wrapping, or altering the source-backed caret/navigation model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-typography`: Visual Edit paragraph spacing must be stable across paragraph/whitespace block splits and merges while empty-line content retains body line height.
- `markdown-editing`: Typing into an existing Visual Edit whitespace row must preserve the vertical position of following rendered content instead of causing a row-height jump.

## Impact

- Affected code: `src/app/preview.rs` whitespace-row sizing and `src/app/tests.rs` rendered interaction coverage.
- Affected documentation: `docs/visual-editing-quality.md` and the two modified OpenSpec capabilities.
- No public API or dependency changes.
- `MarkdownDocument.text` remains canonical; pointer placement without typing remains non-mutating, and per-version `Arc`-shared derived caches remain intact.
