# Design: fix-list-item-block-roundtrip

## Context

`render_to_markdown` (`crates/markdown/src/renderer.rs`) serializes a `Document` AST back to Markdown source. Its `render_list` writes each item as `<marker> <inline content>\n`, then renders `item.blocks` (child blocks such as additional paragraphs or code blocks) on immediately following lines indented to the item's continuation column — with no blank line in between.

CommonMark only attaches an indented block to a list item as a *separate* child block when it is preceded by a blank line; without one, an indented text line is a **lazy continuation** of the item's paragraph. Consequence, reproduced against the real parser:

```
source:    "- first paragraph\n\n  second paragraph\n"
parse:     List item { content: "first paragraph", blocks: [Paragraph "second paragraph"] }
render:    "- first paragraph\n  second paragraph\n"        ← blank line lost
re-parse:  List item { content: "first paragraph second paragraph", blocks: [] }
```

The same missing separation applies between *sibling* child blocks: the loop over `item.blocks` calls `render_block` per block with no blank line, so two child paragraphs would also merge. (Top-level blocks get separation from `render_blocks`; list-item children do not.)

The export engine feeds `render_to_markdown` output to pandoc (`crates/export/src/engine.rs:124`), so the corruption reaches user-facing exports, not only the test suite.

## Goals / Non-Goals

**Goals:**

- List items whose AST contains child blocks render with the blank lines CommonMark requires, so `parse(render(doc))` preserves the item's block structure.
- Keep the existing compact rendering for tight items (inline content only, or content + nested sub-list) byte-identical to today.
- Lock the behavior with unit tests and a persisted proptest regression seed.

**Non-Goals:**

- General escaping of `Inline::Text` payloads (verbatim rendering of text containing Markdown control characters is a broader, separate problem; only `escape_leading_block_marker` exists today).
- Parser changes; AST changes (e.g. ordered nested sub-lists).
- Normalizing tight/loose list style on render.

## Decisions

1. **Blank-line separation inside `render_list`, not in callers.** When `item.blocks` is non-empty, emit one blank line (bare `\n` — an empty line, which CommonMark accepts inside a list item) after the content line, and render child blocks through `render_blocks`-equivalent separation at `continuation_indent`. A bare empty line is simpler than an indent-padded one and cannot be mistaken for content.
2. **Leave sub-item rendering unchanged.** A nested list directly below the item line is valid CommonMark (a list may follow the item paragraph without a blank line) and re-parses correctly today.
3. **Persist the failing seed.** Add seed `8b1f2303c3c82c49e7116a1934c95f3ba3c1d2da4ed68a30e09ec56211df7975` (shrinks to list item `?` + paragraph `  .`) to `crates/markdown/tests/roundtrip_property_test.proptest-regressions` after the fix, so the case replays in every environment regardless of proptest's persistence-path resolution.

## Verification

- New unit tests: loose two-paragraph item, item with paragraph + code-block children, nested sub-list unchanged.
- Reproduction case from the diagnosis: `"- first paragraph\n\n  second paragraph\n"` round-trips structurally.
- `cargo test -p markdown` and `cargo test --workspace` pass from the repository root, including `roundtrip_property_test`.
