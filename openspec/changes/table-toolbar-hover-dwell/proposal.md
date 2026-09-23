## Why

In Visual Edit, the table editing header is an in-flow row inserted at the top of the table chrome the instant the pointer enters the table, and removed the instant it leaves. Moving the pointer across a document with several tables therefore makes the rendered content jump repeatedly (~26 px per show/hide), which users experience as constant visual shaking while simply moving the mouse.

## What Changes

- Gate hover-triggered visibility of the Visual Edit table editing header behind a fixed 250 ms hover dwell: the header appears only after the pointer has stayed over the table's chrome continuously for the dwell window.
- Add a fixed 120 ms hide delay: once shown, the header hides only after the pointer has left the table's chrome continuously for the hide window, so brief excursions do not flicker it away.
- Caret-driven visibility is unchanged: placing the canonical caret in a cell shows the header immediately (no dwell), and it remains shown while the caret belongs to that table.
- Both delays are hardcoded constants; no settings or preferences are added.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tables-outline`: the requirement governing when the Visual Edit table editing header appears on hover now requires a hover dwell before showing and a hide delay before hiding; caret-driven visibility remains immediate.

## Impact

- Code: `src/app/preview.rs` (`visual_table_view` hover handling), `src/app/state.rs` (per-tab hover/dwell state), `src/app/tests.rs` (hover toolbar tests must drive the dwell fire path directly, since `gpui::Timer` is real-time). Follows the existing generation-token + `Timer::after` + `cx.spawn` pattern used by `arm_preview_debounce` / `schedule_autosave`.
- Behavior: Visual Edit only; Split Preview and Read mode have no editing header and are unaffected.
- Non-goals: no floating/overlay toolbar redesign, no changes to toolbar contents or layout, no user-configurable delays.
