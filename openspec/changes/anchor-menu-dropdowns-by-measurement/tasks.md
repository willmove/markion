## 1. Measurement core (mod.rs)

- [x] 1.1 Add `menu_dropdown_offsets_from_label_widths(widths: &[f32; 6]) -> [f32; 6]` implementing `8 + Σ(previous widths + 20)`, plus a measurement helper that shapes the six active-language menu labels with `Font { family: ".SystemUIFont", ..Default::default() }` at 13px via `cx.text_system().layout_line(...)` and feeds their widths to the arithmetic helper; verify with `cargo build`.
- [x] 1.2 Add `menu_dropdown_offsets: [f32; 6]` and its measured-language key to `MarkionApp` (init in `MarkionApp::new` via the measurement helper for the default language, `src/app/application.rs`), plus an `ensure_menu_dropdown_offsets` method that re-measures when the cached language no longer matches; verify with `cargo build`.

## 2. Call-site switch (mod.rs / root_view.rs)

- [x] 2.1 Change `AppMenu::dropdown_left` to a pure index lookup over a passed-in `[f32; 6]` table, delete the per-language constants table, and switch every call site (the dropdown panel builder and the four flyout anchors — Open Recent, Backup and Sync, Advanced Git Tools, Format Images) to pass the app's cached table; verify with `cargo build`.
- [x] 2.2 Call `ensure_menu_dropdown_offsets` at the top of the root render so language changes realign on the next frame; verify with `cargo build` that no borrow conflicts arise (the method takes `&mut self` while render also holds `window`/`cx`).

## 3. Tests (tests.rs)

- [x] 3.1 Add a pure `#[test]` for `menu_dropdown_offsets_from_label_widths` covering the cumulative arithmetic (e.g. equal widths → uniform +20+width steps; unequal widths → verified hand-computed table).
- [x] 3.2 Update the source-contract test currently asserting `AppMenu::File.dropdown_left(self.language)` to assert the new measured-table call shape and that the render entry calls the `ensure` method; verify with `cargo test`.

## 4. Verification

- [x] 4.1 Run `cargo fmt`, `cargo test`, and `cargo test --workspace`; all green with no new warnings.
- [ ] 4.2 GUI smoke across languages: open every top-level dropdown in English, Simplified Chinese, and Japanese (the three previously miscalibrated scripts) and confirm each panel's left edge aligns with its title button; switch language at runtime and confirm realignment on the next open.
