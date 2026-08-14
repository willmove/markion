## 1. Source Layout and Mapping Primitives

- [x] 1.1 Add a versioned per-tab source-layout snapshot containing the shaped editor lines, source line offsets, cumulative line-top positions, line height, and layout identity needed for scroll mapping.
- [x] 1.2 Populate the source-layout snapshot from `EditorElement`'s existing paint geometry and invalidate it on document replacement, typography changes, pane-width/layout changes, and other existing editor measurement resets without triggering extra Markdown derivation.
- [x] 1.3 Implement pure, clamped conversions between source byte offsets and editor content Y positions using binary search plus wrapped-line hit testing/position lookup.
- [x] 1.4 Implement pure preview-block anchor helpers that locate containing blocks, collapse unmapped source gaps to adjacent row boundaries, interpolate within source-height intervals, and recognize document start/end anchors.
- [x] 1.5 Add unit tests for wrapped source lines, zero-length/degenerate ranges, non-uniform block source heights, hidden source gaps, UTF-8 boundaries, and top/bottom clamping.

## 2. Per-Tab Synchronization State Machine

- [x] 2.1 Replace the per-tab cached scroll fractions and app-wide re-entrancy guard with per-tab raw observations, explicit driver hints, expected follower targets, deferred-driver state, and pending preview refinement state.
- [x] 2.2 Reset or seed the transient synchronization state at tab/document replacement, Sync scroll toggles, view-mode transitions, and layout invalidation while preserving both panes' actual per-tab scroll positions.
- [x] 2.3 Implement and unit-test driver selection that prefers explicit input intent, consumes expected follower writes within tolerance, falls back to a single changed raw position, and performs no write when simultaneous changes are ambiguous.
- [x] 2.4 Implement and unit-test version gating so stale editor geometry or debounced preview source ranges retain only the latest driver and reconcile once both mappings are current, without forcing a parse or list reset.

## 3. Bidirectional Source-Mapped Reconciliation

- [x] 3.1 Mark editor and preview driver intent at wheel/trackpad surfaces, both custom scrollbar drags, preview list scroll notifications, and existing editor navigation paths such as find/goto and typewriter scrolling.
- [x] 3.2 Replace proportional editor-to-preview coupling with block lookup and within-block progress mapping, writing a logical preview row target and handling exact document boundaries directly.
- [x] 3.3 Replace proportional preview-to-editor coupling with measured-row progress and source-layout Y mapping, clamped to the editor's valid scroll range.
- [x] 3.4 Record every programmatic follower write as an expected target and update raw observations so the next render cannot reverse the driver or produce a feedback loop.
- [x] 3.5 Keep reconciliation inactive when Sync scroll is disabled, outside Split Preview, when the driving pane cannot scroll, or when no valid current source mapping exists; update implementation comments that still describe proportional coupling.

## 4. Virtualized Preview Refinement

- [x] 4.1 For an unmeasured editor-to-preview target, reveal the matched row at offset zero, store its versioned desired progress, and request one follow-up render without measuring the full list.
- [x] 4.2 Refine the within-row offset after the target row is measured, mark the refinement as an expected follower write, and discard/recompute pending work if the document version, block target, or layout identity changes.
- [x] 4.3 Add focused GPUI tests proving that a distant unmeasured target converges after one refinement frame and that coarse/refined follower movements never drive the editor backward.

## 5. Behavioral and Invariant Verification

- [x] 5.1 Add bidirectional tests with mixed headings, wrapped prose, code fences, tables, and image-sized blocks to verify that both panes remain on the same source-backed block despite deliberately non-uniform total/local heights.
- [x] 5.2 Add regression tests for first-frame seeding, tab switching, preference disable/enable, non-Split modes, one unscrollable pane, stale preview debounce, source gaps, and start/end boundaries.
- [x] 5.3 Verify synchronization leaves document text/version, dirty state, undo/redo history, preview list identity, derived-state `Arc` caches, syntax-highlight cache, and cached text handle unchanged.
- [x] 5.4 Run `cargo fmt --check`, `cargo test`, and `cargo test --workspace`.
- [x] 5.5 Manually exercise wheel, trackpad (where available), both scrollbar drags, find/goto, and typewriter movement in Split Preview on a long mixed-content document; confirm semantic alignment, smooth nearby movement, bounded distant refinement, and independent scrolling when disabled.
- [x] 5.6 Run `openspec validate improve-source-mapped-sync-scroll`.
