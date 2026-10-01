## Context

`arm_file_tree_watch` (`src/app/application.rs`) creates a recursive `notify` (6.1) watcher whose callback ignores the event payload (`move |_| tx.try_send(())`). A debounce loop then calls `refresh_file_tree`, which runs the full background scan. Since the scan speed-up it costs about 40 ms for a 13k-file vault, but each refresh still replaces `file_tree` and redraws the sidebar, and it fires on every autosave.

`src/storage/file_tree.rs` already owns the exclusion rules: `is_always_excluded` (name blacklist) and the dot-name / Windows-hidden-attribute check used during scans.

## Goals / Non-Goals

**Goals:** decide relevance inside the watcher callback from the event alone (kind plus paths), with no filesystem access; keep it a pure function so it is unit-testable without a real watcher.

**Non-Goals:** honoring the Windows hidden *attribute* for watch events (deleted paths cannot be queried, and a stat per event would defeat the purpose); per-path incremental updates.

## Decisions

### 1. Relevance by event kind
Using `notify::EventKind`:
- `Create(_)`, `Remove(_)`, `Modify(ModifyKind::Name(_))` → candidate, then path-filtered.
- `Modify(Data | Metadata | Other | Any)` and `Access(_)` → ignored. On Windows, `ReadDirectoryChangesW` reports content writes as `Modify(Any)`, while renames arrive as `Modify(Name(_))` and creates and deletes as their own kinds, so ignoring `Modify(Any)` is what drops autosave noise there.
- `EventKind::Any`, `EventKind::Other`, an event with no paths, and any event carrying `Flag::Rescan`, or a watcher error → refresh (fail open).

*Alternative:* keep every kind and only path-filter. Rejected: autosave writes visible `.md` files, so path filtering alone would not remove the most frequent spurious refresh.

### 2. Relevance by path
`file_tree_watch_path_is_relevant(root, path, show_hidden) -> bool` in `file_tree.rs` strips `root` and walks the remaining components. It returns false if any component is always excluded, or, when `!show_hidden`, if any component starts with `.`. Paths outside `root` count as relevant (fail open). An event is relevant if any of its paths is relevant, so a rename from `.obsidian/x.md` to `notes/x.md` still refreshes.

The callback captures `root` and the `show_hidden` value at arm time. Toggling hidden entries already rescans and re-arms the watch (the watch generation changes with the workspace refresh path). If a toggle does not re-arm, the stale capture only errs toward refreshing, because a `show_hidden=false` filter drops strictly more events. Task 2.2 confirms which case applies and re-arms on toggle if needed.

### 3. Where the filter runs
In the `notify` callback on the watcher thread, before `try_send`. Irrelevant events never wake the debounce loop. The debounce loop, the poll fallback, and manual Refresh are unchanged.

## Risks / Trade-offs

- [A backend reports a create or delete as `Modify(Any)`] → On Windows, creates and deletes have their own action codes; on Linux (inotify) and macOS (FSEvents), notify maps creates, deletes, and renames to their kinds. Any remaining gap is covered by manual Refresh, and unknown kinds fail open.
- [Entries hidden only by the Windows attribute (not dot-named) still trigger refreshes] → Accepted. They are rare in note vaults, and the refresh is correct, only unnecessary.
