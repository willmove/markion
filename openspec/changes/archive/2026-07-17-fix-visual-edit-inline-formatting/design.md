## Context

Visual Edit derives cached `VisualBlock` values from canonical Markdown and builds an ephemeral `VisualProjection` for each visible row. `inline_runs` already records byte-exact strong, emphasis, strikethrough, code, and link styles, while `visual_text_element` converts their projection spans to GPUI highlights.

Two boundaries currently break the default Inline formatting paragraph:

1. `inline_depth > 1` marks the text inside `***bold italic***` conservative, and `visual_block_view` turns any block containing such a run into a full source island.
2. Markion's highlight, superscript, and subscript extensions are parsed after `pulldown-cmark` for Preview, but Visual Edit reparses only the pulldown event stream. Their delimiters therefore remain visible and no Visual Edit style metadata exists.

The rendering data flow must remain:

```text
MarkdownDocument.text + version
  -> cached VisualBlock / VisualInlineRun / reveal groups
  -> cursor-dependent ephemeral VisualProjection
  -> GPUI StyledText highlights and exact source/display mapping
  -> existing source-backed input and mutation path
```

Cursor movement and local reveal must not mutate the document version or invalidate the cached `Arc<VisualBlock>`.

## Goals / Non-Goals

**Goals:**

- Render byte-exact nested strong/emphasis combinations without degrading unrelated inline runs in the same prose block.
- Use the same delimiter recognition rules for highlight, superscript, and subscript in Preview and Visual Edit.
- Hide supported delimiters while unfocused, reveal the smallest safe containing syntax group at the caret, and preserve monotonic UTF-8-safe mappings.
- Retain full-source fallback for ambiguous or byte-inexact constructs and cover the default welcome paragraph with model and rendered-window tests.

**Non-Goals:**

- Supporting arbitrary malformed or partially overlapping delimiter trees.
- Converting Visual Edit into a mutable rich-text document or changing undo, IME, persistence, or formatting actions.
- Adding new Markdown extensions or changing how Edit, Split Preview, and Read mode parse/render existing syntax.

## Decisions

### 1. Share range-aware extended delimiter recognition

Add a crate-internal range-aware matcher beside the existing extended-inline parser in `src/parse.rs`. It reports the recognized kind plus complete syntax and inner content ranges for `==...==`, `^...^`, and `~...~`, using the same non-empty, newline, length, and `~~` exclusion rules already used by Preview.

Visual Edit will split an exact pulldown text event at those ranges, omit delimiter bytes from rendered runs, apply the corresponding `InlineStyle` flag to content, and add a typed reveal candidate. Keeping delimiter recognition in `parse.rs` avoids a second grammar that can drift. Deriving styles from `PreviewBlock::RichText` was rejected because it no longer retains enough source ranges for precise editing.

### 2. Treat proper nesting as safe and partial overlap as ambiguous

Reveal candidates may overlap only when one complete source range contains the other (including equal ranges emitted for combined `***...***` syntax). Partial/crossing overlaps remain conservative fallback. Runs are no longer rejected solely because more than one supported inline tag is active; exact content lookup and UTF-8 boundary validation remain mandatory.

This permits combined bold-and-italic styling while preserving the existing safety boundary. Accepting every overlap was rejected because the current projection cannot prove a monotonic mapping through crossing ranges.

### 3. Normalize active nested reveals to outermost disjoint ranges

When the caret or selection endpoint activates multiple nested reveal groups, `build_visual_projection` will keep only the outermost containing source range before mixing source and rendered pieces. The complete containing syntax is then identity-mapped once, while unrelated runs remain rendered.

Emitting both nested ranges was rejected because duplicated overlapping source pieces would duplicate visible text and break source/display round trips. Revealing the full block was rejected because it recreates the reported regression.

### 4. Keep rendering styles aligned with Preview

`visual_highlight_style` will apply the same background/color treatment already used by `rich_text_element` for highlight and super/subscript spans. Strong, emphasis, strikethrough, inline code, and links retain their existing GPUI styles.

No per-frame parsing is introduced: all source-derived runs and reveal groups remain part of the per-version visual cache, and only range normalization/projection depends on interaction state.

## Risks / Trade-offs

- [Risk] The shared extended matcher diverges from the recursive Preview output → Refactor both paths to call the same delimiter-consumption helper and add parity tests for valid, invalid, nested, and UTF-8 inputs.
- [Risk] Nested reveal ranges duplicate source or make mapping non-monotonic → Normalize to outermost disjoint active ranges and assert source/display round trips at marker boundaries.
- [Risk] Relaxing `inline_depth` accepts an inexact nested run → Keep `push_run` exact-content validation and mark the block conservative whenever any visible text cannot be located byte-for-byte.
- [Risk] A single ambiguous construct still degrades its full block → Preserve that deliberate fallback and test it; localized ambiguous islands require a richer projection model and remain out of scope.

## Migration Plan

No persisted data migration is required. Implement the shared matcher, cached visual metadata, projection normalization, and renderer styles in that order, then enable the new regression expectations. Rollback restores the previous nested/extended fallback behavior without changing any Markdown file.

## Open Questions

None blocking. Emoji shortcode substitution and custom bare-autolink styling remain separate because their visible text is not always byte-identical to source.
