## 1. Context Menu Model

- [ ] 1.1 Add `CopyPath`, `CopyRelativePath`, and `Properties` variants to the file-tree context action model in `src/main.rs`.
- [ ] 1.2 Include the new actions in file and folder context-menu action arrays while preserving existing Rename and Delete entries.
- [ ] 1.3 Add localized label mapping for the new actions through `file_tree_context_action_label`.

## 2. Path Copy Actions

- [ ] 2.1 Implement absolute path copy for file and folder context-menu targets using the app clipboard API.
- [ ] 2.2 Implement workspace-relative path copy for file and folder context-menu targets.
- [ ] 2.3 Report localized success and failure status messages without changing document text, dirty state, undo history, or derived Markdown caches.

## 3. Properties Surface

- [ ] 3.1 Add transient app state for the selected file-tree entry properties view.
- [ ] 3.2 Collect entry kind, absolute path, workspace-relative path, file size for files, and modified timestamp when available outside the file-tree row render path.
- [ ] 3.3 Render an in-app properties dialog or equivalent surface and support dismissal.
- [ ] 3.4 Ensure folder size is omitted or shown as unavailable unless computed without blocking bounded tree rendering.

## 4. Localization

- [ ] 4.1 Add i18n message variants for Copy Path, Copy Relative Path, Properties, property field labels, unavailable values, and status feedback.
- [ ] 4.2 Provide translations for all supported languages and update the exhaustiveness guard.
- [ ] 4.3 Remove or avoid any hard-coded user-visible strings introduced by this change.

## 5. Verification

- [ ] 5.1 Add or update focused tests for metadata/path helper behavior where practical.
- [ ] 5.2 Run `cargo fmt`.
- [ ] 5.3 Run `cargo test`.
- [ ] 5.4 Run `openspec validate add-file-tree-entry-context-actions`.
- [ ] 5.5 Manually verify that right-clicking files and folders shows Rename, Delete, Copy Path, Copy Relative Path, and Properties, and that each new action reports localized feedback.
