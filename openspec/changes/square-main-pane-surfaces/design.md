## Context

The main workspace constructs three GPUI content surfaces in `src/app/root_view.rs`: the source editor, the visual editor, and the rendered preview. Each surface currently applies the same medium corner radius after setting its theme-derived background and border. The rounded style is presentation-only; pane input handling, scrolling, resizing, and document rendering are attached independently.

Render flow:

1. The active view mode selects the source editor, visual editor, preview, or a split combination.
2. `root_view.rs` builds the selected surface with the active theme background, border, padding, and scroll behavior.
3. The surface hosts the existing editor element or preview list.
4. Document-version caches and per-tab scroll state feed those children and are not read or changed by the corner style.

## Goals / Non-Goals

**Goals:**

- Give every primary document surface square, zero-radius corners.
- Keep the result consistent across Edit, Visual Edit, Split Preview, and Read modes.
- Preserve all existing pane colors, borders, spacing, scrolling, resizing, and input behavior.

**Non-Goals:**

- Do not remove rounded corners from menus, dialogs, buttons, search controls, code blocks, or other secondary UI.
- Do not change theme values, pane density, typography, Markdown rendering, or editing semantics.
- Do not touch derived-state caches, syntax-highlight memoization, cached text handles, or per-tab scroll state.

## Decisions

1. Remove the corner-radius style only from the three primary document surface builders.

   Rationale: GPUI divs are square by default, so omitting the radius produces the requested shape while retaining the existing fill and border declarations. This keeps the implementation local and avoids introducing a new style abstraction for three static call sites.

   Alternative considered: set an explicit zero-radius style. This would document intent inline but adds unnecessary styling when the default already expresses square corners.

2. Include the visual editor surface in the change.

   Rationale: it is the primary editor surface in Visual Edit mode. Leaving it rounded would make the document chrome change shape when switching modes.

   Alternative considered: change only the two surfaces visible in Split Preview. That matches the supplied screenshot but creates inconsistent mode styling.

3. Leave borders and background fills unchanged.

   Rationale: the request concerns shape, while the current theme-derived border and surface background preserve separation and readability across light and dark themes.

   Alternative considered: remove the border or outer padding together with the radius. That would broaden the change into a pane-density redesign.

## Risks / Trade-offs

- [Risk] A less rounded primary surface may visually diverge from secondary controls that remain rounded. → Mitigation: limit the square treatment deliberately to the document workspace and verify it under representative light and dark themes.
- [Risk] One view mode could retain the old shape because its surface is built separately. → Mitigation: cover source, visual, and preview surface call sites and manually switch through all four view modes.

## Migration Plan

Apply the local GPUI style removals and verify all view modes. Rollback consists of restoring the three corner-radius style calls; no data or preference migration is required.

## Open Questions

None.
