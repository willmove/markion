## Context

Markion already writes `session.toml` (`src/storage/session.rs`) with a single `workspace_root`, `open_files`, `active_file`, `recent_files`, and optional `[layout]`. Launch restore (`restore_session_on_startup`) reopens surviving **Markdown** paths and scans the last root. File → Open Recent lists files only. The Files-panel workspace-name header (`file-tree-workspace-root`) is a muted label and drop target, not a picker.

`persist-session-and-recent-files` implemented that global snapshot and listed “recent folders as a separate menu” as a follow-up. `add-open-folder` still says Open Folder must not replace the active document. This change keeps one session file and one window, but stores **per-workspace tab snapshots** and treats an explicit folder choice as a workspace switch.

Constraints: session IO stays off the keystroke path; documents still open through existing tab APIs so per-version Markdown caches, syntax-highlight memoization, and cached text handles are unchanged; dirty work stays on the crash-recovery path; `crates/*` stay GUI-free.

## Goals / Non-Goals

**Goals:**

- Persist a bounded recent-workspace list, each with ordered path-backed tabs and an active path.
- Restore the **current** workspace (root + its tabs) on launch when there is no CLI file/folder intent.
- Let the user pick a recent workspace from the Files header (and from the empty state) and get that folder’s last tab list back.
- Treat File → Open Folder as the same explicit switch (remembered root → restore snapshot; new root → empty welcome session).
- On explicit switch, keep dirty tabs on the tab bar; close only clean tabs. Save / Don't Save / Cancel remains on close-tab and quit.
- Keep implicit rebase (open a file outside the current root) as tree-only: do not replace the live tab set.

**Non-Goals:**

- Cursor, selection, scroll, undo, view-mode, or file-tree expand/collapse persistence.
- Per-workspace chrome layout or multi-root workspaces.
- File → Open Recent Folders submenu (header + empty state are the switchers).
- Folding session data into `config.toml` or wiping it on Preferences reset.

## Decisions

### 1. One `session.toml`, array of workspace snapshots

Keep the existing file and atomic write. Add a bounded `workspaces` list (most recent first, cap `MAX_RECENT_WORKSPACES = 10`, same order of magnitude as `MAX_RECENT_FILES`). Each entry:

```toml
[[workspaces]]
root = "D:/Notes"
open_files = ["D:/Notes/a.md", "D:/Notes/notes.txt", "D:/Notes/cover.png"]
active_file = "D:/Notes/a.md"
```

Continue writing top-level `workspace_root` / `open_files` / `active_file` as aliases of the **current** (first) snapshot so older readers and the existing layout-load path keep working. `recent_files` and `[layout]` stay global.

**Alternative considered:** a separate `workspaces.toml`. Rejected — launch already has one session file; a second file doubles failure modes without a product split.

**Alternative considered:** only `recent_folders = ["…"]` plus one global `open_files`. Rejected — switching would not restore per-folder tabs.

### 2. Snapshot membership is path-backed and in-root

A workspace slot records only tabs that have a filesystem path **and** sit inside that workspace root (Markdown, curated text, and supported images, in live tab order). Untitled / welcome / recovery-only tabs are omitted. `active_file` is the active path only when it is in that in-root list.

Foreign tabs (open files outside the current root after an implicit rebase) may stay on screen but are **not** written into the current slot. Cursor, selection, scroll, dirty text, and undo are never stored; restore re-reads disk through existing open APIs.

**Alternative considered:** record every live path-backed tab on the current slot. Rejected — an implicit rebase would copy Folder A’s files into Folder B’s snapshot.

**Alternative considered:** keep today’s Markdown-only restore. Rejected — the product ask is “the tab list from that time,” which includes text and image tabs already shown in the tree.

### 3. Two ways to change the root

```
                    ┌─────────────────────────┐
                    │  Change workspace root  │
                    └───────────┬─────────────┘
                                │
              explicit          │           implicit
     (header / empty-state /    │     (open file outside
      Open Folder)              │      current root)
              │                 │              │
              ▼                 │              ▼
     1. Write current slot      │     1. Write current slot
        (in-root tabs only)     │        (in-root tabs only)
     2. Close clean tabs only   │     2. Open the file as today
     3. Keep dirty tabs; open   │     3. Rebase tree to parent
        target snapshot paths   │     4. Do NOT load the new
     4. set_workspace_root +    │        root’s stored tabs
        async scan              │     5. Touch new root as
     5. Touch target as current │        current; merge only
                                │        newly contained paths
                                │        into that slot
```

**Explicit switch — remembered root:** close only clean tabs; keep every dirty tab (named or untitled) with its text, undo, and recovery snapshot; open the target slot’s surviving paths in order (reuse an already-open tab for the same path, including a kept dirty tab); focus `active_file` when present among the resulting tabs. If no tabs remain (every previous tab was clean and every snapshot path is missing), leave a welcome tab. Skip missing paths and prune them on the next persist.

