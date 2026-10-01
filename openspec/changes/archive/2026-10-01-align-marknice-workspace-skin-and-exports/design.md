## Context

See `proposal.md` for motivation. The implemented `add-local-marknice-publishing-workspace` and `add-marknice-workspace-content-exports` baselines already give Markion an authenticated, offline, editor-only MarkNice tab: one-way snapshot handoff, pinned themes and typography offsets, Markdown formatting, WeChat rich copy, Copy Markdown, and themed HTML/DOCX downloads. Those changes deliberately omitted MarkNice's marketing chrome, Word/PDF import, print-to-PDF, and CDN-loaded JSZip. The remaining gap is therefore the **editor surface** (layout, fonts, icons, preview chrome) plus two browser-local actions that do not need Node: Word import and themed print-to-PDF.

The sibling MarkNice editor at `D:\Coding\EditorProjects\marknice` remains the visual and Word-import source of truth. Its article `theme-runtime` is already vendored; this change must not fork heading/paragraph/code styles. Caching and versioning stay untouched: Word import and PDF print mutate only the browser textarea/preview, never `MarkdownDocument` or per-version derived caches.

```text
Markion MarkdownDocument (unchanged)
        │ launch snapshot, once
        ▼
browser textarea ◀── Word import (JSZip + docx-parser + htmlToMarkdown)
        │
        ├─ formatting toolbar (existing)
        ▼
themed sanitized preview
        │
        ├─ Copy Markdown / HTML / DOCX (existing)
        └─ Save as PDF ──▶ hidden iframe print of THAT preview clone
                           (theme + font/spacing offsets preserved)

No arrow returns to MarkdownDocument.
```

## Goals / Non-Goals

**Goals:**

- Make the workspace editor section visually track the pinned MarkNice editor as closely as a local, non-marketing shell allows.
- Import `.docx` entirely in the browser into the current session, with an explicit recovery hint (Copy Markdown or otherwise save the session Markdown) and no write-back.
- Print a clone of the current themed, sanitized preview as PDF, closer to the WeChat preview than MarkNice web's generic print stylesheet.
- Keep the workspace offline, capability-scoped, and release-verifiable after adding JSZip and the Word-import runtime.

**Non-Goals:**

- Import Markdown files, PDF (including OCR), or local image pickers; sample and clear actions.
- OSS temporary image upload, Node proxy, landing/guide/navbar/hero/footer.
- Writing browser Markdown back into Markion, a WebView, or a new loopback mutation endpoint.
- Replacing or visually matching Markion's native/Pandoc PDF and DOCX exporters.
- Changing pinned article theme tokens in `theme-runtime.js`.

## Decisions

### 1. Port an editor skin, not the MarkNice site

Copy the **editor-section** visual system from pinned MarkNice `styles.css` into `workspace.css` as CSS variables and component rules: `--accent: #6366f1`, `--radius: 16px`, card border/shadow, traffic-light `panel-header`, `panel-toolbar`, pill `format-btn` + SVG icons, grouped `font-size-ctrl`, SVG `preview-mode-toggle`, 375px phone frame with notch, Inter / PingFang SC / Microsoft YaHei chrome stack, SF Mono / Fira Code / Menlo / Consolas editor stack.

Keep Markion-only chrome that the web app does not have: compact top status row, session-local/privacy disclosure banner, unresolved-image warning. Do not import hero, features, footer, skip-link marketing, or the site navbar. A compact light/dark control may live in the workspace header and MUST drive the same `body[data-mode='dark']` tokens as MarkNice, initializing from `prefers-color-scheme` so the editor cards match without the marketing nav.

Left-panel tools in this change: existing Markdown format toolbar plus **Import Word** only. Right-panel tools: theme select labeled to match MarkNice (`Template` / locale equivalent), font-size and spacing steppers, desktop/phone icons, Copy to WeChat, Copy Markdown, Save as HTML, Save as PDF, Save as Word.

Phone-mode buttons MUST use the MarkNice `.mode-btn` / `.active` contract (the current shell is missing `.mode-btn`, so switching back to desktop can leave a stale active state).

Alternatives considered:

- **Pixel-copy `index.html` from MarkNice:** would reintroduce landing content, CDN tags, PDF-OCR and image-upload controls. Rejected.
- **Keep the flat two-column utility layout and only swap icons:** fails the requested closeness. Rejected.

### 2. Word import stays a browser-session replacement

Vendor `jszip@3.10.1` (`dist/jszip.min.js`) as a checked-in `static/vendor/jszip.min.js`. Extract MarkNice `docx-parser.js` and the `htmlToMarkdown` conversion (the region currently marked around `// ===== HTML to Markdown converter =====` through Word import) into a generated `marknice-word-import-runtime.js` using the same semantic-marker sync used for format/Word-export runtimes. The public `JSZip` global is required; do not attempt to reuse the private copy inside `html-docx.js`.

