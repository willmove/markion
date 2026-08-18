# Design: render-visual-edit-escapes-and-inline-html

## Context

Visual Edit derives `VisualBlock`s per document version through `build_visual_blocks` → `visual_block_from_preview` → `inline_runs` (`src/visual.rs:1355`), which folds pulldown-cmark events into `VisualInlineRun`s plus `VisualRevealGroup`s. Two whole-block demotions exist today:

1. **Escapes**: pulldown-cmark resolves `\*` inside `Event::Text`, so the event's visible text differs from its source slice. `push_text_runs` detects the mismatch and forces the run conservative, and a blunt whole-block scan (`contains_markdown_escape`, `src/visual.rs:1601-1606`) additionally marks the first run conservative and clears all reveal groups. Any single escape collapses the paragraph into the gray source island rendered by `visual_source_island_view` (`src/app/preview.rs:2644`).
2. **Non-image inline HTML**: the `Event::Html | Event::InlineHtml` arm recognizes only exact `<img>` tags; everything else sets `contains_non_image_html`, which assigns `VisualSourceIslandKind::Html` to the whole block (`src/visual.rs:1055-1059`).

The projection (`build_visual_projection_with_marked_range`, `src/visual.rs:258`) assembles `ProjectionPiece::Source` (revealed raw source) and `ProjectionPiece::Rendered(run)` pieces; each run's display text maps linearly onto its `content_range`. Hidden markers are not stored explicitly: `marker_ranges()` (`src/visual.rs:2182`) derives them as the gaps between non-conservative content runs. This is exactly how `**bold**` and `==highlight==` hide their delimiters, and it is the mechanism this change reuses.

## Goals / Non-Goals

**Goals:**

- Render escaped ASCII punctuation and a narrow inline-HTML subset as ordinary styled prose with hidden markers, reusing the existing reveal-group, projection, marker, and caret-affinity machinery without new state or caches.
- Keep a byte-exact proof gate in front of every new rendered form; anything unproven keeps today's conservative whole-block fallback.

**Non-Goals (design-level):**

- No new parser crate or document model; recognition stays a narrow helper beside `parse_inline_html_image` in `src/parse.rs`.
- No changes to Split Preview / Read rendering, exporters, or persistence.
- No entity-reference (`&amp;`) rendering, no attributed/unknown/uppercase-exotic tags, no `<a>` links (navigation plumbing stays Markdown-only).

## Decisions

### D1: Escapes are hidden-marker groups, not non-identity runs

