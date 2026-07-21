## Why

Mermaid diagrams already render as static images in Split Preview and Read mode, but Visual Edit (the WYSIWYG mode) shows their authored source as a plain code island — users see ` ```mermaid ` source instead of the rendered diagram. This breaks the WYSIWYG promise for one of the most visual block types in Markdown and forces users to switch modes to see what a diagram looks like. Visual Edit should show the rendered diagram, the same way it already shows rendered display math.

## What Changes

- Visual Edit renders a recognized diagram fence (`mermaid` and any other registered backend alias) as a static diagram image, reusing the existing async, theme-aware, memoized `markion-diagram` cache — no new rasterizer or backend code.
- The rendered diagram is presented above (or alongside) a **source-backed editable payload editor** so the user can still edit the diagram source inline. This mirrors how Visual Edit already renders display-math blocks (`visual_math_editor`): rendered output on top, editable source below.
- While a diagram is pending, Visual Edit shows a localized loading placeholder; on error it shows a localized error plus the authored source, matching Split Preview / Read mode behavior.
- Diagram rendering, completion, and theme switching in Visual Edit do **not** mutate the document text, increment the document version, invalidate Markdown-derived caches, or reparse the document.
- The existing invariant that diagram fences remain source-backed (canonical Markdown source is the only editing path) is preserved — Visual Edit gains a *presentation* of the diagram, not a second editable tree node.

## Capabilities

### New Capabilities

<!-- None — this change extends an existing capability. -->

### Modified Capabilities

- `diagram-rendering`: The "Diagram blocks remain source-backed in Visual Edit" requirement is revised. Visual Edit will now *present* the rendered diagram image (with pending/error states) on top of an editable source payload, instead of falling back to the raw code island. The source-preservation invariants (no document mutation, no new editable tree, diagram completion cannot overwrite document state) are retained and sharpened.

## Impact

- **Code**: `src/visual.rs` (`visual_block_editor` — attach a `VisualBlockEditor::Code` to diagram fences instead of returning `None`); `src/app/preview.rs` (new `visual_diagram_editor` view mirroring `visual_math_editor`, plus dispatch change in `visual_block_view`); `src/app/diagram.rs` (extend `ensure_diagram_renders` to walk `&[VisualBlock]` in addition to `&[PreviewBlock]`, matching how `ensure_math_renders` already walks both); `src/app/root_view.rs` (pass visual blocks to `ensure_diagram_renders` in Visual Edit mode instead of an empty `Vec`).
- **Existing tests touched**: `src/visual.rs::unclosed_and_diagram_fences_remain_complete_source_islands` currently asserts diagram fences have `editor = None`; this assertion is updated to reflect that the block now carries a `Code` editor while remaining source-backed (source range and island kind unchanged).
- **Invariants touched**: diagram-rendering's "source-backed in Visual Edit" requirement (revised, not removed); the document-mutation / version-isolation invariants of the diagram cache are unchanged but re-asserted for the Visual Edit path.
- **Dependencies**: none new. Reuses `markion-diagram`, the existing `DiagramCache`, and existing GPUI static-image rendering.
- **Non-goals**: adding new diagram backends; changing how Mermaid renders in Split Preview / Read mode; live re-rendering on every keystroke without dedupe (cache + existing debounce already handle this); introducing a non-source-backed visual tree node for diagrams.
