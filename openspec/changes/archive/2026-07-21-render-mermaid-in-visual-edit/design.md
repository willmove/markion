## Context

Markion has two parallel rendering pipelines that share the same document model:

- **Split Preview / Read mode** renders `PreviewBlock`s via `preview_block_view`. A `PreviewBlock::CodeBlock` whose language is a registered diagram alias is dispatched through `MarkionApp::diagram_entry(language, code)` and rendered as a static `ImageSource::Render(..)` image, with `Pending` / `Error(DiagramError)` fallbacks that preserve the authored source (`src/app/preview.rs:3043-3105`).
- **Visual Edit (WYSIWYG)** renders `VisualBlock`s via `visual_block_view`. A diagram fence is *deliberately* routed away from the editor path: `visual_block_editor` returns `None` for any fence whose language resolves to a diagram backend (`src/visual.rs:507-509`), so the block carries no `VisualBlockEditor::Code` and `visual_block_view` falls back to `visual_source_island_view` — the raw authored source in a monospace box (`src/app/preview.rs:2338-2344`).

The diagram cache itself is mode-agnostic: `DiagramCache` is keyed by `(backend_id, source, theme)`, supports `reserve_pending` / `complete` / `get`, and a background executor does sanitize + rasterize off the frame path (`src/app/diagram.rs:78-219`). A late completion can only write its immutable cache key — it cannot mutate the document (`src/app/diagram.rs` test `late_completion_is_key_scoped_and_cannot_mutate_document_state`).

Visual Edit already has a direct template for "rendered presentation + editable source payload" in the form of display-math blocks:

- The model attaches a `VisualBlockEditor::Math { payload, .. }` to the block.
- `visual_math_editor` (`src/app/preview.rs:2490-2548`) renders the cached math image on top, and a bordered, syntax-styled payload editor below; `Pending` shows a localized placeholder, `Error` shows a localized message. Edits in the payload field mutate the canonical Markdown source through the normal visual-editor projection.
- `ensure_math_renders` (`src/app/math_render.rs`) walks *both* `&[PreviewBlock]` and `&[VisualBlock]`, so the math cache is populated regardless of which mode is active.

The diagram path currently does only half of this: the cache is populated from `PreviewBlock`s in Split/Read, but in Visual Edit `MarkionApp::render` passes an empty `Vec` (`src/app/root_view.rs:18-33`), so the cache is never warmed and `diagram_entry` always returns `Pending`.

This change closes that gap by mirroring the math design for diagrams in Visual Edit.

## Goals / Non-Goals

**Goals:**

- Visual Edit presents a rendered diagram (initially Mermaid; any future registered diagram backend falls out for free) on top of an editable source payload, mirroring `visual_math_editor`.
- Reuse the existing `DiagramCache`, `markion-diagram` registry, sanitizer, rasterizer, and theme key — no new rendering or async machinery.
- Keep the source-backed invariant: the only editing path is the Markdown source payload. Diagram preview is presentation-only and cannot mutate the document.
- Pending / error states in Visual Edit match Split Preview / Read mode and go through `src/i18n.rs`.

**Non-Goals:**

- New diagram backends or new Mermaid features.
- Changing Split Preview / Read mode rendering.
- A rendered-tree editing path (clicking the diagram to mutate nodes, drag-edit edges, etc.).
- Per-keystroke re-rasterization without dedupe — the cache key and the existing preview debounce already dedupe; this design does not weaken them.

## Decisions

### Decision 1: Route diagram fences through `VisualBlockEditor::Code` instead of `None`

**Choice:** In `visual_block_editor` (`src/visual.rs:498-519`), remove the early `return None` for diagram fences at `src/visual.rs:507-509` so the block carries a normal `VisualBlockEditor::Code { opening_fence, payload, info_range, closing_fence }`.

**Why:** The payload editor already implements the exact source-backed editing contract the spec requires — it edits the canonical Markdown source via the visual-editor field projection, and it preserves the source range and fences. Returning `None` was the conservative choice when Visual Edit had no way to present a rendered diagram; with a presentation view layered on top, `None` is no longer needed and forces the fallback that hides the diagram.

**Alternatives considered:**
- *Leave `editor = None` and special-case the `else` branch in `visual_block_view` to render the diagram.* Rejected: the payload editor is already correct and tested; re-implementing an inline source editor inside the diagram view would duplicate `visual_editor_field_element` wiring and risk diverging from the math pattern.
- *Introduce a new `VisualBlockEditor::Diagram { .. }` variant.* Rejected: the editor payload is structurally identical to `Code` (fenced source). A new variant would add a case to every `match VisualBlockEditor` without buying anything. The *view* (`visual_diagram_editor`) is what differs, and it is selected by the renderer based on `diagram_backend_id(language)`.

### Decision 2: New view `visual_diagram_editor` mirrors `visual_math_editor`

