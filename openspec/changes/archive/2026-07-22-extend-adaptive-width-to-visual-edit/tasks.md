# Tasks: Extend adaptive width to Visual Edit

## 1. Core width-constraint logic

- [x] 1.1 Update `read_mode_preview_is_constrained` in `src/app/preview.rs` to return true for `ViewMode::VisualEdit` (in addition to `ViewMode::Read`) when `preview_adaptive_width` is disabled.
- [x] 1.2 Pass the constraint flag into `visual_edit_surface_view` in `src/app/root_view.rs` (add a `constrain_width: bool` parameter) so the visual edit list can apply the same `max_w` + `justify_center` wrapper used by the Read/Split preview list.

## 2. Visual Edit surface layout

- [x] 2.1 In `visual_edit_surface_view`, wrap each visual block row in a centered `max_w(READ_MODE_PREVIEW_MAX_WIDTH)` container when the constraint flag is set, mirroring the Read preview list rows (`root_view.rs:447-458`).
- [x] 2.2 Keep the empty-state placeholder ("Type Markdown here...") full-width regardless of the constraint flag (it is not rendered content).
- [x] 2.3 Update the call site in `render` (`root_view.rs:311-321`) to pass `constrain_read_preview` into `visual_edit_surface_view`.

## 3. Tests

- [x] 3.1 Update `read_mode_preview_width_cap_only_applies_without_adaptive_width` in `src/app/tests.rs`: Visual Edit now constrains when adaptive width is off, and uses full width when on. Assert all four modes (Read/VisualEdit/Split/Edit).
- [x] 3.2 Add (or extend) a test asserting `read_mode_preview_is_constrained(ViewMode::VisualEdit, true)` is false (adaptive width on = full width).
- [x] 3.3 Run `cargo test` for the root package and confirm all tests pass.

## 4. Validation and archive prep

- [x] 4.1 Run `openspec validate extend-adaptive-width-to-visual-edit` and fix any reported issues.
- [x] 4.2 Run `cargo build` to confirm the root package compiles cleanly.
