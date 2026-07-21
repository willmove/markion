## Why

In Visual Edit mode, reference-style links (`[text][label]`, `[label][]`, `[label]`) render as literal raw text instead of links, because `inline_runs` (`src/visual.rs`) re-parses each block's source slice in isolation while link reference definitions are document-scoped and live in other blocks. Split Preview and Read modes parse the whole document and render the same links correctly, so Visual Edit breaks the WYSIWYG-first contract for a CommonMark construct the editor otherwise fully supports.

## What Changes

- `build_visual_blocks` collects the document's link reference definition lines once per build (line-level scan that skips fenced code blocks) and passes them down to `inline_runs`.
- `inline_runs` parses `block slice + "\n" + collected definitions` so pulldown-cmark resolves reference-style links (full, collapsed, and shortcut forms) inside per-block parsing. Definitions are appended after the block text, so event offsets inside the block are unchanged and reference definitions produce no events — source-range mapping is unaffected.
- Reference-style links gain the same Visual Edit treatment as inline links: hidden-bracket rendered label when unfocused, reveal group exposing the full `[text][label]` source when focused. `link_target_range` stays `None` when the destination lives outside the block (the URL remains editable in the definition line's own block).
- Two adjacent adjustments discovered during implementation: `reveal_candidate_is_exact` previously accepted only the inline `[text](dest)` form (a resolved reference link would otherwise have flipped the whole block to conservative fallback), and the visual projection marked link spans solely by a local target range, so reference-link labels now count as link spans via their reveal-group membership. Collapsed-form candidates are extended to cover the trailing `[]`, which pulldown-cmark excludes from the tag range.
- Regression tests: full/collapsed/shortcut reference links resolve in Visual Edit; definitions inside fenced code blocks do not create spurious links; appended definitions never shift in-block source ranges.

Non-goals: changing how reference-definition lines themselves render (they remain editable source blocks); footnote-definition block splitting; any change to Split Preview/Read/export behavior.

## Capabilities

### New Capabilities

<!-- None — this sharpens an existing capability. -->

### Modified Capabilities

- `markdown-editing`: The "Visual Edit inline formatting fidelity" requirement is sharpened to state that supported links include reference-style links whose definitions appear elsewhere in the document, and that Visual Edit SHALL resolve them against document-scoped reference definitions while preserving exact in-block source ranges.

## Impact

- **Code:** `src/visual.rs` only — `build_visual_blocks` (definition collection, threaded through `visual_block_from_preview`), `inline_runs` (parse slice + suffix). No new dependencies, no i18n surface, no UI chrome changes.
- **Invariants touched:** none of the cached-per-version invariants are weakened — definition collection is a single line scan inside the already-cached `build_visual_blocks` derivation, and per-block parses stay proportional to block size (the appended suffix is bounded by the document's definition count). The per-block-parse + suffix output remains byte-range equivalent to a full-document parse for the block's events, matching the existing incremental-equivalence requirement.
