## Why

An older autosave can overwrite a completed manual save, a retained draft can collide with a new draft after restart, and Source Select All can select unrelated text in a CRLF file. These failures can silently lose user content.

## What Changes

- Reproduce issues #29, #28, and #32 on the current main branch before changing their implementation.
- Order autosave and explicit save commits, preserving external-change detection and recovery on save failure.
- Allocate independent recovery snapshots safely across launches and keep existing snapshots readable.
- Resolve the Source paragraph selection against the original line endings and byte offsets.

Non-goals are changing autosave preferences, redesigning recovery UI, and changing word movement.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reliable-file-persistence` specifies overlapping save ordering and durability of unrelated snapshots across launches.
- `markdown-editing` specifies progressive selection across LF and CRLF paragraphs.

## Impact

The changes touch storage writes, the tab save and recovery lifecycle, and the source paragraph range helper. Selection uses source bytes directly and preserves per-version Markdown caches and undo behavior.
