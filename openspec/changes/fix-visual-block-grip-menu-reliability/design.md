## Context

Investigation of macOS Visual Edit block-grip click reliability identified four distinct failure mechanisms plus one deterministic chrome omission:

1. **GPUI 2 px drag threshold (`vendor/zed/crates/gpui/src/elements/div.rs:47, 2159`):** `visual-block-drag-grip` binds both `.on_click` and `.on_drag`. Any `NSLeftMouseDragged` displacement > `2.0 px` starts `cx.active_drag` and clears `pending_mouse_down`, cancelling `.on_click`. On macOS Force Touch trackpads and Magic Mouse, clicks routinely drift 2–6 pt, turning ordinary clicks into aborted drags.
2. **Eager mouse-down selection reflow (`src/app/preview.rs:5528-5540`, `div.rs:2219-2233`):** `on_mouse_down` calls `select_visual_block_grip` -> `select_source_blocks` -> `cx.notify()`. Over a real 60–150 ms click, GPUI renders intermediate frames where a previously focused block above collapses (collapsible math/diagram/HTML/YAML source pane, wrapped revealed Markdown syntax, TOC/callout/source-island, or list bottom clamping). Because the grip is 22 px tall, `hitbox.is_hovered(window)` becomes `false` on `MouseUp`, causing GPUI to silently drop `pending_mouse_down`.
3. **Unfiltered scroll dismissal (`src/app/root_view.rs:2736-2738`, `platform/mac/events.rs:217-246`):** Every `ScrollWheelEvent` calls `close_visual_block_menu(cx)`, including macOS `NSEventPhaseMayBegin`/`NSEventPhaseEnded` zero-delta events and tiny inertia/Magic Mouse micro-scrolls arriving right after click release.
4. **macOS pre-dispatch IME commit (`platform/mac/window.rs:1844-1850`, `src/source_mapped.rs:415-467`):** macOS invokes `NSTextInputContext handleEvent:` synchronously before dispatching `MouseDown`. Committing active `marked_range` increments `document.version()` and may allocate fresh `VisualBlockId`s before the stale rendered frame handles `MouseDown`/`MouseUp`, failing version validation or resetting GPUI element state.
5. **MathBlock early return (`src/app/preview.rs:5189-5201`):** `VisualBlockKind::MathBlock` returns `visual_math_editor(...)` early before `visual_block_chrome`, omitting its grip.

## Goals / Non-Goals

**Goals:**
- Make ordinary Visual Edit block-grip clicks reliably open the quick-settings popup across macOS trackpads, Magic Mouse, multi-frame clicks, residual scroll phases, and active IME composition.
- Keep resize handles responsive at GPUI's default 2 px drag threshold while giving dual click+drag grips a 6 px threshold.
- Preserve all cached-per-version Markdown invariants, multi-block Shift-click and drag-reorder semantics, and strict stale-target rejection for external/unrelated document changes.
- Ensure `MathBlock` exposes `visual_block_chrome` like other reorderable blocks.

**Non-Goals:**
- Changing global GPUI `DRAG_THRESHOLD` for non-grip draggable elements.
- Reparsing Markdown or invalidating `visual_blocks_shared` on pointer/menu interactions.
- Adding new menu commands or localization keys.

## Decisions

### 1. Per-element `drag_threshold` in vendored GPUI
Add `pub(crate) drag_threshold: Option<Pixels>` on `Interactivity` in `vendor/zed/crates/gpui/src/elements/div.rs` with `.drag_threshold(Pixels)` on `Interactivity` and `StatefulInteractiveElement`. Default falls back to `DRAG_THRESHOLD` (`2.0`). Set `px(VISUAL_BLOCK_GRIP_DRAG_THRESHOLD)` (`6.0 px`) on `visual-block-drag-grip` and `visual_table_grip`.

### 2. Explicit transient `PendingBlockGripPress` lifecycle in `MarkionApp`

```text
Unmodified MouseDown on grip
  -> validate target (or relocate across same-tick IME commit)
  -> dismiss any open overlay (presentation-only, no list reflow)
  -> store PendingBlockGripPress { document_id, target, down_position } (NO selection mutation yet)

Branch A: pointer moves > 6.0 px -> GPUI on_drag fires
  -> begin_visual_block_grip_drag(): take pending_block_grip, call select_visual_block_grip(target, false, preserve_selected=true)
  -> existing DraggedVisualBlock drag-and-drop runs unchanged

Branch B: MouseUp without active drag & distance <= 6.0 px & !has_grip_click_modifiers(modifiers)
  -> on_mouse_up(Left) in Bubble phase on visual_edit_surface_view runs AFTER GPUI's descendant MouseUp Capture listeners clear `pending_mouse_down` and `clicked_state`
  -> task-checkbox `on_mouse_up` yields while `pending_block_grip.is_some()` so shifted children cannot misfire
  -> finish_visual_block_grip_press(): calls open_visual_block_grip_menu(target, position, cx)
  -> selects canonical single unit + opens popup (if block_handle_menu enabled)

Branch C: Shift + MouseDown on grip
  -> clear pending_block_grip, immediately extend selection via select_visual_block_grip(..., extend=true)

Cancel paths: on_mouse_up_out, Escape, tab/mode switch, document switch, close_menu, non-grip pointer down
  -> clear pending_block_grip without mutating document or selection
```

