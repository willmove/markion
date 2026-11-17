## 1. Preference contract

- [x] 1.1 Expand paragraph spacing to 128px and add normalized 75–300% line spacing with default 100; wire persistence and defaults and verify old/malformed/extreme config plus round-trip tests.

## 2. Presentation and controls

- [x] 2.1 Apply line spacing through shared typography metrics and heading/table helpers; verify default equivalence, proportional line-height changes, glyph-size independence and reflow/cache invariants.
- [x] 2.2 Add Appearance controls, initialization/reset/persistence/summary and all eight translations; verify bounds, saved values, reset and no changes to document text/history.

## 3. Integrated verification

- [x] 3.1 Update the user guide and verify workspace tests, formatting, Clippy and strict OpenSpec validation.
- [x] 3.2 Verify the controls and rendered reflow in an isolated Windows app, including paragraph spacing above 32px, changed line spacing and restart persistence; capture the result and record limitations.
