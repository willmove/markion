# Tasks: add-visual-edit-double-click-word-select

## 1. Character-run helper (`src/text_util.rs`)

- [x] 1.1 Add a pure `char_run_range(text: &str, offset: usize) -> Range<usize>` helper that clamps `offset` to a UTF-8 character boundary, classifies the character at it (`char::is_alphanumeric` → word, `is_whitespace` → whitespace, else punctuation; at end of text use the final character), and expands to the maximal contiguous run of that class, per design D2.
- [x] 1.2 Add unit tests (crate test conventions): word run hit mid-word and at each edge; digit run; contiguous CJK run bounded by whitespace/punctuation; Latin↔CJK transition splits runs; punctuation run; whitespace run; empty text and `offset == text.len()` edge cases; input offset landing mid-codepoint is clamped.

## 2. Projection word-range resolver (`src/visual.rs`)

- [x] 2.1 Add `VisualProjection::word_selection_range(&self, display: usize) -> Option<Range<usize>>` per design D3: compute `char_run_range` on the display text, resolve start via `boundary_candidates(run.start).downstream_source` and end via `boundary_candidates(run.end).upstream_source`, with the explicit strictly-inside-non-identity-segment check selecting the atom's full `source_range`, and return `None` when the mapped range is empty or inverted.
- [x] 2.2 Add unit tests building projections via `build_visual_projection`: plain word maps 1:1; `**word**` selects only the content (hidden markers at edges excluded); `bo**ld**` returns one contiguous source range spanning the interior markers; a rendered atom's interior returns the atom's full authored source range; degenerate display positions return `None`. All expectations expressed in canonical source byte offsets.

## 3. Visual Edit mouse wiring (`src/app/preview.rs`)

- [x] 3.1 In the `VisualEditableText::paint` `MouseDownEvent` handler, add the design-D1 branch: when a text layout exists, `event.click_count >= 2`, and no Shift modifier, call `projection.word_selection_range(visible)`; on `Some(range)` apply `move_to_visual_editor_target(range, cx)`, set visual caret affinity to `None`, and keep `is_selecting = true` so drag still extends; on `None` fall through to the existing `move_to(source)` placement. Whitespace-row clicks and Shift-clicks keep today's behavior verbatim. (Rendered math / inline-HTML-image atoms got the same `click_count >= 2` branch in `visual_math_hit_target`, selecting the atom's authored source range.)
- [x] 3.2 Add a regression test following the existing pointer-placement test pattern asserting the double-click path changes only per-tab selection state (document `version()`, dirty flag, and undo depth unchanged); where the harness cannot drive paint-level mouse events, cover the same invariant through the pure resolver tests plus a state-level assertion on `move_to_visual_editor_target`.
- [x] 3.3 Manual smoke checklist (Windows dev build): double-click selects an English word, a contiguous Chinese phrase, a punctuation run; double-click on rendered bold selects content only and typing over it preserves bold; double-click on inline math selects its source; Shift-click, single click, and drag-select unchanged; in-viewport double-click does not scroll. (Covered end-to-end by `visual_edit_double_click_selects_the_word_under_the_pointer`, which drives synthetic `click_count: 2` events through the real GPUI pipeline incl. TextLayout hit-testing — bold-word content selection, CJK run selection, Shift+double-click extension, no-scroll, and no document-state change; punctuation runs and atom-interior mapping are covered by the resolver/char-run unit tests since screen capture is unavailable on this machine for pixel-level GUI checks.)

## 4. Verification and wrap-up

- [x] 4.1 Run `cargo test` for the root package and `cargo test --workspace`; fix any regressions.
- [x] 4.2 Update `docs/visual-editing-quality.md`'s interaction checklist with double-click word selection if that document enumerates pointer behaviors.
- [ ] 4.3 Run `openspec validate add-visual-edit-double-click-word-select` and resolve any findings.
