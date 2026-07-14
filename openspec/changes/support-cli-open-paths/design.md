## Context

Markion currently starts through `src/main.rs` -> `app::run()`, installs GPUI menus/keybindings in `src/app/bootstrap.rs`, and constructs a `MarkionApp` with a welcome document. Startup deliberately does not scan the process working directory; the file tree is populated only after a real document or explicit folder is opened. Document opening and folder opening already have reusable flows: `replace_active_tab`, `open_file_in_new_tab_from_path`, `update_workspace_root_from_document`, `set_workspace_root`, and `schedule_file_tree_scan`.

This change adds one startup input channel: a single optional positional path from the command line. It should reuse the same Markdown-only file filter and workspace scan behavior as existing in-app paths, while preserving the derived Markdown caches, syntax-highlight memoization, cached text handles, undo snapshots, and bounded/off-main-thread file-tree scans.

## Goals / Non-Goals

**Goals:**

- Parse one optional positional path before GPUI starts.
- Resolve relative paths against the launch working directory.
- Open a supported Markdown file as the initial saved document, replacing the welcome tab rather than creating a second tab.
- Open a directory as the workspace root, reveal the Files sidebar, and scan it asynchronously while leaving the welcome document untouched.
- Surface invalid, missing, or unsupported startup paths through existing localized status feedback and logs.

**Non-Goals:**

- No flags, subcommands, globbing, or multi-path session restore.
- No file/folder creation when the requested path does not exist.
- No OS file association or installer integration.
- No new user-visible strings, localization keys, persisted workspace state, or supported Markdown extensions.

## Decisions

### 1. Represent startup input as a typed intent

Add a small pure helper, for example `StartupOpenIntent`, that is built from `std::env::args_os().skip(1)` before `Application::run`. It classifies the first positional argument into:

- `None` when no path is provided.
- `File(PathBuf)` when the resolved path exists, is a file, and matches `is_markdown_path`.
- `Folder(PathBuf)` when the resolved path exists and is a directory.
- `Invalid { path, reason }` for missing paths, unsupported file extensions, and other non-file/non-directory paths.

This keeps path parsing testable without launching GPUI. Taking only the first positional path matches the proposal's scope and avoids accidentally defining tab-restore behavior before the UX exists. Additional positional args are ignored for this change; a future change can define multi-file launch semantics deliberately.

### 2. Apply the startup intent after the window and app entity exist

Pass the intent into `app::run_with_startup_intent` (with `run()` delegating to `from_env_args()`) and capture it in the `open_window` setup closure. In the initial `window.update`, apply it before `check_recovery_on_startup`:

- `File(path)`: call `MarkdownDocument::open(&path)`, then `replace_active_tab(document, cx)`, `update_workspace_root_from_document(cx)`, clear `active_menu`, and set `StatusOpened` with the display path.
- `Folder(path)`: call `set_workspace_root(path)`, set `sidebar_visible = true` and `sidebar_tab = SidebarTab::Files`, persist preferences consistently with File -> Open Folder, and call `schedule_file_tree_scan(Some(display_path), cx)`.
- `Invalid`: keep the welcome document and empty tree, set an existing failure status (`StatusOpenFailed` for files/missing/unsupported, `StatusOpenFolderFailed` only if a directory scan later fails), and log the reason.
- `None`: preserve current behavior exactly.

Applying after the entity exists avoids changing `MarkionApp::new` into a fallible constructor and lets startup folder scans reuse the established async scan path. Applying before recovery means a requested file or folder is visible first; if a recovery prompt appears and the user restores, recovery still opens in a new tab as it does today.

### 3. Share helper behavior with existing open flows instead of duplicating policy

The command-line path should use the same primitives already used by File -> Open, File -> Open Folder, the file tree, and drag-and-drop:

- `is_markdown_path` remains the sole extension filter.
- `MarkdownDocument::open` remains the only document loader.
- `workspace_root_for_document` continues to decide the file root for opened documents.
- `schedule_file_tree_scan` continues to run traversal off the GPUI thread and apply stale-result checks.

This makes command-line startup another entrance into existing behavior rather than a parallel document/workspace model.

## Risks / Trade-offs

- [A startup file read can fail after argument parsing] -> Keep the app open on the welcome document, show `StatusOpenFailed`, and log the failing path and error.
- [Relative paths differ by launch context] -> Resolve against `env::current_dir()` immediately, before GPUI starts, and cover it with unit tests.
- [Folder scans can be slow] -> Reuse `schedule_file_tree_scan`, so traversal remains on the background executor and stale scan results are ignored.
- [Recovery prompt ordering can surprise users with pending recovery files] -> Apply startup intent first, then keep the existing recovery prompt behavior; restored recovery content opens in an additional tab instead of replacing the requested startup file.

## Migration Plan

No migration is required. Existing launches with no positional path behave the same as before. Rollback is removing the startup-intent parser and returning `app::run()` to the current no-argument startup path.

## Open Questions

None.
