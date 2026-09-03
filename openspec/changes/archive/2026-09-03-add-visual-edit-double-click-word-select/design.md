# Design: add-visual-edit-double-click-word-select

## Context

Visual Edit text interaction has a single choke point: `VisualEditableText::paint` (`src/app/preview.rs:925-981`) registers a `MouseDownEvent` handler that hit-tests via the already-painted `TextLayout` (`preview_index_for_position`), resolves the display offset through `VisualProjection::boundary_candidates` (`src/visual.rs:109`) into a canonical source offset plus caret affinity, and then calls `move_to` / `select_to`. All selection state lives in the per-tab `selected_range: Range<usize>` (canonical source bytes); painting maps it back to display quads through the projection's segments. gpui 0.2.2's `MouseDownEvent` exposes `click_count`, which the handler currently ignores — so a double click behaves exactly like a single click. There is no word-boundary utility anywhere in the workspace; the nearest precedent is the grapheme-level `previous_boundary`/`next_boundary` pair on tab state. `move_to_visual_editor_target(range)` (`src/app/editing.rs:3253`) is the existing precedent for setting a non-collapsed selection range directly with full state hygiene.

## Goals / Non-Goals

**Goals:**

- Double click on Visual Edit editable text selects the same-class character run at the pointer, as a canonical, UTF-8-safe source range that round-trips through the existing selection-painting path.
- All run/mapping logic in pure, GPUI-free functions in the lib crate, unit-testable without a window.
- Preserve every existing pointer-placement invariant: no reparse, no `MarkdownDocument.version()` bump, no derived-cache invalidation, no undo entry, viewport untouched for in-viewport clicks.

**Non-Goals (design-level):**

- Triple-click line/paragraph selection; word-wise drag extension after the double click; dictionary-based CJK segmentation; source-mode and read-only-preview parity; multi-field editors (code/math/HTML/table cell inputs) which have their own input surfaces.

## Decisions

### D1. Hook: `click_count >= 2` branch inside the existing `VisualEditableText::paint` MouseDown handler

The handler already owns hit-testing, affinity resolution, focus, and the shift/plain split. A double-click branch slots in after the `(source, affinity)` computation and before the `move_to`/`select_to` call: when `event.click_count >= 2`, `!event.modifiers.shift`, and a text layout exists, compute the word range and apply it; otherwise the existing path runs unchanged. Whitespace rows (no `text_layout`) and Shift-clicks keep today's behavior verbatim.

*Alternative considered:* a surface-level handler in `root_view.rs` — rejected; it has no access to the row's text layout or projection, so it would need a new plumbing path to duplicate what the row handler already does.

`>= 2` (not `== 2`) means the third click of a triple-click re-selects the same run rather than collapsing the caret — a harmless status-quo fallback until line selection is ever designed.

### D2. Word run = maximal same-class character run in display text; classes from std char classification

New pure helper in `src/text_util.rs`:

```rust
fn char_run_range(text: &str, offset: usize) -> Range<usize>
```

It clamps `offset` to a UTF-8 boundary, classifies the character under it, and expands in both directions over `chars()` to the maximal run of that class, chaining pairwise so results are independent of the anchor. Classes: ASCII word (letters/digits), CJK word (Han, kana, Hangul), other alphanumeric (e.g. `é`, `й` — joins whichever word run it touches, so `café` stays whole), whitespace, punctuation. The ASCII↔CJK script split means double-clicking an embedded acronym in Chinese prose (`使用HBM显存`) selects just `HBM`, while a contiguous Chinese phrase selects as one run — matching mainstream editors (VS Code, Typora) for the cases users hit daily. The run is computed on the **display text** because that is what the user clicked; the visible run, not the source token, defines the selection.

*Alternatives considered:* UAX#29 word boundaries via `unicode-segmentation` (already a dependency) — rejected: without dictionary segmentation it still groups CJK into one long run, i.e. no user-visible benefit over the classification for this feature's cases; dictionary segmentation (e.g. jieba) — rejected: heavy new dependency and out of scope; a single undifferentiated word class — rejected during implementation because it selects `使用HBM显存` as one mega-run when double-clicking the acronym.

### D3. Display→source mapping: new `VisualProjection` resolver with innermost edge resolution

New method next to the existing mapping helpers in `src/visual.rs`:

```rust
impl VisualProjection {
    fn word_selection_range(&self, display: usize) -> Option<Range<usize>>
}
```

