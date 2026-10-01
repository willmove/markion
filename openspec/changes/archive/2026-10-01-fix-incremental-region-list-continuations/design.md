# Design: region boundaries must respect container continuations

## Context

`SourceMappedCache` speeds up typing by splitting the document into regions at blank-line boundaries, reparsing only changed regions, and concatenating region-local blocks with offset shifts (`assemble_regions`). A region boundary is only sound when the text on both sides parses the same standalone as it does inside the whole document. Blank-line-separated *continuations* of a container (list item, quote) violate that: `   continuation` after `1. item` + blank line is part of item 1 in a full parse, but a top-level paragraph when its region is parsed standalone.

`split_regions` already knows this — `starts_with_continuation` suppresses boundaries before lines that look like container content — but its indent rule (`\t` or exactly ≥4 spaces) misses the most common case: list continuation indent equal to the marker width (2–3 spaces). The fence branch bypasses `starts_with_continuation` entirely.

The failure is invisible in debug/test builds: `update()` compares the incremental result against a full parse under `#[cfg(debug_assertions)]` and silently substitutes the full result (incrementing `counters.full_fallbacks`). Release builds publish the corrupted blocks directly into Split Preview and Visual Edit.

## Goals / Non-Goals

- Goal: incremental derivation equals full derivation for indented continuations of list items and quotes, verified by tests that fail in debug builds if regressed.
- Goal: keep the region optimization effective for the dominant case (documents dominated by top-level paragraphs/headings).
- Non-goal: making region splitting understand full CommonMark container structure; conservative merging is acceptable.

## Decisions

### 1. Any leading whitespace is a continuation

`starts_with_continuation` returns `true` when the raw line starts with a space or tab (before the existing marker checks on the trimmed text). Rationale: after a blank line, an indented line is either (a) a container continuation — boundary would corrupt, or (b) an indented code block — already covered, or (c) a 1–3-space-indented top-level paragraph — merging it into the previous region is harmless (regions are an optimization, and a full-region parse of merged text is still correct because the merged text is contiguous source).

### 2. Fence boundaries only at column 0

In `split_regions`, only take the `opening_fence` branch's `pending_break` boundary when the line has no leading whitespace (`raw == trimmed`). An indented fence after a blank line is list-item content; the fence-tracking state (`in_fence`) still updates either way so closing detection is unchanged. Note `opening_fence` receives the trimmed line, so fence tracking itself is indentation-agnostic — only the *boundary insertion* becomes indentation-sensitive.

### 3. Regression tests via the fallback counter

Debug builds can't observe corrupted output (the oracle repairs it), but they can observe that repair: `DerivationCounters::full_fallbacks` increments exactly when the oracle catches a mismatch. Tests seed a cache, apply a single-character edit through `MarkdownDocument::replace_range`, force re-derivation, and assert the fallback counter stayed at its previous value for each fixture. Complement with pure `split_regions` unit tests asserting no boundary lands inside the fixture constructs.

## Risks / Trade-offs

- Larger regions → less incremental reuse for list/quote-heavy documents. Acceptable: correctness over speed, and these documents were previously being *misparsed*, not sped up.
- Rule 1 also merges legitimately separable regions (e.g. a 1-space-indented paragraph). Impact is limited to reparse cost of the merged region.

## Open Questions

- Should the oracle also run (sampled or cheap-hash) in release builds as a safety net? Deferred; out of scope here.
