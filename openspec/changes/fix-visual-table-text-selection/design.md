## Context

The virtual list measures a table as one item, while the caret is painted by an individual cell. A pointer move clears previous caret geometry before the next paint. Using the measured table as the reveal target in that gap can move the viewport by the height of several screenfuls.

## Goals / Non-Goals

Keep clicks and partial selections usable in the visible portion of a tall table. Preserve table editing, including its exact source ranges and existing focused source presentation.

## Decisions

- For a measured table that intersects the viewport, defer reveal calculations to the painted caret. The virtual list can still reveal a fully offscreen target, and post-paint follow handles the caret within a tall row.
- Shape text-only cells with the same text element before and after focusing. Build the rendered hit map with the Visual Edit block's cached source runs, including exact ranges for authored breaks. Use the existing editable source projection if the rendered text cannot be proven against those runs.
- Text selection continues to use canonical source ranges. Pointer movement only updates interaction state and does not regenerate the shared per-version Markdown derivation.

## Risks / Trade-offs

Parser output and preview whitespace normalization can differ. Validate the complete display text before using the rendered hit map. Exercise mixed and quoted tables alongside the reported text-only shape.
