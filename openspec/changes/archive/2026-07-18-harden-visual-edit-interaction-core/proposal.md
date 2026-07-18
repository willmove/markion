## Why

Visual Edit now supports local Markdown reveal and source-backed structural editing, but caret boundaries, wrapped-line navigation, IME composition feedback, and undo grouping still behave more like a projected source editor than a polished WYSIWYG surface. Markion needs an explicit product contract that Visual Edit should stay as close as safely possible to the rendered result while preserving exact Markdown source and conservative fallbacks.

## What Changes

- Clarify that Visual Edit is Markion's source-backed WYSIWYG-oriented authoring surface: it prefers direct rendered editing, reveals only the smallest exact source syntax needed for the active operation, and falls back to source islands only when a lossless mutation cannot be proven.
- Add explicit caret affinity at collapsed projection boundaries so the editor can distinguish the inside and outside of hidden inline syntax without ambiguous typing or arrow-key behavior.
- Make Visual Edit Up/Down and Home/End navigation follow painted wrapped lines and retain a preferred horizontal position across visual lines and adjacent blocks.
- Project and paint IME marked ranges in Visual Edit, keep candidate-window geometry aligned with the active composition, and treat one composition session as one undoable edit.
- Coalesce compatible consecutive text input into semantic undo groups while keeping paste, formatting, structural edits, selection replacement, and mode changes as undo boundaries.
- Add pure mapping/navigation tests and rendered GPUI regressions for UTF-8 text, hidden markers, wrapped lines, cross-block movement, IME, undo/redo, and document-version cache invariants.
- **Non-goals:** incremental Markdown parsing, stable cross-version visual block identities, direct table-cell/image/code-block widgets, a second mutable rich-text model, multi-cursor editing, or changing Edit/Split/Read rendering semantics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Make the WYSIWYG-oriented Visual Edit direction explicit and add requirements for affinity-aware caret behavior, layout-aware navigation, visible IME composition, and semantic undo grouping.

## Impact

- Affects the source/display projection model in `src/model.rs` and `src/visual.rs`, Visual Edit layout and pointer mapping in `src/app/preview.rs`, navigation and history behavior in `src/app/editing.rs` and `src/app/state.rs`, and IME integration in `src/app/editor_element.rs`.
- Adds per-tab interaction state for caret affinity, preferred horizontal navigation position, active visual layout snapshots, and pending IME/typing undo groups.
- Preserves `MarkdownDocument.text` as the only canonical document representation. Cursor, navigation, composition geometry, and undo grouping state must not increment the document version or invalidate the existing `Arc`-shared derived Markdown caches unless source text actually changes.
- Does not add runtime dependencies or change persisted Markdown, preferences, or file formats.