`\X` is modeled as: a one-byte content run for `X` (visible text `X`, identity-mapped) + the `\` byte, which `marker_ranges()` automatically classifies as a hidden marker because no content run covers it. A new `VisualRevealKind::Escape` group spanning both bytes provides caret-activated reveal of the authored `\X`.

*Alternative rejected*: a single run with `visible_text: "X"` and `content_range` covering `\X`. That makes the run's display/source segment non-identity (1 char ↔ 2 bytes), breaking per-character caret mapping and IME geometry for no benefit — the reveal architecture already handles hidden bytes.

### D2: Escape claiming is gated by byte-exact proofs at two levels

pulldown-cmark resolves `\X` in one of two shapes, and each is claimed with its own proof:

1. **Merged-event escapes (the common case)**: pulldown merges the escaped character into the following `Event::Text` whose source range *excludes* the backslash, leaving the `\` byte as an uncovered gap between two leaf events. `inline_runs` tracks the previous leaf event's end; when a one-byte `\` gap sits immediately before a Text event that starts with ASCII punctuation, the gap plus that first byte is claimed as an `Escape` reveal group with a one-byte content run, and the remainder of the event continues through normal processing. An uncovered `\` byte can only be an escape's backslash in CommonMark (a literal backslash stays inside a Text event), so the shape itself is the proof.
2. **Differing visible text** (escapes mixed with other transformations, e.g. entities): when a text event's slice differs from the parser's visible text, `push_text_runs` reconstructs the expected visible text by removing backslashes before ASCII punctuation; only on exact equality is the event split (plain segments keep identity/extended handling, each escape becomes a one-byte run plus an `Escape` candidate). Any mismatch — HTML entities, smart-punctuation residue — keeps the conservative fallback. Extended (`==`/`^`/`~`) markers compose with escapes only within one event; a marker pair spanning an escape gap loses its styling, which exactly matches Split Preview's per-event extended parsing for the same source (verified: preview renders `==a \* b==` as unstyled literal text).

The whole-block `contains_markdown_escape` collapse and its reveal-group clearing are deleted. `push_run`'s existing `escaped_source` heuristic stays as a defense-in-depth net for any remaining path (e.g. odd `Event::Code` slices).

### D3: Narrow inline-HTML recognizer in `src/parse.rs`

`parse_inline_html_style_tag(authored) -> Option<InlineHtmlTagTagKind>` (naming per implementation) accepts only exact, unattributed, case-insensitive tag names from the supported subset — opening/closing `em|i`, `strong|b`, `s|del|strike`, `code`, `mark`, `sub`, `sup`, and the void forms `<br>`, `<br/>`, `<br />` (whitespace before `/` only). Each style tag maps to one existing `InlineStyle` flag (italic, bold, strikethrough, code, highlight, subscript, superscript). Any attribute, other tag name, or malformed form returns `None` and falls through to today's conservative branch. Case-insensitivity is safe because the whole tag is matched exactly against a closed list.

### D4: Style stack in `inline_runs` mirrors the link stack

Opening tags push `(tag name, style flag, open-tag range)` onto an HTML style stack and set the flag on the ambient `style`, exactly as `Tag::Strong` does; closing tags pop the matching entry and must nest strictly (a close that crosses another open tag is unsupported). A successful pair registers one reveal candidate (new `VisualRevealKind::InlineHtml`) spanning open tag start → close tag end; the tag bytes themselves are never covered by content runs, so they become hidden markers automatically. pulldown-cmark parses Markdown between inline-HTML tags, so `<em>` styling composes with `*em*`, math, and images without extra work.

**One bad tag spoils the block**: a close without a matching open, an open left unclosed at event-loop end, or any tag the recognizer rejects sets the existing `contains_non_image_html` flag (whole-block Html island, mixed-image exemption rendering tags as conservative runs preserved). Runs already emitted inside a failed element are demoted to `conservative_fallback` by range containment so the mixed image path cannot show a half-guessed styled form.

### D5: `<br>` is an atomic line-break run

`<br>` variants are pushed directly (the math/image pattern, bypassing `push_run`'s identity checks) as `VisualInlineRun { visible_text: "\n", content_range: <tag bytes>, conservative_fallback: false }`. The single display character participates in the existing stacked wrap-row layout used for authored soft/hard breaks, and the segment maps one display char onto the whole tag range, so pointer/keyboard caret resolution is naturally boundary-only — the same atom semantics as math. An `InlineHtml` reveal group over the tag reveals the authored source when the caret sits at the tag boundary; no `include_end` special case is added (consistent with emphasis groups).

### D6: Data flow, caching, and versioning are untouched

All work happens inside `inline_runs` during the existing per-document-version derivation; output flows through unchanged `Arc`-shared caches, stable block identities, and incremental region reuse. Reveal remains interaction-only state routed through `build_visual_projection_with_marked_range`; cursor movement does not bump the document version or invalidate derived caches. The view-layer gate in `visual_block_view` needs no rule changes — blocks whose runs are all non-conservative and whose island is unset already take the rendered path, and `InlineStyle` already drives `visual_highlight_style`.

## Risks / Trade-offs

- [Reconstruction proof too eager in exotic sources] → It requires exact equality with the parser's visible text; any mismatch (entities, smart-punctuation residue) keeps the conservative fallback. Differential fuzz-style tests compare projection round-trips.
- [`<br>`'s non-identity segment hits code assuming per-char identity] → Display length is 1, so only boundary offsets exist; audit segment consumers (hit testing, selection mapping, IME bounds) for length assumptions and cover click/arrow/selection across `<br>` with view tests.
- [Nested or interleaved supported tags pair incorrectly] → Strict stack nesting; crossing closes mark the block unsupported instead of guessing.
- [Mixed `<img>` + unsupported tag blocks regress] → Preserve the existing exemption behavior explicitly; extend the wrapped-badges view test with unpaired-tag cases.
- [Behavior change vs. old tests] → `escaped_inline_syntax_uses_conservative_fallback` and `visual_edit_marks_non_image_inline_html_for_conservative_runs` asserted the old collapse; they are rewritten to assert the new rendering plus the narrowed conservative triggers, not deleted.

## Migration Plan

Rendering-only change with no persistence, format, or shortcut impact: deploy and roll back as a single revert. No user migration. The support matrix (`docs/visual-editing-quality.md`) rows are updated in the same change so documentation cannot drift from behavior.
