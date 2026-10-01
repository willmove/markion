## Why

Visual Edit can paint the caret below a bare `#` while the canonical caret remains immediately after the marker. Terminal paragraph/heading line endings are assigned to whitespace rows, whose pointer and keyboard mappings can therefore edit the preceding line.

## What Changes

- Make terminal line-ending ownership, blank-row geometry, pointer placement, and keyboard navigation agree on the canonical source line.
- Add regression coverage for bare/empty headings, prose, lists, quotes, LF/CRLF, trailing whitespace, Unicode input, and mode transitions; fix related defects demonstrated by this audit.
- Preserve per-version derived caches, source-backed mutations, and interaction-only document stability.

Non-goals: redesigning the editor, changing Markdown semantics, or unrelated image/resource work.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: require agreement between the painted caret row and its canonical source line at structural and whitespace boundaries.

## Impact

Preview derivation in `src/lib.rs`, visual block derivation in `src/visual.rs`, whitespace/caret layout and mapping in `src/app/preview.rs`, navigation snapshots in `src/app/state.rs`, targeted model/GPUI tests, and `docs/visual-editing-quality.md`. No dependency or persistence changes.
