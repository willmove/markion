## 1. State and timers

- [x] 1.1 Add `TABLE_TOOLBAR_HOVER_DWELL` (250 ms) and `TABLE_TOOLBAR_HIDE_DELAY` (120 ms) constants, and per-tab state `visual_table_toolbar_hover_ready: Option<VisualBlockId>` + `visual_table_hover_generation: u64` in `src/app/state.rs`; verify `cargo build` succeeds
- [x] 1.2 Reset `visual_table_toolbar_hover_ready` and bump the generation at every existing `hovered_visual_table_block` reset site in `src/app/state.rs` (tab switch, mode change, document replace); verify with `cargo build`

## 2. Dwell and hide-delay logic

- [x] 2.1 Implement `arm_visual_table_hover_dwell` + `fire_visual_table_hover_dwell` and `fire_visual_table_hide_delay` on `MarkionApp` following the generation-token + `Timer::after` + `cx.spawn` pattern of `schedule_autosave`/`run_due_autosave` (fire methods validate tab index, generation, and current raw hover before mutating); verify `cargo build` succeeds
- [x] 2.2 Rewire the table chrome `on_hover` listener in `visual_table_view` (`src/app/preview.rs`): enter sets raw hover + arms the dwell; leave clears raw hover, bumps generation, and arms the hide delay when `hover_ready` was set; verify `cargo build` succeeds
- [x] 2.3 Gate `visual_table_toolbar_is_visible` on `(hovered == Some(block_id) && hover_ready == Some(block_id)) || has_caret_target` so caret display stays immediate; verify `cargo test` passes for existing caret-driven toolbar tests

## 3. Tests

- [x] 3.1 Update existing hover-toolbar tests in `src/app/tests.rs` that set `hovered_visual_table_block` directly to also drive the dwell fire path (or set `hover_ready`), and verify `cargo test` passes
- [x] 3.2 Add tests: dwell fire shows the header; firing with a stale generation or after pointer leave shows nothing; hide-delay fire hides only when the pointer is still away; re-entering within the hide window keeps the header; caret display remains immediate without any dwell. Verify `cargo test` passes

## 4. Verification

- [x] 4.1 Run `cargo test` and confirm the full suite is green
- [ ] 4.2 Manual smoke in Visual Edit: sweep the pointer across a document with several tables (no layout shake), dwell on one table (header appears after ~250 ms), click a cell (header appears immediately), and wiggle the pointer briefly off a shown header (it stays)