**Explicit switch — new root (not in the list):** same keep-dirty / close-clean rule, then an empty slot for that root. Show a welcome tab only when no tabs remain.

**Selecting the already-current root:** close the picker; no tab churn.

**Cancel (Open Folder picker):** workspace root, tree, tabs, dirty state, and undo stay unchanged. There is no dirty prompt to cancel on switch.

**Alternative considered:** keep Open Folder’s “do not touch documents” rule and only replace tabs from the header picker. Rejected — choosing the same folder from two entry points would disagree.

**Alternative considered:** treat implicit rebase as a full switch. Rejected — opening one outside file would close the whole tab set.

### 4. Dirty tabs stay open; Save / Don't Save / Cancel waits for close or quit

Explicit switch must not discard or force-save unsaved work. Match Close Others’ keep-dirty default, not quit:

- Close every **clean** tab through the existing close path.
- Leave every **dirty** tab on the tab bar, including dirty untitled / welcome tabs, with text, undo, dirty flag, and recovery snapshot intact.
- Do **not** show Save / Don't Save / Cancel on switch. That prompt stays on close-tab and quit / window close.
- When opening the target snapshot, reuse an already-open tab for the same path (`focus_existing_tab_for_path`) so a kept dirty tab is not duplicated or reloaded from disk.

The restored folder’s list still comes back; leftover dirty tabs from the previous folder may sit beside it until the user closes them or quits. Snapshots still store paths only — unsaved buffer text is never written into `session.toml`.

**Alternative considered:** prompt Save / Don't Save / Cancel on switch (like quit). Rejected — switching folders is not leaving the app; the user asked to keep dirty tabs until they close a tab or quit.

### 5. Launch order stays the same, payload is the current slot

Preferences → CLI `StartupOpenIntent` → session restore → recovery prompt.

- No CLI intent: restore the current workspace root (if it is still a directory) and that slot’s surviving path-backed tabs (not Markdown-only).
- CLI file or folder: skip conflicting document / workspace-root restore; still load recent workspaces and recent files into memory; still apply `[layout]`.
- Legacy file with no `[[workspaces]]`: synthesize one slot from top-level `workspace_root` / `open_files` / `active_file`.

Background existence probes stay off the UI thread (same stall-avoidance as today’s restore).

### 6. Files header is the switcher; File → Open Recent stays files

When a root is established, the workspace-name header opens a localized menu: current folder (marked), other recent roots (display name, most recent first), separator, Open Folder. The header remains a workspace-root drop target; opening the menu is a distinct click that does not start a drag.

When there is no root, the empty state lists recent workspaces (same explicit-switch action) or the existing placeholder if the list is empty.

File → Open Recent and Clear Recent Files are unchanged. No native-menu recent-folder list in this change (same GPUI dynamic-menu caution as the original recent-files work).

### 7. Caching / versioning

Switching closes only clean tabs through the existing close path and opens targets through `MarkdownDocument::open` / image-tab load (or focuses an already-open tab). Each newly opened document gets a fresh version and builds derived caches once, as any other open does. A kept dirty tab is not reloaded and does not increment its document version. Session serialize/write is not on the typing path. File-tree scans stay on the background executor; bounded row rendering is unchanged.

## Risks / Trade-offs

- [Explicit switch leaves mixed live tabs] → Accepted. Dirty tabs stay until close-tab or quit; persist only in-root paths so the next snapshot for the new root stays clean.
- [Implicit rebase leaves mixed live tabs] → Accepted. Persist only in-root paths so the next explicit switch is clean.
- [Stale or missing snapshot paths] → Skip on restore; prune on next write; localized status when the user picks a vanished folder.
- [Large tab restore] → Bound by the previous snapshot size; sequential existing open APIs; tree scan stays async.
- [Header click vs drag-to-root] → Require a click (not a drag) to open the menu; keep `can_drop` / `on_drop` on the header.
- [Unarchived `persist-session-and-recent-files` delta still describes one global Markdown-only snapshot] → This change’s workspace specs are the new contract; do not re-implement the old global-only restore.

## Migration Plan

- Load: if `workspaces` is missing or empty, build at most one slot from legacy top-level fields; heal verbatim `\\?\` prefixes on roots and tab paths the same way as today’s session paths.
- Save: write `[[workspaces]]` plus current-slot aliases; unknown extra keys ignored.
- Rollback: ignore `[[workspaces]]` and keep reading top-level fields; leftover array is harmless.
- Preferences reset still does not delete `session.toml`.

## Open Questions

None. Product choices (path+folder only; explicit switch restores tabs; implicit rebase does not) are fixed in the proposal.
