## Why

Visual Edit still drops fenced code, block math, images, and tables into coarse source islands even though each has a bounded, byte-exact source structure. Adding dedicated editors for those proven structures closes the largest remaining gap between the rendered result and the editing surface without abandoning canonical Markdown or introducing a second document model.

## What Changes

- Replace ordinary fenced-code source islands with a direct code editor that keeps the fence chrome out of the text surface, preserves the authored fence and info string, and shows memoized syntax highlighting while editing the exact code payload.
- Replace block-math source islands with a rendered formula plus an explicit LaTeX editor whose validation, IME, selection, and undo behavior remain source-backed.
- Replace Markdown image source islands with an image presentation and direct alt text, URL, and optional title controls; invalid or unavailable images retain an exact source fallback.
- Make Visual Edit table cells directly editable while retaining the existing row/column toolbar, alignment markers, deterministic GFM serialization, and source-mode commands.
- Introduce a small block-editor mutation protocol: widgets emit one exact `VisualBlockEdit`, and the application owns selection, history, dirty state, autosave, recovery, focus transfer, and structural navigation.
- Keep HTML, front matter, registered diagrams, malformed/ambiguous constructs, and any block whose source structure cannot be proven exact as source-backed islands.
- Add keyboard, pointer, IME, accessibility, lossless round-trip, multi-tab, virtualization, and stale-widget regressions for every direct editor.
- Non-goals: a mutable rich-text document tree, Markdown normalization outside the edited construct, nested rich content in table cells, image upload/cropping, HTML visual editing, or direct editing of rendered diagrams.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Refine the WYSIWYG-oriented fallback rule so exact complex blocks use dedicated direct editors and only ambiguous structures remain raw source islands.
- `code-and-math`: Add source-lossless direct Visual Edit widgets for ordinary fenced code and block math while registered diagram fences remain source-backed.
- `tables-outline`: Require direct cell editing in the Visual Edit grid and preserve alignment/toolbar/source-command behavior.

## Impact

- Affects `VisualBlock` source metadata, visual block rendering and input routing under `src/visual.rs`, `src/app/preview.rs`, `src/app/editor_element.rs`, and `src/app/editing.rs`, plus table/image/fence parsing helpers in the root crate.
- Adds no persisted format and no new runtime dependency; Markdown text remains the only saved representation.
- Direct widgets must use the existing versioned `Arc` derived caches, stable `VisualBlockId`, semantic undo boundaries, per-tab interaction state, preview debounce, memoized highlighting/math/diagram caches, and virtualized `ListState` contracts.
- Registered diagram fences retain the existing `diagram-rendering` source-island requirement and are deliberately excluded from ordinary code direct editing.
