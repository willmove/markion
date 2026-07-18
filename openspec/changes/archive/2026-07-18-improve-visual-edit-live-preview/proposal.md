## Why

Visual Edit can now accept platform input, but focusing an otherwise rendered paragraph or heading still replaces the entire block with raw Markdown source. That mode switch is visually disruptive and falls short of a Live Preview experience, especially for ordinary inline formatting and list-oriented writing.

## What Changes

- Keep supported prose blocks rendered while editing, revealing only the Markdown markers or link destination needed at the active caret or selection.
- Introduce a source-to-display projection that can mix rendered content and temporarily revealed source ranges while retaining exact canonical-source mappings for pointer hit testing, selections, IME, and mutations.
- Add source-backed structural Enter and Backspace behavior for headings, blockquotes, ordered/unordered lists, and task lists so common block transitions feel native in Visual Edit.
- Preserve conservative full-source edit islands for ambiguous nested inline syntax and unsupported or complex constructs.
- Add rendered GPUI regression coverage for marker reveal, caret mapping, structure transitions, undo/redo, and cache/version invariants.
- Depend on the completed `fix-visual-edit-input-and-caret` change for the Visual Edit input bridge, whitespace coverage, caret geometry, and one-shot row reveal foundation.
- **Non-goals:** native rich widgets for tables, images, fenced code, math, HTML, or front matter; a second rich-text document model; marker-free editing when an exact source mapping cannot be proven.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Add progressive inline marker reveal and structure-aware block editing requirements to the existing source-backed Visual Edit mode.

## Impact

- Affects the visual source model in `src/model.rs` and `src/visual.rs`, Visual Edit rendering and hit testing in `src/app/preview.rs`, and mode-aware editing commands in `src/app/editing.rs`.
- Extends per-tab Visual Edit interaction state without changing persisted Markdown or file formats.
- Preserves `MarkdownDocument.text` as the only canonical representation and keeps preview/outline/stats/visual blocks cached per document version; cursor-only marker reveal must not invalidate those caches.
- Uses existing GPUI test support and does not add runtime dependencies.
