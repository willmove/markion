# Design: fix-visual-edit-list-nested-code

## Context

See proposal.md for motivation and the verified root-cause chain. The load-bearing facts established during exploration (confirmed with a pulldown-cmark 0.13.4 event-level diagnostic on the failing document):

- For a list item containing a nested fenced code block, pulldown-cmark emits `Start(Item)` → paragraph events → `Start/End(CodeBlock)` → `End(Item)`, and the item's tag range covers the entire item including the nested fence.
- `derive_preview_and_outline` (`src/lib.rs`) pushes the `CodeBlock` preview block to the top-level stream immediately at `End(CodeBlock)`, but the enclosing `ListItem` is only flushed at `End(Item)` — so the stream order is `CodeBlock, ListItem` (reversed) and the ranges overlap (item swallows code).
- Reading mode renders the block stream in vector order with per-block independent rendering, so the only symptom there is the code box appearing above its bullet.
- `build_visual_blocks` (`src/visual.rs:416`) assumes leaves arrive with monotonically ordered, disjoint source ranges. The violation triggers three cascades: (1) the item's text line becomes an `Unsupported` gap box, (2) `fenced_payload_ranges` rejects the nested fence because payload/closing lines retain the list's 4-space indent (`indent <= 3` check), so the code block falls back to a `Code` source island showing literal fences, (3) the overlap guard force-marks the `ListItem` `Unsupported`, re-rendering the whole item source — the duplication seen in the screenshot.

## Goals / Non-Goals

**Goals:**
- Preview block stream is in document order for list items with nested block constructs, and a list item's source range no longer swallows a nested block that is emitted as its own block.
- Visual Edit renders the item as a normal editable list row and the nested fence as the standard source-backed code editor row — no raw-source fallback, no duplication.
- Reading mode shows the nested code block below its bullet as a side effect of the ordering fix.
- The fix generalizes to other block constructs nested in list items (tables, blockquotes) insofar as the same ordering/partition mechanism covers them.

**Non-Goals:**
- No dedenting of nested code payload for display; the payload editor shows the authored (indented) bytes exactly.
- No redesign of how multiple sibling paragraphs inside one list item are flattened into the item's spans.
- Indented (non-fenced) code blocks nested in lists keep the current conservative source-island presentation.
- No change to canonical source text, persistence, settings, or the caching architecture.

## Decisions

### D1: Restore document order in the shared parse layer with a final stable sort

In `derive_preview_and_outline`, stable-sort the finished `blocks` vector by `source_range.start` before returning (equivalently: insertion-adjust at flush time; the final sort is simpler and covers every nesting reversal at once).

