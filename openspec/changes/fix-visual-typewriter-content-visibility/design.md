## Context

The user runs `target/debug/markion.exe` and reports alternating blank content above the caret when repeatedly typing a line and pressing Enter once in Visual Edit with typewriter mode enabled. The user confirmed that the symptom occurs in existing documents, not newly created empty documents.

`fix-small-window-typewriter-jitter` already supplies same-frame caret centering and terminal whitespace ownership. Its three focused GPUI tests pass, but they check caret painting/centering without proving that preceding visible text paints. This follow-up preserves those changes and addresses that separate coverage failure. The original document and native every-other-Enter cadence have not been replayed; they must not be presented as verified by the headless investigation.

### Observed mechanism

`VisualEditableText::prepaint` in `src/app/preview.rs` requests a viewport-height autoscroll rectangle around the current caret. In `vendor/zed/crates/gpui/src/elements/list.rs`, the upward branch of `StateInner::prepaint_items` uses the requesting item's index with an offset that can be negative. `List::prepaint` installs it and retries. `layout_items` begins at that index, while the leading-overdraw pass only measures earlier items; `paint` draws only `item_layouts`. A preceding paragraph can therefore intersect the viewport without being painted.

Investigation replayed the existing short-document test with its input sequence changed through the debugger, without changing the application algorithm: commit `测试`, press Enter three times, commit `测试`, press Enter, then alternate committing `测试` and a single Enter. This establishes a document with an earlier paragraph and blank lines before the normal input cycle. An equivalent fixture candidate is `测试\n\n\n测试\n` with the caret at the end; the implementation must verify the failing action history rather than assume direct fixture initialization produces identical list state.

At the failing typing frame, the viewport was 324 logical pixels high and measured item heights were `[198, 48, 60, 162]`, including boundary space. Repeated runs captured these states:

| Observation | Start item | Offset within item | Painted items | Caret center error |
| --- | ---: | ---: | --- | ---: |
| Unmodified upward autoscroll result | 2 | -126 | 2 and 3 | 0 |
| Debugger-only substitution of an equivalent anchor | 0 | 120 | 0 through 3 | 0 |

The two anchors represent the same content offset: `198 + 48 - 126 = 120`. The substitution restored the preceding content's participation in painting without moving the caret. Subsequent typing and Enter actions switched between omitted-content and full-coverage states; omitted content persisted across unchanged frames. This establishes the mechanism, but does not establish that every second native Enter has the same timing in the original document.

### Data flow and cache boundary

```text
text / IME / Enter
  -> normal document mutation and version update
  -> existing shared per-version visual blocks
  -> list splice and current row measurement
  -> caret prepaint requests centering
  -> normalize upward autoscroll anchor (proposed)
  -> existing bounded prepaint retry
  -> paint all viewport-intersecting content and the centered caret
```

Only presentation state changes at the proposed normalization step. It must not create another document mutation, parse, undo entry, or cache invalidation. The viewport publisher must continue avoiding reentrant borrowing of `ListState` while GPUI owns its mutable state.

## Goals / Non-Goals

**Goals:**

- Preserve visible preceding content on the first frame after an edit and throughout bounded reconciliation.
- Preserve the requested content-space position while expressing it using the actual first visible list item.
- Cover existing-document typing and single-Enter cycles with a failing content-paint regression before implementation.
- Retain boundary centering, current geometry checks, manual scrolling, ordinary reveal, and per-version caches.

**Non-Goals:**

- Changing inserted Markdown, terminal newline ownership, paragraph identity, focus dimming, or parser behavior.
- Increasing refinement counts/tolerances, painting the entire document, introducing animation, or redesigning the list API.
- Treating the runtime substitution as an implemented fix or the old completed checklist as acceptance evidence for this report.

## Decisions

### Normalize the upward autoscroll result before retrying layout

Make a narrow correction inside the vendored GPUI upward-autoscroll path, where the item tree, current width, renderer, and geometry are already available. If the target offset precedes the requesting item, walk toward preceding items, accumulating their measured heights until the target belongs to the resulting item. Use cached current-width measurements when valid and lay out only unmeasured predecessors needed to resolve the target. Account for list padding, zero-height items, and the applicable document-start clamp. The retry must preserve the same attainable content-space target.

Keep normalization local to this path. A blanket ban on all negative `ListOffset` values would change valid short-content bottom alignment. The existing downward branch, bottom alignment, ordinary in-item reveal, and wheel/scrollbar behavior must retain their contracts. No public API or additional frame loop is planned.

