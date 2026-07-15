## 1. Initial Workspace Expansion State

- [x] 1.1 Add app state that marks a replaced workspace root as awaiting its initial one-level file-tree presentation, while leaving same-root refreshes unmarked.
- [x] 1.2 On the first successful matching scan, seed `collapsed_tree_paths` from depth-zero directory entries and consume the pending state; retain the pending state on failure and preserve surviving collapse paths on later refreshes.

## 2. Filtered Tree Visibility

- [x] 2.1 Update bounded visible-entry derivation so a non-empty filename or relative-path query searches descendants of collapsed directories without mutating `collapsed_tree_paths`.

## 3. Regression Coverage

- [x] 3.1 Add fixture-based tests proving that initial collapse state shows only root children and that expanding one top-level directory reveals its full descendant branch without changing sibling branches.
- [x] 3.2 Add tests proving that same-root refresh state is preserved, a new root receives the default, failed scans remain eligible for initialization, and filtering reveals nested matches before restoring collapse visibility when cleared.
- [x] 3.3 Run `cargo fmt --check`, the focused file-tree tests, and `cargo test`, confirming the existing bounded-row behavior and cached Markdown-state invariants remain unaffected.

## 4. OpenSpec Validation

- [x] 4.1 Run `openspec validate limit-initial-file-tree-expansion` after implementation and reconcile any artifact or behavior mismatch.
