## Why

`stabilize-visual-edit-caret-viewport` stopped Visual Edit from pinning a clicked mid-document row to the viewport top, but typing still jumps the surface. Every in-place edit assigns the touched `VisualBlock` a fresh identity, so `visual_list.splice` marks that row `Unmeasured`; the same render then treats “no bounds and index later than scroll top” as a tail row and pins it. Users typing in the middle of a scrolled document lose the surrounding viewport on the first keystroke, and IME composition repeats the jump on every candidate update.

## What Changes

- In-place Visual Edit mutations that map one source block onto one successor of the same kind keep that block’s ephemeral identity, so the virtualized list does not splice a row that is only changing its text.
- The caret geometry gate no longer treats a just-spliced, previously visible row as an unmeasured tail. Pin-to-top stays reserved for rows that are both unmeasured and outside the previously measured window. In-viewport typing, IME composition, and Enter that keeps the caret on screen MUST NOT change `logical_scroll_top` except for the existing minimum pixel-follow when the painted caret would clip.
- `after_document_changed` stops discarding last-painted caret bounds and forcing a reveal on every mutation. In-viewport edits may still request a post-paint clip check; they MUST NOT pin first.
- Tests cover the missing edit path: typing (and IME-style replacement) in a visible mid-document row leaves `logical_scroll_top` unchanged; last-line growth still pixel-follows; true unmeasured tail rows can still pin.

**Non-goals:** no Typora-style compensation that pins the edited glyph under the pointer when progressive-reveal markers appear; no smoothing of heading/list/code structure transitions; no change to Source, Read, or Split Preview scroll; no typewriter-mode redesign; no Sync scroll changes; no identity reuse across splits, merges, kind changes, or ambiguous reparses.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`:
  - **Visual Edit caret placement preserves the viewport** — caret-moving *edits* inside the viewport inset MUST NOT pin the owning row to the top or otherwise jump already-visible text; pin-to-top stays for rows that are unmeasured *and* below the previously measured window.
  - **Stable source-mapped visual block identity** — a 1:1 in-place successor of the same kind keeps its prior identity so the virtualized list can reuse that row’s measured height; splits, merges, kind changes, and ambiguous reparses still receive new identities.

## Impact

- `src/source_mapped.rs` — `reconcile_visual_block_ids` reuses identity for proven 1:1 in-place successors, not only byte-identical shifted blocks.
- `src/app/application.rs` — `after_document_changed` no longer clears painted caret bounds and unconditionally requests a pin-style reveal.
- `src/app/preview.rs` / `src/app/root_view.rs` — geometry gate distinguishes “spliced but previously visible” from “unmeasured below the measured window.”
- `src/app/state.rs` — reveal/follow flags remain per-tab interaction state; derived Markdown caches stay keyed on `MarkdownDocument.version()`.
- Tests in `src/app/tests.rs` and `src/source_mapped.rs` / `src/lib.rs`: identity reuse on in-place edits; typing in a visible mid-document row does not scroll; tail typing still stays in view.
- Invariants preserved: derived Markdown state (visual blocks, outline, stats) remains cached per document version and shared via `Arc`; identity and scroll adjustments MUST NOT reparse or invalidate those caches; `MarkdownDocument.text` remains the only canonical editable representation.
- Related in-progress change: `fix-visual-edit-tail-fidelity` still describes “changed block receives new identity” for height-signature invalidation. This change narrows that scenario to split/merge/kind/ambiguous cases and keeps whitespace height-signature remasurement.
