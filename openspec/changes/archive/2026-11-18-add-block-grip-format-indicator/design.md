## Context

Visual Edit draws the existing grip outside each eligible row's content box. GPUI clips list items to the viewport, so the wider grip needs space inside that viewport, including at narrow widths.

## Goals / Non-Goals

Fit the format glyph and the drag dots in a single themed control. Keep its events on the existing grip and use its existing block eligibility. The change covers the grip's presentation and uses the existing settings and Markdown model.

## Decisions

- Use a shared document column in Visual Edit and Read mode with a minimum leading gutter inside the virtual list. Wider centered columns can use their existing margin. Use absolute positioning for both grip variants so toggling the preference or revealing the grip keeps the same content geometry.
- Map the cached Visual Edit block to the same format names and icons that the menu uses. Render headings as `H1`–`H6` and cover other existing grip types with their own icons and tooltip labels. Type changes and Undo are reflected when the normal document derivation updates the row.
- Place both halves inside the existing interactive element. The target validation, pointer handlers and menu opening continue through the same selection and reordering APIs.

## Risks / Trade-offs

The fixed gutter reserves some document width in narrow panes. Test that the grip remains fully inside the viewport, that both preference states lay out content identically, and that clicking or dragging the symbol works.
