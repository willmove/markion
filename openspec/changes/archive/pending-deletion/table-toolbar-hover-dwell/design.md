## Context

See proposal.md for motivation. The Visual Edit table editing header is rendered by `visual_table_view` (`src/app/preview.rs`) as an in-flow row gated by `visual_table_toolbar_is_visible(hovered, block_id, has_caret_target)`, where `hovered` is the per-tab `hovered_visual_table_block: Option<VisualBlockId>` set immediately by the table chrome's `on_hover` listener. Every show/hide inserts or removes the row, shifting layout. `gpui::Timer` is real-time and outside the test executor's clock, so the codebase already splits timer arming from firing (`arm_preview_debounce`, `schedule_autosave` / `run_due_autosave`) using a per-tab generation token so stale timers fire into a no-op.

## Goals / Non-Goals

**Goals:**
- Hover shows the header only after a continuous 250 ms dwell over the same table's chrome.
- Once shown, hover hides the header only after the pointer stays off the chrome for a continuous 120 ms.
- Caret-driven visibility stays immediate and is never delayed by the dwell logic.
- Tests can drive the dwell/hide fire paths deterministically without sleeping.

**Non-Goals:**
- No floating/overlay toolbar redesign (the header stays in-flow).
- No user-configurable delays; both are hardcoded constants.
- No change to the code-block `</>` or other hover chrome, which is already overlay-positioned.

## Decisions

- **State model**: keep `hovered_visual_table_block` as the raw, immediate hover truth. Add per-tab derived state: `visual_table_toolbar_hover_ready: Option<VisualBlockId>` (dwell satisfied for this block) plus a `visual_table_hover_generation: u64` token. Visibility becomes `has_caret_target || hover_ready == Some(block_id)` — the ready flag alone gates hover display because it already lags entry by the dwell and exit by the hide delay (gating on the raw hover too would hide instantly on leave and defeat the hide delay). Caret display therefore never waits on the timer.
- **Dwell on enter**: hover enter sets the raw field, bumps the generation, and arms a `TABLE_TOOLBAR_HOVER_DWELL = 250 ms` timer capturing `(tab index, block_id, generation)`. When it fires, `fire_visual_table_hover_dwell` sets `hover_ready = Some(block_id)` only if the generation matches and the raw hover still names that block; then `cx.notify()`.
- **Hide delay on leave**: hover leave clears the raw field and bumps the generation (cancelling any pending dwell). If `hover_ready` was set for that block, arm a `TABLE_TOOLBAR_HIDE_DELAY = 120 ms` timer instead of clearing `hover_ready` immediately; `fire_visual_table_hide_delay` clears it only if the generation matches and the pointer has not re-entered. Re-entering within the window bumps the generation, so the hide timer no-ops and the header stays up without flicker.
- **Arm/fire split**: both timers call plain methods on `MarkionApp` (same shape as `run_due_autosave`), so `gpui::test`s set state and invoke the fire methods directly; no real-time sleeping in tests.
- **Cleanup points**: wherever `hovered_visual_table_block` is reset today (`state.rs` tab switch, mode change, document replace), reset `visual_table_toolbar_hover_ready` and bump the generation alongside it so a stale timer can never resurrect a header.
- **Constants**: `TABLE_TOOLBAR_HOVER_DWELL: Duration = Duration::from_millis(250)` and `TABLE_TOOLBAR_HIDE_DELAY: Duration = Duration::from_millis(120)`, declared next to the other preview timing constants.

## Risks / Trade-offs

- A table the user dwells on still shifts layout once when the header appears → accepted; the annoying repeated shaking came from pass-through hover, which the dwell eliminates. A future overlay toolbar could remove the remaining shift.
- Timers captured against a closed/switched tab must no-op → the fire methods validate the tab index and generation before mutating, mirroring `run_due_autosave`.
- Existing tests set `hovered_visual_table_block` directly and expect the header → update them to also drive the dwell fire path (or set `hover_ready` directly) so the new gating is what gets pinned.
