## 1. Reproduction and fix

- [x] 1.1 Add deterministic regressions for a candidate-selection pause before an IME update and before commit; observe the original failure.
- [x] 1.2 Keep active IME capture grouping independent of ordinary typing timeout while preserving explicit session boundaries and stale-input guards.

## 2. Verification

- [x] 2.1 Run focused IME, semantic undo and mutation regressions, then the full root desktop regression and formatting checks.
- [x] 2.2 Verify physical Pinyin candidate display, delayed commit, one-step Undo/Redo, cancellation and mouse navigation in the actual Windows app.
- [ ] 2.3 Verify actual large-document continuous input and mouse interactions; record native timing/memory and any remaining limitations.
- [ ] 2.4 Validate and archive the change and update the recovered automation evidence.

## 3. Native mouse findings

- [x] 3.1 Regress and fix the visible-table clear/undo viewport jump without disabling off-screen caret reveal.
- [ ] 3.2 Keep the code Copy button right-aligned when the language chip is hidden and verify it in the native app.
- [ ] 3.3 Retest visible-table clear/undo on the final release after the Windows desktop is unlocked.
