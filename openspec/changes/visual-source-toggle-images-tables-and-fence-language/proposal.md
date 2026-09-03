# visual-source-toggle-images-tables-and-fence-language

## Why

Block math, diagrams, and raw-HTML blocks already offer the "rendered by default, hover `</>` to edit the exact source" experience in Visual Edit, but the three remaining everyday constructs still force a mode switch to touch their source: a standalone Markdown image has no in-place path to its `![alt](url …)` syntax (the removed three-field editor was replaced by a read-only caption whose only migration path was "switch to source mode"), a GFM pipe table has a grid editor but no raw-source escape hatch for alignment/structure work or pasting table text, and an ordinary code fence cannot change its language at all — the info string is preserved byte-for-byte with no editing surface.

## What Changes

- **Standalone Markdown images join the collapsible-source family.** An exactly ranged block-level Markdown image renders as today (image + read-only caption + existing caret-owner controls), and gains the hover `</>` control that expands a single monospaced payload editor covering the complete authored image span (`![alt](url "title {width=… align=…}")`). Image load failure forces the payload editor visible (same forced-expand rule as math/diagram errors). This is the completion of `simplify-visual-image-presentation`, not a reversal: default presentation stays clean; low-level syntax appears only on explicit request.
- **GFM tables gain an on-demand raw-source view.** A table with proven cell editors keeps its grid as the default presentation; the hover `</>` expands the complete authored pipe-table source as one payload editor below the grid. While expanded, the grid is read-only presentation, edits route to the payload, and activating a cell collapses the raw view and focuses that cell. Ambiguous tables keep today's source-backed fallback unchanged.
- **Code fences become re-languagable in Visual Edit.** When an ordinary code fence or a diagram fence owns the caret, the language label in its block header becomes an editable field (Typora-inspired, integrated into Markion's existing header rather than a second floating box). Committing rewrites only the first info-string token — preserving the rest of the info string, both fences, and the payload — or inserts a token directly after the opening fence when none is authored. Sanitization rejects whitespace/backtick/newline input, and retyping to `mermaid`/`math` re-dispatches the block naturally on re-parse.
- Expand/collapse state reuses the existing per-tab `expanded_visual_source_blocks` machinery and shared `visual_collapsible_source_block` chrome; all payload edits remain one atomic canonical source replacement through the existing IME, undo, dirty-state, and multi-tab paths.

Non-goals: no language autocomplete dropdown (plain sanitized input; completion list is a follow-up candidate); no change to inline images in prose (caret-proximity reveal already covers them); no collapse mode for ordinary code blocks (the code editor already IS the source; read-mode collapse remains the deferred non-goal from `collapse-visual-math-and-diagram-source`); no change to Read mode, Split Preview, or exports; no new structured image field controls.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `markdown-editing`: adds a requirement for on-demand whole-span source editing of block-level Markdown images in Visual Edit (ADD; deliberately avoids touching the "Direct Markdown image editing in Visual Edit" requirement pending removal by the unarchived `simplify-visual-image-presentation` delta).
- `tables-outline`: extends "GFM table rendering with row/column toolbar editing" with the raw-source toggle, the expanded-mode read-only grid rule, and cell-activation collapse routing.
- `code-and-math`: extends "Direct fenced-code editing in Visual Edit" so the first info-string token is editable through the header while the fence owns the caret, without weakening the existing payload/fence preservation contract.
- `diagram-rendering`: extends "Diagram blocks remain source-backed in Visual Edit" so diagram fences expose the same info-token editing affordance in their payload-editor header.

## Impact

- **`src/visual.rs`** — `visual_block_editor` gains an `Image` payload editor variant (single whole-span field, only when the span is proven) and a whole-block source field on the `Table` editor; fence editor gains an info-token field.
- **`src/model.rs`** — `VisualBlockEditor` shape (image payload, table source field, code info field), new `VisualEditorFieldKind` variants (`ImageSource`, `TableSource`, `CodeInfo`); the unused `ImageAlt`/`ImageDestination`/`ImageTitle` kinds stay as-is.
- **`src/app/preview.rs`** — image and table arms of the visual block view adopt `visual_collapsible_source_block`; `visual_code_editor`/`visual_diagram_editor` headers gain the caret-gated language field; image forced-expand on load failure.
- **`src/app/editing.rs` / `src/lib.rs`** — field validation, sanitization, Tab-traversal, and mutation routing for the new field kinds.
- **`src/i18n.rs`** — new localized strings (image/table source tooltip reuse, language field placeholder/aria).
- **Invariants touched:** none relaxed — payload edits stay one exact source replacement; derived `Arc` caches stay version-keyed; expand state is presentation-only on the tab; image render caches stay content-keyed. The `Maintained Visual Edit support classification` matrix rows for images, tables, and code fences need updating (they gain or regain dedicated payload editors, all staying in the rendered-WYSIWYG class).
- **Ordering note:** this change's image delta must not conflict with the pending `simplify-visual-image-presentation` archive; archiving that change first is recommended.
