## 1. Session storage model

- [x] 1.1 Add `SessionState` (workspace root, open files, active file, recent files) in `src/model.rs` with defaults and a recent-files bound constant
- [x] 1.2 Add `default_session_path()` in `src/paths.rs` pointing at `session.toml` under the config directory
- [x] 1.3 Implement `src/storage/session.rs` load/save (TOML, optional fields, best-effort IO) and export it from `src/storage/mod.rs`
- [x] 1.4 Add unit tests for load defaults, round-trip, recent-list dedupe/cap helpers, and missing-file tolerance

## 2. Persist session during editing

- [x] 2.1 Hold `SessionState` on `MarkionApp` and load it during app construction alongside preferences
- [x] 2.2 Add `persist_session()` and call it when workspace root changes, path-backed tabs open/save/close, active path-backed tab changes, or recent files change
- [x] 2.3 Record recent files (and session open/active paths) from existing open/save/save-as/close/tab-switch flows without recomputing Markdown derived caches

## 3. Restore session on launch

- [x] 3.1 Implement `restore_session_on_startup` that re-establishes a valid workspace root (async scan) and reopens surviving Markdown tabs through existing open/tab APIs
- [x] 3.2 Wire startup order: preferences → CLI `StartupOpenIntent` → session restore (skipped for conflicting CLI fields) → existing recovery prompt
- [x] 3.3 Update empty-state behavior so a restored or CLI workspace root does not show the “no workspace” placeholder
- [x] 3.4 Add tests covering restore with missing paths, CLI override precedence, and untitled tabs omitted from the snapshot

## 4. Open Recent menu and localization

- [x] 4.1 Add i18n `Msg` keys for Open Recent, empty placeholder, Clear Recent Files, and related status strings in every supported language
- [x] 4.2 Add Clear Recent Files action/handler and Open Recent entries to the in-window File dropdown after Open / Open Folder
- [x] 4.3 Open a chosen recent path through the existing open-document/reuse-tab flow; on missing path, show localized failure and prune the entry
- [x] 4.4 Add focused tests for menu wiring / recent open / clear behavior where practical (string or unit-level)

## 5. Verification

- [x] 5.1 Run `cargo test` for the root package and fix regressions related to session/recent changes
- [x] 5.2 Run `openspec validate persist-session-and-recent-files` and ensure tasks reflect completed work
