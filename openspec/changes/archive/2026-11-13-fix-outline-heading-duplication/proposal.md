## Why

Editing a Markdown heading can leave duplicate or obsolete headings in the outline even though canonical source contains each heading only once. Incremental derived state must match a fresh parse.

## What Changes

- Add a deterministic regression for heading edits and cached region reuse.
- Correct the source-mapped outline cache ownership at region boundaries.
- Preserve per-version shared derived caches and incremental reuse.

Non-goals: changing outline presentation, removing legitimately repeated headings, or reparsing the whole document on every keystroke.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

`tables-outline`: make the existing outline-update contract explicit for incremental heading edits, including repeated authored headings and cached-region boundaries.

## Impact

`src/source_mapped.rs` and its regression tests. No new dependencies or persistent data changes.
