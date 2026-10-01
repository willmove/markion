## 1. Baseline, vendors, and sync markers

- [x] 1.1 Confirm the implemented `add-local-marknice-publishing-workspace` and `add-marknice-workspace-content-exports` workspace bundle as the baseline; record that this change is stacked on those unarchived capabilities and must archive after them.
- [x] 1.2 Inventory pinned MarkNice `styles.css` editor-section selectors, `docx-parser.js`, `htmlToMarkdown`, and the Word-import handler; record JSZip 3.10.1 identity, license, and the import size bound to use.
- [x] 1.3 Extend the maintainer sync flow with semantic markers for the editor-skin CSS regions and the Word-import runtime (`docx-parser` + `htmlToMarkdown` + import handler), failing on missing or duplicate markers.
- [x] 1.4 Vendor `jszip.min.js` 3.10.1 and `LICENSE.jszip.txt` under `assets/marknice-workspace`, update third-party notices, and prove normal build/test/packaging/runtime need neither npm/Node nor network access.

## 2. Editor skin chrome

- [x] 2.1 Port MarkNice editor-section CSS variables and card/header/toolbar/textarea/preview rules into `workspace.css`, including indigo accent, 16px radius, shadows, Inter / PingFang SC / Microsoft YaHei chrome fonts, and the SF Mono / Fira Code editor stack, without importing hero/navbar/features/footer.
- [x] 2.2 Restructure `index.html` into dual rounded cards with traffic-light panel headers, keep the session-local disclosure, and wire a compact light/dark control to `body[data-mode='dark']` initialized from `prefers-color-scheme`.
- [x] 2.3 Replace Unicode formatting buttons with MarkNice SVG pill icons (keep H1/H2/H3 text labels), add grouped font-size and paragraph-spacing steppers, and convert desktop/phone controls to SVG `.mode-btn` toggles so only one mode stays active.
- [x] 2.4 Implement the 375px phone frame with notch for phone preview and ensure desktop mode fills the preview card; print and export clones must still exclude that frame.
- [x] 2.5 Add self-tests for required skin class names, `.mode-btn` exclusive active state, 375px phone frame, SVG format-icon presence, and absence of marketing chrome and of Import MD / Import PDF / Import image / sample controls.

## 3. Session-local Word import

- [x] 3.1 Generate `marknice-word-import-runtime.js` from the marked MarkNice parser and HTML-to-Markdown regions, expose `JSZip` from the local vendor file, and load both under the existing self-only CSP.
- [x] 3.2 Add the Import Word file control to the left panel only, with localized label, accept filter, size bound, busy state, and no write-back endpoint.
- [x] 3.3 On success, replace the textarea, rerender, and show a localized session-local recovery hint naming Copy Markdown or another explicit Markdown save; on invalid/oversized/parse failure, keep the current Markdown and show an actionable error.
- [x] 3.4 Add self-tests for a representative `.docx` fixture (headings, lists, table, math, embedded image as data URI), rejection of non-docx and oversized files, no filesystem paths in imported Markdown, and unchanged Markion document invariants when the workspace import runs.

## 4. Themed print-to-PDF

- [x] 4.1 Add Save as PDF to the preview toolbar beside Save as HTML and Save as Word, using localized MarkNice-equivalent wording and an accessible name.
- [x] 4.2 Implement print-document construction from the existing export snapshot: clone the themed sanitized preview, apply only page-box/margin/image/break print CSS, set a sanitized title, and open the browser print dialog without rewriting theme inline styles.
- [x] 4.3 Exclude workspace chrome, disclosure banner, toolbars, and phone bezel from the print document; refuse empty content; report that the print dialog opened rather than that a PDF was saved.
- [x] 4.4 Add self-tests that the print clone retains theme and font/spacing offsets, omits chrome/bezel, contains no session token/loopback/blob/file URLs, and does not invoke a native/Pandoc PDF path.

## 5. Localization, bundle, and documentation

- [x] 5.1 Add seven-locale strings for Import Word, Save as PDF, recovery hint, size/parse errors, print-dialog prompt, empty-content, template/save-as labels, and accessible names, with locale-key parity tests. Change `src/i18n.rs` only if a GPUI-visible string is actually added.
- [x] 5.2 Relabel Copy Markdown / Save as HTML / Save as Word to match pinned MarkNice editor wording per locale without weakening HTML/DOCX safety contracts.
- [x] 5.3 Regenerate LF-normalized bundle digests and extend verification for JSZip, the Word-import runtime, skin assets, licenses, self-only script references, and absence of CDN/OCR/OSS URLs.
- [x] 5.4 Update `README.md`, `README.zh-CN.md`, and `docs/marknice-workspace.md` for editor-skin closeness, session-local Word import with copy/save-MD recovery, themed print-to-PDF versus native PDF, and explicit non-features (MD/PDF/image import, sample).

## 6. Quality and release evidence

- [x] 6.1 Run the automated workspace self-test with external networking disabled and verify skin/toolbar/phone frame, Word import success and failure, themed print clone, HTML/DOCX unchanged behavior, every locale, and no document-mutation side effects in the tab.
- [x] 6.2 Manually verify on Windows default browser: editor-skin closeness to MarkNice, Word import of a real `.docx`, print-dialog Save as PDF matching the themed preview, and that Markion’s open document is unchanged; record outstanding macOS/Linux evidence if deferred.
- [x] 6.3 Add or extend root/GPUI tests proving Word import and Save as PDF in the browser session do not mutate/save the Markion document or change version, selection, dirty state, undo history, syntax memoization, cached text handles, or derived-cache identities.
- [x] 6.4 Run formatting and lint checks, `cargo test -p wechat-workspace`, root-package tests, `cargo test --workspace`, the bundle verifier, and `openspec validate align-marknice-workspace-skin-and-exports` in strict mode; fix every deterministic failure before requesting archive.
