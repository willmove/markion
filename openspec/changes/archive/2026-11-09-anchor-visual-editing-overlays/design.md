## Context

The app already has painted caret bounds, virtual-list block bounds, Visual input bounds, exact selection-format targets, and root-hosted overlays. Reuse those interfaces instead of introducing a parallel overlay state system. Prior removal of the fixed selection toolbar was motivated by occlusion and noise, so the new toolbar must be compact, local, conditional, and suppressed while dragging or editing.

## Goals / Non-Goals

**Goals:**

- Keep document overlays close to their source interaction without changing layout or document state.
- Share one positioning policy across slash, link/token/math, and selection formatting.
- Preserve low-noise context-menu access and suppress transient controls during continuous input.

**Non-Goals:**

- No new overlay widgets beyond the selection toolbar and no new editor-field behavior.
- No pixel-perfect clone of Quiet.md and no cross-platform OS pointer hiding.
- No selection model, source mapping, or asynchronous derivation changes.

## Decisions

### 1. One anchor-resolution policy

Resolve an overlay anchor in this order: current painted caret bounds, the selection/caret-owning VisualBlock's measured bounds, then Visual input bounds. Use the lower-left edge for slash and inline editors. Position the selection toolbar above the owning block when space allows, otherwise below it. `anchored()` supplies viewport clamping; a bounded fallback retains usability before the first paint and for non-document preference editors.

The policy computes anchors but does not store a second geometry cache. Render-time state remains presentation-only.

### 2. Selection toolbar derives from exact targets

A non-empty selection produces a toolbar only when `visual_selection_format_target_for_block` accepts it, no incompatible overlay is active, and no text drag or mutation suppression is active. Bold, Italic, Inline Code, and Link reuse the existing actions. Before dispatch, rebuild the exact target and require the captured document version, range, and block identity to remain current.

The existing right-click block menu and its selection-format group remain available. This restores proximity without restoring the old fixed-position toolbar.

### 3. Document inline editors use the same policy

Slash, document link editing, inline math, and raw-token editing use the resolved source/caret anchor. Preference-only editors can retain a bounded fallback because they have no source range. Hard-coded light editor colors are replaced by the active palette.

### 4. Input suppression is presentation-only

A canonical document change suppresses hover/focus block grips. Moving the pointer over the Visual surface re-arms them. Opening, positioning, navigating, or dismissing an overlay does not change text, selection, version, history, dirty state, or derived caches; a confirmed command still uses the ordinary mutation path and suppresses the toolbar until selection changes again.

## Risks / Trade-offs

- [Geometry is stale or absent during the first frame] → Validate the current version/block where available and use bounded Visual input or preference-dialog fallback.
- [An anchored toolbar can still cover prose] → Prefer above-block placement, flip below when needed, keep a compact single row, and suppress during drag/input.
- [Selection changes while the toolbar is open] → Revalidate version/range/block immediately before every action.
- [Automatic controls add visual noise] → Keep right-click access, show only exact non-empty selections, and hide after mutation or incompatible overlay activity.

## Migration Plan

1. Add anchor resolution and move slash/document inline editors.
2. Add selection toolbar and exact dispatch revalidation.
3. Add input suppression and palette cleanup.
4. Run rendered geometry/action tests and the full quality gate.

## Open Questions

None.
