## 1. Normalization boundary

- [x] 1.1 Add `dunce` to the root `Cargo.toml` `[dependencies]` (std-only, no transitive deps); run `cargo build` and confirm `Cargo.lock` gains only the `dunce` entry.
- [x] 1.2 Switch `comparable_document_path` (`src/app/state.rs`) from `std::fs::canonicalize` to `dunce::canonicalize`, keeping the existing input-as-fallback on error; extend its doc comment to state the normal-form contract (no `\\?\` / `\\?\UNC\` prefix unless the path genuinely requires it).

## 2. Session-state healing

- [x] 2.1 In `src/storage/session.rs`, add a sanitize helper wrapping `dunce::simplified` and apply it in `parse_session_state` to `open_files`, `active_file`, `workspace_root`, and every `recent` entry, so legacy verbatim paths heal on load (identity on non-Windows and for paths that require the verbatim form).

## 3. Regression tests

- [x] 3.1 In `src/app/tests.rs`, test that `comparable_document_path` on an existing temp file returns a path without the `\\?\` prefix (string assertion is cross-platform safe: non-Windows never emits the prefix) and that a non-canonical input with different case/ separators still dedupes to the same key on Windows.
- [x] 3.2 Test the session sanitize path: `parse_session_state` over a `session.toml` fixture containing `\\?\`-prefixed open-file / active-file / workspace-root / recent entries returns prefix-free fields (`#[cfg(windows)]` for the stripping assertion), and normal-form input round-trips unchanged on all platforms.
- [x] 3.3 Add an app-level test following the existing workspace harness: establish a workspace root over a temp dir, scan, open a file through the tree, and assert the resulting tab's stored path — the string `Copy File Path` would write — has no verbatim prefix; confirm existing tab-dedupe and workspace-containment tests still pass unchanged.

## 4. Verification

- [x] 4.1 Run `cargo test --workspace`; on Windows, manually smoke-test: File → Open Folder, copy a tab's file path (expect `C:\...`), Reveal in File Manager, restart to confirm the healed session reopens tabs with normal-form paths. (Automated suite fully green — lib 346, bin 325+2 ignored, all members and doc-tests; the GUI smoke-test checklist was handed to the user for visual confirmation.)
- [x] 4.2 Run `openspec validate fix-verbatim-path-prefix --strict`.
