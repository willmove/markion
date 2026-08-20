# Tasks — add-tab-context-menu

## 1. State & i18n scaffolding

- [x] 1.1 Add `TabContextMenu { index, identity, position }` state, `TabContextAction` enum, and menu mutual-exclusion (clear file-tree/preview menus and `active_menu` on open; clear tab menu when other menus open) in `src/app/mod.rs` / `src/app/workspace.rs`
- [x] 1.2 Add i18n `Msg` entries (EN + zh-Hans): six item labels, close-others summary dialog title/detail/buttons, `StatusCopiedPath`, kept-tabs statuses; wire the `tab_context_action_label` mapping

## 2. Menu open/close plumbing

- [x] 2.1 Add `on_mouse_up(MouseButton::Right, …)` to the tab element in `tab_bar_view` (`src/app/editing.rs`) capturing the clicked index, tab identity (`path()`/title fallback), and event position into `TabContextMenu`
- [x] 2.2 Add `tab_context_menu_view` in `src/app/root_view.rs` (`anchored()` + `occlude()`, disabled-item styling per `PreviewContextMenu`, grouped-slice separators) and wire it into the root view next to the file-tree menu, with click-away dismissal
- [x] 2.3 Add stale-target re-resolution at dispatch (D1): validate stored index against captured identity; on mismatch cancel with status instead of acting

## 3. Actions

- [x] 3.1 Close Tab handler: switch-then-operate reusing `close_tab` unchanged
- [x] 3.2 Batch close helper (D2): close in-scope clean tabs via the shared `close_tab_confirmed`-equivalent removal path (image-claim release, session persist, last-tab-leaves-untitled rule); kept dirty tabs → single summary dialog with Discard-all / Cancel
- [x] 3.3 Close Others and Close to the Right handlers built on the batch helper (clicked tab never in scope)
- [x] 3.4 Rename handler: disabled for untitled; switch-then-operate then reuse `PendingNameKind::Rename` pipeline (prefill file name); render `pending_name_prompt_view` in the tab bar row when the file-tree panel is hidden (D3)
- [x] 3.5 Copy File Path handler (disabled for untitled): clipboard write + status message
- [x] 3.6 Reveal in File Manager handler (disabled for untitled): reuse `reveal_in_system_file_manager`
- [x] 3.7 Middle-click-to-close on tab items (switch + `close_tab`, same as `×`)

## 4. Tests & validation

- [x] 4.1 Menu open/dispatch tests: right-click opens menu; Close Tab switches then closes; disabled items on untitled tabs dispatch nothing; menu exclusivity and click-away
- [x] 4.2 Batch-close tests: all-clean closes silently; dirty tabs kept + dialog; discard-all closes them; last-tab fallback leaves untitled document
- [x] 4.3 Stale-target test: mutate tabs while menu open → dispatch cancels without closing the wrong tab
- [x] 4.4 Rename-from-tab tests: prompt prefilled, dirty refusal status, renamed file re-points open tabs
- [x] 4.5 `cargo test --workspace` green; `openspec validate add-tab-context-menu` passes