On file selection:

1. Refuse empty/non-`.docx` input with a localized error.
2. Bound the file size before `arrayBuffer()` (same order of magnitude as existing export image budgets; fail closed with a size message).
3. `parseDocx` → HTML (including numbering, merged cells, OMML→LaTeX, media as data URIs) → `htmlToMarkdown` → set textarea → synchronous render.
4. Show success plus a recovery hint: the import lives only in this tab; bring it into Markion via Copy Markdown or by saving the session Markdown elsewhere.
5. Never POST the file or Markdown to the loopback service.

Word-embedded images become data URIs in session Markdown. They are not managed loopback resources, so the existing copy-without-local-images gate does not apply to them. Preview, HTML, DOCX, and themed PDF may include those data URIs subject to existing export size fallbacks. Data URIs MUST NOT encode filesystem paths.

Alternatives considered:

- **Write imported Markdown back into Markion:** violates the one-way snapshot invariant and would dirty/version the GPUI document. Rejected.
- **Reuse Markion/Pandoc DOCX import if any:** there is no equivalent in-process Word→Markdown path, and it could not run inside the authenticated tab without a new mutation API. Rejected.

### 3. PDF prints the themed sanitized preview, not a generic article stylesheet

Save as PDF reuses the export snapshot builder (cancel debounce, sync render, clone sanitized preview). It then loads that clone into a hidden iframe whose print CSS only supplies page box, body margin, image max-width, and `break-inside` hints. It MUST NOT rewrite heading/paragraph/code colors or font-sizes the way MarkNice web currently does. Inline theme styles, font-size offset, paragraph-spacing offset, and rendered math SVG travel with the clone.

Print the article only: workspace chrome, disclosure banner, phone bezel, and toolbars stay out of the print document. Filename/`document.title` follow the existing export title sanitization. Status wording reports that the print dialog was opened and that the user should choose “Save as PDF”; JavaScript cannot prove a PDF was written.

This path is labeled as a browser print-to-PDF of the MarkNice preview and is distinct from **Export → PDF** in Markion.

Alternatives considered:

- **Port MarkNice web's generic print stylesheet unchanged:** worse fidelity to the WeChat preview, which is the user's actual goal. Rejected.
- **Generate a PDF blob in-browser (e.g. html2pdf):** extra vendor, font embedding, and WeChat-CSS impedance. The print dialog is what MarkNice already uses and needs no Node. Rejected.
- **Call Markion's native PDF writer:** would require posting browser-edited themed HTML back to the process. Rejected.

### 4. Keep bundle, CSP, and isolation contracts

New scripts load under the existing self-only CSP. Manifest, licenses (`LICENSE.jszip.txt`), and the verifier gain JSZip plus the import runtime. Sync/check continues to fail on missing semantic markers. No new HTTP routes. Root/GPUI tests (or an extension of the existing unchanged-state assertions) MUST show that Word import and PDF print inside a fake/browser session do not change document bytes, version, selection, dirty flag, undo, or derived-cache identity.

## Risks / Trade-offs

- [Editor-skin CSS copied from MarkNice can drift on the next bundle refresh] → Import skin rules by documented selectors/markers in the maintainer sync notes; golden self-tests assert class names, phone-frame width, and icon presence rather than screenshot pixels.
- [JSZip + Word parser increase installer size and XSS surface] → Pin exact bytes, parse only inside the tab, sanitize via the existing MarkNice render path before preview/copy/export, bound input size, never send the file to loopback.
- [Huge data-URI images from Word can exhaust tab memory or fail WeChat paste] → Size gate on import; reuse export fallbacks; keep the existing local-image copy policy for loopback blobs; document that Word-embedded images are session data URIs.
- [Print output still depends on the browser's print engine and user-chosen paper] → Disclose print-dialog semantics; do not claim pixel-identical PDF across browsers; verify Windows first, then other default browsers as release evidence.
- [Users may assume Word import updated the Markion file] → Persistent session-local banner plus a post-import recovery hint naming Copy Markdown / save-MD.

## Migration Plan

1. Implement against the current checked-in workspace bundle; do not wait for the two baseline changes to archive, but carry their outstanding manual evidence forward and archive this change after those capabilities exist in `openspec/specs/`.
2. Add skin CSS/HTML/icons, Word-import vendor+runtime, PDF print path, locales, self-tests, manifest/licenses, and documentation in one additive bundle refresh.
3. No stored Markion document or preference migration. Rollback restores the previous shell/CSS/bundle files; active tabs vanish when the process exits.

## Open Questions

None. Visual closeness, session-only Word import with a copy/save-MD hint, and themed print-to-PDF are decided in the proposal.
