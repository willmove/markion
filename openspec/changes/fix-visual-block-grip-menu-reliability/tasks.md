# Tasks

## 1. Per-element GPUI drag threshold

- [x] 1.1 Add optional `drag_threshold: Option<Pixels>` with `.drag_threshold(Pixels)` on `Interactivity` and `StatefulInteractiveElement` in `vendor/zed/crates/gpui/src/elements/div.rs` (defaulting to `DRAG_THRESHOLD = 2.`), and apply `px(6.)` to `visual-block-drag-grip` and `visual_table_grip` in `src/app/preview.rs`. Verify `cargo test block_grip` builds and passes with small-jitter click tests confirming no premature drag below 6 px and drag activation past 6 px.

## 2. Explicit block-grip press lifecycle, IME commit relocation, and MathBlock chrome

- [x] 2.1 Add transient `PendingBlockGripPress` and `RecentImeCommit` state on `MarkionApp` (`src/app/mod.rs`, `src/app/application.rs`, `src/app/editor_element.rs`), defer unmodified single-block selection from grip `MouseDown` to drag start (`on_drag`) or click release (`on_mouse_up` / `on_mouse_up_out` on `visual_edit_surface_view`), and relocate stale grip targets across same-tick IME commits via `last_change_set()`. Verify with GPUI tests covering intermediate-frame layout collapse between `MouseDown` and `MouseUp`, Shift-click, multi-block drag, post-click unpressed pointer movement (no phantom drag), IME pre-dispatch commit, and strict rejection of genuinely stale targets.
- [x] 2.2 Remove the early `return` for `VisualBlockKind::MathBlock` in `src/app/preview.rs` so display math blocks pass through `visual_block_chrome`, list indentation, and quote wrapping, ignore macOS `modifiers.function` (`NSFunctionKeyMask`) on grip click checks, and gate `caret_in_payload` in `visual_collapsible_source_block` against exclusive block-selection ends. Verify with GPUI tests confirming display math block grips, `modifiers.function == true` grip clicks, and collapsed following HTML/collapsible blocks.

## 3. Scroll-wheel dismissal noise filtering

- [x] 3.1 Filter zero-delta `ScrollWheelEvent` phases and require accumulated scroll displacement `>= 4.0 px` before closing `block_menu` / `visual_table_menu` in `src/app/root_view.rs` and `src/app/editing.rs`. Verify with GPUI tests confirming zero-delta (`Started`/`Ended`) and sub-threshold micro scroll deltas keep the menu open while deliberate scrolling closes it presentation-only.

## 4. Full verification and macOS native check

- [x] 4.1 Run `cargo test --workspace` and confirm every crate test suite passes (all member crates and `markion` lib/integration/block_grip suites 0 failures; note the 5 pre-existing macOS-only failures in `src/app/tests.rs` from `/var` vs `/private/var` temp paths and Windows/Linux `ctrl-b` default shortcut assumption).
- [x] 4.2 Build and verify the macOS desktop app natively on this machine (6 live `CGEvent` scenarios for display math grip, 4 pt jitter click, unpressed post-click move + micro-scroll vs deliberate scroll, 180 ms held click with `NSFunctionKeyMask` across 142 pt HTML collapse, and 30 pt drag, plus `MARKION_NATIVE_SMOKE_DIR` native window smoke check: `passed: true`, 9 observations), then validate the OpenSpec change with `openspec validate fix-visual-block-grip-menu-reliability`.
