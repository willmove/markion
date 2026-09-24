## Context

See proposal.md for motivation. The file tree already scans on a background executor (`schedule_file_tree_scan` in `src/app/application.rs`) and keeps collapse state across a successful scan. Context actions live in `FileTreeContextAction` (`src/app/mod.rs`) and are dispatched from `handle_file_tree_context_action` (`src/app/workspace.rs`). Create, rename, move, and delete live in `src/storage/file_tree.rs`. There is no filesystem watcher dependency today. `schedule_file_tree_scan` drops a result whose root no longer matches, but two scans of the same root can finish out of order.

## Goals / Non-Goals

**Goals:**

- Duplicate a file or folder from the context menu onto a free sibling name, off the UI thread.
- Keep the open workspace tree aligned with disk by watching that root, with a timed refresh only when the watch cannot start.
- Coalesce scans so a burst of writes produces one refresh, and a stale scan cannot overwrite a newer tree.

**Non-Goals:**

- Reloading open editors when their files change on disk.
- Copying unsaved buffers, opening the duplicate, or remapping tabs.
- Watching paths outside the current workspace root.

## Decisions

### Duplicate is a filesystem copy with a localized free name

Copy the on-disk entry in the same parent. The name is `{stem} - {marker}{ext}`, then `{stem} - {marker} ({n}){ext}` for `n` starting at 2, where `{marker}` comes from i18n (`副本` in Simplified Chinese, `copy` in English, and the existing other locales). Folders use the same pattern with no extension. The copy includes every on-disk child, not only tree-visible files. Symlinks are copied as links and are not followed. On failure, delete the destination if this operation created it, then report status.

The menu item is `FileTreeContextAction::Duplicate`, placed after Rename on the file and folder menus only. Success selects the new path, calls the existing refresh, and sets a localized status. Source tabs are not retargeted.

Alternative considered: prompt for a name. Rejected because Duplicate should finish in one click, and collision handling is the free-name sequence.

### Watch the workspace root; poll only as fallback

Add `notify` to the root app package only. When `workspace_root` is set, recursively watch that directory from a background task. Debounce events (about 400 ms) and then call the existing `refresh_file_tree` path. Do not set `StatusFileTreeRefreshed` for these refreshes. If the watcher fails to start, run a `Timer::after` loop (about 2 seconds) that uses the same silent refresh until the root changes. Restart or stop the task when the root changes or clears. Editor-originated create, rename, delete, and move already refresh; extra watch events are absorbed by coalescing.

Alternative considered: polling only. Rejected as the primary path because the request prefers watching, and idle polling walks large trees for no change. Polling remains the failure fallback.

### One scan generation for the current root

Give each requested scan a generation. A background result applies only when its generation is still the latest for the current workspace root. If a scan is already running, record that another scan is needed and start at most one follow-up when the in-flight scan finishes. This keeps auto-save and external bursts from stacking full walks, and it stops a late scan of a previous root from replacing the current tree.

Data flow, and what it does not touch:

```
watch event or fallback tick
        |
        v
debounce + scan generation
        |
        v
background FileTree::scan_with_options
        |
        v
apply only if generation matches current root
        |
        v
replace file_tree, keep collapse state, cx.notify()
        |
        v
bounded row render (unchanged)

open tab text / dirty / undo / derived Markdown caches: not on this path
```

Duplicate uses the same refresh after the copy returns; it does not write document state.

## Risks / Trade-offs

- [A full rescan on every debounced burst is still O(tree)] → Coalesce to one in-flight scan plus one follow-up; do not scan on each event.
- [Watchers miss events on some network or virtual filesystems] → Fall back to the timed refresh when the watch cannot start; manual Refresh stays available.
- [A failed folder copy could leave a partial tree] → Remove the destination created by this operation before reporting failure.
- [Auto-save writes retrigger the watcher] → Debounce and coalescing absorb them; document caches stay on the existing per-version path.

## Migration Plan

No stored data or config migration. Removing the watcher dependency and the menu action restores the previous manual-refresh behavior.

## Open Questions

None. The localized marker and the watch-then-poll fallback are fixed above so tasks can implement them directly.
