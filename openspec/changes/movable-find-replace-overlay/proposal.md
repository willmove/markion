## Why

The Find / Replace overlay is pinned to the upper-right corner of the editor workspace. When it covers the text the user is reading or matching against, the only way to see that text is to close the overlay, which also clears the match highlighting the user was relying on.

## What Changes

- The Find / Replace overlay can be dragged to another position inside the window by a dedicated grip on the overlay. Fields, toggles, and buttons keep their current mouse behavior.
- The default position is unchanged: the overlay first appears at the upper right, as today.
- The dragged position is kept for the rest of the session (closing and reopening the overlay reopens it there) and is not written to `config.toml`.
- The overlay stays fully inside the window: positions are clamped while dragging and again whenever the window is resized, so it cannot be lost off-screen.
- Double-clicking the grip returns the overlay to its default upper-right position.
- Moving the overlay never shifts workspace layout and never touches the document.

Non-goals: no docking, resizing, or snapping; no persistence across launches; no change to search behavior, shortcuts, or the overlay's contents; no movable variants of other overlays (Git panel, menus, dialogs).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `chrome-platform`: adds a requirement that the Find / Replace overlay is movable by a grip, clamped to the window, session-persistent, and resettable. The existing "Find and replace" requirement (upper-right default, no layout shift) is not modified; the unarchived `improve-find-replace-and-read-search` change also modifies that requirement, so this change adds a separate requirement instead.

## Impact

- Code: `src/app/root_view.rs` (`search_panel_view` positioning and grip), `src/app/mod.rs` / `src/app/application.rs` (session overlay position state and a drag payload type), `src/app/tests.rs`, `src/i18n.rs` (grip tooltip / accessible label in every supported language).
- Invariants: moving the overlay is UI state only. It does not mutate document text, dirty state, undo history, or document version, and does not invalidate per-version derived Markdown caches.
