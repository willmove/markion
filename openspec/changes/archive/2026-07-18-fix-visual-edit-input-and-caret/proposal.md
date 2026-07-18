## Why

Visual Edit renders source-backed Markdown blocks and maps pointer positions to source offsets, but it never registers a GPUI text input handler after replacing the source `EditorElement`. Normal character and IME input therefore cannot reach the existing document mutation path, while whitespace-only positions and cursor-driven navigation can also leave the visual caret without a rendered block or visible scroll target.

## What Changes

- Register one Visual Edit text-input bridge per rendered frame, including for empty documents, while keeping Read mode non-editable.
- Route character input and IME composition through the existing source-backed selection, undo/redo, dirty-state, autosave/recovery, and cache invalidation paths.
- Represent whitespace-only gaps and trailing editable positions in the visual model so every valid source caret position has a visual editing affordance.
- Track the active visual block and keep the virtualized Visual Edit list aligned with cursor moves, search results, and outline jumps.
- Provide Visual Edit-aware caret bounds for IME candidate placement where the active visual layout is available.
- Add GPUI input integration tests plus focused model/navigation tests so rendered Visual Edit behavior, rather than only source-range helpers, is verified.

Non-goals: full Typora-style WYSIWYG behavior, direct rich table-cell editing, removal of conservative source islands, or changing Markdown as the canonical persisted representation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Clarify that Visual Edit must accept platform text/IME input, expose whitespace caret positions, provide usable caret geometry, and keep the active edit position visible without invalidating document-version caches.

## Impact

- Affected code: Visual Edit elements and input registration in `src/app/preview.rs` and `src/app/root_view.rs`, mode-aware input geometry in `src/app/editor_element.rs`, visual list/caret state in `src/app/state.rs`, navigation in `src/app/application.rs` and `src/app/editing.rs`, and whitespace block construction in `src/visual.rs`.
- Tests: GPUI test support may be enabled for dev builds to simulate real text input; existing model tests remain in place.
- Architecture invariants: visual blocks remain cached per `MarkdownDocument.version()` and shared through `Arc`; cursor, focus, scroll, and IME-geometry changes must not reparse Markdown or invalidate derived caches.
- No persisted document, preferences, or public file-format migration is required.
