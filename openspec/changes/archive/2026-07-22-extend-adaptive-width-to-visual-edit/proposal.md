## Why

Visual Edit mode currently always fills the full pane width regardless of the "Preview adaptive width" preference, producing over-long prose lines on wide windows that diverge from the readable, centered layout Read mode provides. The preference already governs Read mode width and is on by default disabled; extending it to Visual Edit makes the two rendered-content modes visually consistent and gives users one control for readable content width.

## What Changes

- Visual Edit mode SHALL constrain rendered content to the same maximum width (860px) and center it within the available pane, matching Read mode, when "Preview adaptive width" is disabled (the default).
- When "Preview adaptive width" is enabled, Visual Edit mode SHALL use the full available pane width, matching Read mode behavior.
- The shared width-constraint helper (`read_mode_preview_is_constrained`) SHALL apply to both Read and Visual Edit modes.
- Split Preview and Edit modes remain unaffected by the preference, as today.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `chrome-platform`: The "Read mode preview width" requirement is renamed/expanded to cover Visual Edit mode — both rendered-content modes (Read and Visual Edit) constrain content width by default and respond to the "Preview adaptive width" preference identically. Split Preview and Edit remain unaffected.
- `theme-preferences`: The "Preview adaptive width" toggle description is updated to state it applies to both Read and Visual Edit rendered content (no behavioral change to the toggle itself, which already defaults to disabled and persists as-is).

## Impact

- `src/app/preview.rs` — `read_mode_preview_is_constrained` now also returns true for `ViewMode::VisualEdit` when the preference is off.
- `src/app/root_view.rs` — `visual_edit_surface_view` applies the same `max_w` + `justify_center` wrapper the Read/Split preview list already uses; the function needs access to the constraint flag (passed in, not read from a global).
- `src/app/tests.rs` — existing `read_mode_preview_width_cap_only_applies_without_adaptive_width` test assertions for `VisualEdit` flip to expect constraint; add a scenario for Visual Edit + adaptive width enabled.
- The cached derived-state invariants are untouched: width is a pure layout concern layered on top of the existing visual block list, not a re-parse.
- Non-goals: changing the max-width value (860px), changing Edit or Split Preview width, or introducing a per-mode width preference.