- **Why the parse layer, not the visual layer**: the reversed order is visible to *every* consumer — reading mode order, preview copy/selection ordering, export document order, and sync-scroll source anchoring. Sorting expanded leaves inside `build_visual_blocks` would fix only Visual Edit and leave the shared stream wrong.
- **Why a final sort, not early-flush**: flushing the item draft at `Start(CodeBlock)` would require draft surgery for content that follows the nested block inside the same item (today trailing paragraphs are flattened into the item's spans) and could emit a second bullet row for one authored item. The final sort changes no accumulation logic at all.
- **Stable** sort preserves event order for blocks with equal starts, and for all well-formed documents the stream is already in source order, so the sort is a no-op except for the nesting case.
- Consumers that assign ranges during the event loop (e.g. table range assignment, outline heading collection) run before the sort and are unaffected. The incremental source-mapped derivation contract ("incremental output equals full parse") is preserved because ordering becomes part of the full-parse output that incremental derivation must equal; the incremental path derives through the same routine.

**Alternatives considered**: (a) visual-layer-only sort of expanded leaves — rejected, leaves reading mode and other consumers reversed; (b) early-flush the item at nested-block start — rejected, splits one authored item into multiple blocks and complicates trailing-content handling; (c) give `ListItem` a `children` field like `BlockQuote` — rejected as a cross-cutting model refactor (preview rendering, export, memory accounting) disproportionate to the bug.

### D2: Truncate the swallowed list-item range in the parse layer, not the visual layer

The plan during exploration was to generalize the nested-list partition pass in `build_visual_blocks` (`src/visual.rs:494-514`) to truncate a `ListItem` leaf at any following nested leaf. Implementation revealed the parse layer is the better home: `derive_preview_and_outline` tracks the earliest nested block start (`CodeBlock`, `MathBlock` via fenced code, `Table`) while a list item draft is open and truncates the item's `source_range.end` there at flush time. Combined with D1's sort, the preview stream itself becomes ordered and disjoint, so every consumer (reading mode, export, preview copy, sync scroll, Visual Edit) sees the corrected shape and no visual-layer partition change is needed — the visual regression tests pass with `build_visual_blocks` untouched.

- The existing `ListItem`→`ListItem` visual partition stays as-is: nested list items are flushed after their parent, so that overlap pattern never reaches the parse-layer truncation and continues to be partitioned in visual land.
- The truncated item row covers only its direct text; `inline_runs` re-parses exactly that slice, so bullet prefix, links, and reveal groups work as for any flat list item. Trailing blank lines inside the item stay inside the truncated range and are absorbed by the existing trailing-whitespace run handling — no new gap boxes.
- Inline `![images]()` inside list items are deliberately NOT treated as nested blocks for truncation (they are inline constructs; truncating there would cut the item text mid-sentence). Their pre-existing overlap quirks are unchanged and out of scope.
- Blockquotes nested in list items keep their current presentation; the tracking deliberately covers code/table blocks only.
- The overlap guard (`src/visual.rs:627`) stays as a defense-in-depth safety net; after D1+D2 it simply never fires for this pattern.

### D3: Teach `fenced_payload_ranges` the list-relative indentation of nested fences

pulldown-cmark reports a nested fence's `CodeBlock` range starting at the opening backticks (excluding the list indent), while payload and closing-fence lines retain their list indentation. The current closing-fence scan requires `indent <= 3` (CommonMark's top-level fence rule) and therefore never finds the indented closing fence, so `visual_block_editor` returns `None` and the block degrades to a raw `Code` source island.

Fix: keep the strict `indent <= 3` scan first (top-level behavior unchanged). If no closing fence was found, and only when the opening fence itself is indented 4+ in the document (spaces between its line start and the parser-reported range start — the signature of container nesting), measure the common leading-space indentation of non-blank payload lines; when it exceeds 3, re-scan accepting a closing fence whose indentation is at most that payload indentation, whose marker run is at least the opening run length, and whose remainder is empty. All returned ranges stay byte-exact against the canonical source, so the payload editor edits the authored (indented) bytes losslessly. The indentation gate prevents a pathological top-level unclosed fence whose payload merely looks like an indented fence from being misread as closed (pinned by a dedicated guard test).

**Alternative considered**: display-time dedent of the payload — rejected; it would decouple displayed columns from source columns and complicate caret mapping for no functional gain.

## Risks / Trade-offs

- [Stable sort reorders blocks for documents that previously relied on event-order quirks] → Event order equals source order for every construct except container-nested blocks, which are exactly the broken cases; add characterization tests over representative fixtures (tables, alerts, footnotes, nested lists, quotes) pinning the expected order before and after.
- [Truncated list-item ranges no longer cover the item's trailing structural bytes, so caret/selection math that assumed item-end == item-content-end could shift] → The existing trailing-whitespace run and whitespace gap rows already model those bytes as valid caret positions; verify with caret-mapping tests around the partition boundary.
- [The lenient closing-fence scan could misidentify a payload line as the closing fence (e.g. an indented line of only backticks inside the payload)] → pulldown-cmark already decided the block's extent; we only re-locate its fence lines within that exact range, and a payload line consisting solely of a longer backtick run would have extended the real fence. Unit-test such payloads.
- [Reading-mode vertical order changes for affected documents (code moves below its bullet)] → This is the author-intended order and matches every other Markdown renderer; called out in the proposal as expected.

## Migration Plan

Pure derivation/rendering fix. No data, settings, or API migration. Rollback = revert the change; no state is persisted from the block stream.

## Open Questions

None blocking. Whether blockquote/table constructs nested in list items need additional visual-row polish beyond the shared ordering/partition mechanism can be assessed during implementation testing; the mechanism itself is construct-agnostic.
