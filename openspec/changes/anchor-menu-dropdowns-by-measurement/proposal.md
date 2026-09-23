## Why

The in-window menu dropdowns are positioned by a hand-tuned, per-language constants table that assumes one specific system UI font. It drifted the moment either the language or the machine's UI font differed from the tuning setup (Japanese Help was off by 46px; English Export/Help by ~30px), and each fix requires pixel forensics on screenshots. Measuring the actual rendered label widths at runtime with gpui's own text system makes the alignment correct by construction for every language, system font, and machine, and deletes the maintenance burden.

## What Changes

- Replace the hand-tuned per-language `AppMenu::dropdown_left` constants table with offsets computed at runtime from the measured widths of the six menu-bar labels, using the same text system and font (`.SystemUIFont`, resolved per platform) that renders the menu bar, combined with the fixed layout parameters (menu-bar left padding 8, per-button padding 2×8, inter-button gap 4).
- Cache the computed six-entry offset table on the app and re-measure lazily when the interface language changes (at the top of the root render), so no call-site bookkeeping is needed; the current table stays valid across frames.
- Keep everything else about dropdown placement unchanged: dropdown widths, the flyout submenus' row-based vertical offsets, paint order, theme styling, and hover/click behavior.
- Update the source-contract test that asserts the old `dropdown_left(self.language)` call shape, and add a pure-function unit test for the width→offset arithmetic in the repository's existing pure-fn test style.

Non-goals: restructuring the element tree (no `deferred`/popover rework — gpui 0.2.2 has no popover-anchor API and nesting dropdowns inside the menu-bar band would break paint order); changing dropdown widths or the flyout submenus' vertical row offsets (fixed 24px rows, not font-dependent); touching the native macOS menu; changing menu labels, actions, or behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Add a requirement that in-window dropdown panels align with their title buttons in every interface language and under the machine's actual system UI font, derived from runtime measurement rather than fixed constants.

## Impact

- Affected application code: `src/app/mod.rs` (`AppMenu` offsets API, new measurement + pure-arithmetic helpers, cache fields on `MarkionApp`), `src/app/application.rs` (initial cache construction), `src/app/root_view.rs` (lazy re-measure on language change at render entry; all `dropdown_left` call sites), and source-contract tests in `src/app/tests.rs`.
- The measurement uses `TextSystem::layout_line` (already used elsewhere in the app for font metrics), so no new dependencies; rendering cost is six cached line layouts per language, amortized to once per language switch.
- No document data, preferences, shortcuts, workspace members, or user-visible menu content change — only the horizontal anchor of the dropdown panels becomes measurement-derived.
