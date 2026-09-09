## 1. GPUI clipboard HTML model

- [x] 1.1 Extend `ClipboardItem` in the vendored GPUI (`vendor/zed/crates/gpui/src/platform.rs`) with an optional HTML representation: `html: Option<String>` field, `html()` accessor, `pub(crate)` setter for platform backends, a public `new_html(text, html)` constructor for tests, and `html: None` in every existing constructor and literal construction site (macOS, Wayland, Windows backends).

## 2. Platform clipboard readers

- [x] 2.1 Wayland (`platform/linux/wayland/clipboard.rs`): read the `text/html` offer mime alongside the text read and attach it to the returned item; image-only offers and self-owned clipboards are unaffected.
- [x] 2.2 X11 (`platform/linux/x11/clipboard.rs`): enable the `text/html` atom, and after a successful text read in `get_any`, attempt an HTML target read that fails quietly when unoffered (including when we own the selection).
- [x] 2.3 macOS (`platform/mac/platform.rs`): when the pasteboard has a string, also read `public.html` and attach it; image-only pasteboards unaffected.
- [x] 2.4 Windows (`platform/windows/clipboard.rs`): register `"HTML Format"`, read it when a text entry is present, and extract the fragment using the CF_HTML byte-offset header with a whole-payload fallback.

## 3. HTML→Markdown converter crate

- [x] 3.1 Add GUI-free `crates/html-import` (package `markion-html-import`, zero external dependencies, headless `cargo test -p markion-html-import`), wired into the root manifest with a `[profile.dev.package.markion-html-import]` override.
- [x] 3.2 Implement a tolerant tokenizer + tree builder: comments/doctype/processing instructions skipped, `script`/`style` raw-text skipped, void elements, mismatched-tag tolerance, `p`/`li`/`td`/`th`/`tr` auto-closing, and HTML entity decoding (named common set + numeric, `&nbsp;`).
- [x] 3.3 Implement the Markdown renderer: headings 1–6, paragraphs, `br` line breaks, bold/italic/strikethrough/underline/sub/sup (tags plus `font-weight`/`font-style`/`text-decoration` style attributes), safe-scheme links, referenced images (data-URI sources reduced to alt text), nested ordered/unordered/task lists with correct continuation indent, blockquotes, fenced code with language from `class="language-*"` and inline code with backtick-run sizing, GFM tables (first header row or empty header; merged cells fall back to one raw HTML table), and horizontal rules.
- [x] 3.4 Apply context-aware Markdown escaping (mirroring the docx-import policy, plus line-start markers and table pipes), HTML whitespace collapsing outside `pre`, adjacent-marker merging, and blank-line collapsing; output has no leading/trailing blank lines.
- [x] 3.5 Add converter tests: Word `MsoNormal`/mso-comment/`o:p` fixture, Google Docs styled-span fixture, ordinary web-page fixture, entities, nested lists with `start`, task lists, tables (GFM + merged-cell raw fallback + headerless), code fencing edge cases, escaping cases, CRLF input, and empty/non-content HTML.

## 4. Application integration

- [x] 4.1 In `MarkionApp::paste` (`src/app/editing.rs`), when no text input has focus and the clipboard item carries HTML, insert `markion_html_import::html_to_markdown` output through the existing checked-mutation path as one atomic undo step; fall back to plain text when conversion is empty; keep image paste, text-field paste, and plain-text-only paste byte-identical to today.
- [x] 4.2 Add an app-level test mirroring `pasted_clipboard_image_uses_managed_asset_and_one_undo`: clipboard item with text+HTML pastes as converted Markdown in one undo step, and a plain-text-only item still pastes verbatim.

## 5. Documentation and verification

- [x] 5.1 Mention rich-text paste in README/README.zh-CN feature lists (wording parity with existing entries).
- [x] 5.2 Run `cargo fmt`, clippy, `cargo test -p markion-html-import`, the root paste/editing tests, and `cargo test --workspace`; record any pre-existing unrelated failures explicitly.
- [ ] 5.3 Runtime clipboard verification on a live session (X11/Wayland paste from a real rich-text source, macOS `public.html`, Windows `CF_HTML`). NOT done in this change: the implementation environment is headless (no display, xvfb, xclip, or wl-paste), so platform readers are compile-covered and unit-covered only; record live-session evidence before claiming platform passes.
- [ ] 5.4 Run `openspec validate add-rich-text-paste` when the CLI is available (it is not installed in the implementation environment) and reconcile proposal/design/specs/tasks with the implemented scope; do not edit stable specs directly. Archive the change via the OpenSpec CLI after validation.
