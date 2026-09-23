## 1. Localization Contract

- [x] 1.1 Add a dedicated localized Sync menu-title message in `src/i18n/git.rs`, keep the existing Backup and Sync translations unchanged for non-menu content, and verify the Git localization exhaustiveness tests pass for every supported language.
- [x] 1.2 Replace the separate Source Mode and Split Preview Mode menu/shortcut labels with one localized Source/Split Preview label in `src/i18n.rs`, update the shortcut catalog mapping, and verify `cargo test -p markion localization_ --lib` passes.

## 2. Combined View Action and Shortcut

- [x] 2.1 Replace the separate Source and Split Preview menu-facing GPUI actions with one Source/Split Preview toggle whose transitions are Source → Split and Split/Visual Edit/Read → Source, and verify focused action tests cover all four starting modes plus unchanged document version, editor state, and derived-cache ownership.
- [x] 2.2 Register the combined action on `secondary-/`, retain the persisted shortcut id `set-edit-mode`, remove the former `set-split-preview-mode` descriptor and `secondary-p` default, and verify shortcut tests cover the platform labels, preserved Source override, sanitized obsolete Split override, unique registry ids, and absence of the old `Ctrl+P` binding.

## 3. Menu Integration

- [x] 3.1 Update the native View menu and in-window View dropdown to show exactly one Source/Split Preview row wired to the combined action while retaining Visual Edit, Read, and the all-mode cycle, and verify menu/action source-contract tests pass for both surfaces.
- [x] 3.2 Use the dedicated Sync title for the native and in-window top-level synchronization menus without changing Backup and Sync panel/setup/status text, and verify focused menu and localization tests distinguish the two message usages.

## 4. Validation

- [x] 4.1 Run `cargo fmt --all -- --check`, the focused combined-view/menu/localization tests, `cargo test -p markion`, and `cargo test --workspace`; resolve any regressions while preserving the per-document-version cache invariants.
- [x] 4.2 Run `openspec validate simplify-sync-and-view-menu` and verify every implementation task and delta-spec scenario is satisfied before marking the change ready to archive.
