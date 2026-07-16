## Context

The source editor owns the canonical `EntityInputHandler` implementation. Its `EditorElement` registers `ElementInputHandler<MarkionApp>` during paint, which lets GPUI forward platform text and IME events into `MarkdownDocument::replace_range` and the existing undo, dirty-state, autosave, and cache invalidation flow.

Visual Edit replaces `EditorElement` with a virtualized `ListState` of `VisualBlock` rows. `VisualEditableText` maps visible glyph positions back to source ranges and updates the shared source selection, but no Visual Edit element calls `Window::handle_input`. GPUI therefore has no platform input target in this mode. The visual model also drops whitespace-only gaps, and cursor/navigation updates do not request that the active visual row be revealed.

Current broken flow:

```text
platform text / IME
  -> no Visual Edit input registration
  -X-> EntityInputHandler
      -> source mutation / undo / dirty / autosave / visual-cache invalidation
```

Target flow:

```text
Visual Edit surface paint
  -> one input bridge registers ElementInputHandler<MarkionApp>
  -> existing EntityInputHandler mutates MarkdownDocument.text
  -> document version changes
  -> cached Arc<VisualBlock> refreshes once for that version
  -> visual list splices changed rows and reveals the active row when requested

pointer / keyboard-only cursor change
  -> source selection + pending visual reveal + ephemeral caret bounds
  -> no document version change and no derived-cache invalidation
```

## Goals / Non-Goals

**Goals:**

- Make normal character input, selection replacement, paste, and IME composition functional in Visual Edit through the existing source-backed mutation path.
- Register exactly one active platform input handler for the Visual Edit surface, including when the document is empty.
- Give whitespace-only gaps and trailing positions source-ranged visual rows so every valid non-empty-document caret offset can be focused.
- Reveal the active virtualized row after keyboard navigation, document mutation, mode entry, search selection, or outline jump without forcing scroll after unrelated manual scrolling.
- Expose the painted visual caret rectangle to GPUI for IME candidate placement without treating it as Markdown-derived state.
- Add tests that exercise platform input at the rendered-window boundary as well as pure source-range helpers.

**Non-Goals:**

- Full Typora-style marker-free WYSIWYG editing.
- A second rich-text document model or a separate visual undo stack.
- Direct rich editing of table cells, images, code, math, HTML, or other conservative source islands.
- Replacing list virtualization or changing preview/read behavior.

## Decisions

### 1. Register input once at the Visual Edit surface boundary

Add a lightweight Visual Edit input element whose paint method calls `Window::handle_input` with the app focus handle and the existing `ElementInputHandler<MarkionApp>`. Render it once as a non-hit-testing overlay/sibling of the visual list, not once per `VisualEditableText` row.

GPUI keeps the last input handler registered during a frame. Per-row registration would make the last painted virtual row own IME geometry and would fail for an empty document or an off-screen active row. A single surface bridge has stable ownership and continues to reuse all current `EntityInputHandler` mutation methods. Read mode does not render the bridge and remains non-editable.

### 2. Model whitespace gaps as explicit visual blocks

Add a `VisualBlockKind::Whitespace` row for whitespace-only source ranges between parsed blocks and after the last parsed block. Unfocused whitespace rows render as compact clickable spacing; when focused they use the same source-backed island element as other precise-edit regions. Non-whitespace parser gaps remain conservative unsupported source islands.

This preserves source coverage without rendering every ordinary inter-block newline as a permanently bordered raw-source card. Empty documents continue to use the existing placeholder surface plus the input bridge.

### 3. Use a one-shot reveal request for virtual-list cursor following

Store `visual_cursor_reveal_pending` per tab. Cursor/navigation and document-mutation paths set it; the next Visual Edit render finds the visual block containing the active source cursor, calls `ListState::scroll_to_reveal_item`, then clears the flag.

Always revealing on every render was rejected because manual wheel scrolling would snap back to the caret. Direct pixel mapping was rejected because variable-height virtual rows already provide item-level reveal semantics.

### 4. Keep visual caret geometry ephemeral and mode-aware

When the focused `VisualEditableText` row paints its cursor, record that painted caret rectangle in per-tab `visual_caret_bounds`. In Visual Edit, `EntityInputHandler::bounds_for_range` returns this rectangle when available and otherwise falls back conservatively to the Visual Edit surface bounds. Source Edit/Split retain their existing shaped-line geometry.

Caret bounds and reveal flags are interaction state, not document-derived Markdown state. Updating them MUST NOT increment `MarkdownDocument.version()` or clear preview, outline, stats, highlight, or visual-block caches.

### 5. Verify the platform boundary

Enable GPUI test support for dev/test builds and add a rendered-window test that enters Visual Edit, focuses the app, simulates text input, and asserts canonical Markdown text, dirty state, and undo behavior. Add a companion empty-document case and retain helper tests for whitespace coverage, active-block lookup, and one-shot reveal state.

Pure calls to `MarkdownDocument` or visual mapping helpers cannot catch a missing `Window::handle_input`, which is why the previous suite passed despite the mode being non-editable.

## Risks / Trade-offs

- [Risk] A transparent input bridge could intercept pointer events → Keep it free of hitboxes and continue handling pointer mapping in `VisualEditableText` and whitespace rows.
- [Risk] Whitespace rows could add excessive visual chrome → Render them as compact spacing while unfocused and expose raw whitespace only when focused.
- [Risk] Virtualization can leave an off-screen caret without fresh geometry for one frame → Issue a one-shot row reveal first and use the surface bounds only as a temporary IME fallback.
- [Risk] GPUI test support increases test compile cost → Enable it only through dev-dependency feature unification and keep rendered integration cases focused.
- [Risk] Cursor changes accidentally trigger Markdown recomputation → Store reveal/caret data outside `MarkdownDocument` and assert version/cache reuse in tests.

## Migration Plan

1. Add interaction state and whitespace block coverage without changing persisted Markdown.
2. Add the Visual Edit input bridge and mode-aware caret bounds.
3. Wire one-shot reveal requests into cursor, navigation, and mutation paths.
4. Add rendered GPUI input tests and run the root and workspace suites.

Rollback removes the bridge and ephemeral state plus the whitespace variant; no file or preference migration is required.

## Open Questions

None for this repair. Richer inline marker reveal and direct complex-block editing remain separate future changes.
