## Context

The in-window menu bar renders six title buttons in a fixed-parameter flex row (root `div().px_2().gap_1()`; each button `px_2()` with `text_size(px(13.))` over `.SystemUIFont`), so a button's left edge is exactly `8 + Σ(previous label widths + 20)` logical pixels. Dropdown panels and flyout submenus are absolutely positioned against the root and currently read that address from a hand-tuned per-language table in `AppMenu::dropdown_left` (`src/app/mod.rs`). The table was recalibrated twice in 2026-09 by screenshot pixel forensics after drift was reported (English/Japanese/Chinese misalignment up to ~46px), because fixed constants silently assume one system UI font — on Windows gpui resolves `.SystemUIFont` via `SPI_GETICONTITLELOGFONT`, which is Microsoft YaHei UI on a zh-CN machine and Segoe UI elsewhere. See proposal.md — Why.

gpui 0.2.2 (vendored) has no popover-anchor API: `div` exposes no anchor position, `.occlude()` only blocks mouse hits, and nesting dropdowns inside the menu-bar band would place them earlier in paint order than the workspace content that follows. However, `WindowTextSystem::layout_line(text, font_size, runs, None)` (reached via `window.text_system()`) returns the exact line layout the same engine renders with (its `Arc<LineLayout>` carries `.width`), `Font { family: ".SystemUIFont".into(), ..Default::default() }` resolves through the same per-platform system-font path, and the app already calls the text system for font metrics in `src/app/preview.rs` and `src/app/appearance.rs`.

## Goals / Non-Goals

**Goals:**

- Dropdown horizontal alignment that is correct by construction for all seven languages, any system UI font, and any machine.
- Delete the per-language constants table and its tuning burden.
- A pure, unit-testable arithmetic core (widths → offsets) in the repository's existing pure-fn test style.

**Non-Goals:**

- Restructuring the element tree (deferred/popover layers, anchoring flyouts to their parent rows' live bounds).
- Changing dropdown widths (`dropdown_width`) or the flyout submenus' `SUBMENU_TOP` row arithmetic — rows have fixed 24px heights and 9px separators, independent of fonts.
- The native macOS menu (positioned by the OS).

## Decisions

- **Measure with gpui's own text system, not an external metric source.** PIL/GDI font metrics disagreed with DirectWrite by ~1.34px per label, which is exactly the drift class being eliminated; `layout_line` is the renderer's own measurement, so button boxes and dropdown anchors share one source of truth. `Font` carries only the `.SystemUIFont` family with default weight/style, matching the menu-bar text style.
- **Compute offsets from the verified layout formula, cache per language, re-measure lazily.** `MarkionApp` stores `menu_dropdown_offsets: [f32; 6]` plus the language it was measured for; a small `ensure` method at the top of the root render re-measures only when the language differs from the cached one. This covers both startup preferences load and runtime language switches without touching any language-mutation site, and steady-state frames do zero text layout. Alternative — measuring inside `dropdown_left` per call — rejected: it would re-layout six lines on every render of any open menu.
- **`AppMenu::dropdown_left` becomes a pure index lookup taking the cached table**, so all existing call sites (the dropdown panel builder and the four flyout anchors in `root_view.rs`) keep a one-line change, and the flyouts inherit the corrected horizontal anchor for free since they position relative to `dropdown_left + dropdown_width`.
- **Pure arithmetic helper kept free of gpui types** (`fn menu_dropdown_offsets_from_label_widths(widths: &[f32; 6]) -> [f32; 6]` implementing `8 + Σ(w + 20)`) so it can be asserted in a plain `#[test]` without a GPU or window, following the repo's pure-fn test convention. `layout_line` resolves fonts through its fallback chain and always returns a positive width, so no degenerate-width fallback is needed; the initial construction in `MarkionApp::new` measures the default language immediately so the first frame is already anchored.

## Risks / Trade-offs

- [A platform resolves `.SystemUIFont` unexpectedly (headless test context, exotic font)] → `layout_line` still returns its fallback-font layout, so offsets degrade gracefully to fallback-font metrics rather than breaking; no code path can produce an unpositioned dropdown.
- [Language changes between two values repeatedly] → re-measurement happens per switch, six line layouts each — negligible, and cached layouts live in gpui's own line-layout cache anyway.
- [Font or app zoom changes at runtime without a language change] → out of scope; label widths are measured per language, matching today's behavior where zoom applies uniformly. If a future zoom feature lands, the cache key gains the zoom factor.

## Migration Plan

No persisted data or preferences involved; the constants table is replaced in place. Rollback is reverting the implementing commit. The hand-tuned table's provenance notes live in git history (commits 5e330d6, e9c1cc5) if a fallback reference is ever needed.
