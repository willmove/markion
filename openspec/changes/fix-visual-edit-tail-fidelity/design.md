# Design: fix-visual-edit-tail-fidelity

## Context

Diagnosis established three facts (see proposal.md — Why):

1. **Invisible tail growth.** All blank source after the last content block becomes one `VisualBlockKind::Whitespace` block (`build_visual_blocks`, `src/visual.rs` tail-gap fallback). Its row height is `(newline_count * 12.).clamp(12., 72.)` in `src/app/preview.rs`. Enter at the document tail really inserts `"\n" + continuation` into the source every time (`insert_markdown_newline`), but past ~6 blank lines the row stops growing — the surface stops representing the document.
2. **Stale row heights survive identity-preserving splices.** `sync_visual_list` reconciles the virtualized list through `visual_block_splice`, which compares only block IDs (`src/app/preview.rs`). GPUI 0.2.2's `ListState` caches a `Measured` size per item and re-renders an item only when it is visible or has no cached size; overdraw (800px) items keep their cached size. A whitespace row whose covered blank-line count changed while its ID stayed stable therefore keeps its old height forever, and `items.summary().height` (the scroll extent source) understates the true content.
3. **Scroll extent excludes the list's own padding.** The visual/preview lists carry `.pt(9).pb(9)` in their own style; GPUI computes `max_offset_for_scrollbar() = summary.height − bounds.height` where `bounds` includes that padding. Content occupying `summary.height + pt + pb` can never fully scroll into view — a standing ~18px shortfall that clips the last line at the pane's bottom edge.

The outline-duplication incident is real in-memory text corruption with no identified writing path; disk, session, recovery, undo-snapshot, and background-parse paths were all verified clean. This change adds evidence capture rather than a speculative fix.

**Data flow (where caching/versioning is touched):**

```
text edit → replace_source_range (version++, derived caches dropped)
  → background derive (debounced) → install_derived (version-gated)
  → build_visual_blocks (incremental stable IDs) ── NEW: height_signature stamped on Whitespace blocks
  → sync_visual_list → visual_block_splice ── CHANGED: identity comparison becomes (id, height_signature)
  → ListState.splice → rows re-measured when visible or within overdraw
  → summary.height → max_offset_for_scrollbar / pane scrollbar view ── CHANGED: padding moved off the list element
```

Per-version derived caching, memoized highlighting, and the cached text handle are untouched; no additional recomputation is added to the keystroke path.

## Goals / Non-Goals

Goals: make the Visual Edit tail honest (visible blank-line growth), make cached row heights incapable of going stale for height-mutable rows, make the scroll extent cover real content including padding, and make the next heading-duplication repro identifiable from logs.

Non-Goals (design-level): no GPUI fork or version bump; no per-blank-line block splitting; no change to the 12px-per-blank-line visual language or to Enter semantics; no speculative fix for the duplication.

## Decisions

### D1 — Faithful whitespace row height, generous bound only
Keep the compact 12px-per-blank-line representation but drop the 72px clamp; clamp only at a generous bound (≈ 4096 lines) to keep pathological files from producing unbounded rows. Bound lives next to the current constant in the `Whitespace` render arm.

*Alternatives:* cap blank-line insertion in `insert_markdown_newline` (rejected — changes editing semantics and hides data); split the tail into one block per blank line (rejected — churns block IDs and splice behavior for no benefit); per-line height from typography metrics (deferred — visual language change, no fidelity gain).

### D2 — Height signature participates in splice identity
Add a small `height_signature: Option<u32>` field to `VisualBlock`, stamped during `build_visual_blocks` for Whitespace blocks (covered newline count), `None` for all other kinds whose geometry is determined by their proven-unchanged content. `visual_block_splice` then compares `(id, height_signature)` instead of `id`. A whitespace row that grew or shrank lands inside the spliced middle range, so GPUI marks it `Unmeasured`; the 800px overdraw re-measures it on the next frame even below the viewport, and `summary.height` self-corrects.

The signature is stamped at build time because the old slice's line count cannot be recomputed from current text — the block carries its own geometry provenance. `document_memory` byte accounting is updated for the new field if its tests assert exact sizes.

