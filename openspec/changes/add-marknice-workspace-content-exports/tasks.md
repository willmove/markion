## 1. Prerequisite and pinned source boundary

- [x] 1.1 Confirm the implemented `add-local-marknice-publishing-workspace` baseline and its strict validation; document its outstanding cross-platform/browser/WeChat manual evidence, carry it forward as a release/archive prerequisite, and reconcile both changes against the archived workspace and release-packaging specs before release.
- [x] 1.2 Inventory the pinned MarkNice Markdown-formatting, Copy Markdown, HTML-download, Word-preparation, and DOCX-download source sections and record the exact `html-docx-js` version, distribution identity, license, and Microsoft Word compatibility target used by this change.
- [x] 1.3 Update the maintainer sync flow to select new MarkNice formatting/export code by stable semantic markers or an explicit curated module, fail on missing/duplicate markers, and add regression coverage that prevents numeric source-line drift.
- [ ] 1.4 Vendor the pinned browser DOCX distribution and license under `assets/marknice-workspace`, update third-party notices, and prove normal build, test, packaging, and runtime need neither npm/Node nor network access.

## 2. Shared browser export snapshot and safety boundary

- [x] 2.1 Add Copy Markdown, themed HTML, and browser DOCX controls to the editor-only workspace shell with accessible names, per-action busy state, and responsive toolbar layout.
- [x] 2.2 Implement an operation-local export snapshot builder that cancels pending debounce, synchronously renders the exact current textarea value, clones the sanitized preview, and captures the current title, safe filename, theme, typography offsets, locale, and resource metadata without mutating the live editor or preview.
- [x] 2.3 Implement deterministic title and cross-platform filename sanitization with launch-name fallback, control/invalid-character removal, length bounds, and exactly one target-format extension; add edge-case tests for empty, Unicode, dotted, and platform-invalid titles.
- [x] 2.4 Implement the durable-artifact sanitization pass for scripts, authored styles, handlers, frames/plugins, forms, meta refresh, unsafe URL schemes, workspace-only attributes, session data, and unconverted loopback/blob/file references.
- [x] 2.5 Retain a bounded protected-resource-to-Blob mapping and implement sequential data-URI conversion for managed images with per-image and aggregate export limits, canonical revalidation when refetching, safe alt-text fallback, fallback counts, and no allowlist widening.
- [x] 2.6 Add shared browser tests for current-state capture, stale-debounce prevention, safe filenames, active-content removal, URL-scheme filtering, token/path/reference leakage, remote-resource classification, image embedding/fallback limits, and unchanged live workspace state.

## 3. Exact Markdown copy

- [x] 3.1 Port the pinned Copy Markdown behavior to copy the exact current textarea value as `text/plain`, using the preferred browser clipboard API plus a verified fallback and reporting success only after completion.
- [x] 3.2 Add tests for session edits versus launch snapshots, preserved whitespace and line endings as exposed by the textarea, empty content, preferred/fallback success, permission denial, and no mutation of editor text, selection, preview, or presentation settings.

## 3A. P0 Markdown formatting toolbar and shortcuts

- [x] 3A.1 Extract the pinned MarkNice selection/caret formatting command layer semantically into a generated runtime and make marker drift fail synchronization.
- [x] 3A.2 Add the responsive, accessible toolbar for H1/H2/H3, bold, italic, underline, lists, code, link, quote, code block, image syntax, and table; exclude local-image upload.
- [x] 3A.3 Add Ctrl/Cmd+B, Ctrl/Cmd+I, Ctrl/Cmd+U, and Ctrl/Cmd+K handling plus complete seven-locale labels, placeholders, and accessibility metadata.
- [x] 3A.4 Add deterministic self-tests for selection toggles, empty-selection templates, line formatting, shortcuts, immediate preview refresh, and browser-session-only state; document the behavior and packaged runtime provenance.

## 4. Portable themed HTML download

- [x] 4.1 Implement the standalone UTF-8 HTML wrapper with escaped title, responsive viewport, restrictive artifact CSP, current prepared MarkNice article, and no workspace chrome or application script.
- [x] 4.2 Embed the bundled KaTeX stylesheet and required WOFF2 font data so rendered math and themed article output remain usable after Markion exits and without a CDN or loopback asset request.
- [x] 4.3 Implement the `.html` Blob download with delayed object-URL revocation, empty-content refusal, remote-resource disclosure, managed-image fallback reporting, and wording that accurately reports download initiation rather than disk-save completion.
- [x] 4.4 Extend the HTML compatibility corpus and browser tests for all MarkNice themes, font/spacing offsets, headings, lists, tables, code, links, math, managed images, remote images, unresolved resources, offline reopen, CSP inertness, and absence of session/local references.

