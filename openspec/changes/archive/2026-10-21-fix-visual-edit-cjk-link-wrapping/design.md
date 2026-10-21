## Context

`visual_text_with_math_element` sends blocks containing any navigation target to a mixed flex layout. That layout splits identity-mapped text into whitespace-delimited children. CJK text without spaces therefore stays in a single child and cannot use the shaped text layout's natural line breaking. The regular `visual_text_element` already owns a complete source projection and handles caret mapping, editing, IME, and secondary-modifier navigation.

## Goals / Non-Goals

**Goals:** Use shaped text for ordinary URL and heading links when no other mixed-layout feature is present. Keep link interaction and source mapping intact.

**Non-Goals:** Change stable inline link edit icons, footnote tooltip rendering, or the mixed layout for math, images, scripts, and authored HTML breaks.

## Decisions

- Remove ordinary link navigation as a reason to choose the mixed layout. Continue requiring the mixed layout when footnote navigation is present because its fragments provide footnote tooltips.
- Keep the `nav_icons.is_empty()` gate. Stable inline mode creates link edit icons in the mixed row, retaining the explicit edit affordance and its click handling.
- Keep all other mixed-layout gates unchanged. Both renderers use the existing version-cached `VisualBlock` and `VisualProjection`; no parsing or cache lifecycle changes.

## Risks / Trade-offs

- A shaped link row must still activate URLs and headings from source offsets. `VisualEditableText` already calls `activate_visual_navigation_at` with the mapped source position; integration coverage will guard this path.
- Existing tests inspect mixed-row debug selectors. Update them to verify shaped navigation geometry and line breaks instead of renderer implementation details.
