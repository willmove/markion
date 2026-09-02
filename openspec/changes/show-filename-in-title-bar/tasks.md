## 1. Window-title projection

- [x] 1.1 Add a pure `window_title(title, is_dirty)` helper in `src/app/status_bar.rs` that returns `Markion - {title}` and appends ` *` only when `is_dirty` is true, reusing `EditorTab::title()` / `title_from_path` rather than parsing document text.
- [x] 1.2 Add unit tests for saved filenames, `Untitled.md`, dirty vs clean documents, and image-style titles with `is_dirty = false` (no `*` suffix).

## 2. Native title sync

- [x] 2.1 Track the last applied window-title string on `MarkionApp` and add a `sync_window_title` that calls `window.set_window_title` only when the desired caption differs, using the active tab title plus document dirty state (image tabs never dirty).
- [x] 2.2 Invoke that sync from the root-view render path so tab switches, Save / Save As, rename, and dirty transitions update the native caption without touching Markdown caches, highlight memoization, or text handles.
- [x] 2.3 Add an app-level test that activating another tab or toggling dirty state updates the window title and that a no-op render does not require a title change.

## 3. Status-bar identity removal

- [x] 3.1 Change `status_bar_feedback` so document tabs render `{save_state} | {status}` and image tabs render `{status}` only; stop passing brand, file name, and dirty marker into the status-bar feedback region in `src/app/root_view.rs`.
- [x] 3.2 Update status-bar tests so they assert identity tokens are absent from feedback while save-state, transient messages, and persistent context (counts, caret, Git) still render on a single clipped row.

## 4. Documentation

- [x] 4.1 Update `README.md` and `README.zh-CN.md` so chrome identity is the native window title after the Markion brand, and the status bar is no longer described as the document-identity surface.
- [x] 4.2 Align `docs/faq.md` so the unsaved-changes `*` suffix is documented on the title bar next to the file name.

## 5. Verification

- [x] 5.1 Run `cargo fmt --check` and the targeted window-title / status-bar tests; fix failures without weakening per-document-version cache invariants.
- [x] 5.2 Run `cargo test --workspace` and `openspec validate show-filename-in-title-bar`; record completion only after both pass.
