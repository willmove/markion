# Design: fix-visual-data-uri-source-toggle-freeze

## Context

Expanding the image `</>` source toggle feeds the image's complete authored span — including megabytes of base64 — through the verbatim payload-editor pipeline. The measured cost chain (see proposal.md for motivation):

```
click </>  (preview.rs:5192, no caret move)
   └─ show_payload = true  (visual_collapsible_source_block, preview.rs:5119)
        └─ VisualEditableText::prepaint → StyledText layout
             └─ gpui layout_line: whole span to DirectWrite CreateTextLayout + per-glyph Draw
                (Windows direct_write.rs:546-665) → seconds, millions of glyphs resident
        └─ VisualEditableText::paint, navigation_active = caret_active || !block_owns_caret
             (preview.rs:4505)  — true because the toggle does not move the caret
             └─ visual_navigation_snapshot (preview.rs:1052-1137)
                  ├─ position_for_index per grapheme: N calls
                  │    └─ WrappedLineLayout::position_for_index is O(W) per call
                  │       (gpui line_layout.rs:362-389, linear wrap-boundary scan)
                  │    → O(N × W) ≈ 10^10–10^11 iterations, EVERY frame
                  └─ display_boundaries Vec of all N grapheme indices (N × 8 bytes),
                     then per-line filter rescans it → another O(N × W)
```

Additionally, while collapsed, every frame already does span-length work: `visual_image_source_editor` (preview.rs:5062-5065) probes `preview_image_entry` → `PreviewImageKey::from_url` → `format!("data:{}", url)` (preview_image.rs:50) — a multi-MB string clone per frame — and eagerly builds the payload editor, whose projection clones the span again (`visual_editor_field_projection`, preview.rs:4583).

N = base64 characters (≈ 1.37 × image bytes); W = wrapped lines (N / ~chars-per-line). A 1 MB PNG → N ≈ 1.37M, W ≈ 10–20k → ~10¹⁰ boundary iterations per frame.

Constraints:

- Derived state is cached per document version and shared via `Arc`; nothing span-length-sized may run per frame.
- gpui 0.2.2 is an external crate — we cannot fix `position_for_index` upstream; we must stop calling it per character.
- The projection layer (`VisualProjection` in model.rs:1302, mapping helpers in visual.rs:110-247) already supports display ≠ source lengths; edits are canonical source replacements via `direct_visual_block_edit` (editor_element.rs:178-191).

Data flow / versioning sketch (where the new state lives):

```
document text ──(edit)──▶ version bump ──▶ derivation (off typing path, Arc-shared)
                                               ├─ VisualBlockEditor::Image { payload, elision }
                                               │     elision = { visible head range, token source range }
                                               │     computed once per version by scanning the span
                                               └─ image destination fingerprint (u64 hash)
decode completion (preview_image_cache.complete, background→UI)
            └─▶ failed_fingerprints: HashSet<u64>   (app state, not versioned)

render frame (collapsed): reads elision? nothing built. forced = failed_fingerprints.contains(fingerprint)
render frame (expanded):  projection = visible head verbatim + token display + tail verbatim
                          snapshot  = per wrapped line (O(W)), caret x resolved lazily on navigation
```

## Goals / Non-Goals

**Goals:**

- Expanding any image source — data URI or oversized — stays interactive; per-frame work is bounded by display text.
- The elided token is unmistakably not authored text (ellipsis marks + size label + styling) and edits atomically through the existing canonical-replacement path.
- All source-revealing payload editors (code/math/diagram/HTML/image) stop paying O(N × W) for navigation snapshots.
- Collapsed image blocks do zero span-length work per frame.

**Non-Goals:**

- No elision inside code/math/diagram/HTML payloads (only the snapshot/cost fix applies to them).
- No changes to Read/Split rendering, inline-in-prose image atoms, or the source editing modes.
- No background threading of text layout, no gpui fork/patch, no virtualization of payload editors.

## Decisions

### D1. Elide at derivation time, not render time

`visual_block_editor` construction (visual.rs, the `PreviewBlock::Image` arm at :1286) additionally computes an `ElisionPolicy` when the proven image span qualifies:

- **Data-URI destination (any size):** keep `data:` + media type + `;base64,` (everything through the first `,` after `data:`) verbatim; the token covers the rest of the destination data up to the closing `)` or title delimiter.
- **Non-data-URI span ≥ 64 KiB:** keep a 48-character verbatim head of the destination interior; the token covers the remainder.