**Choice:** Add `visual_diagram_editor(app, block, block_index, language, payload, cx) -> Div` in `src/app/preview.rs`. Layout: bordered rounded container; presentation slot on top (`Ready` → centered image with intrinsic-size + `max_w_full` + horizontal scroll for wide diagrams, matching Split Preview; `Pending` → localized `Msg::DiagramLoading`; `Error` → localized `diagram_error_message`); payload editor below (same `visual_editor_field_element` call as `visual_code_editor`, no syntax styling on the payload because Mermaid isn't a highlighted language — pass `None` for `styled_text`, matching `visual_math_editor`).

**Why:** This is a straight structural copy of the proven math pattern. The diagram cache entry type already exposes the three states we need.

**Alternatives considered:**
- *Render the diagram image with no source editor (click-to-edit).* Rejected: violates the source-backed spec invariant; users would lose inline editing.
- *Reuse `visual_code_editor` and prepend the image.* Rejected: `visual_code_editor` styles the payload as a dark highlighted code block, which is wrong for an un-highlighted diagram source and would couple diagram presentation to code styling.

### Decision 3: Extend `ensure_diagram_renders` to walk `&[VisualBlock]` like `ensure_math_renders`

**Choice:** Change the signature of `MarkionApp::ensure_diagram_renders` (`src/app/diagram.rs:183`) to accept the visual blocks too (either a second slice or a small struct mirroring `ensure_math_renders`). In `MarkionApp::render` (`src/app/root_view.rs:18-33`), pass the visual blocks even in `ViewMode::VisualEdit` so the cache warms. The existing `PreviewBlock` path in Split/Read is unchanged.

**Why:** This is the exact pattern `ensure_math_renders` already uses (`src/app/math_render.rs:319-342`). It keeps the cache population logic in one place and ensures dedupe through `reserve_pending`.

**Alternatives considered:**
- *Call `reserve_pending` inline from `visual_diagram_editor`.* Rejected: the view path runs every frame and must not spawn renders; `ensure_*_renders` is deliberately called once per render pass from `MarkionApp::render` and dedupes via `reserve_pending` returning `false` for known keys.

### Decision 4: Preserve the source-backed spec invariant explicitly

**Choice:** The "Diagram blocks remain source-backed in Visual Edit" requirement in `openspec/specs/diagram-rendering/spec.md` is **modified**, not removed. It continues to mandate: (a) edits mutate the canonical Markdown source through the normal document mutation paths; (b) diagram completion / theme switch / pointer interaction cannot rewrite the fenced source or create a second editable tree; (c) document text, dirty flag, undo history, and version are unchanged by diagram completion. The *presentation* of the rendered diagram is added as an explicit new scenario.

**Why:** The spec is the source of truth and currently *forbids* what this change introduces. A delta is required; softening the wording without a delta would leave the spec contradicting the code.

### Data flow (where caching/versioning is affected)

```
Document text (version N)
   │
   ├─> PreviewBlock parse (cached per version, Arc-shared) ──> ensure_diagram_renders (Split/Read)
   │                                                              │
   │                                                              └─> reserve_pending → background spawn → sanitize+rasterize → complete(key)
   │
   └─> VisualBlock projection (cached per version, Arc-shared)
            │
            ├─> visual_block_editor: diagram fence now yields VisualBlockEditor::Code (was None)
            │     (source_range, payload source_range, fences unchanged)
            │
            └─> ensure_diagram_renders (Visual Edit, NEW)
                    │
                    └─> reserve_pending → same background spawn → same cache → complete(key)

Visual frame render:
   visual_block_view(CodeBlock with editor=Code, language=mermaid)
      └─> visual_diagram_editor
             ├─ diagram_entry(language, code)  → Ready(image,size) | Pending | Error(err)
             └─ visual_editor_field_element(payload)  → edits mutate document via projection
```

**Caching/versioning invariants preserved:**

- The diagram cache key does not include the document version; completed entries are reusable across versions and tabs (already true today; this change does not alter it).
- Diagram completion writes only its immutable cache key — the existing test `late_completion_is_key_scoped_and_cannot_mutate_document_state` covers the Visual Edit path unchanged because the completion path is unchanged.
- Visual Edit's `VisualBlock` projection is cached per version and shared via `Arc`; adding an editor to diagram blocks changes the *shape* of a `VisualBlock` but not *when* it is rebuilt (still per document version, not per keystroke inside the debounced window).
- The visual editor field element mutates the document through the same projection used by every other visual editor; no new mutation path is introduced.

## Risks / Trade-offs

- **[Spec delta is load-bearing]** The current spec *forbids* presenting diagrams in Visual Edit. → Mitigation: ship the MODIFIED requirement in the same change; do not archive without the delta synced.
- **[Test `unclosed_and_diagram_fences_remain_complete_source_islands` will break]** It asserts `editor.is_none()` for ` ```mermaid `. → Mitigation: update the assertion to check `editor` is `Some(Code)` with the expected payload source range, and add a sibling assertion that the block is still source-backed (`source_range` covers the full fence; `VisualBlockEditor::Code.payload.source_range` lies inside it).
- **[Visual layout regression on very tall diagrams]** A rendered Mermaid image can be tall; placing the payload editor below it pushes the editor below the fold. → Mitigation: this is the same trade-off as display math and is acceptable; wide diagrams already scroll horizontally via the existing `overflow_x_scroll` wrapper used for math. Reuse the same wrapper for the diagram presentation slot.
- **[Cache pressure with many diagrams]** Visual Edit now also warms the cache; in a document with many diagrams the LRU eviction (`DIAGRAM_CACHE_CAPACITY = 128`) could churn. → Mitigation: the key is unchanged and the cache is shared across modes, so worst case is the same as having the document open in Split Preview. No action needed for v1; revisit if telemetry shows churn.
- **[Payload editor without syntax highlighting looks plain]** Mermaid source in the payload editor won't be syntax-highlighted. → Mitigation: matches math payload editor (also unstyled); consistent with the WYSIWYG source-backed pattern.

## Migration Plan

Single-commit code change; no persistence format, settings, or network change. Rollback is `git revert`. No user-facing migration.

## Open Questions

- Should the diagram presentation slot show a label/badge (e.g. "Mermaid") the way `visual_code_editor` shows a code-block header? **Default:** yes — reuse `code_block_header` for parity with code blocks and to make the diagram type visible at a glance. Confirm during implementation; cheap to adjust.
