## Why

After Enter exits a nested empty list item, whitespace-only lines containing spaces or tabs lose their line breaks in the visual projection. Subsequent Enter presses change the source but leave the displayed caret at the preceding item's end.

## What Changes

- Preserve source-backed line breaks and horizontal whitespace in list tails, including mixed blank lines and CRLF.
- Cover nested-list continuation, exit, repeated Enter, pointer placement, typing and undo with regression tests.
- Non-goals: changing list indentation/exit semantics or normalizing authored whitespace.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: list-tail blank lines remain individually editable even when they contain spaces or tabs.

## Impact

List projection construction in `src/visual.rs` and model/GPUI regression tests. Preserve canonical Markdown bytes, per-version derived caches, and stable source mapping; no new dependency or persistence format.