Rationale: the span is scanned once per document version (derivation already touches the full text), keeping the render path pure and O(display). A render-time check would re-scan megabytes per frame.

`VisualBlockEditor::Image { payload, elision: Option<ImageElision> }` where `ImageElision { visible_head: Range<usize>, token: Range<usize> }` (source coordinates, absolute offsets). Default (`None`) preserves today's verbatim behavior.

Alternative rejected: eliding only above a size threshold even for data URIs — a 400-byte base64 icon is still noise to read, uniform behavior is simpler to reason about and to test.

### D2. Token display: `…` + localized size label + `…`, styled

Display text of the token: `…` + size label + `…`, e.g. `…4.2 MB…` (non-data-URI: `…120 KB…`; the label alone does not need to say "base64" — the verbatim `;base64,` prefix right before it does). Size label is localized through `src/i18n.rs` (new message with a `{size}` placeholder; bytes formatted human-readable, 1 decimal, binary units).

Styling: a `VisualProjectionSpan` over the token's display range with a dedicated `InlineStyle`/highlight (soft background tint + dimmed text color, matching the source-island chrome palette, e.g. `0xf1f5f9` bg / `0x64748b` fg). The leading/trailing `…` plus the tint make the token distinguishable from authored bytes at any zoom/theme; this was an explicit user requirement.

The token's display text is computed once per version alongside the elision policy (stored as the display string + its byte length), so the frame only copies O(display) text.

### D3. Atomic segment semantics in the projection

`VisualProjectionSegment` (model.rs:1262) gains an `atomic: bool` flag (default false; all existing constructors unchanged). For atomic segments the mapping helpers in visual.rs change behavior:

- `source_for_display(display)` / `display_for_source(source)`: positions strictly inside the token snap to the nearest boundary (start or end), never to an interior offset. The existing upstream/downstream `VisualBoundaryCandidates` pair expresses the two boundaries so caret affinity machinery (preview.rs:804-813) works unchanged.
- Clicks: `preview_index_for_position` yields a display index possibly inside the token; `boundary_candidates` returns the token's start/end pair and the click resolves by x proximity to either boundary (same pattern as today's ambiguity resolution, preview.rs:949-964).
- `word_selection_range`: double-click inside the token selects the whole token display range (which maps to the whole elided source range).
- Edits: `replace_text_in_range` → the canonical source selection covers either token boundary exactly or the whole token range; a replacement intersecting the token replaces the entire `token` source range in one canonical replacement (`sanitize_visual_field_replacement` stays identity for `ImageSource` — the projection already produced source coordinates). Adjacent Backspace/Delete therefore deletes the whole token, and single Undo restores it (existing history path).

Alternative rejected: making interior offsets reachable with linear mapping (typing "inside" base64) — mechanically possible but produces a 5 MB diff for one keystroke and makes caret rendering meaningless; atomic matches how users think about an opaque blob.

### D4. Navigation snapshots become line-driven

Rebuild `visual_navigation_snapshot` (preview.rs:1052) around wrapped lines instead of graphemes:

- Enumerate lines by y: `wrapped_line_ix = floor((y - bounds.top) / line_height)`; per line, resolve start/end display indices via `index_for_position` at the line's left edge (gpui `_index_for_position` is O(1): direct wrap-boundary indexing, line_layout.rs:310). Total O(W) per paint, no per-character layout queries.
- Per line, store `{ y, start_display, end_display }`; drop the all-grapheme `line_ys`/`display_boundaries` Vecs entirely.
- Caret x positions are resolved lazily: `current_visual_navigation_snapshot` consumers (editing.rs:2822-2853 Up/Down movement, :3046, :3099) compute `position_for_index` for the specific offsets involved at keypress time — a handful of O(W) calls per keystroke (≈ tens of thousands of boundary iterations worst case ≈ well under a frame).
- Grapheme-boundary carets within a line are still needed for click→caret granularity; clicks already resolve through `preview_index_for_position` directly on the layout (preview.rs:945), not through snapshot carets, so the snapshot no longer needs per-grapheme entries. If Up/Down target resolution needs candidate offsets on a line, compute them from that line's `[start_display, end_display]` window on demand (grapheme iteration is bounded by one line's characters).

`VisualNavigationLine` changes shape accordingly (`carets` replaced by the line window + lazy resolution helper). `register_visual_navigation_snapshot` retention/invalidation logic (state.rs:1242-1262) is unchanged.

