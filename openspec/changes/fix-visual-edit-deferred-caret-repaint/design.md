## Context

See `proposal.md` for motivation and `specs/markdown-editing/spec.md` for the observable contract.

Visual Edit keeps canonical selection in `DocumentTabState` and caches painted-line navigation snapshots per document version and visual block identity. Same-block Up/Down can resolve from the active snapshot during the action. Cross-block navigation instead records `PendingVisualNavigation`, reveals the adjacent list item, and waits for an element paint to publish target geometry. `VisualEditableText::paint` currently calls `complete_pending_visual_navigation`, which can call `move_to` or `select_to` and notify the application while GPUI is already drawing.

GPUI records the dirty view during a draw but does not mark the window dirty or enqueue the ordinary notify effect until `DrawPhase::None`. The current frame was also constructed from the pre-move selection. The canonical caret can therefore move during paint while the displayed scene retains the old caret, and no subsequent draw is guaranteed until another external invalidation occurs. `VisualInputElement::paint` has the same scheduling hazard when it decrements `visual_caret_follow_frames` and calls ordinary notification to request another refinement frame.

This is presentation state, not a second editing model:

```text
keyboard action
  -> canonical selection / pending navigation
  -> cached geometry, or reveal virtualized row
  -> paint publishes target geometry only
  -> post-paint, version-checked completion
  -> explicit next-frame invalidation
  -> scene paints the resolved caret
```

Real source edits retain the existing path from `MarkdownDocument.text` through one version increment and one derived-state invalidation. Deferred navigation and follow frames remain per-tab interaction state and do not rebuild per-version Markdown data.

## Goals / Non-Goals

**Goals:**

- Keep selection mutation and navigation completion outside GPUI element paint.
- Complete a cached adjacent-block move during the originating action and an unmeasured move immediately after valid target geometry is published.
- Guarantee the required follow-up draw even when the compositor and application are otherwise idle.
- Preserve blank-line stops, preferred horizontal position, selection extension, minimal reveal scrolling, typewriter behavior, and stale-request rejection.
- Test the caret that was actually painted in each controlled frame, not only the model offset or a cached bounds value.

**Non-Goals:**

- Changing terminal-newline ownership, which is already handled by the existing terminal whitespace-row correction.
- Skipping authored whitespace rows or changing how many Up/Down actions cross them.
- Reworking GPUI's global invalidation contract, list virtualization, input dispatch, or Wayland backend.
- Adding an animation, new preference, persisted state, or GPUI dependency to a workspace member.

## Decisions

### 1. Resolve current geometry synchronously in the input action

Factor target resolution from `complete_pending_visual_navigation` into a helper that validates document version, target block index and identity, snapshot version, direction, and preferred X before returning a source offset and painted-line index. When the adjacent block already has a current snapshot, the Up/Down or selection action applies the result immediately, while GPUI is outside a draw. It then performs the existing one-shot reveal and ordinary notification.

This removes an unnecessary frame of latency for already-painted targets and ensures the next element tree is built from the new selection. It must retain the existing source-offset path for `Whitespace` and marker-only callout-title rows, which do not publish wrapped-line snapshots.

Alternative considered: always preserve the pending path even with cached geometry. That keeps selection changes coupled to paint and leaves the stale-frame window open for the common visible-block case.

### 2. Paint publishes geometry; a deduplicated deferred callback mutates selection

When the target snapshot does not exist, retain `PendingVisualNavigation` and reveal the target list item. Target paint may publish its immutable navigation snapshot, but it must not call `move_to`, `select_to`, or any other caret-mutating method inline.

After publishing a snapshot for the pending target, queue one post-paint completion through the window/entity defer mechanism. Track enough request identity to deduplicate scheduling. The callback re-reads current state and completes only if all of the following still match:

- active tab identity;
- document version;
- target block index and stable `VisualBlockId`;
- navigation direction and selection-extension mode;
- the still-current pending request.

On success it applies the resolved selection, preferred X and line position, clears the pending request and queue marker, and issues notification while GPUI is no longer drawing. On mismatch it only clears a matching obsolete queue marker; a newer request remains authoritative. Document edits, tab/mode changes, pointer placement, and newer navigation already clear or replace navigation intent and therefore invalidate the callback.

