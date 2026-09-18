## 1. P0 — GFM tables from TSV and headerless HTML

- [x] 1.1 Add `tsv_to_markdown` in `crates/html-import` (rectangular ≥2-row TSV → GFM with first row as header; 1×1 and non-tabular return `None`; escape `|` and newlines) and verify with crate tests covering Excel-like CJK TSV, jagged rejection, and single-cell `None`
- [x] 1.2 Change headerless HTML table rendering to use the first data row as the GFM header (no empty `|  |  |` row) and verify by updating `headerless_table_gets_empty_header` plus a two-row Excel-`<td>` fixture
- [x] 1.3 Flatten `colspan`/`rowspan` into a rectangular GFM grid (origin value, remaining spanned cells empty) instead of raw HTML; keep nested tables as raw HTML; verify by replacing `merged_cells_fall_back_to_raw_html` expectations and keeping the nested-table test
- [x] 1.4 Add Excel / Google Sheets / WPS-shaped HTML fixtures (ProgId Excel.Sheet, `google-sheets-html-origin`, `xl*` class + `<col>`) that convert to GFM tables and verify `cargo test -p markion-html-import`

## 2. P0 — Paste orchestration

- [x] 2.1 In `MarkionApp::paste`, after HTML conversion (or when HTML is missing/empty), if the Unicode text is rectangular TSV insert `tsv_to_markdown` through the existing atomic undo path; verify with an app test that TSV-only clipboard pastes a GFM table in one undo and a single-cell clipboard stays plain text
- [x] 2.2 Add an app test that HTML+TSV spreadsheet clipboard pastes the GFM table (not raw TSV, not empty-header) and verify `pasted_clipboard_html_converts_to_markdown_with_one_undo` still passes

## 3. P1 — Word lists and encoding fallback

- [x] 3.1 Reconstruct `MsoListParagraph` / `mso-list` paragraphs into nested Markdown lists in `html-import` and verify with Word numbered, bulleted, and nested-level fixtures; real `<ul>`/`<ol>` tests stay green
- [x] 3.2 Confirm Windows CF_HTML invalid UTF-8 still omits HTML (no new decoder) and that paste then uses Unicode text + TSV conversion; add an app test with HTML `None` + CJK TSV if the clipboard test helper cannot inject invalid bytes, otherwise document that the reader already returns `None`

## 4. P1 — Paste as plain text

- [x] 4.1 Add `PastePlainText` action, `menu_shortcuts::PASTE_PLAIN` (`secondary-alt-v` / `Ctrl+Alt+V` / `Cmd+Option+V`), Edit menu item under Paste, keybinding, and i18n `ItemPastePlain` in every language plus the shortcut catalog; verify labels compile per language and `Ctrl+Shift+V` remains Toggle View Mode
- [x] 4.2 Implement paste-as-plain-text to insert Unicode text with no HTML and no TSV conversion (text fields and image paste unchanged) and verify an app test: HTML+TSV clipboard + PastePlainText yields tabs, while Paste on the same item yields a GFM table

## 5. Verification

- [x] 5.1 Run `cargo fmt`, `cargo test -p markion-html-import`, and the root paste/editing tests; record any pre-existing unrelated failures
- [x] 5.2 Run `openspec validate improve-rich-text-paste` and reconcile artifacts with implemented scope
