## Why

The local WeChat publishing workspace already ships the pinned MarkNice themes, typography offsets, formatting commands, rich copy, and HTML/DOCX downloads, but its chrome still looks like a stripped utility page: Unicode toolbar glyphs, unlabeled font-size buttons, a max-width phone preview, and no Word import or print-to-PDF. Users who know the sibling MarkNice web editor therefore meet a different layout, icon language, and action set even though the article body is the same. Closing that gap in the editor surface — without bringing back Node, OCR, OSS, or the marketing site — makes the bundled workspace recognizable and complete for the remaining browser-local workflows.

## What Changes

- Restyle the editor-only workspace shell as a MarkNice **editor skin**: dual rounded cards, traffic-light panel headers, CSS-variable palette (indigo accent `#6366f1`), Inter / PingFang SC / Microsoft YaHei chrome fonts, SF Mono editor stack, SVG formatting and preview-mode icons, grouped font-size and paragraph-spacing steppers, and a 375px phone frame with notch. Visual fidelity targets the sibling MarkNice editor section as closely as the local workspace constraints allow.
- Keep the existing session-local / privacy disclosure; do not restore the marketing navbar, hero, features, guide, or footer.
- Add a **session-local Word (.docx) import** that runs entirely in the browser (pinned JSZip + the pinned MarkNice `docx-parser` / HTML-to-Markdown path). Import replaces only the workspace textarea and preview. It SHALL NOT write back to the Markion document. After a successful import, the workspace SHALL tell the user that recovery into Markion requires Copy Markdown or another explicit save of the session Markdown.
- Add **Save as PDF** that prints a clone of the current **themed, sanitized** publishing preview (including the selected theme and font-size / paragraph-spacing offsets), rather than MarkNice web's generic print stylesheet. The browser print dialog remains the save mechanism; no Node PDF engine is introduced. This PDF path is distinct from Markion's native/Pandoc PDF export.
- Relabel HTML/DOCX actions to match MarkNice wording where localization allows (`Save as HTML` / `Save as Word`), while keeping the existing safety, embedding, and offline-bundle contracts.
- Vendor the new browser-only runtimes (JSZip and the curated Word-import module) into the checked-in workspace bundle with provenance, digests, and licenses. Regenerating the manifest remains the maintainer refresh path; normal build, test, packaging, and runtime still require neither Node.js nor the sibling MarkNice checkout.
- Update bilingual user documentation so the workspace description covers the editor-skin chrome, session-local Word import (no write-back), and themed print-to-PDF.

**Non-goals:** importing Markdown files, importing PDF (including OCR), importing local images, sample/clear document actions, OSS temporary image upload, the MarkNice Node proxy, landing/guide/navbar chrome, bidirectional sync or writing browser edits back to `MarkdownDocument`, embedding a WebView, replacing Markion's native HTML/PDF/DOCX exporters, or changing the pinned article `theme-runtime` typography (already aligned).

This change does not touch per-version derived Markdown caches, syntax-highlight memoization, or cached text handles: Word import and PDF print stay inside the authenticated browser tab.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `wechat-publishing-workspace`: Requires the editor-skin chrome (layout, fonts, icons, typography controls, phone frame) to track the pinned MarkNice editor section, and adds session-local Word import that never mutates the Markion document. (This capability is pending sync from the unarchived `add-local-marknice-publishing-workspace` change; this delta is stacked on that implemented baseline and on `relax-runtime-workspace-verification`.)
- `wechat-workspace-content-export`: Adds Save as PDF from the current themed sanitized preview and aligns export control labels with the pinned MarkNice editor actions. (This capability is pending sync from the unarchived `add-marknice-workspace-content-exports` change; this delta is stacked on that implemented baseline.)
- `project-documentation`: Updates the bilingual README and local MarkNice workspace guide so they describe the closer editor chrome, session-local Word import with copy/save-MD recovery, and themed print-to-PDF as distinct from Markion native PDF.

## Impact

- Browser assets under `assets/marknice-workspace/` (shell HTML, `workspace.css`, formatting toolbar markup, bridge/export scripts, locales, self-tests, compatibility fixtures, bundle manifest, and third-party notices).
- New vendored files: pinned `jszip` browser distribution and a semantically extracted MarkNice Word-import runtime (`docx-parser` plus the HTML-to-Markdown conversion used after parse). `html-docx-js` already bundles a private JSZip; the import path still needs a public `JSZip` global and must not rely on CDN.
- Loopback service, snapshot allowlist, GPUI document model, and native Export-menu launch action stay unchanged except for any status/docs that mention the new workspace actions.
- User-facing workspace strings (Word import progress/success/failure, session-local recovery hint, PDF empty-content/print prompt, control labels, accessible names) are localized in every workspace locale. Markion `src/i18n.rs` changes only if a GPUI-visible string is added; none are expected.
- Package verification and the bundle verifier gain the new vendor files; installer size increases by the JSZip + parser payload.
- Invariants preserved: launching still snapshots `MarkdownDocument` once; Word import, skin changes, and PDF print never increment document version, dirty state, selection, undo history, or shared derived-cache identity.