Alternative rejected: caching snapshots per (version, block) — correct but adds invalidation surface; the per-paint O(W) build is already cheap (W ≈ tens of thousands of arithmetic steps).

### D5. Lazy payload construction

`visual_image_source_editor` (preview.rs:5051) stops eagerly building `payload_editor`: `visual_collapsible_source_block` takes the payload editor as an `Option`/closure and only constructs it when `show_payload` is true. Same treatment for the other collapsible-source call sites (math/diagram/HTML) where they are currently eager. This removes the per-frame projection clone (`authored.to_string()`, preview.rs:4583) and the hidden-field element tree while collapsed.

Alternative rejected: keeping eager construction but skipping clones via `Arc<str>` — larger refactor of `VisualProjection` for less benefit; laziness also skips the `StyledText` construction.

### D6. Forced-expand via destination fingerprint

Replace the per-frame `preview_image_entry(url, dir)` probe in `visual_image_source_editor` with:

- Derivation computes a 64-bit fingerprint (e.g. fxhash/xxhash of the destination URL) and stores it on the image block / editor payload (per version, off typing path).
- `preview_image_cache.complete(key, result)` (preview_image.rs:748) already lands on the UI thread; on an `Error` result it additionally inserts the key's URL fingerprint into `app.failed_image_fingerprints: HashSet<u64>` (bounded, cleared alongside other caches on tab eviction, mirroring `preview_probe_results` hygiene at preview_image.rs:717-720).
- Render: `forced = failed_image_fingerprints.contains(&fingerprint)` — O(1), no string work.

`PreviewImageKey` itself is unchanged (decode still needs the full URL; `ensure_preview_images` already builds keys off the typing path at preview_image.rs:676-690).

### D7. Testing strategy (deterministic, no wall-clock)

- Extend the `visual_image_source_toggle_expands_collapses_and_edits_exactly` test pattern (tests.rs:12659) with data-URI cases: expansion shows prefix + token, token selection replacement applies one exact source replacement, undo restores, caret cannot enter the token interior, forced-visible error path still elides.
- `#[cfg(test)]` counters (same pattern as `visual_caret_paint_count`, preview.rs:831-833): a `visual_navigation_position_queries` counter incremented wherever the snapshot code queries layout positions; tests assert it stays ≤ O(W) (bounded by lines × small constant) and that elided display text length ≤ threshold-derived bound. Collapsed-frame tests assert a `visual_payload_projection_builds` counter stays zero while collapsed.
- Snapshot-equivalence test: for representative documents, caret Up/Down via the new line-driven snapshots lands on the same source offsets as the previous per-grapheme snapshots produced (golden offsets computed by linear mapping in the test).

## Risks / Trade-offs

- [Atomic mapping regressions in existing projections] → the `atomic` flag defaults to false and no existing constructor sets it; mapping-helper behavior for non-atomic segments is byte-identical, guarded by the existing projection test suite plus new differential tests.
- [Up/Down navigation fidelity after the snapshot redesign] → D7's snapshot-equivalence test compares new navigation results against per-grapheme golden offsets before removing the old path.
- [Backspace deleting megabytes feels dangerous] → it is one canonical replacement with single-undo restore; the token's tinted chip affordance signals "one unit". If device testing disagrees, restricting deletion to explicit selection is a contained follow-up.
- [Elision policy computes on a huge span during derivation] → derivation already scans the full text per version; the extra pass is one memchr-style comma/`)` scan over the destination, O(span) once per edit — same class as existing parsing.
- [Fingerprint hash collisions mark an unrelated image as failed] → 64-bit hash over destinations; collision odds are negligible for realistic document counts, and the failure mode is a harmless forced-visible editor. Note in code comment.
- [Existing test expectations with data-URI fixtures] → tests asserting verbatim payload text for data-URI images update to the token form (they are exercising the behavior this change intentionally modifies).

## Migration Plan

Pure presentation/interaction change; no persisted state, no document format impact. Rollback is reverting the commit — collapsed rendering and verbatim (sub-threshold) behavior are unchanged for non-data-URI images. Ships in the next patch release.

## Open Questions

- Exact token label wording/units (e.g. `…4.2 MB…` vs `…base64 4.2 MB…`) — final copy decided during implementation alongside the i18n message; the spec only fixes "size label framed by ellipsis marks, visually distinct".
- Whether the 48-char verbatim head for oversized non-data-URI destinations should adapt to the field width — safe to tune later; constant is fine for v1.
