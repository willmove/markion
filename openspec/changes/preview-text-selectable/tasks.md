## 1. Selection model

- [x] 1.1 Add a `PreviewSelection` (block index, text-run id, byte range) on the active tab or `MarkionApp`, with helpers to normalize/clamp ranges and clear invalid selections after preview splice/tab switch
- [x] 1.2 Add unit tests for range normalization, plain-text extraction for a selection, copy-precedence (preview vs editor), and invalidation when the block index is out of range

## 2. Pointer selection in preview text

- [x] 2.1 Introduce a selectable preview text helper (wrapping `StyledText` / `InteractiveText`) that maps pointer down/move/up to character indices via `TextLayout` and updates `PreviewSelection`
- [x] 2.2 Wire the helper into common textual preview blocks (headings, paragraphs, list item bodies, blockquotes) while excluding decorative markers from the selectable string
- [x] 2.3 Extend selection to code block bodies, math source/fallback text, table cell text, and image captions/URLs as shown
- [x] 2.4 Preserve link-click behavior: open URL only when the gesture does not create a meaningful non-empty selection

## 3. Highlight paint and copy routing

- [x] 3.1 Paint selection highlight quads for the active `PreviewSelection` without rebuilding `preview_list` or recomputing derived Markdown state
- [x] 3.2 Update `MarkionApp::copy` so a non-empty preview selection is written to the clipboard (plain text) and takes precedence over the editor selection; leave cut/paste/typing editor-only
- [x] 3.3 Clear preview selection when the user starts selecting in the source editor, when the active tab changes, or when the selected block/run becomes invalid

## 4. Verification

- [x] 4.1 Run `cargo test` for the new helpers and ensure existing preview/editor tests still pass
- [x] 4.2 Manually verify Split Preview and Read modes: drag-select + Copy for paragraph and code; link click still works; Read mode copy does not dirty or edit the document
