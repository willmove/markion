## Context

Rendered paragraphs with inline formulas, images, script text, or certain other inline features use flex rows. Each style span is currently split at whitespace. Flex can wrap at a style boundary, and unspaced text following that boundary is measured as one fragment. Ordinary prose uses GPUI shaped lines, whose character-based fallback can also put closing punctuation at the start of a row.

## Goals / Non-Goals

Derive wrap opportunities from visible text while retaining the existing element and source ranges used for painting and interaction. The change is scoped to layout and requires no migration.

## Decisions

- Compute Unicode line break opportunities over the displayed text before splitting mixed prose into styled fragments. Emit adjacent fragments in one flex item when they share a wrap unit. This keeps each fragment's selection, link, and caret behavior intact and allows unspaced CJK prose to wrap.
- Keep explicit newlines and inline widgets on the existing row-building path. Keep non-identity visual projection segments atomic so display offsets cannot split their source ranges.
- In the shared GPUI shaped-line wrapper, keep the existing wrap heuristic and exclude closing punctuation from its candidate and fallback positions. Use the Unicode line-break classes, available from a dependency already present in the lockfile.
- Compute mixed layout opportunities from the displayed text during layout. Parsing, derived Markdown state, and document version keep their existing lifecycle.

## Risks / Trade-offs

- Unspaced CJK produces more mixed-layout text fragments. The ordinary shaped path continues to use one shaped element. In mixed layout a wrapper is needed only when a wrap unit spans multiple children.
- Wrap boundaries change navigation geometry. Verify actual painted positions and existing tests for authored breaks, inline atoms, and visual navigation.
