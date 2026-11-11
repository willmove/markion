## Context

See proposal.md. Windows IMM delivers marked text through `replace_and_mark_text_in_range` and the result through `replace_text_in_range`. Both enter the root tab's semantic undo capture after document/version validation. The current compatibility predicate applies the ordinary 900 ms timeout to every capture kind, including an active IME session.

## Goals / Non-Goals

**Goals:** keep candidate-selection pauses inside a live composition's undo entry, preserve exact Unicode source/selection restoration and visible-table viewport position, keep fresh sessions and ordinary typing separate, and give native text input a bounded frame-dispatch path.

**Non-Goals:** extend ordinary typing timeouts, retain composition ownership across tab/mode/selection changes, or change derived-cache ownership.

## Decisions

- Use composition lifecycle for `Ime` captures. An existing IME capture remains compatible until its input handler explicitly finishes it; elapsed time does not split that live capture. Other capture kinds keep their current timeout and continuity rules.
- Keep input generation validation and commit/unmark cleanup at their existing boundaries. Windows does not need a new input protocol or a special-case platform flag.
- Regress the real callback seam with simulated expired capture timestamps before intermediate updates and before final commitment. Test cancellation/fresh-session and ordinary typing boundaries as well.

IME data flow: physical key → Windows IMM marked/result callback → document/version guard → semantic capture → checked canonical edit → per-version derivation and paint. The undo fix changes only capture compatibility; snapshots continue to omit derived caches.

## Native mouse findings

Clearing a visible table rectangle invalidates the table row's measured height. The pre-layout caret reveal mistakes this already-visible row for an unmeasured off-screen target and pins it to the top; it can also reuse geometry from a previously focused field. Before synchronizing the list, retain only a boolean witness that the same source-owned table was fully within the viewport inset. Validate that the last checked change is wholly within both the old and new table ranges, with the same start and index. Consume the usual reveal request, skip coarse pinning for that witnessed row, discard the old caret bounds, and let the existing bounded post-paint caret follow handle any actual overflow. Off-screen/new rows retain the existing reveal path. This adds no document-sized cache or retained geometry.

The Copy button uses an automatic left margin so hiding the language chip does not move it from the right edge. This is an obvious one-line layout correction.

## Long-input tail diagnosis

Actual OS input produced a short tail-latency cluster despite exact source restoration and a passing native callback replay. System ETW recording was rejected by Windows policy (`0xc5585011`). Extend the existing opt-in, text-free performance observations with cumulative mutation/render/model checkpoints and current UI-thread CPU time. This separates CPU work from wall-clock waits without changing system policy, adding dependencies, or enabling profiling for normal users. Use the evidence before selecting any further optimization.

The checkpoints reproduced a 498 ms sample with about 1.3 ms of mutation work, a 468 ms wall-clock wait before render began (no additional UI-thread CPU), and about 24 ms to prepare the visual model. The Windows backend invalidates for vsync and renders through low-priority `WM_PAINT`. Add one posted application message after a successful character/composition callback or consumed key action. A per-window gate coalesces requests, resets on post failure, and is consumed before drawing. Drawing runs after the input handler has been restored, uses the existing dirty-frame callback without `force_render`, and retains the normal vsync/paint path. There is no synchronous draw from the input callback, polling timer, or idle redraw loop. Plain unconsumed keydown does not request an extra pre-character frame.

Reference: [GetMessage dispatch order](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessage) documents that posted messages precede input and `WM_PAINT`. This evidence supports testing a queued frame request; final native measurements must establish its effect.

## Risks / Trade-offs

- [A stale or ended capture could merge with a new session] → retain matching capture kinds, explicit commit/unmark/command cleanup and stale-event tests.
- [Synthetic callbacks miss timing seen in real IMEs] → age capture timestamps deterministically, then repeat physical Pinyin input, candidate selection, Undo/Redo, and large-document typing on Windows.
- [Long pauses retain one snapshot longer] → the session already owns that cache-free snapshot; there is no additional full-document or derived-state retention.
