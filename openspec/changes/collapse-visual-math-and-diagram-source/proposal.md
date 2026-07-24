## Why

In Visual Edit, block math and Mermaid diagrams always show a rendered preview stacked above an editable source payload. That permanent dual pane is visually noisy—especially when the formula or diagram is already correct—and duplicates information the user only needs while editing. An Obsidian-style collapsed presentation (render-only by default, expand source on demand) keeps the reading surface clean while preserving the existing direct payload editors.

## What Changes

- **BREAKING** (spec-level): Visual Edit block math no longer always shows the LaTeX payload editor together with the formula. Default presentation is the rendered formula (or pending/error chrome) only.
- **BREAKING** (spec-level): Visual Edit registered diagram fences no longer always show the source payload editor beneath the diagram. Default presentation is the rendered diagram (or pending/error chrome) only.
- On hover, a compact source-toggle control (`</>`-style) appears in the top-right of the block chrome for exactly ranged block-math and registered diagram fences.
- Clicking that control expands the block to the current dual presentation: rendered result above, monospaced payload editor below. Clicking the formula/diagram surface itself does not expand source.
- Clicking outside the expanded block collapses it back to render-only. While the caret owns the payload editor, the block stays expanded until the pointer click lands outside the block.
- Invalid or pending renders force the payload editor to remain available (auto-expanded / non-collapsible) so the user can always correct authored source.
- Split Preview and Read mode stay render-only (unchanged). Ordinary non-diagram fenced code editors are unchanged.

## Non-goals

- Inline math source-toggle UI; Typora-style replace-formula-with-raw-`$...$` on focus; editing via a rendered formula/diagram tree; changing HTML/export math or diagram output; collapsing ordinary (non-diagram) code blocks.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `code-and-math`: Replace always-visible block-math dual pane with collapsed-by-default + hover `</>` expand, click-outside collapse, and forced expand for invalid/pending.
- `diagram-rendering`: Apply the same collapsed/expand/collapse contract to registered diagram fences in Visual Edit.

## Impact

- **`src/app/preview.rs`**: Refactor `visual_math_editor` / `visual_diagram_editor` into collapsed vs expanded presentations; add hover affordance and outside-click collapse handling.
- **Tab / presentation state**: Per-tab expanded set keyed by `VisualBlockId` (and document version), presentation-only—must not mutate `MarkdownDocument.text`, dirty flag, undo, or derived `Arc` caches.
- **`openspec/specs/code-and-math/spec.md`**, **`openspec/specs/diagram-rendering/spec.md`**: requirement rewrites via delta specs.
- **Invariants**: Markdown canonical source and versioned derived caches unchanged; math/diagram render caches remain content-keyed presentation data.
