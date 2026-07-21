## Context

Visual Edit renders each `VisualBlock` through `visual_block_view` (`src/app/preview.rs:2087`). Before the per-kind `match`, two predicates route blocks to `visual_source_island_view` (`src/app/preview.rs:1997-2042`):

```rust
let always_source = matches!(block.source_island,
        Some(FrontMatter | Code | Html | Unsupported))
    || (block.editor.is_none()
        && block.editable_runs.iter().any(|run| run.conservative_fallback));
let focused_conservative = owns_caret
    && block.editor.is_none()
    && (block.source_island.is_some() || block.editable_runs.is_empty());
if focused_conservative || always_source {
    return visual_source_island_view(app, block, block_index, cx);
}
```

`visual_source_island_view` wraps the row in `mb_2 .p_3 .rounded_md .border_1 .border_color(0xcbd5e1) .bg(0xf8fafc) .font_family("JetBrains Mono")` — visibly a "code-like" box.

A `Whitespace` block is produced by `gap_block` (`src/visual.rs:353-373`) with empty `editable_runs`, `editor = None`, `source_island = None`. The user reaches it in two ways:

1. **Creating a blank line by pressing Enter at the end of a paragraph that already has a trailing newline.** A single Enter at the end of `"Body"` extends the Paragraph range to include the new `\n` (so no Whitespace row appears yet). But once the document already ends in `"\n"`, the next Enter inserts a second `\n` outside any Paragraph range and `gap_block` creates a Whitespace row that owns the caret at end-of-document. The same applies to a second Enter in the middle of multi-paragraph content.
2. **Down/Up arrow across a blank line.** `move_visual_vertical` (`src/app/editing.rs:1422-1436`) deliberately moves the caret onto the Whitespace row of an existing blank line.

In both cases the Whitespace block ends up owning the caret, `editable_runs.is_empty()` is true, so `focused_conservative` is true and the row is promoted to `visual_source_island_view`. That is the "large edit box" the user sees.

The caret is drawn by `VisualEditableText` (`src/app/preview.rs:480-583`) — specifically at `caret_active` time, as a 2px blue quad at the layout position for the source cursor. This element already handles empty text: `position_for_index(0)` returns the layout origin, so an empty `StyledText` paints a lone caret line.

Text input is document-level: `EntityInputHandler::replace_text_in_range` (`src/app/editor_element.rs:66-90`) routes to `push_text_input` / `replace_range` on the document, independent of which visual block owns the caret. So removing the source-island wrapper does not affect typing.

## Goals / Non-Goals

**Goals:**

- A Whitespace block that owns the caret renders as the same passive-height row it uses when passive, plus a thin caret line at the row's start — visually consistent with an empty paragraph.
- Typing, undo, autosave, dirty flag, and document version behavior on a caret-owning Whitespace row are unchanged.
- Existing passive-whitespace behavior (click is passive when the caret is elsewhere) is unchanged.
- Existing source-island rendering for FrontMatter/Code/Html/Unsupported and `conservative_fallback` runs is unchanged.

**Non-Goals:**

- Changing `move_visual_vertical` so the caret skips blank lines (the caret can still land on a blank line — users confirmed they want a visible caret, not navigation skipping).
- Changing passive-whitespace rendering.
- Changing source-island rendering for genuinely source-only blocks.
- A new caret-painting primitive — we reuse `VisualEditableText`.

## Decisions

### Decision 1: Exclude `Whitespace` from `focused_conservative`

**Choice:** In `visual_block_view`, compute `is_whitespace = matches!(block.kind, VisualBlockKind::Whitespace)` and require `!is_whitespace` in the `focused_conservative` predicate.

**Why:** `focused_conservative` was added (commit `b93afb2`) so that an empty block owning the caret still has a caret anchor. It works by routing the block to `visual_source_island_view`, which paints a caret via its `VisualEditableText`. The side effect is that the row also picks up the full source-island chrome (border + padding + monospace + gray background). For FrontMatter/Code/Html/Unsupported this chrome is correct (they genuinely are source). For Whitespace it is wrong — a blank line is layout, not source.

