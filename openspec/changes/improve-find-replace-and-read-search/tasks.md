## 1. Search State and Matcher Foundations

- [x] 1.1 Add a UTF-8-safe `SearchFieldState` for buffer, caret, anchor/selection, and IME composition, with focused unit tests for insertion, selection replacement, movement, Backspace, Delete, and multibyte boundaries.
- [x] 1.2 Replace the untyped search match/current-index state with explicit source and Read-preview target variants, result states (idle, pending, invalid, no matches, ready), requested panel form, and a generation key covering tab identity, document version, mode, query, and options.
- [x] 1.3 Extract a shared compiled literal/regex matcher used by source and preview inputs so case sensitivity, Unicode, invalid-pattern, and zero-width behavior stay consistent without adding a dependency.
- [x] 1.4 Add preview-search run enumeration and matching helpers for headings, rich prose, lists, quotations, code, tables, footnotes, and visible HTML text while excluding decorative markers, hidden link destinations, image paths, and non-text atoms.
- [x] 1.5 Test that inline styling does not split a visible phrase, code-line-number presentation does not duplicate matches, run ordering is deterministic, and source-only Markdown syntax is absent from Read-preview results.

## 2. Real Find and Replace Field Editing

- [x] 2.1 Route committed text and marked-text/IME updates into the focused `SearchFieldState`, replacing the append-only search branch while leaving document, link-editor, file-tree, and inline-name routing unchanged.
- [x] 2.2 Route Left/Right/Home/End, shifted selection, Select All, Backspace, Delete, Cut, Copy, and Paste to the focused search field before any document action, and verify those commands cannot mutate the document.
- [x] 2.3 Route Enter and Shift+Enter to next/previous search navigation, Tab and Shift+Tab through available overlay controls, and Escape through the existing close path without inserting field or document line breaks.
- [x] 2.4 Implement a shaped single-line field element with pointer-to-caret placement, selection and caret painting, IME-safe focus behavior, and horizontal visibility for long values.
- [x] 2.5 Render only the exact field buffer inside the editable area; move Find/Replace identity to adjacent chrome and add regression tests proving fixed labels, colons, and placeholders are not part of empty or non-empty field text.

## 3. Source Search, Navigation, Replacement, and Highlighting

- [x] 3.1 Rework source refresh so every valid non-empty query immediately chooses the first match at/after the source caret with wraparound, while empty, invalid, and no-match results clear stale current state and highlights.
- [x] 3.2 Centralize next/previous wraparound and visible-pane reveal so toolbar actions, Enter/Shift+Enter, and F3/Shift+F3 share one transition path and never display `0/N` when matches exist.
- [x] 3.3 Update replace-current to use only the current source target, refresh against the new document version, and select the next surviving result; preserve replace-all as one undoable snapshot and disable both operations outside valid source-match states.
- [x] 3.4 Paint subdued highlights for all source matches and a stronger current treatment in the source editor without modifying cached document text, shaped-text handles, derived Markdown state, or syntax-highlight caches.
- [x] 3.5 Map visible source matches into Visual Edit projections for all/current highlighting and use the existing source-reveal/caret path for a current match that requires authored syntax to become visible.
- [x] 3.6 Add action/state tests for initial-current selection, wraparound, query/option generation changes, invalid regex, zero-width regex progress, replace-current continuation, replace-all undo, and stale target rejection.

## 4. Read-Mode Rendered-Text Search

- [x] 4.1 Select the Read-preview search domain only when current-version cached preview blocks are installed; expose a pending state while debounced blocks are stale and retrigger matching when the preview list advances without forcing a synchronous parse.
- [x] 4.2 Build ordered Read-preview targets from cached selectable runs and choose the first result at/after the top visible preview row, wrapping when necessary.
- [x] 4.3 Add preview-run all/current search highlighting separate from `PreviewSelection`, including correct translation of canonical code-block ranges into line-split rendering when code line numbers are enabled.
- [x] 4.4 Reveal the current Read result by scrolling the virtualized preview list to its owning block and exact highlighted range, without relying on hidden source-editor scrolling.
- [x] 4.5 Preserve query, replacement value, options, and requested panel form across mode changes; recompute the domain on entry/exit from Read, hide mutating controls in Read, and restore the requested Replace form on return to an editable mode.
- [x] 4.6 Handle Replace invocation in Read as Find-only focus plus localized external guidance, and add tests proving replace actions cannot change the document while the replacement buffer remains preserved.
- [x] 4.7 Add Read-mode coverage for literal, case-sensitive, regex, Unicode, styled phrase, code, table, visible HTML, link-label-versus-destination, navigation wrap, close/highlight clearing, and stale-preview pending transitions.

## 5. Overlay Feedback, Theme, and Localization

- [x] 5.1 Restructure the compact overlay into responsive Find and optional Replace rows with external field identity, current/total or state feedback, navigation, active option toggles, disabled actions, and close control while preserving absolute upper-right placement and workspace geometry.
- [x] 5.2 Add localized messages for field identity, navigation/option/close tooltips and accessibility labels, pending/empty/no-match/invalid states, replacement actions, and Read-mode replacement guidance in every supported language.
- [x] 5.3 Remove hard-coded `No query`, case-sensitive/case-insensitive status strings, and other user-visible English from the search path; extend all-language catalog/exhaustiveness tests for the new messages.
- [x] 5.4 Apply theme palette roles to field caret/selection, all/current match highlights, invalid treatment, active options, disabled controls, hover states, and guidance; verify theme switching updates an open overlay immediately.
- [x] 5.5 Verify the two-row Replace form remains usable without overlapping or shifting editor/preview content in narrow windows and variable-width interface languages.

## 6. Integration Verification and Documentation

- [x] 6.1 Add GPUI action tests covering field-focused keyboard and clipboard routing, Enter/Shift+Enter, Tab/Shift+Tab, Escape, document non-mutation, and query/replacement retention across close/reopen.
- [x] 6.2 Add integration tests covering Edit, Split Preview, Visual Edit, and Read domain selection, all/current highlighting state, visible-pane reveal, mode switching, and replacement availability.
- [x] 6.3 Update `docs/keyboard-shortcuts.md` with Enter/Shift+Enter, field Tab behavior, and the Read-mode Find/Replace distinction.
- [x] 6.4 Run `cargo fmt --check`, `cargo test`, and `cargo test --workspace`, fixing any regressions without weakening the cached-per-version or virtualized-rendering invariants.
- [x] 6.5 Manually verify English and Simplified Chinese overlays with IME input in every view mode, including a narrow window, a dark theme, invalid regex, no matches, long values, navigation, and replacement gating.
- [x] 6.6 Run `openspec validate improve-find-replace-and-read-search` and resolve every proposal/spec/design/task consistency error before implementation is considered complete.
