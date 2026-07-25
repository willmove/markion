# Fix incremental region splitting for indented continuations

## Why

In release builds, Visual Edit and Split Preview intermittently render ordered/unordered list items and block quotes incorrectly after edits: a list item's indented continuation paragraph detaches into a stand-alone paragraph, items lose their continuation content, and quotes nested in list items break out of the item. Reproduced deterministically with a release-mode diff harness (incremental `SourceMappedCache::update` vs. full `derive_preview_and_outline`):

- `1. item\n\n   continuation\n\n2. two` — 123/149 single-character edits produce corrupted blocks.
- `1. item\n\n   > quoted note\n\n   more text\n\n2. two` — 118 corrupting edits (the user-visible "引用显示不正确").
- `- item\n\n  continuation\n\n- two` — 114 corrupting edits.
- `1. item\n\n   ```rust …` (fence inside a list item) — 115 corrupting edits.

Root cause is `split_regions` in `src/source_mapped.rs` placing region boundaries inside multi-part list items:

1. `starts_with_continuation` treats only `\t`/4-space indents as continuations, but list-item continuation indent is the marker width (2 spaces for `- `, 3 for `1. `). A blank line followed by a 2–3-space-indented line gets a region boundary, so the continuation is reparsed standalone as a top-level paragraph/quote.
2. The fence-opening branch inserts a boundary after a blank line unconditionally, even when the fence is indented as list-item content.

The bug ships in every release build because the correctness oracle in `SourceMappedCache::update` is `#[cfg(debug_assertions)]`-only: debug/test builds silently repair the mismatch via full fallback, so no existing test can fail on it. It predates the memory-retention work but was surfaced by the same release testing.

## What Changes

- `starts_with_continuation` SHALL treat any line that begins with whitespace (one or more spaces or a tab) as a continuation, so indented list-item content never opens a new region. Boundaries are an optimization; being conservative only reduces incremental reuse, never correctness.
- The fence-opening branch of `split_regions` SHALL only insert a boundary for fences that start at column 0 (unindented). Indented fences are list-item content and must stay in the containing region.
- Add regression coverage that works in debug builds despite the oracle: after `SourceMappedCache::update`, assert `counters.full_fallbacks` did not increase for these fixtures (the oracle increments it when it has to repair a mismatch), plus direct `split_regions` boundary unit tests for the four fixture shapes above.

Non-goals: changing the assembled block model, pulldown-cmark options, the debug oracle itself, or Visual Edit rendering; performance tuning of region reuse.

## Capabilities

### Modified Capabilities

- `markdown-editing`: the incremental preview/visual derivation must be observably equivalent to full derivation for list items with blank-line-separated indented continuations (paragraphs, quotes, fences) — in release builds, not only under the debug oracle.

## Impact

- `src/source_mapped.rs` (`split_regions`, `starts_with_continuation`) and its tests.
- No app-side (`src/app/**`) changes required.
