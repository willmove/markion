## 1. Untitled image resolution

- [x] 1.1 Add a helper that locates a relative packaged path by trying `{current_exe parent}/`, then `{CARGO_MANIFEST_DIR}/`, returning the first existing file (same search order as `bundled_reference_doc_path`). Prefer `src/paths.rs` so preview and export can share it later.
- [x] 1.2 When `PreviewImageKey::from_url` receives a relative local URL and `document_dir` is `None`, use the CWD-relative path if that file exists; otherwise use the bundled helper. Leave saved-document resolution (join `document_dir`) unchanged. Do not reparse Markdown or change cache keys except for the resolved filesystem identity.

## 2. Welcome document content

- [x] 2.1 Replace `![Local image placeholder](markion-example.png …)` in `DEFAULT_WELCOME_MARKDOWN` with a Markdown image whose destination is `assets/markion.png` and whose alt text names the Markion logo.
- [x] 2.2 Add a compact HTML `<table>` section (one CommonMark HTML block, no inner blank lines) with header cells and at least one `colspan` or `rowspan`, using short Markion-oriented labels.
- [x] 2.3 Add a compact “other HTML” section covering `<strong>`/`<em>`, `<br>`, a list, `<kbd>` or `<code>`, and an `<img src="assets/markion.png" …>`. Keep attributed/unsupported tags out of the existing “Write with *italic*…” Visual Edit paragraph.

## 3. Tests

- [x] 3.1 Update `welcome_prose_stays_visual_outside_the_focused_block` and any other welcome-marker assertions to require `assets/markion.png`, HTML `<table`, a span attribute, and another HTML tag such as `<kbd>`, while keeping the “Write with” paragraph visually editable.
- [x] 3.2 Add a unit test that untitled resolution of `assets/markion.png` finds a real file through the bundled helper (and that a missing relative name still does not invent a path under `assets/`).

## 4. Verification

- [x] 4.1 Run the focused welcome/image tests (`welcome_prose_stays_visual_outside_the_focused_block` and the new resolution test) and confirm they pass without invalidating per-version derived caches.
- [x] 4.2 Manually launch on a clean untitled welcome tab: Markdown logo image displays; HTML table shows distinct cells/spans; other HTML tags render; Visual Edit still hides markers in the inline-formatting paragraph.
