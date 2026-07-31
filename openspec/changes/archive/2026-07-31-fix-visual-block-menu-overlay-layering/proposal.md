## Why

The Visual Edit block menu is currently rendered inside the focused virtualized document row, so later rows, formatted text, and images can paint over the menu and make its commands unreadable even though they remain interactive. Contextual block operations need a reliable overlay presentation that remains visually above document content without changing canonical Markdown or derived document state.

## What Changes

- Render the Visual Edit block menu through an editor-level overlay host instead of as a child of an individual virtualized row.
- Anchor the overlay to the invoking block-menu button, keep it within the usable viewport, and constrain overflowing command content so every action remains reachable.
- Define the block menu's stacking and dismissal behavior relative to document content, other contextual UI, scrolling, tab/mode changes, stale targets, and application modals.
- Add rendered GPUI regression evidence covering menus opened above following headings, formatted prose, and images while preserving the existing exact block-operation path.
- **Non-goals:** changing block transformation/duplicate/delete/reorder semantics, introducing a new document model, persisting overlay state, or redesigning unrelated application menus and contextual tools.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Require the Visual Edit block-operation menu to remain legible, reachable, and above all document rows and media while its presentation-only state leaves canonical source, document version, history, and derived-cache identity unchanged.

## Impact

- Affected UI seams: `src/app/preview.rs`, `src/app/root_view.rs`, block-menu state and open/close handling in `src/app/mod.rs` and `src/app/editing.rs`, plus rendered GPUI tests in `src/app/tests.rs`.
- The change adds only ephemeral positioning/visibility state and reuses existing GPUI overlay/anchoring patterns; it introduces no external dependency, public API, persistence migration, or workspace-member change.
- The per-document-version `Arc` caches, memoized highlighting, cached text handle, exact `BlockTarget` validation, and one-mutation/one-undo invariants remain unchanged.
