## Context

See `proposal.md` for the user-facing problem. The existing `reconcile_sync_scroll` runs once per render after the debounced preview blocks have been spliced into `EditorTab::preview_list`. It reads the editor and preview pixel offsets, converts each to a fraction of that pane's total scrollable range, infers the driver from two cached fractions, and writes the same fraction to the follower. This converges mechanically but loses document locality whenever source and rendered heights differ non-uniformly.

The data needed for a source-mapped algorithm already exists:

- Every top-level `PreviewBlock` has an ordered source byte range.
- The source editor retains shaped logical lines, source line offsets, per-line wrapped heights, and its current `ScrollHandle` offset from the last paint.
- The virtual preview exposes a logical top row and within-row offset, rendered row bounds for measured items, and item-based scrolling without materializing the full document.
- `preview_reflects_version` identifies the document version represented by the debounced preview list.

The reconciliation path must remain presentation-only. It cannot force parsing, reset/splice lists, mutate document/history state, or rebuild derived Markdown data on the typing path.

## Goals / Non-Goals

**Goals:**

- Align the source and preview viewport tops to the same source-backed block and a stable relative position inside that block.
- Make editor-to-preview and preview-to-editor mapping symmetric enough that a follower write cannot reverse the driver on the next frame.
- Preserve smooth nearby scrolling and make large virtual-list jumps converge after at most one measurement/refinement frame.
- Keep steady-state reconciliation O(log logical lines + log preview blocks), with no full-document parse or full-list measurement.
- Preserve per-tab isolation, preview virtualization, debounced preview parsing, and per-version caches.

**Non-Goals:**

- Character-perfect mapping inside rendered inline layout; preview blocks expose block-level source ranges, not per-glyph source geometry.
- Synchronizing horizontal scroll or applying sync outside Split Preview.
- Replacing GPUI's `ScrollHandle` or `ListState`, or sharing one physical scroll state between panes.
- Changing the Sync scroll preference schema, label, default, or persistence.

## Decisions

### 1. Use the top content edge as a semantic viewport anchor

Each pane is interpreted at the top edge of its scrollable content viewport. The anchor is represented transiently as a preview block index plus normalized progress through that block's source-height interval. Top and bottom document boundaries are explicit special anchors.

For an editor-driven scroll:

1. Convert the editor's positive scroll offset into a source byte position using the current shaped editor layout.
2. Binary-search the ordered preview block ranges for the containing or adjacent block.
3. Convert the source layout Y coordinate to normalized progress between that block range's start and end Y coordinates.
4. Convert `(block index, progress)` to a preview `ListOffset`.

For a preview-driven scroll:

1. Read `logical_scroll_top()` to obtain the top row and within-row pixel offset.
2. Divide by the measured full row height to obtain normalized progress, clamped to `[0, 1]`.
3. Interpolate between the editor-layout Y coordinates of that block's source-range start and end.
4. Write the resulting clamped pixel offset to the editor scroll handle.

If a source gap has no preview block (blank separators, link definitions, or other non-rendered syntax), the gap collapses to the shared boundary between the preceding row end and following row start. Leading and trailing gaps map to document start/end. Thus scrolling through hidden source may leave the preview momentarily stationary, but it cannot jump to an unrelated block.

At exact document start or end, boundary mapping overrides within-block interpolation and sends the follower directly to its own start or maximum scroll offset. This makes both panes agree at boundaries even when the final block cannot be top-aligned because the follower viewport is taller than the remaining content.

**Alternatives considered:**

- Keep whole-document percentage and add correction checkpoints. This still accumulates drift between checkpoints and introduces discontinuities.
- Scroll only to the matching block start. This is source-aware but visibly jumps inside long paragraphs, tables, and code blocks.
- Add inline render markers for every source line. This could be more exact, but it expands parser/render output and measurement cost beyond the current block-level requirement.

### 2. Promote the editor's painted line geometry to a versioned layout snapshot

