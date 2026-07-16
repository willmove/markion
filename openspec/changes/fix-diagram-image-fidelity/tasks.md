# Tasks

## 1. Dependencies

- [x] 1.1 Add `resvg`, `usvg`, and `image` to the root crate at the versions already resolved in `Cargo.lock` via GPUI, and confirm `cargo tree` reports one version of each shared with `gpui`.

## 2. Rasterization

- [x] 2.1 Add a diagram rasterizer in `src/app/diagram.rs` that parses sanitized SVG with `usvg` options whose `fontdb` is populated by `load_system_fonts()` behind a `LazyLock`, mirroring `gpui::SvgRenderer::new` including its fallback selector.
- [x] 2.2 Rasterize at a 2x supersample with `resvg`, convert the premultiplied RGBA pixmap to BGRA, and build an `Arc<RenderImage>` via `RenderImage::new`.
- [x] 2.3 Return the presentation size (sanitized intrinsic size, falling back to the rasterized size divided by the supersample factor when the SVG declares none) alongside the image.
- [x] 2.4 Map rasterization failure to `DiagramErrorKind::RenderFailed` so preview keeps its localized error plus authored-source fallback.
- [x] 2.5 Unit-test that an SVG with a known red-dominant fill produces BGRA bytes in the expected channel order, that a text-bearing SVG (including a CJK label) rasterizes to non-blank pixels, and that a 2x raster reports a 1x presentation size.

## 3. Cache

- [x] 3.1 Change `DiagramCacheEntry::Ready` to carry the rasterized `Arc<RenderImage>` plus its presentation size, replacing `Arc<Image>`.
- [x] 3.2 Rasterize inside the existing `cx.background_spawn` in `ensure_diagram_renders` so no rasterization reaches the frame path.
- [x] 3.3 Confirm the existing cache tests still cover deduplication, FIFO eviction that never evicts pending work, independent light/dark keys, and key-scoped stale completion.

## 4. Preview

- [x] 4.1 Update the `DiagramCacheEntry::Ready` branch in `src/app/preview.rs` to build `img()` from `ImageSource::Render` with a definite intrinsic width and auto height, so GPUI derives an aspect-correct height and the supersampled raster is not presented at double size.
- [x] 4.2 Keep a wider-than-column diagram fully visible and undistorted via `max_w_full()` and the default `ObjectFit::Contain`. Note in `design.md` that such a diagram still reserves its intrinsic height: GPUI resolves both image dimensions to definite lengths before layout, and taffy's aspect-ratio rule (`height = max(height, width / ratio)`) can only grow that height, never shrink it. Eliminating the residual vertical padding needs container-width measurement, which is out of scope here.
- [x] 4.3 Confirm pending, error, and non-diagram code-block branches are unchanged, and that Visual Edit still treats the fence as a source island.

## 5. Verification

- [x] 5.1 Run `cargo test -p markion-diagram`, the targeted root diagram/cache tests, `cargo test`, and `cargo test --workspace`.
- [x] 5.2 Run `cargo fmt --all -- --check` (clean) and `cargo clippy --workspace --all-targets -- -D warnings`. Clippy reports no finding in this change's files. It does still fail overall on six pre-existing `collapsible_if` errors in `crates/markdown/` (`extended_inline.rs`, `highlight.rs`, `incremental.rs`) that this change does not touch and that also fail on a clean tree under Rust 1.96; fixing them belongs to its own change.
- [x] 5.3 Confirm the rendered result out of band: a CJK light-theme flowchart driven through `builtin_diagram_registry` + `rasterize_diagram`, with the resulting buffer decoded as BGRA the way GPUI does, shows lavender nodes with navy edges (not the pink/brown of the old path), intact CJK labels, and a 1191x364 raster reported for presentation at 595x182.
- [ ] 5.4 Still to do by a human on a machine with a display: launch the app and confirm the same diagram in Split Preview and Read mode, and check how a wider-than-column diagram looks with the known intrinsic-height padding (task 4.2). This environment has no display, so the GUI itself was never exercised.
