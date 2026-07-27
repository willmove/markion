## Why

The Preferences language picker can compress its seven pills until the selected check mark and native-language labels paint outside their borders on Windows systems whose UI font has wider metrics than the usual Segoe UI. Resolution, display scaling, and Markion version alone do not determine the result, so the picker must remain bounded across supported system-font and viewport variations instead of relying on one machine's text widths.

## What Changes

- Make every language pill keep enough horizontal space for its native label and selected indicator rather than shrinking the text below its measured content width.
- Render the selected check mark as a compact element beside the language name instead of embedding it and multiple spaces in one string; reserve the same leading slot for inactive pills so labels stay aligned.
- Give language pills a more comfortable minimum width, make the language row wrap as complete pills when the available width is insufficient, and modestly widen the General Preferences panel while keeping it bounded by the window.
- Add regression coverage for the language-picker structure and manually verify normal and wide/monospaced Windows UI-font metrics at 125% display scaling.

Non-goals: this change does not hard-code Segoe UI, replace GPUI's application-wide `.SystemUIFont` selection, change the supported language set, alter translations or persistence, or redesign unrelated Preferences controls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `theme-preferences`: strengthen the language-picker contract so every supported native-language label and its active marker remain contained and usable across font-metric, display-scale, and constrained-width variations.

## Impact

- **Code:** `src/app/root_view.rs` language-row rendering and a language-specific pill helper; focused structural/layout tests in `src/app/tests.rs`.
- **Behavior:** the General Preferences panel becomes slightly wider when space permits, and language pills wrap instead of compressing or overflowing when space is constrained.
- **APIs, persistence, and dependencies:** no changes.
- **Architecture invariants:** the cached Markdown-derived state, syntax-highlighting cache, cached editor text handles, and GPUI-free workspace-member boundary are not on this UI layout path and remain untouched.
