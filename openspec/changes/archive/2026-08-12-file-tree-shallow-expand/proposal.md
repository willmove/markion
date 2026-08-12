## Why

Clicking a folder in the leftmost file tree currently pops the **entire** subtree open down to the deepest files in a single action. This is not caused by an explicit "expand all" routine — it is emergent: the click handler only toggles the clicked folder's own path in `collapsed_tree_paths`, and because the visibility filter hides collapsed subtrees by depth (never recording any state for the descendants), re-expanding a folder exposes every nested folder already-fully-open. Users cannot drill into a workspace one level at a time, which is overwhelming for deeply nested projects and inconsistent with how native file explorers behave.

## What Changes

- Expanding a folder (clicking it while collapsed) SHALL reveal only its immediate children — direct subfolders and files — while every deeper subfolder stays collapsed.
- Collapsing a folder keeps today's behavior (its whole subtree is hidden by the depth filter).
- The expand/collapse toggle logic is extracted into a pure, unit-testable helper alongside the existing collapse-state helpers in `src/app/state.rs`.
- No change to the initial-collapse policy (the root's depth-0 folders stay collapsed on workspace open, established by the earlier `limit-initial-file-tree-expansion` change) and no change to filename filtering.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `workspace`: add a requirement governing interactive folder expansion depth — expanding a folder reveals exactly one level of children, with scenarios for one-level expand, progressive drill-down, and collapse.

## Impact

- `src/app/root_view.rs` — the folder click handler (around lines 1656-1665) delegates to the new helper instead of toggling a single path in the set.
- `src/app/state.rs` — new pure helper (e.g. `toggle_tree_folder`) next to `update_file_tree_collapse_state_from_scan`, taking the folder path, the `FileTree`, and the mutable `collapsed_tree_paths` set.
- `src/app/tests.rs` — new regression tests for one-level expansion, reusing the existing `nested_file_tree_fixture` / `visible_tree_entry_names` helpers.
- Preserves the invariant that the file tree renders a bounded number of rows per frame (one-level expansion only *reduces* the visible row count). No Markdown-derived caches or syntax-highlighting memoization are touched.

## Non-goals

- Not adding expand-all / collapse-all toolbar actions or context-menu entries.
- Not introducing lazy/per-level disk scanning — the tree stays a fully-scanned flat depth-tagged list held in memory.
- Not preserving per-descendant expanded state across collapse/expand cycles — the model is strict one-level (each click drills down exactly one more level).
