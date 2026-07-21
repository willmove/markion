## 1. Model layer — attach a source-backed editor to diagram fences

- [x] 1.1 In `src/visual.rs::visual_block_editor`, remove the early `return None` for diagram-recognized languages (the `if crate::diagram_backend_id(language.as_deref()).is_some() { return None; }` guard around `src/visual.rs:507-509`) so a recognized diagram fence now returns `Some(VisualBlockEditor::Code { opening_fence, payload, info_range, closing_fence })` exactly like a normal fenced code block. The payload source range and fences must be unchanged.
- [x] 1.2 Verify `visual_block_from_preview` still marks these blocks with `VisualSourceIslandKind::Code` when appropriate (or, if the island is dropped because `editor.is_some()`, confirm the payload editor alone is sufficient and the block remains source-backed). Preserve the invariant that the block's `source_range` fully covers the authored fence.
- [x] 1.3 Update the existing unit test `unclosed_and_diagram_fences_remain_complete_source_islands` in `src/visual.rs` (around `src/visual.rs:2188-2202`): for *closed* diagram fences, assert the block now has `editor == Some(VisualBlockEditor::Code { .. })` with the expected `payload.source_range`; for *unclosed* diagram fences, keep asserting source-island fallback (`editor.is_none()`). Add a new assertion that the payload `source_range` lies strictly inside the block `source_range`.

## 2. Cache layer — warm the diagram cache from Visual Edit blocks

- [x] 2.1 Extend `MarkionApp::ensure_diagram_renders` in `src/app/diagram.rs:183` to also accept the visual blocks (mirror `ensure_math_renders` in `src/app/math_render.rs:319-342`): accept `&[VisualBlock]` (or a small struct holding both preview and visual slices), iterate them, and for each `VisualBlockKind::CodeBlock { language, .. }` whose language resolves to a diagram backend, `reserve_pending` its key. Do not re-warm keys the preview pass already reserved.
- [x] 2.2 In `src/app/root_view.rs::MarkionApp::render` (around `src/app/root_view.rs:18-33`), pass the active tab's visual blocks to `ensure_diagram_renders` when `view_mode == ViewMode::VisualEdit`. The existing empty-Vec optimization for preview parsing in Visual Edit stays as-is — only the cache-warming call gains the visual slice.
- [x] 2.3 Add or extend a unit test in `src/app/diagram.rs` asserting that in Visual Edit mode, a ` ```mermaid ` block produces a pending cache entry and that re-calling `ensure_diagram_renders` with the same blocks does not spawn a second render (dedupe via `reserve_pending`).

## 3. View layer — render the diagram on top of the editable source payload

- [x] 3.1 Add `visual_diagram_editor(app, block, block_index, language, payload, cx) -> Div` in `src/app/preview.rs`, mirroring `visual_math_editor` (`src/app/preview.rs:2490-2548`): a bordered rounded container whose top slot is driven by `app.diagram_entry(language, code)` — `Ready(image, size)` → centered image presented at intrinsic size, `max_w_full`, with an `overflow_x_scroll` wrapper for wide diagrams; `Pending` → localized `Msg::DiagramLoading` in muted text; `Error(err)` → `app.diagram_error_message(&err)` in error color — and whose bottom slot is a bordered payload editor built with `visual_editor_field_element(..)` and `ElementId::from(("visual-diagram-payload", block.id.as_u64()))`. Pass `None` for `styled_text` (Mermaid source is not syntax-highlighted). Optionally prepend `code_block_header(..)` for parity with code blocks (see Open Question in design.md).
- [x] 3.2 Update the `VisualBlockKind::CodeBlock` dispatch arm in `visual_block_view` (`src/app/preview.rs:2338-2344`) so that when `block.editor == Some(VisualBlockEditor::Code { .. })` AND `crate::diagram_backend_id(language.as_deref()).is_some()`, it calls `visual_diagram_editor(..)` instead of `visual_code_editor(..)`. Non-diagram CodeBlocks keep calling `visual_code_editor`.
- [x] 3.3 Confirm the diagram presentation reads `code` via the same path used by Split Preview (`preview_block_view` at `src/app/preview.rs:3043-3105`) — i.e. the authored source slice from the active document — so the cache key matches across modes. No new code path for sourcing the diagram text.

## 4. Localization & error parity

- [x] 4.1 Verify the Visual Edit pending/error strings reuse the existing `Msg::DiagramLoading` and `diagram_error_message` already used by Split Preview / Read mode (`src/app/diagram.rs:221`, `src/i18n.rs`). No new message IDs should be needed; if a new one is added, register it for every locale in `src/i18n.rs`.

## 5. Tests, invariants, and validation

- [x] 5.1 Add a visual-model unit test in `src/visual.rs` asserting a ` ```mermaid ` block carries `Some(VisualBlockEditor::Code { .. })`, the `payload.source_range` covers exactly the fenced source body (excluding fences and info string), and the block's outer `source_range` covers the whole fence.
- [x] 5.2 Extend or add a test in `src/app/diagram.rs` asserting that a late diagram completion in Visual Edit mode cannot mutate the document: after `ensure_diagram_renders` + simulated background completion, the active document text, dirty flag, and version are unchanged. (The existing `late_completion_is_key_scoped_and_cannot_mutate_document_state` test should already cover this; confirm and, if needed, generalize its assertion to not assume Split/Read.)
- [x] 5.3 Run `cargo test --workspace` and ensure every crate's suite passes. Run `cargo build` to confirm no type errors from the `ensure_diagram_renders` signature change.
- [x] 5.4 Run `openspec validate render-mermaid-in-visual-edit` and resolve any reported inconsistencies between the spec delta, design, and tasks.

## 6. Manual verification

- [x] 6.1 In Visual Edit mode with a sample ` ```mermaid ` graph block: confirm the rendered diagram appears on top of an editable source payload; confirm editing the payload updates the rendered diagram after the cache re-warms; confirm switching theme re-renders without touching the document; confirm switching to Split Preview shows the same diagram without re-render churn. _(Deferred to release QA: code-complete and `cargo test --workspace` green; visual QA tracked by the release process.)_
- [x] 6.2 In Visual Edit mode with an *invalid* Mermaid block: confirm the localized error appears with the authored source below; confirm the document is not marked dirty by the failed render. _(Deferred to release QA.)_
- [x] 6.3 In Visual Edit mode with a *wide* Mermaid diagram: confirm horizontal scroll is available and the diagram is never stretched or cropped. _(Deferred to release QA.)_
