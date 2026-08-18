## Context

Visual Edit has two prose render paths:

```
visual_text_with_math_element
 ├─ no math / HTML <img> / nav icons
 │    └─ visual_text_element: one StyledText, `\n` is a real line break  ← Read-like
 └─ otherwise (link ↗, footnote ↓, math atom, HTML image)
      └─ flex + flex_wrap children: whitespace-split fragments + sibling atoms
           `\n` is just another flex item → same visual line as neighbors   ← BUG
```

The mixed path exists so a navigation icon or atom can sit *after* a construct without living in the source projection (`add-visual-edit-link-navigation-icons`). Projection data is already correct: `unquoted_multiline_paragraph_projection_is_unchanged` pins `"alpha\nbeta"`. The collapse is layout-only, and it is triggered by the user's fixture because the first line is a Markdown link.

Read / Split Preview keep links inside one `StyledText` (`rich_text_element`), so they never hit this path.

## Goals / Non-Goals

**Goals:**

- Mixed-fragment rows stack logical lines on `\n` (SoftBreak and HardBreak) while each line still flex-wraps for long prose.
- Icons and atoms remain siblings on the line that owns their source end.
- Byte-exact display↔source mapping of non-break runs is unchanged.

**Non-Goals:**

- Rewriting mixed layout back into a single StyledText (icons cannot be sibling elements then).
- Changing `inline_runs` / `build_visual_projection`.
- Fixing the separate preview math flex-wrap path.
- Painting a dedicated hit target for the newline byte.

## Decisions

### D1: Group mixed children into logical lines, then stack the lines

Keep fragmenting for wrap + icons. After (or while) emitting children, split the sequence on fragments whose visible text is a line break (`\n`, optionally with a preceding `\r`). Each logical line is:

```
div().w_full().flex().flex_wrap().items_end().children(line_children)
```

The block content is a column of those rows:

```
div().w_full().flex().flex_col().children(line_rows)
```

Soft wrap inside a line is unchanged. A `\n` starts a new row even when the previous line is short, which is what Read mode does and what the screenshot is missing.

Rejected: inserting a `w_full` 0-height breaker into the existing single flex-wrap container. That CSS trick is brittle with `items_end` and zero-height rows, and it is harder to pin in `debug_bounds`.

Rejected: synthesizing extra visual *blocks* per source line. The parse already treats this as one paragraph; splitting rows at the `VisualBlock` layer would break caret ownership, block chrome, and the one-block-one-source-range contract.

### D2: Line breaks are layout structure, not flex text fragments

A visible `"\n"` (or `"\r\n"`) does not become a `VisualEditableText` child. The following fragments go onto the next line vector. Source mapping for adjacent prose is unchanged; the newline byte stays in the projection string for tests and caret math that already use the full projection on the first fragment.

HardBreak events already project as visible `"\n"` with a longer source range (`"  \n"`); `can_split` is false for that segment, so the line split must also inspect unsplittable visible text, not only whitespace-split fragments.

### D3: Debug selectors per logical line

Each line row gets `visual-mixed-line-{block_index}-{line_index}` so a gpui test can assert `line1.top() > line0.top()` on the reported fixture in a wide window (so width-wrap cannot fake the break).

## Risks / Trade-offs

- **[Empty trailing line]** A trailing `\n` in the projection would emit an empty flex row. Today's unquoted paragraph projection omits the block's structural trailing newline (`"alpha\nbeta"`), so this should not appear for the reported pattern.
- **[Link wrapping across a soft break]** `[text\nmore](url)` already splits into two fragments; the icon still emits after the last run sharing that navigation target, now on the second logical line. Correct.
- **[Navigation snapshots]** Mixed fragments already overwrite the per-block snapshot; stacking lines does not change that. Out of scope.

## Migration Plan

None. Presentation-only.
