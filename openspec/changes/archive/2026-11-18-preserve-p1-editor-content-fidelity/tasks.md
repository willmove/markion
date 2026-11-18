## 1. Baseline

- [x] 1.1 Verify #30, #31 and #32 are already closed with fixes on main. Reproduce #34, #36, #38, #41, #47, #52, #53 and #54 through the released code paths and reproduce #27 in a real Windows window at `a2fd67d`.

The baseline changed link targets, lost DOCX cell content and drawings, interpreted literal prose as markup, changed list ownership, and showed white table cells against a dark theme.

## 2. Implementation

- [x] 2.1 Verify list ownership, legal one-level transitions, and moves beside independent prose (#52, #53, #54).
- [x] 2.2 Verify explicit link destinations and literal imported prose through the relevant preview and export paths (#34, #41, #47).
- [x] 2.3 Verify rich Word cells, text and drawing order, and reported loss on unsupported content (#36, #38).
- [x] 2.4 Verify table colors and controls in dark and light presets (#27).

## 3. Verification

- [x] 3.1 Run regressions, locked workspace tests, formatting, OpenSpec validation, and native Windows checks.

The locked workspace suite passed 2,263 tests with 5 ignored. The Windows app verified list nesting, movement, saved Markdown, One Dark and One Light table rendering, Copy Link Address, rich clipboard paste, and DOCX import, save and reopen with embedded image assets. Rich paste passed with HTML and Unicode text clipboard flavors. The existing Windows clipboard adapter returned an empty result for the HTML-only fixture.
