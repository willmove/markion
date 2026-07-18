## Context

The completed `fix-visual-edit-input-and-caret` change gives Visual Edit one platform input bridge, source-backed caret geometry, complete whitespace coverage, and virtual-row following. Its remaining visual discontinuity is in `visual_block_view`: any focused supported block is replaced by `visual_source_island_view`, so clicking one bold word turns the entire paragraph, heading, list item, or blockquote into raw Markdown.

`MarkdownDocument.text` must remain the single canonical representation. `VisualBlock` and its inline runs are derived once per document version and shared through `Arc`; cursor movement and marker reveal are interaction state and must not invalidate preview, outline, stats, highlighting, cached text handles, or visual blocks. Existing editing commands already own undo, dirty state, autosave, IME, and Markdown-aware newline behavior.

Target data flow:

```text
MarkdownDocument.text + version
  -> cached VisualBlock + typed syntax/prefix ranges
  -> active cursor/selection chooses a safe local reveal group
  -> ephemeral display projection for the visible row
     (rendered spans + locally revealed source spans + exact mappings)
  -> GPUI text layout / hit testing / caret bounds
  -> existing source-backed edit commands
  -> MarkdownDocument.text + next version
```

## Goals / Non-Goals

**Goals:**

- Keep supported prose blocks visually rendered while the caret moves and text is edited.
- Reveal only the safe inline syntax group needed to understand or edit the active construct, including a link destination.
- Maintain exact source/display mappings for pointer placement, selections, keyboard navigation, IME, and replacement.
- Make Enter and Backspace at common heading, blockquote, list, and task-list boundaries behave like structural editing rather than raw marker deletion.
- Preserve per-version cache reuse when only the cursor, selection, reveal state, or visual layout changes.

**Non-Goals:**

- Native rich widgets for tables, images, fenced code, math, HTML, or front matter.
- A second mutable rich-text tree, a second undo stack, or serialization from rendered content back to Markdown.
- Guessing through nested or byte-inexact Markdown syntax; ambiguous cases remain full source islands.
- Changing Edit, Split Preview, or Read mode editing semantics.

## Decisions

### 1. Derive typed reveal groups once per document version

Extend the GPUI-independent visual model with typed, source-ranged syntax metadata. A reveal group describes one safely mapped inline construct (strong, emphasis, strikethrough, inline code, or link), its complete source range, its editable content ranges, and any link destination range. Supported block kinds also expose their structural prefix range and kind where it can be identified exactly.

All ranges must be UTF-8 boundaries, contained by the owning block, and validated against the canonical source slice. Nested, overlapping, or parser events that cannot prove an exact mapping set the existing conservative fallback instead of producing a reveal group.

This metadata belongs in the cached `VisualBlock` because it is derived from Markdown text and changes only with the document version. Re-scanning delimiters inside GPUI paint was rejected: it would duplicate parsing work, mishandle escapes and nesting, and weaken the cache invariant.

### 2. Build an ephemeral mixed display projection for each visible row

Replace the current all-rendered-versus-all-source choice with a small `VisualProjection` built from one cached block plus the active source selection. The projection contains display text, highlight spans, and monotonic mapping segments from display byte ranges to canonical source byte ranges.

For ordinary text, the projection uses the existing rendered run and its exact `content_range`. When the collapsed caret is inside a safe syntax group, or a selection endpoint lies inside it, that complete local group is emitted from canonical source with identity mapping and a restrained source-style highlight. A link group includes its brackets, label, destination, and optional title so the URL is directly editable. Other supported runs in the same block stay rendered.

Hidden marker positions map deterministically to the nearest adjacent displayed boundary. If keyboard navigation moves the source caret into a hidden marker, the next render reveals its group and restores identity mapping before further edits. Non-empty selections paint every covered projected segment; interior hidden markers do not require the entire paragraph to become source.

The projection is interaction-derived and is built only for virtualized visible rows. It is not stored in `MarkdownDocument` and does not change the document version. Caching the projection by document version was rejected because its output also depends on cursor and selection state.

### 3. Full source islands are an explicit fallback, not the focused default

Remove `focused` as a sufficient reason to call `visual_source_island_view`. Full source islands remain for code, math, HTML, front matter, unsupported parser gaps, byte-inexact inline runs, and ambiguous nested reveal groups. Images and tables retain their current conservative affordances until separate widget changes.

Block markers such as heading hashes, quote prefixes, and list bullets continue to render as visual chrome. Their raw prefix is revealed only if the source cursor actually enters that prefix or an ambiguous edit requires fallback; normal content focus does not expose the whole line.

### 4. Route structural Enter and Backspace through source-aware helpers

Editing actions remain source mutations and continue to take exactly one undo snapshot and call the existing post-change path once. Visual Edit adds a mode-aware block context lookup from the active cached `VisualBlock`:

- Enter in a heading splits to a following plain paragraph without copying the heading prefix.
- Enter in a non-empty ordered, unordered, task-list, or blockquote line continues the appropriate prefix; ordered numbering advances and a new task is unchecked.
- Enter on an empty list, task-list, or blockquote item removes the empty prefix and exits that structure.
- Backspace at the first visible content position outdents one nested list level when indentation remains; at top level it removes the complete heading, quote, list, or task prefix in one edit rather than deleting a single marker byte.
- Backspace elsewhere and any non-empty-selection replacement keep the existing grapheme-safe path.

Reusable source helpers live with `MarkdownDocument` or the visual model rather than GPUI rendering. Source Edit behavior is left unchanged unless an existing helper already provides the same Markdown semantics.

### 5. Test both the projection model and the rendered platform boundary

Pure tests cover reveal-group extraction, source/display round trips at UTF-8 boundaries, nested fallback, link destinations, structural edit ranges, and unchanged `Arc` reuse for cursor-only projection changes. GPUI rendered-window tests keep a supported block visual while focused, send platform input through the existing bridge, verify local reveal/hide transitions, and assert canonical text, caret/IME bounds, undo/redo, dirty state, and Read-mode non-mutation.

## Risks / Trade-offs

- [Risk] A projection boundary maps a click or selection to the wrong Markdown byte → Require monotonic UTF-8-safe segments, round-trip property tests, and conservative fallback whenever exactness is not provable.
- [Risk] Nested syntax produces overlapping reveal groups → Treat overlap/nesting as conservative fallback in this change rather than attempting partial rich editing.
- [Risk] Revealing a local group changes line wrapping and moves the caret → Reuse one-shot row reveal and painted caret geometry from the prerequisite change; reveal the smallest complete safe group.
- [Risk] Structural Backspace removes more source than expected → Activate it only at an exact collapsed-caret prefix boundary and cover each prefix kind and nesting level with source tests.
- [Risk] Per-frame projection adds typing cost → Build only virtualized visible rows from cached ranges and benchmark a large document without reparsing or cloning whole-document text.

## Migration Plan

1. Land after `fix-visual-edit-input-and-caret` so the input bridge and visual caret foundation are present.
2. Add typed syntax/prefix metadata and pure mapping tests without changing rendering.
3. Switch supported prose rows to mixed projections and add rendered regression tests.
4. Enable mode-aware structural commands and their undo/cache tests.

No persisted data or preference migration is required. Rollback restores the focused full-source-island path and removes ephemeral projection/structural helpers; Markdown files remain unchanged.

## Open Questions

No blocking questions. Highlight, superscript, and subscript marker groups may join the safe set during implementation only if their existing parser events provide the same exact range guarantees; otherwise they retain conservative rendering for a later change.
