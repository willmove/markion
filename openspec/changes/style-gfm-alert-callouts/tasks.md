## 1. Shared callout style

- [ ] 1.1 Add Lucide `info`, `lightbulb`, `message-square-warning`, `triangle-alert`, and `octagon-alert` SVGs under `assets/icons/ui/` and to `icon_set!`, plus `callout_icon`, `callout_background`, and a callout title-row helper in `src/app/preview.rs`; verify with `cargo build` and the icon asset tests (`cargo test --bin markion icon`)

## 2. Split Preview and Read mode

- [ ] 2.1 Render `PreviewBlock::BlockQuote { alert: Some(kind) }` as a callout card (accent 2 px left border, tinted gradient background, icon + label title row, body below); plain quotes unchanged; verify with an app test that an alert has `preview-callout-title-*` and a plain quote does not
- [ ] 2.2 Add per-tab `folded_preview_alerts` and make the title row toggle it (chevron icon, notify only); add an app test that clicking the title hides and restores the body while document text, version, dirty state, and undo history are unchanged; verify with `cargo test --bin markion preview_callout`

## 3. Visual Edit

- [ ] 3.1 Add `alert: Option<AlertKind>` to `VisualQuoteContext`, set it where `visual.rs` builds quote contexts, and add a lib test that title and body rows of an alert carry the kind while a plain quote carries `None`; verify with `cargo test --lib visual_quote_alert`
- [ ] 3.2 Use the kind accent border and tint in `visual_quote_row` for alert groups and add the icon to the `CalloutTitle` row; add an app test asserting the icon renders in Visual Edit and the marker reveal still works; verify with `cargo test --bin markion visual_callout`

## 4. Integration

- [ ] 4.1 Run `cargo fmt --check`, `cargo test --bin markion`, `cargo test --lib`, and confirm no new clippy lints on changed lines; validate with `openspec validate style-gfm-alert-callouts --strict`