Alternatives considered:

- Clamping the negative offset to zero at the requesting item moves the caret and still skips preceding content.
- Correcting the scrollbar offset on a later app frame exposes an incomplete first frame, and the existing caret-only refinement can stop while the content is still missing.
- Increasing leading overdraw does not fix coverage because its current pass measures predecessors without adding them to the paint list.
- Replacing the app's prepaint centering mechanism would reopen the earlier one-frame jump; the local normalization preserves that mechanism with a smaller change.

### Test actual painting independently of the invalid anchor

Add a root-package GPUI regression that exercises the real edit, layout, prepaint, and paint path with the observed action history. Use a paint-stage observation tied to a stable source range, or scene primitives for the expected text, to prove that a preceding paragraph whose content intersects the viewport actually paints. Row construction, measurement, caret presence, and `ListState::bounds_for_item` alone are insufficient: they can miss or inherit the same invalid-anchor assumption. Use relative geometry so content legitimately scrolled out of view is not required to remain visible.

Inspect the first frame after each text commit and Enter, then unchanged frames through the existing refinement budget. Assert content coverage and centered-caret geometry together; retain the current caret tolerance instead of widening it. Compare document text, dirty state, version, undo length, shared derived caches, and cached text handle before and after scroll-only frames.

Also exercise the generic list autoscroll path with variable-height rows and a child requesting a rectangle extending above itself. Verify actual predecessor painting, unchanged reachable pixel target, document-start clamping, zero-height predecessors, padding, and unaffected bottom alignment/downward reveal. This integration test may live in the root GPUI test host: the patched GPUI crate is excluded from the root workspace, so root test success must not be reported as running its private unit suite.

### Keep the previous change and the native verification boundary explicit

The apply-phase typography regression exposed an additional integration omission: the rendered-font-size and paragraph-spacing setters invalidate measurements without requesting a new typewriter recenter. After the old request expires, the next frame therefore does not use the prepaint centering path (observed displacement: 26.428589 logical pixels). Both setters will explicitly request recentering after invalidation when Visual Edit is active, following the existing editor-font-size setter's pattern for its affected surface. Rendered typography changes must not introduce source-pane recentering. This is limited to the already specified typography-change behavior and preserves the same request lifecycle and cache boundaries.

Add a separate `chrome-platform` requirement for content coverage. Do not replace the existing focus/typewriter requirement or duplicate the previous change's added requirement. At archive time, preserve both follow-up and earlier deltas; the earlier checklist's completion is not evidence that this report is resolved.

Native acceptance uses a newly built `target/debug/markion.exe`, an existing document containing multiple blocks, and the user's line-then-single-Enter operation in a small window. Record build identity, document structure, viewport, typography, and the action/frame where missing content appears. If the original document is unavailable, distinguish the representative native check from verification of the user's exact cadence; leave the latter outstanding rather than silently completing it.

## Risks / Trade-offs

- Shared GPUI behavior affects other lists -> scope the correction to upward autoscroll and include ordinary reveal, padding, bottom-alignment, and boundary checks in the same regression work.
- Unmeasured predecessor heights or stale widths produce a wrong target -> measure only the needed rows at the current width, reuse current measurements, and exercise resize/typography changes.
- A centered caret hides an omitted paragraph -> make actual viewport content coverage a separate failure condition on every inspected frame.
- Test-platform fonts differ from Windows -> use relative visibility geometry, cover 640 x 400 and 480 x 300 windows, default typography and explicit font size 15 / paragraph spacing 13, then run native acceptance.
- An integration probe accidentally changes layout -> keep observations at paint time and avoid extra rendering or measurement solely to satisfy assertions.
- Concurrent changes share app rendering/tests -> preserve existing focus-mode and unrelated edits; do not rebuild broad derived state to repair presentation.

## Migration Plan

No data or preferences migration. Implement the failing regressions, apply the scoped normalization, run focused and root-package verification, and build the debug executable for native acceptance. Record commands and results in this change's verification notes during apply. Rollback reverts only this follow-up's normalization and associated integration adjustments, preserving the earlier typewriter and terminal-whitespace work.

## Open Questions

- Which content structure and native frame timing produce the user's precise every-other-Enter cadence? The confirmed mechanism is a starting point; any remaining difference requires a further captured case before claiming full resolution.
- Which existing root-test paint observation best exposes source-backed content without adding production overhead? Resolve this while establishing the failing regression, before implementing normalization.
