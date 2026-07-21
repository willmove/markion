# Tasks

## 1. Definition collection

- [x] 1.1 In `src/visual.rs`, add a helper that scans the full document line by line and collects link reference definition lines (`[label]: dest` with optional title), skipping lines inside fenced code blocks (reusing the fence-tracking helpers from `src/source_mapped.rs`, now `pub(crate)`), excluding footnote definitions (`[^label]:`), and allowing at most three leading spaces.
- [x] 1.2 Unit-test the collector: top-level definitions are collected; definitions inside fenced code blocks, footnote definitions, and four-space-indented (code block) lines are not.

## 2. Resolve references during per-block parsing

- [x] 2.1 In `build_visual_blocks`, run the collector once and thread the resulting definition suffix through `visual_block_from_preview` into `inline_runs`.
- [x] 2.2 In `inline_runs`, when the suffix is non-empty, parse `block slice + blank line + suffix` so pulldown-cmark resolves reference-style links; event ranges stay relative to the original block slice, and the event loop stops at the first event past the block slice (e.g. from a malformed definition line the parser could not consume).
- [x] 2.3 Widen `reveal_candidate_is_exact` for `VisualRevealKind::Link` to accept reference forms (full `[text][label]`, collapsed `[label][]`, shortcut `[label]`) without requiring a local `link_target_range`; keep the strict in-range destination check for inline links. Extend collapsed-form candidates to cover the trailing `[]`, which pulldown-cmark excludes from the tag range.
- [x] 2.4 Style reference-link labels as links in the visual projection: a rendered run counts as a link span when it carries a local target range (inline link) or lies inside a link reveal group's source range (reference link).
- [x] 2.5 Regression test: full `[text][label]`, collapsed `[label][]`, and shortcut `[label]` forms render as links (rendered label run, link reveal group, in-block ranges only, link-styled projection span) with definitions in a trailing block.
- [x] 2.6 Regression test: a `[label]: dest`-shaped line inside a fenced code block does not turn matching prose `[text][label]` into a link.
- [x] 2.7 Regression test: an undefined reference stays literal text.

## 3. Validate

- [x] 3.1 `cargo test --workspace` green.
- [x] 3.2 `openspec validate resolve-reference-links-in-visual-edit` passes.
