## Why

Pasting content copied from Word, Google Docs, browsers, or other rich-text sources currently inserts only plain text: Markion reads the clipboard's `text/plain` flavor and silently drops every formatting flavor (headings, bold/italic, links, lists, tables) that the source application offered. Editors like Typora convert the clipboard's `text/html` flavor into Markdown on paste, so users migrating content from Word or web pages keep their structure. Two gaps block this today: the vendored GPUI clipboard abstraction only models plain text and images (no HTML flavor is read on any platform), and the workspace has no Rust HTML→Markdown converter.

## What Changes

- Extend the vendored GPUI clipboard model with an optional HTML representation (`ClipboardItem::html()`) and read it in all four platform backends: macOS (`public.html` pasteboard type), Windows (`CF_HTML` with fragment-offset parsing), Wayland (`text/html` offer mime), and X11 (`text/html` selection target). Clipboard *writes* are unchanged.
- Add a GUI-free `markion-html-import` workspace member that converts clipboard HTML into Markion-compatible Markdown: headings 1–6, paragraphs and line breaks, bold/italic/strikethrough/underline/sub/sup (including style-attribute formatting used by Google Docs/Word), safe-scheme hyperlinks, images by reference, nested ordered/unordered/task lists, blockquotes, fenced and inline code, GFM tables with a merged-cell raw-HTML fallback, horizontal rules, and HTML entities — with context-aware Markdown escaping so pasted text cannot form unintended structures.
- Convert on paste: when the clipboard carries HTML alongside text, Edit → Paste on the document surface inserts the converted Markdown as one atomic undo step. Plain-text-only clipboards paste verbatim as before, text-input fields keep receiving raw text, image paste keeps the managed-asset flow, and an empty conversion result falls back to the plain text.
- Add converter fixtures (Word `MsoNormal`/mso-comment HTML, Google Docs styled-span HTML, ordinary web pages), and an app-level paste test covering conversion, plain-text fallback, and single-undo behavior.

**Non-goals:** Writing an HTML clipboard flavor on copy (Copy as HTML keeps writing a plain string); downloading or importing images referenced by pasted HTML; rich-text drag-and-drop; a separate "paste as plain text" action; RTF/RTFD flavors; changing the browser workspace (MarkNice) import path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Paste converts an offered HTML clipboard flavor to Markdown instead of dropping formatting; plain-text, image, and text-field paste stay unchanged.
- `crate-architecture`: New GUI-free member crate `crates/html-import` (`markion-html-import`) with root dependency wiring and a dev-profile optimization override.

## Impact

- Vendored GPUI patch (`vendor/zed/crates/gpui`): `ClipboardItem` gains an `html` field plus accessors; four platform clipboard readers populate it. This extends the existing `[patch.crates-io]` gpui arrangement; no new external dependency is introduced anywhere.
- New `crates/html-import` member plus root `Cargo.toml` dependency and `[profile.dev.package.markion-html-import]` override, per the crate-architecture spec.
- Application touch points: `MarkionApp::paste` (`src/app/editing.rs`) gains an HTML branch ahead of the plain-text branch; no menu, keybinding, or i18n changes are required (the Paste action itself is unchanged).
- The pasted Markdown enters the document through the existing checked-mutation + atomic undo path; derived-state caches, preview, and Visual/Source modes behave as for any other inserted Markdown.
- Platform verification: only the Linux backends compile and run in this environment; macOS/Windows reader code mirrors existing backend patterns and is covered by CI builds, not by local runtime verification (recorded as a task caveat, mirroring the docx-import change's platform-evidence policy).
