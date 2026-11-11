## Why

P1 improved text editing, but a single byte range cannot express selected blocks or a table rectangle. P3 operations require an explicit, stale-safe selection and transaction boundary first.

## What Changes

- Add GPUI-free text/block/cell selection values with exact source ownership and version validation.
- Route structured edits through a checked transaction with one undo entry and explicit resulting selection.
- Support block selection, expansion, multi-block copy/delete/move and plain-text plus HTML clipboard flavors.
- Preserve canonical Markdown, UTF-8 ranges, per-tab isolation and shared per-version caches.

Non-goals: a second rendered document model, merged GFM cells, or changes to unrelated data-file editing.

## Capabilities

### New Capabilities

### Modified Capabilities
- `markdown-editing`: explicit structured selection, atomic structured commands and multi-format copy.

## Impact

New pure selection/transaction helpers; tab editing state; keyboard, pointer, clipboard and history integration; focused pure and GPUI tests.
