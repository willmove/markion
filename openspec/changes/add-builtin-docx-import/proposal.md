## Why

Markion can export Word documents, but receiving a Word manuscript currently requires conversion elsewhere or importing into the session-only MarkNice browser workspace and manually copying the result back. Native, offline DOCX import would let users migrate existing notes and manuscripts into durable Markdown documents with managed images, while making conversion losses visible before saving.

## What Changes

- Add File → Import Word (.docx) to the native and in-app menus. Convert one explicitly selected file in the background, show a conversion report, select a new Markdown destination, and open the saved result in a new tab.
- Introduce a GUI-free Rust DOCX import engine that returns Markdown, embedded image resources, and structured diagnostics. Evaluate a pinned `docx-rs` reader against a representative corpus before selecting it; retain a bounded ZIP/XML reader as the alternative if required structure or resource controls are unavailable.
- Preserve body text, headings, inline emphasis, hyperlinks, nested lists and numbering, ordinary tables, supported merged tables, common embedded images, and footnotes. Support an explicitly tested subset of native Word equations and import tracked changes as the accepted revision view with a disclosure.
- Detect unsupported or simplified content and distinguish informational normalization, formatting loss, content loss, and fatal errors. Content-loss cases require a concrete report and explicit continuation; unreadable documents never become successful imports.
- Save images under the target document's managed `.assets` directory, publish the Markdown only after its resources exist, and clean up import-owned files on handled failures or cancellation. Refuse overwriting existing documents or asset directories in this first version.
- Add real-producer compatibility fixtures, resource-limit and persistence tests, localized UI, and bilingual documentation distinguishing native import from browser-session import.

**Non-goals:** Editing or overwriting Word files; lossless DOCX round trips or page-layout fidelity; `.doc`, `.docm`, encrypted documents, PDF/OCR, batch import, drag/drop or OS file associations; a browser write-back endpoint; bundling Word, LibreOffice, Node, or Pandoc; changing existing exports or browser import behavior. Pandoc may be used as a development comparison tool, not a runtime backend in this change.

## Capabilities

### New Capabilities

- `docx-import`: Offline DOCX-to-Markdown conversion, supported semantics, explicit fidelity diagnostics, bounded background execution, native import interaction, and creation of a separate saved document.

### Modified Capabilities

- `document-resources`: Add staged DOCX resource persistence with safe relative references, create-only output, rollback of import-owned files, and Markdown-last publication.
- `project-documentation`: Document native Word import, its fidelity matrix and resource layout, and its distinction from the existing session-local MarkNice import.

## Impact

- New `crates/docx-import` member (`markion-docx-import`) plus root dependency wiring and an explicit dev-profile optimization override. Parsing-library versions and features will be recorded after the first compatibility task; no JS/GUI runtime or conversion service is introduced.
- Application integration around `src/app/documents.rs`, native menus in `src/app/bootstrap.rs`, actions and in-app menus, save-dialog profiles, and the existing `src/i18n.rs` entry point. Keep the import coordinator in its own module rather than expanding the editor's typing path.
- Extend `src/storage/resources.rs` and persistence helpers for staged imports without weakening existing paste/drop resource or ordinary save behavior. Existing `.md` documents and preferences need no migration.
- The imported Markdown remains the sole durable editing source. The importer has only a transient conversion model; existing tabs, dirty state, undo/redo, per-version `Arc` caches, syntax memoization, and cached text handles remain unchanged. The new tab enters the normal document lifecycle once after a successful save.
- Add a compatibility report and fixtures, a user-facing import guide, and equivalent English/Simplified Chinese README coverage. Preserve the browser workspace's one-way snapshot contract and current export preferences.
