## Context

`FileTree::scan` intentionally builds one flat, depth-annotated list containing every Markdown file and every directory that has a Markdown descendant. `MarkionApp` separately owns `collapsed_tree_paths`, and `filtered_visible_file_tree_entries` omits descendants of any directory in that set. Because a workspace-root change currently clears the set and the completed background scan installs the full tree without repopulating it, every branch is initially expanded.

The change affects the handoff between asynchronous scanning and sidebar visibility, but it does not change filesystem traversal or the `FileTree` data model. The bounded 300-row render cap must remain in place. The data flow will be: establish a new root and mark it for initial collapse → scan off the GPUI thread → install the matching result → seed collapse state from depth-zero directory entries → derive visible rows during rendering. Same-root refreshes skip the seeding step. Markdown document versions, derived preview state, highlighting, and cached text handles are unaffected.

## Goals / Non-Goals

**Goals:**

- Show only the opened root's immediate Markdown-bearing files and directories on the first successful tree render.
- Preserve normal per-directory expansion and collapse interactions.
- Preserve the current workspace's expansion state across refreshes.
- Allow a filename filter to reveal nested matches without mutating collapse state.
- Preserve asynchronous scanning and bounded row construction.

**Non-Goals:**

- Persist expansion state across launches or workspace switches.
- Change Markdown-only collection, directory pruning, sorting, or the visible-row cap.
- Add recursive expand/collapse commands or automatically reveal the active document's ancestors.

## Decisions

### 1. Seed collapsed paths after the first successful scan for a new root

Track whether the current root still needs its initial one-level presentation. A root change clears root-specific selection, scroll, and collapse state as today and marks the new root as pending initial collapse. When a matching background scan succeeds, collect every `FileTreeEntryKind::Directory` entry whose `depth == 0` into `collapsed_tree_paths`, then clear the pending marker.

Seeding from scanned entries keeps the storage model independent of UI state and exactly represents the requested boundary: root-level files and directories remain visible, while descendants of each root-level directory are hidden. Initializing all directory entries as collapsed was rejected because it would not change the first render but would make a newly expanded top-level directory show only one additional level, turning a normal expansion click into a recursive series of clicks.

The pending state is tied to workspace-root replacement rather than every scan invocation. A failed scan leaves it pending so a retry still receives the default. A stale result for another root remains ignored by the existing root-match guard.

### 2. Treat same-root scans as refreshes and retain interaction state

On a successful refresh of the existing root, retain collapsed paths that still exist, matching current behavior, and do not seed top-level directories again. This preserves directories that the user expanded before refreshing. Switching to a different root discards the old root's state and applies the one-level default to the replacement root.

Resetting top-level collapse state after every refresh was rejected because refresh is expected to update filesystem contents, not undo navigation choices.

### 3. Make an active filename filter temporarily bypass collapse visibility

When the normalized query is non-empty, `filtered_visible_file_tree_entries` will evaluate matches across the scanned flat list without skipping descendants of collapsed directories. Clearing the filter resumes visibility from the unchanged `collapsed_tree_paths` set.

This keeps nested files discoverable after top-level directories become collapsed by default. Permanently expanding ancestors for search results was rejected because clearing a query would unexpectedly alter the user's navigation state.

### 4. Test state derivation separately from GPUI rendering

Use small `FileTree` fixtures to verify the initial collapsed-path set and the visible entries for empty queries, manual expansion, and active filtering. Add application-state coverage where practical for new-root versus same-root scan behavior. This tests the behavior without requiring pixel-level UI assertions and leaves the existing bounded-row tests intact.

## Risks / Trade-offs

- [A deeply nested active document can be hidden immediately after Open Folder] → Keep the requested one-level default deterministic; the user can expand its branch, and filename filtering can reveal it without changing state.
- [A new top-level directory discovered during a same-root refresh is not distinguishable from an explicitly expanded directory when expansion is represented by absence from a collapsed set] → Limit automatic seeding to new roots; preserving existing navigation state is more important than forcing refreshed content into the initial default.
- [Concurrent same-root scans can complete out of order under the existing root-only stale-result guard] → Ensure initial-collapse seeding is idempotent and consumes the pending marker only once; broader scan generation ordering remains outside this change.
- [Filtering more nested entries can reach the render cap sooner] → Continue counting and cloning only up to the existing bounded limit and report hidden matches with the existing indicator.

## Migration Plan

No persisted data or dependency migration is required. Implement the pending-initial-collapse state, scan-result seeding, filter visibility rule, and tests. Rollback removes those additions and restores the current empty collapsed-path initialization; no user data needs cleanup.

## Open Questions

None.
