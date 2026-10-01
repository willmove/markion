## Context

The affected and unaffected screenshots use the same reported 2560×1600 resolution, 125% Windows scaling, and Markion 0.1.12, but their application-owned text has materially different metrics. The unaffected UI uses a proportional sans-serif face; the affected UI uses a wider, heavier monospaced face across the menu bar, sidebar, Preferences title, language labels, and theme names.

That difference follows the current font path. `src/app/root_view.rs` sets `.SystemUIFont` on the root element. GPUI 0.2.2 resolves that special family on Windows by calling `SystemParametersInfoW(SPI_GETICONTITLELOGFONT)` and using the returned face (falling back to Segoe UI only if the call fails). A customized icon-title font, font substitution, or comparable system configuration can therefore give Markion a very different UI font without changing display resolution, scale, or app version. The screenshot alone cannot identify which system setting changed the returned face, but it does establish that the resulting metrics are the necessary environmental difference; GPUI also logs the selected family as `Use <family> as UI font.` for confirmation on the affected machine.

The font difference exposes three coupled assumptions in `preferences_panel_view`:

1. The General panel is fixed at 560 logical pixels and its padded body has about 528 logical pixels for the language row.
2. The row is a non-wrapping flex container. `preference_option_button` has only a 48-pixel minimum and does not opt out of flex shrinking.
3. Each pill receives one centered string produced by `format!("{}  {}", marker, native_name)`. The selected label contains `✓` plus two spaces; an inactive label contains a placeholder space plus those two spaces. When the flex items shrink below the string's intrinsic width, the centered glyph run paints past both sides of the pill.

The code point is U+2713 CHECK MARK (`✓`), not U+221A SQUARE ROOT (`√`). The visual report calls it a root symbol, but changing the mathematical glyph is not part of the fix.

This is a render-only path. It reads the existing `Language::all()` metadata and dispatches the existing selection action; it does not touch document state, derived Markdown caches, syntax highlighting, editor text handles, or persistence.

## Goals / Non-Goals

**Goals:**

- Keep every check mark and complete native-language name inside its own pill under proportional, wide, and monospaced UI-font metrics.
- Use compact, deterministic spacing between the marker and label without encoding layout as whitespace characters.
- Give the seven choices more room when the window permits, while wrapping complete pills at constrained widths instead of compressing their text.
- Preserve the current language ordering, selection behavior, localization metadata, and immediate persistence.

**Non-Goals:**

- Do not force Segoe UI or otherwise replace `.SystemUIFont` application-wide.
- Do not diagnose or rewrite the affected user's Windows font configuration from inside Markion.
- Do not change translations, supported languages, language codes, preference storage, theme cards, or other segmented controls.
- Do not add a dependency or a general responsive-layout framework.

## Decisions

### Decision 1: Make the language picker metric-tolerant instead of normalizing the global font

The fix will respect the inherited `.SystemUIFont` and remove the picker's dependency on Segoe-like widths. A hard-coded Windows face would make this one row look predictable, but it would override user/platform conventions, require a separate cross-platform fallback policy, and still leave CJK fallback metrics variable. Layout containment is the direct fix for the reported failure.

The affected machine's resolved family should be confirmed from Markion's existing GPUI log when available, but implementation does not depend on obtaining that machine-specific name.

### Decision 2: Introduce a language-specific pill renderer

The language row will call a focused helper that accepts `native_name`, `active`, palette, and listener rather than passing a preformatted string to the generic `preference_option_button`.

Each language pill will:

- be `flex_none` (or equivalently have zero shrink) so the parent cannot make it narrower than its content;
- use a 72-logical-pixel minimum width and the existing 12-pixel label size;
- render a small fixed leading marker slot, containing `✓` only when active;
- place the native name in a separate child with a compact 2-pixel visual gap from the marker slot;
- center the combined marker-plus-label group and retain the existing active colors, border, hover, and click listener.

An empty inactive marker slot, rather than leading text spaces, keeps every language name aligned without contributing font-dependent whitespace advances. Keeping this helper separate avoids widening or changing the many non-language controls that reuse `preference_option_button`.

Alternatives considered:

- Reducing the current two spaces to one would improve the screenshot but would leave a shrinkable, indivisible text run and would regress again under wider metrics.
- Removing the inactive marker placeholder would save width but make labels jump horizontally when selection changes.
- Measuring every label manually through GPUI's text system would add render-time complexity that the flex engine's intrinsic content sizing already provides once shrinking is disabled.

### Decision 3: Combine a wider responsive panel with whole-pill wrapping

The General Preferences target width will increase from 560 to 640 logical pixels. The modal will use the available overlay width up to that cap, with horizontal inset for windows narrower than the target; the 720-pixel Shortcuts width remains unchanged. The language row will enable flex wrapping while preserving the existing inter-pill gap.

At ordinary Segoe UI metrics, the increased width and 72-pixel pill minima keep the seven options on one comfortable row. If a wider font or constrained window makes the aggregate intrinsic width exceed the row, one or more complete pills move to the next line. The scrollable Preferences body and existing 560-pixel height cap already bound the extra vertical space.

Widening alone is not sufficient because no finite desktop width guarantees containment for arbitrary accessibility or substituted font metrics. Wrapping alone would prevent overflow, but the modest width increase and longer minimums address the reported density and make wrapping uncommon on the stated 2560×1600 setup.

### Decision 4: Verify structure automatically and font behavior manually

Focused tests will guard that the language row wraps, uses the dedicated helper, does not reconstruct `✓` plus whitespace in a formatted label, and gives language pills non-shrinking/content-preserving structure. Existing language selection and persistence tests remain authoritative for behavior.

The manual verification matrix will include:

- Windows at 2560×1600 and 125% with the normal proportional UI font;
- Windows at the same resolution/scale with a wide or monospaced UI font (or an equivalent local override/test fixture);
- a constrained window that forces wrapping;
- each of the seven languages selected in turn, confirming exactly one marker, compact spacing, complete borders and labels, and working clicks.

Pixel screenshot tests are not introduced because the repository has no cross-platform golden-image harness and font rasterization differs by platform. The structural test plus targeted manual matrix isolates the regression without creating brittle glyph snapshots.

## Risks / Trade-offs

- **[Risk] A very wide font causes a second language row and pushes later settings downward.** → The body already scrolls vertically and has a maximum height; wrapping is preferable to clipping or overlap.
- **[Risk] A fixed 72-pixel minimum is still narrower than an extreme label.** → It is a minimum, not a fixed width; zero shrink lets intrinsic content width expand, and the row wraps the complete pill.
- **[Risk] The affected UI remains monospaced even after overflow is fixed.** → This change deliberately respects the system-selected font. A consistent bundled/application-wide UI font would be a separate product decision with cross-platform and CJK-fallback implications.
- **[Risk] Responsive panel sizing interacts with very small windows.** → Cap the panel at its target while allowing it to use the overlay's available width with horizontal inset; verify the forced-wrap case manually.
- **[Trade-off] Inactive pills reserve a small empty leading slot.** → The few pixels are intentional to prevent label movement and maintain consistent alignment when selection changes.

## Migration Plan

No data migration is required. Ship the render-only change with the existing preferences format. Rollback is a source revert; saved language and theme values remain compatible in either direction.

## Open Questions

None. The exact visual constants may receive small implementation-time tuning if the stated verification matrix shows clipping, but the containment, zero-shrink, compact marker slot, responsive width, and whole-pill wrapping decisions are fixed.
