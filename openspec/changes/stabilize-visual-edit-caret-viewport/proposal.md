## Why

In Visual Edit, a pointer click that already hit-tests a painted row still goes through `move_to`, which always sets `visual_cursor_reveal_pending`. The next render then treats any later item than the current scroll top as an unmeasured tail row and pins it to the viewport top — so clicking the middle or lower part of the visible document jumps the whole surface and the caret leaves the mouse. Users have to hunt for the caret after every click. Typora-class WYSIWYG editors keep the viewport still unless the caret would leave the visible area, and they pad the document end so the last line is not glued to the clip.

## What Changes

- Visual Edit caret movement becomes geometry-gated: if the painted (or soon-to-be-painted) caret already sits inside the viewport plus a small inset margin, the virtualized list MUST NOT change its scroll offset.
- When the caret would sit outside that inset — last line growing below the clip, keyboard/search/outline landing off-screen, unmeasured tail rows — the list scrolls the minimum amount needed to bring the caret into the inset. Pinning a later item to the top is reserved for unmeasured rows that cannot be revealed by bounds.
- The Visual Edit list gains a document-end padding band (about half a viewport) so the last rendered line can sit away from the pane bottom. Last-line clicks and typing then rarely need to move already-visible text.
- Pointer placement and in-viewport drag selection share this contract. Keyboard navigation, search, outline jumps, mode entry, and caret-moving edits keep the existing “reveal if off-screen” duty, but they use the same geometry gate instead of always requesting a reveal.
- The lost “reveal the active visual block when it is outside the viewport” scenario is restored in `markdown-editing` and narrowed so an already-visible caret is not a reveal trigger.

**Non-goals:** no change to progressive-reveal layout (markers may still shift a few glyphs under the mouse), no Typora-style “keep the clicked glyph pinned under the pointer” compensation, no Sync scroll changes, no change to source-editor or Read/Split preview click/scroll, no typewriter-mode redesign (typewriter remains an explicit opt-in that may still center the caret).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`:
  - **Source-backed Visual Edit mode** — restore and narrow caret-follow: off-screen caret moves still reveal the active row; in-viewport pointer placement and other in-viewport caret moves MUST NOT change Visual Edit scroll.
  - **Pane scroll state with visible scrollbars** — Visual Edit scroll extent includes a trailing document-end padding band so the last rendered line can be scrolled away from the clip and so last-line pointer placement does not have to jump the viewport.

## Impact

- `src/app/editing.rs` — `move_to` / `select_to` stop unconditionally requesting a pin-to-top reveal; caret moves share a geometry gate.
- `src/app/root_view.rs` — consume reveal only when the target is off-screen or unmeasured; add document-end padding to the Visual Edit list extent.
- `src/app/preview.rs` — keep pixel-follow for clipped last-line growth; do not follow when the caret is already inside the inset.
- `src/app/state.rs` — reveal/follow flags remain ephemeral per-tab interaction state, not document-version caches.
- Tests in `src/app/tests.rs`: clicking a visible mid-document row leaves `logical_scroll_top` unchanged; last-line typing still stays inside the viewport; off-screen keyboard/search/outline still reveal.
- Invariants preserved: derived Markdown state stays cached per `MarkdownDocument.version()`; pointer, caret, and scroll changes MUST NOT reparse or invalidate those caches.
