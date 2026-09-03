# visual-source-toggle-images-tables-and-fence-language

## Why

Block math, diagrams, and raw-HTML blocks already offer the "rendered by default, hover `</>` to edit the exact source" experience in Visual Edit, but the three remaining everyday constructs still force a mode switch to touch their source: a standalone Markdown image has no in-place path to its `![alt](url …)` syntax (the removed three-field editor was replaced by a read-only caption whose only migration path was "switch to source mode"), a GFM pipe table has a grid editor but no raw-source escape hatch for alignment/structure work or pasting table text, and an ordinary code fence cannot change its language at all — the info string is preserved byte-for-byte with no editing surface.

## What Changes

- **Standalone Markdown images join the collapsible-source family.** An exactly ranged block-level Markdown image renders as today (image + read-only caption + existing caret-owner controls), and gains the hover `</>` control that expands a single monospaced payload editor covering the complete authored image span (`![alt](url "title {width=… align=…}")`), rendered **above** the image so the source sits directly under the toggle the user clicked. Image load failure forces the payload editor visible (same forced-expand rule as math/diagram errors). This is the completion of `simplify-visual-image-presentation`, not a reversal: default presentation stays clean; low-level syntax appears only on explicit request.
- **Code fences become re-languagable in Visual Edit.** The fence's language label is hidden while the pointer is outside the block and the fence does not own the caret; moving the pointer into the fence (or clicking into it) reveals the label as an explicit click target over the first info-string token, with a localized placeholder for bare fences. Committing rewrites only the first info-string token — preserving the rest of the info string, both fences, and the payload — or inserts a token directly after the opening fence when none is authored. Sanitization rejects whitespace/backtick/newline input, and retyping to `mermaid`/`math` re-dispatches the block naturally on re-parse. Diagram fences expose the same chip inside their payload editor.
- Expand/collapse state reuses the existing per-tab `expanded_visual_source_blocks` machinery and shared `visual_collapsible_source_block` chrome; all payload edits remain one atomic canonical source replacement through the existing IME, undo, dirty-state, and multi-tab paths.

Non-goals: no language autocomplete dropdown (plain sanitized input; completion list is a follow-up candidate); no change to inline images in prose (caret-proximity reveal already covers them); no change to GFM tables (an earlier raw-source toggle was reverted after user testing — the always-editable grid stays as-is); no collapse mode for ordinary code blocks (the code editor already IS the source; read-mode collapse remains the deferred non-goal from `collapse-visual-math-and-diagram-source`); no change to Read mode, Split Preview, or exports; no new structured image field controls.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `markdown-editing`: adds a requirement for on-demand whole-span source editing of block-level Markdown images in Visual Edit (ADD; deliberately avoids touching the "Direct Markdown image editing in Visual Edit" requirement pending removal by the unarchived `simplify-visual-image-presentation` delta).
- `code-and-math`: extends "Direct fenced-code editing in Visual Edit" so the language label is hover/caret-revealed as an editable first-info-token field, without weakening the existing payload/fence preservation contract.
- `diagram-rendering`: extends "Diagram blocks remain source-backed in Visual Edit" so diagram fences expose the same info-token editing affordance in their payload-editor header.

## Impact

- **`src/visual.rs`** — `visual_block_editor` gains an `Image` payload editor variant (single whole-span field, only when the span is proven); the fence editor gains an info-token field.
- **`src/model.rs`** — `VisualBlockEditor` shape (image payload, code info field), new `VisualEditorFieldKind` variants (`ImageSource`, `CodeInfo`); the unused `ImageAlt`/`ImageDestination`/`ImageTitle` kinds stay as-is.
- **`src/app/preview.rs`** — image arm of the visual block view adopts `visual_collapsible_source_block` with payload-first ordering; `visual_code_editor` gains a hover-revealed language chip (`hovered_visual_code_block` on the tab) and `visual_diagram_editor` mounts the same chip in its payload header; image forced-expand on load failure.
- **`src/app/editing.rs` / `src/lib.rs`** — field validation, sanitization, Enter handling, and mutation routing for the new field kinds; code-payload edge handoff filters out the unmounted info field.
- **`src/app/state.rs`** — per-tab `hovered_visual_code_block` (pruned/cleared with the other hover state).
- **`src/i18n.rs`** — new localized `CodeLanguage` label (all seven languages) for the bare-fence placeholder.
- **Invariants touched:** none relaxed — payload edits stay one exact source replacement; derived `Arc` caches stay version-keyed; expand/hover state is presentation-only on the tab; image render caches stay content-keyed. The `Maintained Visual Edit support classification` matrix rows for images and code fences need updating (they gain payload/chip editors, staying in the rendered-WYSIWYG class).
- **Ordering note:** this change's image delta must not conflict with the pending `simplify-visual-image-presentation` archive; archiving that change first is recommended.
