## Context

Markion already persists UI preferences in `config.toml` (including sidebar visibility/tab) and writes crash-recovery copies for dirty documents. It does **not** persist the file-tree workspace root or open document tabs across launches: startup always begins with the welcome document and an empty file-tree placeholder unless a CLI open intent or recovery prompt intervenes.

Users expect desktop-editor continuity: reopen the last folder in the Files panel, restore saved Markdown tabs, and reach recent files from File → Open Recent. This change adds a small session/recent persistence layer beside preferences, without folding session paths into `config.toml` or changing Markdown derived-state caches.

## Goals / Non-Goals

**Goals:**

- Persist and restore the last file-tree workspace root so the left Files panel returns to the previous folder on launch.
- Persist and restore open *saved* Markdown document tabs and the active-tab path.
- Maintain a bounded recent-files list and expose File → Open Recent (+ Clear Recent Files).
- Define a clear restore order relative to preferences, CLI `StartupOpenIntent`, and crash recovery.
- Keep all new user-visible strings localized.

**Non-Goals:**

- Restoring unsaved/untitled buffer text, dirty flags, undo history, cursor, selection, scroll, or view mode.
- Persisting file-tree collapse/expansion, filter query, or sidebar width (root restore is the required folder continuity).
- Recent folders as a separate menu (recent files plus workspace-root restore cover the requested UX).
- Multi-window or cloud-synced sessions.
- Changing Preferences reset to wipe session/recent data (optional later); preferences reset continues to cover `config.toml` fields only.

## Decisions

### 1. Store session + recent files in a dedicated TOML file, not `config.toml`

Add `session.toml` under `default_config_dir()` (same directory as `config.toml`), owned by a new `src/storage/session.rs` module. Shape (conceptual):

```toml
workspace_root = "D:/Notes"
open_files = ["D:/Notes/a.md", "D:/Notes/b.md"]
active_file = "D:/Notes/b.md"
recent_files = ["D:/Notes/b.md", "D:/Other/c.md"]
```

Rationale: preferences are user settings; session/recent are workspace continuity. Keeping them separate avoids bloating Preferences reset/migration and matches the existing recovery-vs-preferences split. Alternatives considered: embedding under `[session]` in `config.toml` (rejected: couples launch restore to preference save paths and reset semantics); JSON (rejected: TOML already used for config).

Missing/invalid fields default to “no session / empty recent.” Paths are stored as absolute UTF-8 path strings after normalization used elsewhere in the app.

### 2. Session payload is path-only for saved documents

Only document tabs with a real file path participate in `open_files` / `active_file`. Untitled welcome tabs and recovery-only tabs are omitted. On restore, each path is opened only if it still exists and is a Markdown file; missing paths are skipped and pruned from the next persist. If every tab is missing but a workspace root remains, restore the folder scan alone and keep the welcome document.

This avoids inventing a second recovery format and keeps crash recovery as the sole owner of unsaved text.

### 3. Startup order: preferences → CLI intent → session restore → recovery prompt

1. Load preferences (unchanged).
2. If CLI `StartupOpenIntent` is File or Folder, apply it and **do not** restore conflicting session pieces (CLI file replaces initial document/tabs; CLI folder sets workspace root). Still load recent files into memory for the menu.
3. Else if a session file exists, restore workspace root (async scan) and reopen surviving open files; focus `active_file` when present among restored tabs.
4. Then run the existing recovery prompt (`check_recovery_on_startup`) so dirty-buffer recovery still works on top of the restored session.

Empty-state rule becomes: show the placeholder only when there is **no** restored or CLI-established workspace root (and no in-session root yet).

### 4. Persist opportunistically on session-affecting mutations

Write `session.toml` when the workspace root changes, when a saved document is opened/saved-as/closed, when the active tab changes among path-backed tabs, and when the recent list changes. Debounce is optional; writes are small. Prefer best-effort IO (log and continue on failure) so persistence never blocks editing or derived-state updates.

Cap `recent_files` at a fixed bound (e.g. 10). Opening or successfully saving a Markdown path moves it to the front and deduplicates. Clear Recent Files empties the list and persists.

### 5. Open Recent is in-window File menu first; reuse existing open path

Add an Open Recent submenu (or nested section) to the in-window File dropdown after Open / Open Folder. Each entry opens that path through the same reuse-existing-tab / open-document path used by File → Open and the file tree. If the path is missing, show localized failure status and remove it from recent. Also add Clear Recent Files.

Native OS menu parity is best-effort in the same change if `install_menus` can host a dynamic list; if GPUI native menus cannot easily rebuild dynamic items, ship the in-window submenu first and keep native File menu without a dynamic recent list until a follow-up. Spec will require the in-window surface.

### 6. No impact on Markdown derived-state caches

Session restore opens documents through existing `MarkdownDocument::open` / tab APIs, so preview/outline/stats caches build once per restored document version as today. Session IO never runs on the keystroke path.

## Risks / Trade-offs

- [Stale paths after moves/deletes] → Skip missing files on restore; prune from session/recent on next write; surface localized open-failed status for Open Recent.
- [CLI vs session surprise] → Documented precedence: CLI wins for the requested file/folder; recent list still loads.
- [Large tab restore cost] → Bound by previous session size; opens are sequential via existing APIs; tree scan stays on the background executor.
- [Native menu dynamic items awkward] → Prioritize in-window File → Open Recent; native parity optional if the menu API makes dynamic rebuilds fragile.
- [Writing session on every tab switch] → Accept small TOML writes; if noisy, debounce later without changing the on-disk schema.

## Migration Plan

- No migration from older releases: absence of `session.toml` means current welcome + empty tree behavior.
- Rollback: stop reading/writing `session.toml` and hide Open Recent; leftover file is harmless.
- Preferences reset does not delete `session.toml` in this change.

## Open Questions

None blocking implementation. Optional follow-ups: persist collapse state; recent folders submenu; preference to disable session restore.
