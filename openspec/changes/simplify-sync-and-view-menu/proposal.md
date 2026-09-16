## Why

The top-level “Backup and Sync” label is longer than the action-oriented menu needs, and separate Source and Split Preview entries make two closely related source-oriented layouts feel like independent destinations. A shorter Sync category and one Source/Split Preview toggle reduce menu clutter while keeping all four view modes available.

## What Changes

- Rename only the native and in-window top-level “Backup and Sync” menu category to the localized equivalent of “Sync”; retain “Backup and Sync” wording inside setup, status, settings, and the sync center where it describes the broader feature.
- Replace the separate Source Mode and Split Preview Mode rows in both View menus with one localized “Source/Split Preview” row.
- Make that combined row and `Ctrl+/` (`Cmd+/` on macOS through the existing `secondary` convention) alternate between Source and Split Preview. From Visual Edit or Read, the first invocation enters Source; subsequent invocations alternate Source and Split Preview.
- Remove the old default `Ctrl+P` direct Split Preview binding and its separate shortcut-reference row while retaining the existing Visual Edit, Read, and all-mode cycle controls.
- Preserve a user’s customized Source shortcut when migrating to the combined action where practical; obsolete Split Preview overrides are ignored by the existing unknown-id sanitization path.
- **BREAKING UI/shortcut:** Source and Split Preview no longer have separate menu commands or separate default shortcuts.

Non-goals: changing Git synchronization behavior, renaming Backup and Sync content outside the top-level menu title, changing the four view-mode layouts, or altering the existing all-mode cycle order.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Replace direct Source and Split Preview shortcuts with one source-layout toggle while preserving document and per-version cache state.
- `chrome-platform`: Consolidate the View menu rows and shorten the top-level synchronization menu title on both menu surfaces.
- `ui-i18n`: Localize the new Source/Split Preview row and Sync menu title, and keep shortcut-reference content aligned with the consolidated action.

## Impact

- Affected application code: GPUI actions and shortcut registry in `src/app/mod.rs`, menu/keymap installation in `src/app/bootstrap.rs`, view-mode handlers in `src/app/workspace.rs`, in-window menu rendering in `src/app/root_view.rs`, and related source-contract/GPUI tests in `src/app/tests.rs`.
- Affected localization code: `src/i18n.rs` and `src/i18n/git.rs`, with distinct menu-title wording so existing Backup and Sync panel language remains unchanged.
- Existing customized shortcut storage may need a narrow legacy-id migration for the former Source action; no document data, view-mode persistence, Markdown text, undo history, or derived Markdown caches are changed by this work.
- No new dependencies or workspace-member changes are expected.
