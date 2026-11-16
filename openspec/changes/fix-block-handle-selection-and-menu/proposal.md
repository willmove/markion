## Why

Clicking a Visual Edit block grip selects the correct half-open source unit, but its ending boundary activates the following heading's prefix reveal. This makes the next block's `## ` or `### ` appear to belong to the selection. The grip also lacks the direct quick-settings popup requested by users, despite an existing block context menu.

## What Changes

- Keep a block selection and its visual effects within the selected source units: an exclusive end at the next block's start must neither select nor reveal that next block's structural prefix.
- Preserve whole-block source ownership, including the selected block's own structural markers, nested descendants and deterministic separators, for copying, deleting, moving and undo.
- Make an ordinary completed grip click select that block and open an anchored compact quick-settings popup, with direct Text/Heading and List/Quote/Code/Divider/Table transforms plus the existing Duplicate, Move and Delete operations and block Copy/Cut/Add Below actions.
- Preserve Shift-click block-range extension, drag reordering, right-click and keyboard context-menu entry points; a drag or range-extension gesture must not open the click popup.
- Add a localized General preferences toggle, **Show block menu on grip click**, persisted as `block_handle_menu`, defaulting to on. Disabling it retains click-to-select and drag behavior and leaves explicit context menus available.
- Retain version/identity validation, atomic edits, exact undo, localized labels and active-theme styling.

Non-goals: reproducing the reference application's entire feature set, adding arbitrary text color/alignment serialization, changing Source/Read behavior, or redesigning Markdown parsing and caches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: correct exclusive-end syntax reveal at block boundaries and add a compact, source-safe grip-click block popup with selection/drag-compatible interactions.
- `theme-preferences`: expose, persist and reset the grip-click menu toggle with safe handling of missing or invalid configuration values.

## Impact

Projection boundary handling in `src/visual.rs`; grip interactions and block menu rendering/state in `src/app/preview.rs`, `src/app/editing.rs`, `src/app/structured_editing.rs`, `src/app/root_view.rs` and `src/app/mod.rs`; preferences in `src/model.rs`, `src/storage/preferences.rs`, `src/app/application.rs` and `src/app/appearance.rs`; localized strings in `src/i18n.rs`; focused library and GPUI regressions. Exact block insertion/copy/cut may reuse or extend `src/block_edit.rs` and the existing clipboard transaction path.

No new dependencies are expected. Selection, popup lifecycle and preference changes must preserve document versions, shared per-version derived caches, highlight memoization and cached text handles. This change must integrate with, and preserve, the in-progress `compact-slash-command-panel` edits in shared files.
