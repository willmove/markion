## 1. Establish presented-frame regressions

- [x] 1.1 Add a test-only paint observation that records the source selection head, owning visual block identity, document version, caret bounds, and explicit frame generation for the caret actually emitted by `VisualEditableText`; prove it does not alter production layout or state.
- [x] 1.2 Add failing one-action regressions for Enter, cached adjacent-block Up/Down, Select Up/Down, and an otherwise idle window. Inspect the first eligible presented frame and demonstrate that no second input is allowed to make the assertion pass.
- [x] 1.3 Add a failing virtualized-target regression that drives reveal, measurement, deferred effects, and the next frame explicitly; include replacement/cancellation of a stale pending request after a document, tab, mode, pointer, or newer-navigation change.
- [x] 1.4 Retain first-class whitespace semantics in controls: one action paints the caret on the blank row and a separate second action crosses it, with preferred X, text, version, dirty state, history, and derived cache identity preserved.

## 2. Move navigation completion outside paint

- [x] 2.1 Extract a side-effect-free pending-navigation resolver that validates document version, block index and stable ID, snapshot version, direction, extension mode, preferred X, and marker-only/whitespace fallbacks.
- [x] 2.2 Resolve adjacent blocks with current geometry synchronously in the originating Up/Down or selection action, updating selection and line-position state once while preserving minimal reveal and the existing whitespace/callout behavior.
- [x] 2.3 Add per-request deferred-completion identity and cancellation state to the active tab, clearing or replacing it through the existing document, tab, mode, pointer, and navigation intent paths without touching persisted or per-version document state.
- [x] 2.4 Change visual text paint to publish target geometry only and queue at most one post-paint completion for the matching pending target; revalidate the complete request in the callback, then move/select and notify outside the GPUI draw phase.

## 3. Schedule bounded caret-follow frames explicitly

- [x] 3.1 Replace draw-phase ordinary notification in `VisualInputElement` caret-follow reconciliation with an explicit GPUI animation-frame request whenever scrolling occurred or the bounded follow budget remains.
- [x] 3.2 Verify that the follow budget reaches zero without an idle redraw loop, manual scroll remains stable until later caret activity, and typewriter enabled/disabled behavior retains its existing centering, content-coverage, and request-expiration contracts.

## 4. Verify integration and platform behavior

- [x] 4.1 Run the new presented-frame tests plus existing Visual Edit Enter, blank-line navigation, virtualized navigation, viewport reveal, and small-window typewriter/content-coverage suites; record exact commands and before/after evidence in `verification.md`.
- [x] 4.2 Run `cargo fmt --check`, `cargo test`, `cargo test --workspace`, scoped diff/whitespace checks, and `openspec validate fix-visual-edit-deferred-caret-repaint --strict`; do not modify unrelated files to satisfy checks.
- [ ] 4.3 Build a current debug or Linux artifact and smoke-test the minimal fixtures on COSMIC Wayland, another available Wayland compositor, and XWayland with typewriter mode both enabled and disabled. Record the Markion build, display backend, scaling, fixture, and first presented frame; keep unavailable environments explicitly pending rather than claiming native verification.
