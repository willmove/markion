## Context

See `proposal.md` — Why for the symptom. The mechanics that produce it:

- `comparable_document_path` (`src/app/state.rs`) is the app's single path-normalization boundary, meant as a **comparison key** (tab dedupe, workspace containment). It calls `std::fs::canonicalize`, which on Windows returns verbatim `\\?\`-prefixed paths.
- That canonical form is not only compared, it is **stored**: `set_workspace_root` keeps it as `workspace_root` and `FileTree.root` (tree entries are `root.join(name)`, so every entry inherits the prefix); `sync_and_persist_session` / `session_open_files_from_paths` / `record_recent_path` write it into `session.toml` (open files, active file, workspace root, recent files).
- On session restore, recent-open, and open-from-tree, `MarkdownDocument::open` stores whatever path it was given — verbatim included. Display surfaces (`CopyPath` clipboard + `StatusCopiedPath`, `RevealInFileManager` status + the `explorer /select,...` argument, image-tab path displays) then print `path.display()` verbatim.

Path data flow today (the leak is every arrow out of `comparable_document_path` into stored state):

```
OS dialogs ──normal form──► open document / workspace root
comparable_document_path (std::canonicalize ⇒ \\?\ on Windows)
   ├─► workspace_root + FileTree.root ──join──► tree entries ──► documents (verbatim)
   ├─► session.toml: open_files / active_file / workspace_root / recent (verbatim)
   └─► comparison keys (intended use)
documents ──tab.path──► CopyPath / Reveal / status messages (prints verbatim)
```

This change does not touch rendering or editing state: paths are identity metadata, none of this runs on the typing path, and derived-state caches are unaffected.

## Goals / Non-Goals

**Goals:**

- Every path the app stores on content, workspace state, or persisted session is in normal form, so all current and future display surfaces are healed transitively.
- Users with an existing verbatim-laden `session.toml` are healed on the first launch after upgrade.
- Comparison semantics (case/symlink normalization on Windows) remain exactly as strong as today.

**Non-Goals:**

- No per-display-site stripping (the stored form is fixed instead; see D1).
- No general long-path (> 260 chars) support work; verbatim is retained when genuinely required.
- No changes to the internal-only `canonicalize` sites that never reach UI or disk.

## Decisions

### D1 — Fix at the normalization boundary, not at display sites

Swap `comparable_document_path` to return normal-form canonical paths rather than stripping `\\?\` in `CopyPath`, status formatting, and the explorer argument.
*Why:* display-site stripping leaves verbatim paths stored on documents and in `session.toml`, and every future surface that prints a path re-introduces the bug. One boundary edit fixes all sources (workspace root → tree entries → documents; session persist; recent) and keeps a single place to test. `CopyPath` keeps reading `tab.path` directly; tests assert the invariant (stored paths are prefix-free) instead of papering over it at the last hop.
*Alternative rejected:* strip in each `path.display()` call site — whack-a-mole, and the clipboard would be clean while `session.toml` stays polluted.

### D2 — Use `dunce::canonicalize` rather than hand-rolled prefix stripping

`dunce` returns the canonical path without the verbatim prefix **only when the shortened form still resolves to the same file**; it keeps verbatim for paths that truly need it (over `MAX_PATH`, components with trailing dots/spaces). Unconditional `strip_prefix("\\?\")` would break file access for exactly those paths, and lexical normalization (`path-clean`-style) cannot resolve Windows case/symlink identity, weakening the comparison key. `dunce` is std-only with zero transitive dependencies and is the established community solution. On non-Windows it is a passthrough, so macOS/Linux behavior is byte-identical.
*Alternative rejected:* manual string stripping (unsafe for exotic paths); keeping std + a "display path" converter at every edge (D1 rejects the shape).

### D3 — Heal persisted state at session parse time

`parse_session_state` (`src/storage/session.rs`) sanitizes `open_files`, `active_file`, `workspace_root`, and `recent` entries with `dunce::simplified` (equivalence-checked prefix removal, identity on failure). One choke point heals all four fields on the first launch after upgrade; the next save rewrites them in normal form.
*Alternative rejected:* sanitize at each consumption site (`restore_session_on_startup`, recent menu, `open_recent_path`) — three call sites to keep in sync, easy to miss one.

### D4 — Leave internal-only canonicalize sites on `std::fs::canonicalize`

Git-dir resolution (`src/app/status_bar.rs:180`), file-tree containment validation (`src/storage/file_tree.rs` `safe_child_path` / `ensure_existing_path_within_root`), and image-cache identity (`src/app/preview_image.rs:38`) never reach user-visible strings or persisted state; verbatim is the most robust form there, and each already canonicalizes both sides of its comparison so mixed input forms compare correctly. Swapping them is churn with no behavioral gain.

## Risks / Trade-offs

- [`dunce` performs an equivalence check beyond std's canonicalize → slightly more work per call] → The boundary runs on open/restore/workspace-set/save paths, never per keystroke; the extra stat is noise. Noted as an invariant: keep `comparable_document_path` off hot loops.
- [Paths that genuinely require verbatim still display with the prefix] → Accepted and spec'd (correct file access wins); rare, and copy-paste of a verbatim path still works in shells that accept it.
- [Running instances upgraded in place keep verbatim tab paths until restart/reopen] → Session-load healing covers the restart; any newly opened file is normal-form immediately.
- [New dependency in the root crate] → std-only, no transitive deps, not passed to workspace members, so the crates/* no-gpui and profile-override invariants are untouched.

## Migration Plan

1. Add `dunce` to the root `Cargo.toml` dependencies.
2. Swap the canonicalize call inside `comparable_document_path`; no signature change.
3. Add sanitization to `parse_session_state` (and thereby `load_session_state`).
4. Regression tests; `cargo test --workspace`.
5. Rollback is a plain revert of those two edits — session files written in normal form during the interim remain perfectly valid inputs for the old code (normal-form paths canonicalize the same), so no data rollback is needed.