Keyboard clicks (`ClickEvent::Keyboard`) remain wired on `.on_click` so keyboard activation still opens the menu.

### 3. Scroll-wheel noise filtering for menu dismissal
In `MarkionApp`, track `visual_menu_scroll_dismiss_accum: f32` (reset to `0.0` whenever a block or table menu opens or closes). In `handle_visual_block_menu_scroll(event, cx)`:
- Convert `event.delta.pixel_delta(px(20.))` (matching `gpui::List`) and take `delta.x.abs().max(delta.y.abs())`.
- Ignore `magnitude <= f32::EPSILON` (zero-delta `MayBegin`/`Ended` events).
- Accumulate `magnitude` and call `close_visual_block_menu(cx)` once accumulated distance `>= 4.0 px`.

### 4. Bounded IME pre-dispatch target relocation
Record `recent_ime_commit: Option<RecentImeCommit>` (`document: (DocumentInstanceId, u64)`, `before_version: u64`, `at: Instant`) when `replace_text_in_range` applies a mutation while `active_marked_range.is_some()`.
During `begin_visual_block_grip_press`, if `validate_block_target` fails because `target.document_version != current_version`, only if `recent_ime_commit` matches `(current_doc_id, current_version)` with `before_version == target.document_version` and `at.elapsed() <= Duration::from_millis(250)`:
- Read `tab.document.last_change_set()` and verify `change.before_version == target.document_version && change.after_version == current_version`.
- Map `target.source_range` unambiguously across `change` (before edit: unchanged; after edit: shifted by signed length delta; containing edit: match the unique reorderable block covering the relocated start).
- Construct `BlockTarget::from_block(current_version, &blocks[index])`.
All other stale targets (external reload, programmatic stale calls, non-IME edits) continue to fail validation and are rejected.

### 5. `MathBlock` chrome fallthrough, `NSFunctionKeyMask` filtering, and exclusive collapsible ends
- In `src/app/preview.rs`, change the `VisualBlockKind::MathBlock` arm in `visual_block_content_view` from `return visual_math_editor(...)` to an expression returning the `Div` so it flows through `visual_block_chrome`, `indent_list_nested_row`, and `quote_context` like `CodeBlock`, `Html`, `FrontMatter`, and `Table`.
- Check explicit pointer modifiers (`shift || control || alt || platform`) via `MarkionApp::has_grip_click_modifiers` instead of `Modifiers::modified()`, because GPUI includes `modifiers.function` (`NSFunctionKeyMask` on macOS) which navigation/Fn/Globe keys can assert without being a pointer click modifier.
- Gate `caret_in_payload` in `visual_collapsible_source_block` on `tab.structured_selection.is_none()` and half-open selection intersection so selecting a preceding block via its grip (whose exclusive `selected_range.end` equals the next block's start) does not falsely expand a following collapsible source pane.

## Caching & Versioning Impact

- `PendingBlockGripPress`, `visual_menu_scroll_dismiss_accum`, and `RecentImeCommit` are lightweight transient UI fields on `MarkionApp` (root crate only).
- No Markdown reparsing, no `VisualBlock` rebuild, no highlight/text-handle invalidation, and no undo snapshot overhead occur during hover, press, click, scroll filtering, or menu open/dismiss.
- All document edits from the opened menu continue through the checked versioned `BlockTarget` mutation pipeline.

## Risks / Trade-offs

- [Deferring unmodified selection from `MouseDown` to `MouseUp` / drag start] -> Keeps drag-selection preservation (`preserve_selected = true`) identical once drag starts; verified against all existing tests in `block_grip_tests.rs` and `tests.rs`.
- [Resolving grip clicks in Bubble `on_mouse_up` on `visual_edit_surface_view`] -> Keeps GPUI's descendant Capture-phase `MouseUp` listeners intact so `pending_mouse_down` and `clicked_state` always reset (preventing phantom post-click drags), while guarding `toggle_visual_task_checkbox` on `pending_block_grip.is_some()` so shifted checkboxes never misfire.
- [6 px drag threshold on grips] -> Large enough to absorb macOS trackpad/Magic Mouse click jitter, small enough that deliberate block reordering feels immediate.
