# Proposal: fix-visual-edit-tail-fidelity

## Why

At the end of a document in Visual Edit, the surface silently stops representing the real Markdown source. Every Enter beyond the first appears dead: all trailing blank lines collapse into one whitespace row whose height is hard-capped at 72px (`src/app/preview.rs`), while `insert_markdown_newline` keeps really inserting `"\n" + continuation` into the source — the user's own file carried ~40 invisible tail lines/continuations after a short Enter session. The same tail region cannot be scrolled fully into view (the last text line stays clipped), because identity-preserved list rows reuse stale cached heights and the list's scroll extent does not cover the full rendered content. In one observed session the outline additionally showed every heading from §1.1 to §1.9 doubled — proven to be real in-memory text duplication at that moment (disk file clean, outline pipeline is a strict 1:1 mirror) — with no identified writing path, so evidence capture is needed before that corruption can be fixed.

## What Changes

- Tail whitespace rows in Visual Edit render faithfully: the passive row's height tracks the actual number of blank source lines it covers (removing the 72px clamp, with only a generous sanity bound), so each Enter visibly grows the tail region and hidden document growth becomes visible.
- The virtualized Visual Edit list stops reusing stale cached row heights for identity-preserved rows whose rendered height derives from mutable source text (whitespace rows): when such a row's height-relevant signature changes, its cached height is invalidated and re-measured even though its block identity is stable.
- The Visual Edit list's scroll extent covers the full rendered content including the list's vertical padding, so the last rendered line can always be scrolled completely into view.
- Document mutation entry points gain debug-level diagnostic tracing (document version, edit range, replacement length, call-site tag) so the next in-memory heading-duplication repro identifies the exact writing path from logs.

**Non-goals:** no change to Enter/editing semantics (newline + continuation insertion stays as-is), no new visual block kinds, no outline UI redesign, no speculative fix for the heading duplication itself (diagnostics first), no Sync scroll changes.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`:
  - **Visual Edit whitespace activation** — the passive whitespace row's height SHALL reflect the amount of blank source it covers (no fixed small cap), including while it owns the caret.
  - **Stable source-mapped visual block identity** — the "unchanged rows keep reusable cached heights" guarantee gains an exception: rows whose rendered height derives from mutable source (whitespace) must have cached heights invalidated when their height signature changes despite stable identity.
  - **Pane scroll state with visible scrollbars** — the Visual Edit virtualized list's scroll extent SHALL cover the full rendered content including list padding so the final rendered line is fully scrollable into view.

## Impact

- `src/app/preview.rs` — Whitespace row rendering height; `visual_block_splice` identity signature for height-mutable rows.
- `src/app/root_view.rs` — Visual Edit list padding/scroll-extent interaction (`visual_edit_surface_view`, `list_pane_scrollbar_view`).
- `src/app/state.rs` — `sync_visual_list` splice/height invalidation coordination.
- `src/lib.rs` / `src/app/editor_element.rs` — debug tracing on canonical mutation entry points.
- Tests: `src/app/tests.rs`, `src/lib.rs`, `src/visual.rs` unit tests for splice signature, whitespace height, and scroll-extent math.
- Invariants preserved: derived state stays cached per document version (no reparse per keystroke); block identity stability semantics are refined, not removed; tracing is debug-level and off the hot rendering path.
