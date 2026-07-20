## Context

Markion currently embeds typography values directly in GPUI view construction: the source editor uses 15px text with a nominal 24px line height, while Visual Edit and preview/read blocks use hard-coded body, heading, code, math, line-height, and margin values. `AppPreferences` already flows from optional/defaulted TOML fields into `MarkionApp`, and the Preferences panel can update and persist display settings immediately.

Typography is presentation state, not Markdown document state. The implementation must remeasure editor and virtualized block geometry when a value changes, but it must not mutate `MarkdownDocument`, bump its version, rebuild per-version preview/outline/stat caches, evict syntax highlighting, or replace the cached `SharedString` text handle.

The intended data flow is:

```text
config.toml / Preferences controls
              ↓
normalized AppPreferences values
              ↓
MarkionApp typography state
              ↓
DocumentTypographyMetrics for the current render
       ↙              ↓                 ↘
EditorElement   Visual Edit rows   Preview/Read rows + math
       ↘              ↓                 ↙
presentation-only reflow and scroll-geometry refresh
```

## Goals / Non-Goals

**Goals:**

- Let users independently set the source-editor font size, rendered-document font size, and rendered paragraph spacing from localized Preferences controls.
- Apply changes immediately and consistently across the view modes that display each surface.
- Preserve today's 15px source text, 14px rendered body text, and 12px rendered paragraph gap as defaults.
- Persist safe values in `config.toml`, tolerate older or malformed fields, and include the settings in reset and preference summaries.
- Keep caret placement, selection, wrapping, list virtualization, scrollbars, inline math, and focus/typewriter behavior aligned with the painted geometry.

**Non-Goals:**

- Font-family selection, line-height as a separate preference, per-document/per-tab values, or theme-specific typography.
- Adding artificial paragraph gaps to the source editor; source spacing continues to follow authored Markdown lines.
- Changing Markdown parsing, document contents, cached derived state, or exported document typography.
- Scaling images or authored diagram contents.

## Decisions

### 1. Persist three bounded logical-pixel values

Add `editor_font_size`, `rendered_font_size`, and `paragraph_spacing` to `AppPreferences` and the serde-facing TOML shape. Values are integer logical pixels so the file remains understandable and equality/testing stay simple.

- `editor_font_size`: default 15, accepted range 10–32.
- `rendered_font_size`: default 14, accepted range 10–32.
- `paragraph_spacing`: default 12, accepted range 0–32.

Missing or non-numeric values use the default; numeric values outside the range are clamped. Central normalization helpers are used by TOML loading, app setters, reset, and tests so UI and hand-edited files cannot diverge.

Alternative considered: store an overall zoom percentage. Zoom is less direct than the requested font size, couples unrelated chrome or image scaling, and cannot independently preserve source and reading preferences.

### 2. Use independent source and rendered font controls

The Preferences panel adds compact minus/value/plus controls for Source font size, Reading font size, and Paragraph spacing. Each click changes one logical pixel, disables decrement/increment at its bound, persists through the existing save path, and requests a repaint. Labels, units, tooltips/status feedback, and accessible button text go through `src/i18n.rs`.

The source and rendered defaults differ today, and users commonly need dense source editing but larger reading text. Independent controls preserve existing defaults and avoid hiding an unexplained offset inside a single nominal font-size value.

Alternative considered: one font-size control for every surface. It is simpler, but either changes an existing default or makes the displayed value disagree with one of the two actual body sizes.

### 3. Derive all dependent metrics from one per-render metrics object

Introduce a small root-crate-only `DocumentTypographyMetrics` value derived from normalized preferences. Source editor text and nominal line height use the source size with the current 24/15 line-height ratio. Rendered body text uses the reading size; headings, lists, block quotes, tables, code, inline/display math, and their line heights derive from the same base by preserving the current default ratios or deltas. Paragraph block bottom margin uses `paragraph_spacing` directly in Visual Edit and Preview/Read.

Pass the metrics into the existing view/element builders instead of reading globals or altering `PreviewBlock`/`VisualBlock` domain types. The same resolved font sizes must be used by GPUI shaping, painting, hit testing, selection geometry, and math cache keys.

Alternative considered: replace only the three most visible hard-coded `.text_size`/`.mb_3` calls. That would leave lists, block quotes, code, math, caret geometry, and scroll calculations inconsistent at non-default sizes.

### 4. Invalidate presentation measurements without invalidating Markdown caches

When a typography setter changes a value, clear or refresh only cached line heights and virtual-list item measurements needed for layout, preserving each tab's document, version, derived `Arc` state, syntax-highlight cache, and cached shared text. Preserve the user's approximate scroll position by recording/restoring the affected pane's scroll fraction when a list measurement reset is necessary. The next render reshapes text using the new metrics and recomputes content extents.

Inline/display math already keys rendered results by logical font size, zoom/display scale, and foreground. Feeding it the resolved typography size naturally produces the correct cache identity without flushing unrelated entries.

Alternative considered: bump every document version or rebuild preview blocks. Typography does not affect parsing and doing so would violate the typing-path cache invariant.

### 5. Keep preferences global and exports unchanged

Typography values live in global application preferences and apply to all tabs on the next render. They affect only the interactive GPUI workspace. Exporters retain their own document-format styles so a personal reading preference does not silently alter shared output artifacts.

Alternative considered: per-tab values. That complicates tab state, save semantics, preference reset, and user expectations without being requested.

## Risks / Trade-offs

- [Risk] Large fonts can expose stale editor/list measurements, incorrect scrollbar ranges, or caret/selection drift. → Mitigation: centralize metrics, invalidate presentation measurements explicitly, and add non-default-size geometry tests plus manual checks in every view mode.
- [Risk] Resetting a virtual list for remeasurement can jump the viewport. → Mitigation: preserve and restore scroll fraction, and test a long document while changing each value.
- [Risk] Scaling every rendered text class can make heading, code, or math proportions uneven. → Mitigation: preserve current default ratios/deltas in one metrics object and verify minimum/maximum values visually.
- [Risk] Frequent preference clicks can rewrite the preferences file repeatedly. → Mitigation: persist only when the normalized value actually changes; the existing writes are small and user-driven.
- [Risk] A malformed hand-edited TOML value can prevent startup. → Mitigation: use tolerant numeric deserializers and normalize after parsing, with focused missing/invalid/out-of-range tests.

## Migration Plan

Add optional/defaulted TOML fields and load older files without migration. On the first save after upgrade, the normalized values are written in the standard TOML output. Apply the model/storage plumbing first, then metrics and render paths, then Preferences controls and localization. Rollback removes the fields and restores constants; older binaries ignore the extra TOML keys.

## Open Questions

None. The proposed defaults and bounds preserve current rendering while providing useful adjustment ranges; they can be revised during implementation if GPUI platform testing reveals a hard layout limit.
