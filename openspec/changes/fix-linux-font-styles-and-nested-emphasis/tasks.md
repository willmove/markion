## 1. GPUI Font Fallback & Linux Text System

- [x] 1.1 Update `TextSystem::resolve_font` in `vendor/zed/crates/gpui/src/text_system.rs` to check `font.fallbacks` before `fallback_font_stack` and preserve `weight`, `style`, `features`, and `fallbacks` when resolving fallback families.
- [x] 1.2 Update `CosmicTextSystem` in `vendor/zed/crates/gpui/src/platform/linux/text_system.rs` to track `requested_weight`, `requested_style`, and `fake_italic` per `LoadedFont` with deduplicated `FontId` variants, propagate weight/italic intent across `layout_line` fallback runs, and pass `CacheKeyFlags::FAKE_ITALIC` in `raster_bounds` and `rasterize_glyph`.
- [x] 1.3 Append Linux monospace fallbacks (`Liberation Mono`, `Noto Sans Mono`) after `DejaVu Sans Mono` in `code_slot_font` (`src/app/mod.rs`).

## 2. Visual Edit Nested Emphasis Depth Counters

- [x] 2.1 Replace boolean `bold`/`italic`/`strikethrough` toggles in `src/visual.rs` with `usize` depth counters matching `InlineStateDraft` in `src/lib.rs`.

## 3. Tests & Verification

- [x] 3.1 Add Visual Edit regression tests for nested strong, emphasis, and strikethrough runs in `src/visual.rs`.
- [x] 3.2 Add Linux GPUI integration tests verifying `.SystemUIFont` bold/italic/bold-italic resolution, `code_slot_font` monospace fallback, CJK bold fallback, and `FAKE_ITALIC` non-aliasing rasterization.
- [x] 3.3 Run `cargo test --workspace` and verify all tests pass, then commit the change cleanly.