1. `let run = char_run_range(&self.text, display);` (empty/degenerate runs handled by step 3).
2. Resolve source edges using the **existing** `boundary_candidates`, with one explicit segment check:
   - **start**: if `run.start` lies strictly inside a non-identity segment (rendered atom), use that segment's `source_range.start`; otherwise use `boundary_candidates(run.start).downstream_source`.
   - **end**: if `run.end` lies strictly inside a non-identity segment, use that segment's `source_range.end`; otherwise use `boundary_candidates(run.end).upstream_source`.
3. Return `Some(start..end)` only when `start < end`; otherwise `None` → the handler falls back to today's caret placement.

Why innermost (downstream for start, upstream for end): at a segment edge, `upstream_source`/`downstream_source` point into the adjacent hidden/visible sides; picking the inner side excludes hidden Markdown syntax sitting at the selection's edges. So double-clicking `**word**` (display "word") selects only the content — typing over it preserves the bold markers, matching Typora. Hidden syntax *inside* the run (e.g. `bo**ld**` → "bold") is unavoidably and correctly inside the contiguous source range. The strictly-inside-non-identity check exists because there `boundary_candidates` resolves to the atom's two source edges, and innermost would invert the range; instead the whole atom source range is selected (double-clicking rendered inline math selects `$x=1$` — the coherent editable unit). This was verified against the current `boundary_candidates` implementation: identity-segment interiors resolve exactly, so ordinary runs map 1:1 with no clamping needed.

*Alternative considered:* outermost resolution (markers included at edges) — rejected: typing over a double-clicked word would delete its formatting markers, which reads as corruption to a Typora user.

### D4. Apply via `move_to_visual_editor_target(range)`

The existing method already performs exactly the required state hygiene for setting a selection range: writes `selected_range`, resets `selection_reversed`, clears caret affinity and navigation intent, closes any open undo capture (pointer-only, so no new undo entry), clears `marked_range` (consistent with how a click today interrupts IME state via `input_marked_len = 0`), and honors caret-reveal/typewriter state. No new app method is needed.

*Alternative considered:* a dedicated `select_word_range` on `MarkionApp` — rejected as a wrapper that would only re-state this method.

### D5. Drag after double click stays character-wise

The handler keeps setting `is_selecting = true` on the second click, so a drag after the double click extends the selection through the existing `select_to` MouseMove path (character-wise from the run edge). Word-wise drag extension (Typora does this) is scope creep for no spec requirement; the simple behavior is still strictly better than today's no-op.

### D6. Layering: pure logic in the lib crate, thin wiring in the app crate

`char_run_range` (text_util) and `word_selection_range` (visual.rs) are GPUI-free and unit-testable. The `preview.rs` change is a small branch. No code under `crates/*` changes; no workspace invariant is touched.

## Data flow and caching

Double click → hit test against the **already-painted** `TextLayout` (no layout rebuild) → `word_selection_range` on the **existing per-frame** `Arc<VisualProjection>` (built during paint; reused, never recomputed) → `move_to_visual_editor_target` writes per-tab `selected_range` → repaint maps the source selection back to display quads via the existing segment mapping in `VisualEditableText::paint`. The path performs no `MarkdownDocument` mutation, no version bump, no derived-cache invalidation, and no reparse — it is on the same interaction tier as single-click placement, which the spec already covers ("Pointer placement does not reparse").

## Risks / Trade-offs

- [Contiguous CJK run ≠ linguistic 词语 (no dictionary segmentation)] → Accepted and specified as run semantics; dictionary segmentation stays out of scope. The spec scenario deliberately describes run behavior.
- [`_` and `'` classify as punctuation, splitting `snake_case` / `don't` on double click] → Matches the specified class table (letters/digits vs punctuation); consistent with several mainstream editors. If ever revisited it is a one-line classification change plus a spec MODIFIED requirement, not an architectural change.
- [Platform `click_count` quirks (slow double click, trackpad taps)] → Degradation is bounded: a missed count falls back to today's caret placement; there is no crash or state-corruption path. Fallback is also the behavior for degenerate `None` resolutions (e.g. clicking a lone atom boundary).
- [First click of the pair still places the caret before the second selects the word] → Identical to native editors; transient caret flash is expected and harmless.

## Migration Plan

None: pure additive interaction change, no persisted state, preferences, or document-format impact. Rollback is reverting the implementing commit.

## Open Questions

None — the classification table and mapping rule are decided; anything beyond them would change observable behavior and therefore belongs in a future change.
