## 1. Status context projection

- [x] 1.1 Add a pure `StatusBarContext` projection in the root application crate that reads character and word counts once from the active document's version-cached statistics, derives the active caret's one-based line/column from `EditorTab::cursor_offset()`, and omits caret data in Read mode without mutating any document or cache state.
- [x] 1.2 Add focused tests for Unicode-scalar character counts, whitespace-delimited word counts, UTF-8 line/column values, forward and reversed selection caret ends, Read-mode omission, repeated same-version metric reuse, and active-tab context switching.

## 2. Asynchronous Git branch context

- [x] 2.1 Implement an isolated, fallible Git metadata resolver that walks parent directories, supports `.git` directories and relative/absolute `gitdir:` indirection files, returns complete symbolic `refs/heads/*` names, and returns no branch for detached, malformed, unreadable, or absent repositories; cover ordinary, nested, linked-worktree/submodule-style, detached, and missing repository fixtures with temporary-directory tests.
- [x] 2.2 Add app-level Git context state that chooses the active saved document's parent before an established workspace fallback, launches discovery only on the background executor, caches the result independently of Markdown state, and accepts a result only when its generation and requested context still match.
- [x] 2.3 Schedule Git context lookup when the active tab or workspace context changes and add a low-frequency background HEAD refresh that updates only changed branch values; test stale-result rejection, saved-document precedence, unsaved-workspace fallback, repository removal, and branch switching without putting filesystem work in render or input handlers.

## 3. Localized status-bar rendering

- [x] 3.1 Add compact branch, character, word, line, and column message keys and non-empty translations for every supported language in `src/i18n.rs`, keeping branch names and numeric values as interpolation arguments and extending localization tests where needed.
- [x] 3.2 Refactor the existing 28px status row in `src/app/root_view.rs` into a flexible clipped document/save/transient-feedback region and a non-wrapping persistent-context region; render counts in every mode, caret only in Edit/Visual Edit/Split Preview, and the branch only when cached, with bounded clipping for long branch names.
- [x] 3.3 Add app/render tests proving that save/export/search/error-style transient feedback remains present alongside persistent context, missing Git state creates no placeholder or error, long context stays single-row without overlap, and language changes reformat labels while leaving branch/document content unchanged.

## 4. Verification

- [x] 4.1 Run `cargo fmt --check`, `cargo test`, and targeted status/Git/i18n tests; fix all failures without weakening the per-document-version cache or background-I/O guarantees.
- [x] 4.2 Run `cargo test --workspace` and `openspec validate show-document-context-in-status-bar`; record completion only after every workspace member and OpenSpec validation pass.