Alternative considered: change `WindowInvalidator` so notifications during any draw automatically dirty the following frame. Upstream GPUI deliberately distinguishes draw-time invalidation, and a global change could introduce redraw loops or affect every list and element. The application should instead respect the framework phase boundary.

### 3. Use an explicit animation-frame request for bounded caret following

`visual_caret_follow_frames` remains a small bounded reconciliation budget. When a paint consumes a follow frame and either scrolls or leaves work outstanding, request a GPUI animation frame rather than calling ordinary entity notification from inside paint. The scheduled callback notifies the owning application entity outside the completed draw, so the next compositor callback sees a dirty window.

Do not request frames after the counter reaches zero, when no scroll occurred and no deferred navigation remains. This preserves manual-scroll independence and prevents an idle redraw loop. Typewriter prepaint centering and the existing request-expiration rules remain unchanged.

Alternative considered: unconditional refresh after every key event. That can mask the lost post-layout handoff, causes unnecessary whole-window work, and still does not encode when virtualized geometry becomes ready.

### 4. Observe actual painted caret identity at controlled frame boundaries

Add test-only paint observation that records the document version, source caret offset or selection head, owning block identity, and caret bounds for the caret quad emitted by the current frame. Cached `visual_caret_bounds` alone is insufficient because it can survive from another frame or be updated without proving that the correct scene was presented.

Regressions explicitly drive the action, layout/paint, deferred effect, and next eligible frame without sending a second user action. Cover:

- one Enter at paragraph and document tail positions;
- same-block and cached cross-block Up/Down;
- an unmeasured virtualized adjacent block;
- Select Up/Down;
- one press onto a whitespace row followed by a separate press across it;
- typewriter mode enabled and disabled;
- an idle interval with no incidental hover or background notification.

The tests compare source text, version, dirty flag, undo/redo length, shared visual/preview `Arc` identity, highlight memoization and cached text identity across presentation-only frames. Existing terminal-newline and viewport-content tests remain controls rather than being rewritten to accept extra frames or inputs.

### 5. Keep platform verification separate from deterministic acceptance

The root GPUI test host provides deterministic frame inspection and is the implementation gate. Native smoke testing on COSMIC Wayland, another Wayland compositor, and XWayland confirms platform behavior but must not replace the first-frame regression. Record the exact Markion build, typewriter setting, display backend, document fixture and action where a stale caret is observed.

## Risks / Trade-offs

- [A deferred callback targets state that has since changed] → Revalidate tab, version, block identity and the complete pending request before applying it.
- [Multiple target elements queue duplicate completion] → Queue only from the pending target and retain a per-request queued marker cleared on completion or cancellation.
- [Explicit follow requests create an idle redraw loop] → Keep the existing bounded counter and schedule only while scrolling or work remains.
- [Tests pass by reading updated model state rather than rendered output] → Observe the emitted caret at explicit frame boundaries and assert its source owner and bounds.
- [Immediate cached resolution changes whitespace semantics] → Keep whitespace and marker-only source-offset branches separate and retain their existing navigation tests.
- [Concurrent typewriter work overlaps the same rendering state] → Preserve its centering and content-coverage contracts, and run both focused suites together.

## Migration Plan

No data migration is required. First establish a failing first-painted-frame regression, then move cached resolution into the action path, introduce the deduplicated post-paint handoff, and correct bounded follow scheduling. Run focused Visual Edit navigation/Enter/typewriter tests, the root test suite, `cargo test --workspace`, formatting checks, and strict OpenSpec validation. Build a Linux artifact or equivalent debug binary for native Wayland/XWayland smoke testing.

Rollback removes the deferred-completion scheduling and immediate cached path while leaving canonical Markdown, terminal whitespace ownership, and persisted data unchanged.

## Open Questions

- Which Markion version, typewriter setting, display scale, and document structure produced the reporter's exact COSMIC cadence? This affects native verification coverage but not the specified scheduling contract or implementation approach.