The mapping helpers need fast bidirectional conversion between source bytes and editor content Y. `EditorElement` already produces the expensive inputs during layout/paint. Store them as a versioned per-tab source-layout snapshot rather than rescanning or reshaping during reconciliation:

- document version and the layout-affecting editor bounds/line height;
- `line_offsets` and shaped `WrappedLine`s;
- a cumulative `line_tops` prefix array derived from `line_heights`.

`source_offset -> Y` binary-searches `line_offsets`, then uses the selected wrapped line's `position_for_index`. `Y -> source_offset` binary-searches `line_tops`, then uses the wrapped line's closest-position hit testing at the left edge of the wrapped visual line. Both conversions clamp to UTF-8/document boundaries. A zero-height source interval uses at least one editor line height so interpolation remains defined.

The snapshot is valid only for its document version and layout key. Typography changes, pane-width changes, document replacement, or missing paint geometry invalidate it and seed reconciliation without moving either pane on the first valid frame.

**Alternative considered:** derive source Y from logical line number times a constant line height. That repeats the current class of error for soft-wrapped source lines and would make mapping width-dependent in the wrong way.

### 3. Replace cached equal fractions with a per-tab driver/follower state machine

Equal fraction caches cannot represent a converged semantic mapping because the two correct follower offsets generally have different fractions. Replace them with transient per-tab synchronization state containing:

- last observed raw editor offset;
- last observed preview logical top `(item index, within-item offset)`;
- an optional explicit driver hint (`Editor` or `Preview`);
- an optional expected follower target, used to consume the programmatic write on the next observation;
- an optional deferred driver while preview/source layout versions are stale;
- an optional pending preview refinement target.

Driver hints are recorded at existing input/mutation boundaries: editor and preview wheel/trackpad surfaces, both custom scrollbar drags, preview list scroll notifications, and editor navigation helpers such as find/goto and typewriter scrolling. Raw-offset comparison remains a fallback for scroll mutations without an explicit hint.

Reconciliation proceeds in this order:

1. Read both pane positions and consume any follower movement matching an expected target within a pixel/logical-offset tolerance.
2. Prefer a current explicit driver hint; otherwise select the only pane whose raw position changed.
3. If both panes changed ambiguously (for example during simultaneous reflow), refresh observations without writing either pane.
4. Validate source-layout and preview versions.
5. Map the driver anchor and write only the follower, immediately recording the expected target and updated observations.

Enabling Sync scroll, entering Split Preview, or switching tabs seeds observations without an immediate jump. Disabling it clears only transient synchronization metadata; the pane scroll positions remain intact.

This removes the app-wide `syncing_scroll` guard. Loop prevention becomes per-tab and target-aware, which also handles delayed virtual-list refinement without blocking unrelated tabs.

**Alternative considered:** rely only on render-time raw-offset comparison. It cannot reliably disambiguate a user scroll from a follower write, preview measurement adjustment, or two-pane reflow. Explicit hints plus observation fallback keep coverage without coupling the core mapper to one input device.

### 4. Use a bounded two-phase target for unmeasured preview rows

`ListState::bounds_for_item` can provide a full height only for a row that has been laid out. Editor-driven nearby scrolling normally targets a visible/measured row and can set the precise within-row offset immediately. A large source scrollbar jump may target an unmeasured row.

For an unmeasured target:

1. Call `scroll_to(ListOffset { item_ix, offset_in_item: 0 })` to reveal the correct semantic row.
2. Store `(document version, item index, desired progress)` as the pending refinement, mark the coarse list movement as an expected follower write, and request one follow-up render.
3. On the next reconciliation after layout, read the row height, set `offset_in_item = progress * row_height`, update the expected follower target, and clear the pending refinement.

If the document version, block identity/index, or layout validity changes before refinement, discard the pending value and recompute from the current driving pane when a valid map is available. There is no percentage fallback and no request to measure all preview rows.