*Alternatives:* invalidate all cached heights on every sync (rejected — destroys scroll anchoring and the incremental-cache invariant); app-side extent correction that adds the known-but-unmeasured tail height to the scrollbar math (rejected — duplicates layout truth in two places and drifts).

### D3 — Move list vertical padding off the list element
Change `visual_edit_surface_view` and the preview list container from `list(...).pt(9).pb(9)` to an outer padded `div` wrapping an unpadded `list`. The list's bounds then exclude the padding, so `summary.height − bounds.height` is the exact max scroll and the final row scrolls fully into view with the 9px bottom inset visible below it. Applies to the preview list too (same latent shortfall; same spec requirement).

Scrollbar geometry (`list_pane_scrollbar_view`) and Split Preview sync-scroll coupling read list offsets; both must be re-verified after the move (existing tests + manual check) since viewport-relative math shifts by the padding.

*Alternatives:* fork/patch gpui to include padding in extent (rejected — registry dependency, out of scope); remove the padding entirely (rejected — visual regression at pane edges).

### D11 — Caret Y on whitespace rows; post-paint pixel follow
The 72px-cap / splice work made the tail *row* grow, but two remaining seams still hid tail edits: the whitespace caret was painted at the row origin (empty projection → display 0), and `scroll_to_reveal_item` only reveals a *block* using pre-layout heights (unmeasured suffix rows contribute 0). Paint the caret on the insertion line that matches newlines before the source caret; when reveal targets a later item, `scroll_to` that index so it can be measured; after paint, pixel-follow `visual_caret_bounds` into the list viewport for two frames.

### D4 — Mutation choke-point diagnostics for the duplication
Add `tracing::debug!` at the two canonical document choke points (`replace_source_range`, `set_text`): document version, edit range, old/new lengths — never content. Add one tagged `debug!` line per high-level mutation entry point (`insert_newline`, `replace_text_in_range` / IME mark path, `apply_markdown_format`, `apply_exact_block_edit`, table edits, undo/redo, `reload_from_disk`) so a repro log reconstructs the writing sequence. All logging is off the render path and free when the subscriber filters the level.

Implementation must verify the default subscriber (see `src/app/bootstrap.rs` / log setup) actually records `debug` for the `markion` target in the installed build; if it filters to `info`, enable debug for the `markion` target by default rather than promoting per-keystroke lines to `info`.

*Alternatives:* full audit logging with content snapshots (rejected — log noise and content exposure); runtime invariant checker comparing heading-line counts between versions (rejected — needs a second parse per edit, violates the derived-state invariants).

## Risks / Trade-offs

- [Documents saved with large invisible tails suddenly show a tall blank region] → Intended one-time honesty change; the region is scrollable and its height tracks deletions/undo symmetrically.
- [Unmeasured far-below-viewport rows contribute 0 to summary height until overdraw reaches them] → Inherent GPUI virtualization behavior, unchanged by this design; the user-facing guarantee (last line fully scrollable, tail growth visible while editing there) holds because caret reveal and overdraw measure the tail before extent accuracy matters. Manual verification covers scroll-to-bottom via scrollbar drag, whose extent freezes during a drag.
- [Padding move shifts scrollbar/sync-scroll anchor math] → Re-run existing pane-scroll and sync-scroll tests; manual check of thumb position at top/bottom for both panes.
- [`height_signature` widens `VisualBlock` and touches memory accounting] → Update `document_memory` accounting and its tests; field is 8 bytes, no persisted format changes.
- [Diagnostics never fire because the default log level filters debug] → D4 mandates verifying the subscriber filter and enabling debug for the `markion` target if needed.

## Migration Plan

Single-commit behavior change, no persisted format or preference migration. Rollback = revert; documents written between release and rollback keep their extra tail blank lines (valid Markdown, user-visible and editable).

## Open Questions

- Per-blank-line height: keep the literal 12px or derive from `typography.preview_row_line_height` — decide during implementation; spec only requires proportional, uncapped height.
- Whether the preview list needs the D2 signature as well (its rows are content-driven and already re-measured when visible; decide from test evidence during implementation).
