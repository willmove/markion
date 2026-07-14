## Why

Users who launch Markion from a terminal or OS shell integration need to open a specific Markdown file or workspace folder directly, instead of starting on the welcome document and then using the in-app Open / Open Folder flows.

## What Changes

- Accept a single optional positional path when Markion starts from the command line, e.g. `markion notes.md` or `markion C:\Notes`.
- If the path is a Markdown file (`.md`, `.markdown`, or `.mdown`, case-insensitive), open it as the initial document and scan the appropriate file-tree workspace root.
- If the path is a directory, establish it as the file-tree workspace root, reveal the Files sidebar, and scan it with the existing Markdown-only tree behavior without replacing the welcome document.
- If the path is missing, keep the current startup behavior: show the in-memory welcome document and do not scan the working directory.
- Reuse existing open/open-folder status feedback and preserve off-main-thread file-tree scans plus per-document derived-state cache invariants.

Non-goals: parsing flags or subcommands, opening multiple positional paths, creating files or folders that do not exist, registering OS file associations, persisting the startup workspace across launches, or changing the Markdown extensions Markion accepts.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: Add command-line startup paths as a way to open an initial Markdown document or establish the file-tree workspace root.

## Impact

- `src/main.rs` / `src/app/bootstrap.rs`: capture startup arguments before GPUI launch and pass the resolved path into the initial window setup.
- `src/app/application.rs` / `src/app/documents.rs` / `src/app/workspace.rs`: reuse existing document-open and workspace-root flows so startup file opens focus an initial saved document and startup folder opens only the workspace tree.
- `src/app/state.rs` and tests: add focused path-classification / startup-intent tests for files, folders, missing paths, unsupported file types, and relative path resolution.
- No new dependency, storage format, public API, localization surface, or Markdown derived-state recomputation is expected.
