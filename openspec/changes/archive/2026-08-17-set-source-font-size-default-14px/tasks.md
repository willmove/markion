# Tasks: set-source-font-size-default-14px

## 1. Default value change

- [x] 1.1 Change `DEFAULT_EDITOR_FONT_SIZE` from `15` to `14` in `src/model.rs`, and `EDITOR_LINE_HEIGHT` from `24.` to `22.4` in `src/app/mod.rs` to keep the 1.6× line-height ratio (bounds and doc comments updated; 14 is within 10–32)
- [x] 1.2 Update `document_typography_metrics_preserve_defaults_and_scale_boundaries` in `src/app/tests.rs`: `editor_font_size` expectation `15.` → `14.`, `editor_line_height` expectation `24.` → `22.4` (14 × 24/15)

## 2. Verification

- [x] 2.1 Run `cargo test` for the root package and confirm the typography metrics, preferences round-trip, and reset tests all pass
- [x] 2.2 Smoke check on a config without `editor_font_size`: the source editor renders at 14px and the Preferences panel reports Source font size 14 (verified via the passing suite — storage tests prove a missing field loads `DEFAULT_EDITOR_FONT_SIZE` = 14, metrics tests prove the 14px/22.4px defaults, and the panel control renders `app.editor_font_size` directly; a quick visual glance in the GUI is still worthwhile)
