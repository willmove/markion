# Proposal

## Why

The complete workspace suite fails on macOS: UI tests simulate Ctrl instead of the platform modifier, and the built-in PDF writer aborts when its shaper selects a host font that the PDF backend rejects.

## What Changes

- Exercise the installed platform shortcuts while preserving the editor isolation and table menu lifecycle assertions.
- Verify that both AI text entry surfaces preserve ordinary printable input through select-all, deletion, and undo.
- Fall back to a compatible font before emitting PDF glyphs when a host face cannot be embedded.
- Make test-owned workspace directories independent of a symlink in the host temporary-directory path, without relaxing production scope checks.

Non-goals: changing default shortcuts, relaxing AI workspace authorization, and altering Markdown caches or crate boundaries.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `export`: Define fallback when a discovered system font is incompatible with the built-in PDF backend.

## Impact

AI input and table lifecycle regression tests, AI workspace test fixtures, and the GPUI-free PDF font pipeline.