### 5. Gate mapping on current versions and defer the latest driver

Source ranges are safe only when `preview_reflects_version == document.version()` and the current `preview_list_blocks` are the blocks represented by the list. Editor mapping is safe only when the source-layout snapshot has the same document version and current layout key.

When either side is stale, reconciliation records the latest driver and current raw observations but does not move the follower. It does not invoke `preview_blocks_shared`, reset the list, or bypass the existing debounce. Once both mappings are current, the deferred driver is recomputed from that pane's then-current position and applied once. A newer explicit user input replaces an older deferred driver.

Preview row remeasurement caused by images, math, diagrams, typography, or resize is treated as geometry change rather than user intent unless an explicit driver or raw movement independent of the expected follower target exists. Logical list anchoring therefore continues to preserve the visible block while measurements settle.

### 6. Keep pure mapping and reconciliation decisions testable

Separate GPUI reads/writes from pure helpers for:

- locating the preview block or collapsed gap boundary for a source offset;
- clamped within-interval progress and inverse interpolation;
- document-boundary selection;
- driver selection from hints, observations, and expected follower targets;
- stale-version deferral and pending-refinement validation.

Use synthetic source-layout Y values and preview block ranges in unit tests to prove that non-uniform block heights map to the same block, gaps collapse deterministically, endpoints clamp, and forward/inverse mappings remain stable within tolerance. Add focused GPUI tests for the two-phase list refinement and for document/cache immutability during reconciliation.

### Data flow and cache impact

```text
input or navigation marks driver
  -> existing pane scroll state changes
  -> render syncs debounced preview list (existing)
  -> reconcile validates preview version + source-layout snapshot
  -> driver position becomes (preview block, within-block progress)
  -> follower scroll state is written and recorded as expected
  -> optional next-frame virtual-row refinement
  -> render/layout continues normally
```

The only new cache is lightweight presentation geometry/observation state on `EditorTab`. Markdown-derived `Arc` values, syntax highlighting, shared text handles, document versions, undo snapshots, dirty state, and autosave/recovery state are neither read through a new derivation path nor invalidated.

## Risks / Trade-offs

- **[Block-level source ranges cannot reproduce inline rendered geometry exactly]** → Anchor block boundaries exactly and interpolate within each block. This yields stable semantic alignment without expanding parser output; inline markers can be a future capability if evidence justifies them.
- **[A large jump needs one extra frame before its within-row position is exact]** → Reveal the correct row immediately, refine once after measurement, and tag both movements as follower writes so no oscillation occurs.
- **[Rapid alternating input in both panes can make driver order ambiguous]** → Prefer the most recent explicit interaction hint; when only raw observations exist and both changed, seed without writing rather than choosing arbitrarily.
- **[Stale debounced preview temporarily pauses coupling while typing]** → Retain only the latest driver and reconcile as soon as the normal preview result lands, preserving responsiveness and cache invariants.
- **[Editor or preview reflow may invalidate prior pixel geometry]** → Version/layout-key snapshots and target validation discard stale mappings; first valid frame seeds observations before further user movement drives synchronization.
- **[Additional line-top storage scales with logical line count]** → It is one `Pixels` prefix per already-shaped logical line and is rebuilt alongside existing line geometry, avoiding a second document scan.

## Migration Plan

1. Introduce the source-layout snapshot and pure mapping/state-machine helpers behind the existing Sync scroll preference.
2. Replace the per-tab fraction fields and app-wide re-entrancy guard with per-tab observation/target state; no persisted preference migration is needed.
3. Wire driver hints and the render-time reconciliation, then add the bounded preview refinement render.
4. Run focused mapping/state tests, existing GPUI tests, `cargo test --workspace`, and manual Split Preview checks with long mixed-content documents and both scrollbar directions.

Rollback is code-only: restore proportional reconciliation and its transient fraction fields. The persisted `sync_scroll` value remains compatible in either implementation.
