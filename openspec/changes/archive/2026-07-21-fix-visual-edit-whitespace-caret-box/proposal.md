## Why

In Visual Edit mode, pressing Enter to create a blank line, or pressing Down/Up arrow across an existing blank line, drops the caret onto a `Whitespace` row. That row is currently rendered through `visual_source_island_view` — a bordered, padded, monospace, gray-background box visually indistinguishable from a code/source island. Users see what used to be ordinary paragraph spacing turn into a "large edit box" and reasonably conclude the paragraph is no longer a normal paragraph. The recent default-mode flip to Visual Edit (`50deb4d`) makes this the first impression for every new user.

The Markdown-editing spec never required this affordance to be a source-island box — it only requires that a whitespace range "provide the source-backed editing affordance" when the caret enters it. A 2px caret line with normal text input is a valid, less confusing affordance.

## What Changes

- A `Whitespace` visual block that owns the caret is rendered as a passive-height row containing a thin caret line, **not** as a `visual_source_island_view` box. Typing still inserts into the canonical Markdown source at the caret's source position — no behavior change to input, undo, autosave, or document mutations.
- The `focused_conservative` routing in `visual_block_view` no longer promotes a `Whitespace` block to `visual_source_island_view`. Other conservative-fallback blocks (empty non-Whitespace blocks, `FrontMatter` / `Code` / `Html` / `Unsupported` source islands, `conservative_fallback` runs) keep their existing rendering.
- Whitespace blocks that do **not** own the caret keep the existing passive layout (no change).
- The Markdown-editing spec's "Visual Edit whitespace activation" requirement is sharpened to state the affordance is a caret line, not a source island.

## Capabilities

### New Capabilities

<!-- None — this change sharpens an existing capability. -->

### Modified Capabilities

- `markdown-editing`: The "Visual Edit whitespace activation" requirement is updated to specify that when a whitespace range owns the caret, Visual Edit renders a caret line (same visual as a paragraph/heading caret) and accepts typed text at the source position, **without** wrapping the row in a source-island box. The "passive until caret entry" semantics and the source-backed-editing guarantee are retained.

## Impact

- **Code**: `src/app/preview.rs` — `visual_block_view` `focused_conservative` predicate (`!is_whitespace`); `VisualBlockKind::Whitespace` render arm (paint a caret when the block owns the caret, reuse `VisualEditableText` with an empty projection); new helper `visual_whitespace_caret_element`.
- **Invariants touched**: none. The caret is still drawn by the existing `VisualEditableText` element; input still flows through the document-level `replace_text_in_range` path; per-version visual-block caching is unchanged.
- **Dependencies**: none new.
- **Non-goals**: changing `move_visual_vertical` navigation (caret can still land on blank lines); changing passive whitespace rendering when the caret is elsewhere; touching `always_source` source islands (FrontMatter/Code/Html/Unsupported), which legitimately render as source islands because they have no visual rendering.
