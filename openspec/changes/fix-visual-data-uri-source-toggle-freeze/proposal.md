# Proposal: fix-visual-data-uri-source-toggle-freeze

## Why

Expanding the `</>` source toggle on a block-level Markdown image whose destination is a base64 data URI freezes the UI for tens of seconds to minutes (the app looks hung) because the expanded payload editor lays out the multi-megabyte authored span verbatim while an O(N×W) navigation-snapshot loop runs on the UI thread every frame (N = authored characters, W = wrapped lines). Hover alone already janks, because collapsed image blocks still clone the multi-megabyte span and re-format the data URI into a cache key on every frame.

## What Changes

- **Elided payload projection for image source editors.** When the destination is a data URI, or the authored span exceeds an elision threshold (64 KiB), the expanded payload editor shows `![alt](data:image/png;base64,…payload summary…)`: the prefix (label, `data:`, media type, `;base64,`) stays verbatim and editable, while the opaque payload collapses into one atomic summary token carrying a human-readable size label (e.g. `…4.2 MB…`) with distinct styling (background tint + dimmed text) plus leading/trailing ellipsis characters so it can never be mistaken for authored bytes.
- **Atomic token edit semantics.** The caret snaps to the token's boundaries (never inside it); double-click selects the whole token; typing over a selection that covers the token replaces the entire payload span through one exact canonical source replacement; adjacent Backspace/Delete removes the whole token (single Undo restores). Raw bytes remain fully editable in Source/Split mode.
- **Line-driven navigation snapshots.** `visual_navigation_snapshot` is rebuilt from wrapped-line boundaries instead of querying `position_for_index` per grapheme (O(N×W) → O(display text)), benefiting every source-revealing payload editor (image, fenced code, math, diagram, HTML).
- **Lazy payload construction.** Collapsed source-toggle blocks stop building and cloning the payload projection each frame; the payload editor element is constructed only while shown.
- **Display-bounded forced-expand check.** The "destination failed to load" check that forces the payload editor visible stops re-deriving a multi-megabyte `PreviewImageKey` per frame; the decision moves to a derivation-time destination fingerprint plus a completion-time failure set.

Non-goals: no change to how Read mode / Split Preview render data-URI images; no background threading of text layout; no gpui upstream changes; no elision inside code/math/HTML/diagram payloads (they only gain the cost bound); no change to inline-in-prose image atoms; no new search behavior inside elided bytes from Visual Edit.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: The requirement "On-demand Markdown image source editing in Visual Edit" (introduced by the complete-but-not-yet-archived change `visual-source-toggle-images-tables-and-fence-language`) gains elided rendering and atomic token edit semantics for data-URI and oversized spans, and a responsiveness obligation for expanding such sources.
- `engineering-quality`: Adds a requirement that Visual Edit source-revealing payload editors keep per-frame layout, projection, and navigation work bounded by the display text length rather than the authored span length, gated by deterministic counters (consistent with the existing deterministic performance gates).

## Impact

- `src/visual.rs` — block editor construction computes the elision policy (visible head / token source ranges) once per document version; projection mapping helpers (`boundary_candidates`, `source_for_display`, `display_for_source`, `word_selection_range`) gain atomic-segment snapping.
- `src/model.rs` — `VisualProjectionSegment`/`VisualProjectionSpan` carry an atomic marker (or equivalent) so mapping snaps to segment boundaries.
- `src/app/preview.rs` — `visual_editor_field_projection` emits the elided display text; `visual_image_source_editor` builds the payload editor lazily; `visual_navigation_snapshot` becomes line-driven; `VisualEditableText` styles the token span.
- `src/lib.rs` — `sanitize_visual_field_replacement` / `direct_visual_block_edit` mapping for atomic segments (whole-segment replacement).
- `src/app/editing.rs` — navigation snapshot consumers (`current_visual_navigation_snapshot` and Up/Down caret movement) adapt to per-line snapshots with lazily resolved caret x positions.
- `src/app/preview_image.rs`, `src/app/state.rs` — destination fingerprint + failed-destination set replacing the per-frame `preview_image_entry` probe in `visual_image_source_editor`.
- `src/i18n.rs` — localized size label for the token.
- Invariants touched: elision policy and fingerprints are computed at derivation time (per document version, `Arc`-shared), never per frame; memoized highlighting and the cached text handle per version are unchanged; navigation snapshots stay paint-time but linear in display text.
