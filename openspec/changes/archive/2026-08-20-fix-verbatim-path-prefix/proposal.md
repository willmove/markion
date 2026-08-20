# Strip Windows verbatim (`\\?\`) path prefixes from user-facing and persisted file paths

## Why

On Windows, `Path::canonicalize()` returns a *verbatim* path carrying the extended-length prefix `\\?\` (e.g. `\\?\C:\Workspace\Vaults\articles\AGENTS.md`). Markion's path-identity helper `comparable_document_path` (`src/app/state.rs`) uses `std::fs::canonicalize` for tab dedupe / workspace-containment comparisons — which is correct — but that canonical form then leaks into paths the app **stores and shows**: the workspace root and file-tree entries joined onto it, `session.toml` (open files, active file, workspace root, recent files), and consequently the `MarkdownDocument` paths opened through those flows. Users see the raw `\\?\` prefix when copying a file path (tab context menu → Copy File Path), in the status feedback that follows, and in Reveal-in-File-Manager feedback — the path is also passed verbatim to `explorer /select,...`, which does not reliably accept it.

## What Changes

- Normalize canonical paths to **platform-normal form** at the single source: `comparable_document_path` switches from `std::fs::canonicalize` to `dunce::canonicalize`, which returns the identical path minus the `\\?\` / `\\?\UNC\` prefix whenever the shortened form is provably equivalent, and keeps the verbatim form only when the path genuinely requires it (e.g. beyond `MAX_PATH`). Non-Windows platforms are a passthrough. This heals every leak source at once: stored workspace roots, file-tree entry paths, session-persisted paths, recent files, and document paths opened from those flows.
- Sanitize **already-persisted** state on session load (`parse_session_state` / `load_session_state` in `src/storage/session.rs`): existing users' `session.toml` files contain verbatim-prefixed entries today; strip the prefix (via `dunce::simplified`) from open files, active file, workspace root, and recent entries so sessions heal on the first launch after upgrade.
- Comparison semantics are unchanged: every path comparison in the app already routes both sides through `comparable_document_path`, so tab dedupe, workspace containment, and session-restore focus behavior behave exactly as before.

Non-goals: no changes to tab identity/dedupe rules, workspace containment logic, file-tree scanning behavior, or any UI strings (paths are data, not localizable text); no new path-display surfaces; no long-path (> 260 char) support work beyond what `dunce` already preserves.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `chrome-platform`: add a requirement that file paths shown to the user or written to the clipboard are presented in platform-normal form — on Windows without the `\\?\` verbatim prefix — and that persisted session state bearing legacy verbatim paths is healed to normal form on load.

## Impact

- `Cargo.toml` — add the `dunce` crate (std-only, no transitive dependencies; it simplifies verbatim prefixes on Windows and is an identity function elsewhere). No `gpui` dependency, so workspace-member invariants are untouched.
- `src/app/state.rs` — `comparable_document_path` implementation swap; the one-line helper stays the app's single normalization boundary.
- `src/storage/session.rs` — sanitize open-files / active-file / workspace-root / recent entries during session parsing.
- `src/app/tests.rs` — regression tests (normalization output has no verbatim prefix; session-load sanitizes verbatim entries; existing tab-dedupe / workspace-containment tests keep passing).
- Invariants preserved: no derived Markdown state is touched (paths are identity metadata, not typing-path work); internal-only canonicalize call sites that never reach the UI (git-dir resolution, file-tree containment validation, image-cache identity) are left on `std::fs::canonicalize` deliberately.
