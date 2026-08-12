## Context

See `proposal.md` for motivation. The relevant current state:

- `FileTree` (`src/storage/file_tree.rs`) is a **flat, depth-tagged, pre-order `Vec<FileTreeEntry>`** containing every folder/file at every depth, built once per scan by `collect_file_tree_entries`. There is no nested node graph and no lazy loading.
- Interactive expand/collapse state lives on `MarkionApp` as `collapsed_tree_paths: HashSet<PathBuf>` — the set holds **collapsed** folder paths; an absent folder is treated as expanded.
- The visibility filter `filtered_visible_file_tree_entries` (`src/app/root_view.rs`) renders entries while skipping the subtree of any collapsed directory **by depth**: once a collapsed dir at depth `D` is seen, every following entry with `depth > D` is skipped until depth drops back. It never reads or writes descendant state.
- The folder click handler (`src/app/root_view.rs`, the `FileTreeEntryKind::Directory` arm inside the `on_mouse_up(Left)` closure) currently toggles **only the clicked folder's own path** in the set.
- Because the depth filter hides a collapsed parent's whole subtree without recording any state for the descendants, those descendants are implicitly "expanded". Re-expanding the parent (removing it from the set) therefore exposes the entire subtree already-fully-open in a single click. This is the emergent bug.

## Goals / Non-Goals

**Goals:**
- Make a single expand click reveal exactly one level of children.
- Keep the change localized to interactive toggle state; do not touch the scan, the flat data model, the visibility filter's depth-skip logic, the initial-collapse policy, or filename filtering.
- Keep the toggle logic pure and unit-testable (same pattern as the existing `update_file_tree_collapse_state_from_scan` helper).

**Non-Goals:**
- No expand-all / collapse-all actions or context-menu entries.
- No lazy/per-level disk scanning.
- No preservation of per-descendant expanded state across collapse/expand cycles (strict one-level model).

## Decisions

### Decision 1 — Fix the state mutation, not the filter
The filter is a pure read of the collapse set and is correct: it hides a collapsed dir's subtree by depth. The bug is upstream — descendant folders are simply never recorded as collapsed when their parent is expanded. So the fix belongs in the toggle that mutates `collapsed_tree_paths`, called from the click handler.

- *Alternative considered:* track an "expanded depth frontier" in the filter. Rejected — it would still need per-folder state and would complicate the existing depth-skip loop, which also has to interplay with filename filtering.

### Decision 2 — Extract a pure helper `toggle_tree_folder`
Add `toggle_tree_folder(folder: &Path, tree: &FileTree, collapsed_paths: &mut HashSet<PathBuf>)` in `src/app/state.rs`, next to `update_file_tree_collapse_state_from_scan`. Semantics:
- **Folder currently collapsed (in set) → expand:** remove it from the set, then insert every descendant **directory** (entries whose path is component-wise under `folder` and whose path ≠ `folder`) so only immediate children show.
- **Folder currently expanded (not in set) → collapse:** insert it into the set (depth filter hides the subtree as today).

The click handler's `Directory` arm calls `toggle_tree_folder(&path, &app.file_tree, &mut app.collapsed_tree_paths)` instead of the inline single-path toggle. `app.file_tree` is already accessible inside the existing `update(cx, |app, cx| …)` closure.

- *Rationale:* matches the established pattern of pure, `tests.rs`-testable state helpers, and keeps the GPUI closure thin.
- *Alternative considered:* inline the logic in the closure. Rejected — untestable without a GPUI harness.

### Decision 3 — Identify descendants from the flat entry list via component-wise `starts_with`
The flat `tree.entries` already contains every directory, so no recursion or disk walk is needed. Descendant detection = `entry.kind == Directory && entry.path != folder && entry.path.starts_with(folder)`. `Path::starts_with` is component-wise, so `workspace/docs` does not wrongly match `workspace/docs-extra`.

- *Alternative considered:* store a parent index / children ranges on `FileTreeEntry`. Rejected — changes the entry struct and scan code for a single call site; the linear scan over entries is negligible.

### Decision 4 — Strict one-level model
Each expand collapses all descendants. This is predictable and matches the user's request; it also matches the mental model of native file explorers where reopening a folder starts shallow.

## Data flow / caching

The toggle mutates only `app.collapsed_tree_paths` (a `HashSet<PathBuf>` on app state). It does **not** touch:
- The scanned `FileTree` (read-only reference).
- Any Markdown-derived cache (preview blocks, outline, stats, line count), which are keyed by document version and shared via `Arc`.
- Syntax-highlighting memoization or the cached editor text handle.

After the mutation the existing `cx.notify()` triggers a re-render; the visibility filter recomputes the bounded visible-row list from the updated set. No rescan is scheduled and no document version changes. The "bounded number of rows per frame" invariant is preserved — one-level expansion strictly *reduces* the visible row count versus today.

## Risks / Trade-offs

- **[Risk] Large subtrees insert many paths into the set.** → Mitigation: bounded by the number of directories already held in memory from the scan; cost is dwarfed by the per-frame filter pass. No pre-sizing needed.
- **[Risk] `Path::starts_with` consistency on Windows (case / separators).** → Mitigation: both `collapsed_paths` and `entry.path` originate from the same `read_dir` scan, and the existing collapse logic already relies on exact `PathBuf` equality/contains. No new normalization is introduced.
- **[Trade-off] Re-expanding a previously drilled branch restarts at one level.** → Accepted per Non-Goals; consistent with native explorers.
- **[Risk] Existing test `initial_file_tree_collapse_shows_root_children_and_expands_one_branch` asserts the old full-expansion visibility.** → Mitigation: that test drives the filter directly (`visible_tree_entry_names` with a hand-mutated `collapsed` set), not the new toggle helper; the filter is unchanged, so the assertion stays valid. The one-level behavior is covered by **new** tests on `toggle_tree_folder`. Verify during implementation and adjust only if the assertion actually exercises toggle semantics.

## Migration Plan

None — pure interactive behavior change, no persisted state, config, or on-disk format. Rollback is reverting the click handler's `Directory` arm to the inline single-path toggle.