## 5. Browser-generated MarkNice DOCX

- [x] 5.1 Port the pinned MarkNice Word preprocessing helpers for soft breaks, list alignment, compact tables/cells, Word-oriented styles, bounded image dimensions, and current-session title/filename handling into the workspace export module.
- [x] 5.2 Integrate the locally bundled DOCX converter to generate the document entirely in the browser from the prepared export clone, embed managed data images, retain only permitted authored HTTP(S) references, and start a `.docx` download with safe object-URL cleanup.
- [x] 5.3 Add structured package tests that unzip generated DOCX fixtures and verify required package parts, content type, safe relationship targets, embedded image parts, absence of loopback/blob/file/token strings, and deterministic failure for missing or malformed converter output.
- [x] 5.4 Add normalized DOCX compatibility fixtures for headings, paragraphs, nested lists, blockquotes, inline formatting, links, tables, soft/hard breaks, code, math, managed images, remote images, and resource fallbacks.
- [x] 5.5 Add localized browser-generated-versus-native DOCX labeling and documentation of the Microsoft Word target plus best-effort LibreOffice, Pages, web-viewer, and OS-preview compatibility.

## 6. Localization, bundle verification, and documentation

- [x] 6.1 Add labels, accessible descriptions, progress, success, empty-content, clipboard, remote-resource, image-fallback, compatibility, and failure strings for English, Simplified Chinese, Traditional Chinese, Japanese, French, German, and Spanish with locale-key parity tests.
- [x] 6.2 Regenerate the bundle manifest with LF-normalized digests and extend verification for the export module, DOCX runtime, licenses, embedded-asset closure, self-only runtime references, and absence of CDN/hosted converter URLs.
- [x] 6.3 Update source-tree and staged/package verification so Windows NSIS, macOS application/DMG, Linux DEB, and Linux AppImage include the same verified export assets and exclude npm archives, Node code, credentials, temporary files, and generated article artifacts.
- [x] 6.4 Update user and maintainer documentation for exact Markdown recovery, themed HTML portability, remote-resource behavior, managed-image embedding/fallback, browser DOCX compatibility, the distinction from Markion native DOCX, and semantic MarkNice refresh steps.

## 7. Quality and release evidence

- [x] 7.1 Run the automated browser self-test with external networking disabled and verify the formatting toolbar/shortcuts, all three export actions, every locale, representative themes/content, clipboard preferred/fallback paths, HTML offline reopen, DOCX package inspection, resource limits, and active-content/token leakage defenses.
- [ ] 7.2 Manually verify Copy Markdown and HTML/DOCX downloads in the supported default browsers on Windows, macOS, Linux X11, and Linux Wayland, including permission denial, cancelled downloads, repeated actions, session expiry, and large-image fallback; record evidence and bundle/package size deltas.
  - Windows evidence (2026-08-23): the user reports that Copy Markdown and browser-generated Word export are OK. HTML, denial/cancel/repeat/session/image edge cases, and other supported browser/platform combinations remain outstanding.
- [ ] 7.3 Open representative generated DOCX files in the documented Microsoft Word targets, inspect formatting and embedded images, record best-effort results for available non-Word viewers, and verify the UI never presents browser output as native/Pandoc DOCX.
  - Windows evidence (2026-08-23): the user reports the exported Word result is OK; the Word version, secondary viewer, and structured formatting/image compatibility matrix are not yet recorded.
- [ ] 7.4 Add or extend root/GPUI tests proving successful, failed, and cancelled browser export workflows do not mutate/save the Markion document or change version, selection, dirty state, undo history, syntax memoization, cached text handles, or existing derived-cache identities.
- [ ] 7.5 Run formatting and lint checks, `cargo test -p wechat-workspace`, root-package tests, `cargo test --workspace`, the bundle verifier, staged/package workspace verification, and the repository quality gate; fix every deterministic failure.
- [x] 7.6 Run `openspec validate add-marknice-workspace-content-exports` in strict mode and update all completed checkboxes and release evidence before requesting archive.
