## 1. Reproduction and fix

- [x] 1.1 Add deterministic regressions for a candidate-selection pause before an IME update and before commit; observe the original failure.
- [x] 1.2 Keep active IME capture grouping independent of ordinary typing timeout while preserving explicit session boundaries and stale-input guards.

## 2. Verification

- [x] 2.1 Run focused IME, semantic undo and mutation regressions, then the full root desktop regression and formatting checks.
- [x] 2.2 Verify physical Pinyin candidate display, delayed commit, one-step Undo/Redo, cancellation and mouse navigation in the actual Windows app.
- [x] 2.3 Verify actual large-document continuous input and mouse interactions; record native timing/memory and any remaining limitations.
- [x] 2.4 Validate and update recovered automation evidence; complete the change for archive.

## 3. Native mouse findings

- [x] 3.1 Regress and fix the visible-table clear/undo viewport jump without disabling off-screen caret reveal.
- [x] 3.2 Keep the code Copy button right-aligned when the language chip is hidden and verify it in the native app.
- [x] 3.3 Retest visible-table clear/undo on the final release after the Windows desktop is unlocked.

## 4. Long-input tail diagnosis

- [x] 4.1 Add opt-in application checkpoints and UI-thread CPU attribution; collect evidence for the native-input tail without changing Windows policy.
- [x] 4.2 Resolve any confirmed application bottleneck and revalidate, or document a demonstrated external limit with the measured evidence.
- [x] 4.3 Verify coalescing, post-failure recovery and reentrant frame requests; repeat long OS input and native IME verification after the Windows frame-scheduling fix.