**Alternatives considered:**
- *Change `move_visual_vertical` to skip Whitespace.* Rejected by the user — they want a visible caret on the blank line.
- *Render Whitespace owning the caret through a brand-new lightweight element.* Rejected — `VisualEditableText` already paints the caret correctly for empty text; reusing it avoids duplicating caret-blink, IME-bounds, and selection-paint logic.

### Decision 2: Reuse `VisualEditableText` with an empty projection for the Whitespace caret

**Choice:** New helper `visual_whitespace_caret_element(app, block, block_index, cx) -> AnyElement` builds a `VisualEditableText` with:
- `text: StyledText::new("".into())` (empty),
- `projection` covering `block.source_range` (single segment `display 0..0 ↔ source block.source_range.start..block.source_range.start`, `source_anchor = block.source_range.start`, empty spans/revealed ranges),
- `source_island: false`,
- `caret_active: visual_block_owns_caret(app, block_index)` (always true in this path, but computed for consistency),
- `navigation_active: true`.

The Whitespace render arm wraps it in the existing passive `div().h(px(row_height))` and adds `.cursor(CursorStyle::IBeam)` so the pointer matches neighboring prose.

**Why:** The caret line is drawn by `VisualEditableText`'s existing paint logic (`src/app/preview.rs:532-557`), which reads `caret_active`, `source_cursor`, and the projection. With an empty projection whose single segment maps source `start..start` to display `0..0`, the caret resolves to `position_for_index(0)` — the row's origin. Clicks route through the same element's hit-testing, landing the caret at `block.source_range.start`. IME bounds keep working via the existing `visual_input_bounds` fallback (`src/app/editor_element.rs:252`).

**Alternatives considered:**
- *A raw `div().w(px(2.)).h(px(row_height)).bg(0x2563eb)`.* Rejected — bypasses caret-blink, selection paint, and click-to-place-caret; would regress the click affordance and visual consistency.
- *Reuse `visual_text_with_math_element` with an empty Paragraph block.* Rejected — it is tied to the Paragraph block kind and pulls in math/highlight wiring; the empty Whitespace case is simpler.

### Data flow (caching/versioning impact)

None. `VisualBlockKind::Whitespace` is still produced by `gap_block` per document version, cached via `Arc<Vec<VisualBlock>>`. Adding an `is_whitespace` check in `visual_block_view` is a per-frame render-time branch; it does not mutate blocks or invalidate caches. The caret-owning Whitespace row continues to be the same immutable block it was before — only its rendered appearance changes.

## Risks / Trade-offs

- **[Click affordance on a caret-owning Whitespace row]** Previously clicking the big box moved the caret into the row; after the fix, clicking the thin row must do the same. → Mitigation: `VisualEditableText` already handles click-to-place-caret; the row keeps `cursor(IBeam)`; integration test asserts click moves the caret into the row (covered by extending the existing passive-gap test family).
- **[IME bounds on an empty row]** A caret-owning Whitespace row has no text glyphs, so the caret bounds come from `VisualEditableText` painting at the origin. → Mitigation: `visual_input_bounds` already provides a surface-level fallback (`src/app/editor_element.rs:252`); the existing `visual_edit_heading_enter_activates_insertion_line_for_typing` test asserts `visual_input_bounds.is_some()` after Enter, and the new test asserts the same for the paragraph-Enter path.
- **[Visual asymmetry with Heading Enter]** Heading range includes `\n`, so Enter after a heading lands the caret on the heading row itself, not a Whitespace row. → Trade-off: acceptable; both paths now show the same thin caret, so the asymmetry is invisible to users.
- **[Spec wording]** The spec previously said the affordance "provides the source-backed editing affordance" without specifying the visual. → Mitigation: the MODIFIED requirement sharpens this to "a caret line, not a source-island box", with a new scenario pinning the behavior.

## Migration Plan

Single-commit code change; no persistence, settings, or network impact. Rollback is `git revert`. No user-facing migration.

## Open Questions

None — the user confirmed (a) paint a thin caret, don't skip the blank line; (b) go through the OpenSpec flow.
