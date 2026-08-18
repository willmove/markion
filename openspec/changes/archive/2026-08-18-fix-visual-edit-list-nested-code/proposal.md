# Proposal: fix-visual-edit-list-nested-code

## Why

In Visual Edit mode, a list item containing a nested fenced code block renders as a cascade of raw-source boxes: the bullet line appears as unrendered Markdown in one box, the fenced block appears with literal ` ``` ` fences in another, and the entire list item source is then duplicated in a third box. This was observed on a real-world document (`- 推荐使用 Anthropic API 兼容…` followed by an indented ` ``` ` fence) and makes a common note-taking pattern (API setup instructions under a bullet) unreadable in Visual Edit.

Root cause (verified with a pulldown-cmark event-level diagnostic): when a fenced code block is nested inside a list item, `derive_preview_and_outline` pushes the `CodeBlock` preview block to the top-level block stream *before* the enclosing `ListItem` is flushed at `End(Item)`, and the `ListItem`'s `source_range` still covers the whole item including the nested block. The shared block stream is therefore out of document order with overlapping ranges. Reading mode tolerates this silently (it renders each block independently, only showing the code box slightly out of place above its bullet), but Visual Edit's `build_visual_blocks` assumes leaf source ranges are monotonically ordered and disjoint; the violation triggers gap fallback boxes, fails fenced-editor derivation, and force-marks the list item as an `Unsupported` source island.

## What Changes

- **Parser ordering**: the shared preview block stream SHALL be emitted in document (source) order when a list item contains nested block constructs such as fenced code blocks, so the list item row precedes its nested blocks.
- **Visual range partitioning**: `build_visual_blocks` SHALL partition a list item's source range at the start of a following nested block (generalizing the existing nested-*list* partition), so the item's text renders as a normal list row and the nested block renders as its own visual row with no overlap and no duplication.
- **Nested fence editing**: `fenced_payload_ranges` SHALL accept the list-relative indentation that pulldown-cmark retains on payload and closing-fence lines of a nested fence, so a nested code block gets the standard source-backed code editor instead of a raw source island.
- **Regression tests** covering parser ordering, visual partitioning, and fence editor ranges for the reported pattern.
- **Non-goals**: no change to how reading mode styles these blocks beyond the corrected ordering; no dedenting of code payload display; no redesign of blockquote/table nesting inside list items beyond what the ordering + partition fixes already cover; no change to the canonical source text or any persisted format.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: strengthen *Markdown parsing via CommonMark + GFM* — nested blocks inside list items must yield a document-ordered block stream whose ranges do not swallow nested sibling blocks; extend *Source-backed Visual Edit mode* — a list item containing a nested fenced code block must render the item as an editable list row and the code block as its source-backed editor row, exactly once each, without conservative raw-source fallback.

## Impact

- **Code**: `src/lib.rs` (`derive_preview_and_outline` block emission order), `src/visual.rs` (`build_visual_blocks` partition pass, `fenced_payload_ranges` indentation handling). Reading-mode rendering (`src/app/preview.rs`, `src/app/root_view.rs`) consumes the same stream unchanged and benefits from corrected ordering automatically.
- **Invariants touched**: derived-state caching per document version is preserved (same single parse, same caches); the incremental source-mapped derivation contract ("incremental output equals full parse") is preserved because ordering becomes part of the full-parse output that incremental derivation must match.
- **Specs**: `openspec/specs/markdown-editing/spec.md` delta as above.
- **Compatibility**: pure rendering/derivation fix — no file format, settings, or API changes; no migration needed.
