## Context

The editor and preview content areas already use GPUI scroll containers (`editor-scroll` and `preview-scroll`) with per-tab `ScrollHandle`s. The current layout gives each pane generous outer padding (`p_4`) plus inner content padding, and the split/sidebar resize controls use a narrow visible rule with a wider invisible drag target.

The change is a layout and interaction refinement: make vertical scrolling discoverable and draggable in both source and preview panes, and reclaim space currently consumed by pane chrome. It should not affect the document model, Markdown parsing, preview-block computation, syntax highlighting, undo/redo, or tab isolation.

Data flow:

1. The active tab owns editor and preview scroll handles.
2. The render path attaches those handles to the editor and preview scroll containers.
3. GPUI handles wheel/trackpad scrolling and scrollbar drag input against those containers.
4. Pane density changes alter spacing and visible chrome only; the same editor element and preview block data are rendered.

## Goals / Non-Goals

**Goals:**

- Show usable right-side vertical scrollbars for both editor and preview panes when document content exceeds the visible area.
- Preserve existing wheel/trackpad scrolling and per-tab scroll restoration.
- Reduce main content pane gaps and outer padding to roughly 15% of the current visual spacing.
- Keep resize handles easy to drag while reducing their visible visual footprint.
- Keep Edit and Read modes full-width within the available workspace.

**Non-Goals:**

- No virtualized/lazy rendering rewrite.
- No Markdown parser, derived cache, text handle, export, or theme-palette changes.
- No new preferences for density or scrollbar behavior.
- No change to file-tree row limiting or search behavior.

## Decisions

1. Keep GPUI scroll containers as the interaction primitive.

   Rationale: the app already tracks pane scroll state through `ScrollHandle`s, so improving scrollbar visibility should happen in the existing `overflow_y_scroll` containers rather than introducing separate custom scroll widgets.

   Alternative considered: implement custom scrollbar elements. That would increase input-handling complexity and risk diverging from GPUI scroll behavior.

2. Make scrollbars visually discoverable without consuming excessive content width.

   Rationale: visible right-side scrollbars solve the large-document navigation problem, but wide chrome would fight the density goal. Use a compact but draggable width and leave the pane content padding modest.

   Alternative considered: always show oversized scrollbars. Easier to hit, but wastes horizontal reading/editing space.

3. Tighten only the editor shell spacing, not typography.

   Rationale: the user asked for smaller gaps between panes and borders, not smaller text. Reducing pane padding and gutter spacing preserves readability while fitting more lines/blocks into the viewport.

   Alternative considered: reduce font size or line height. That would change reading/editing ergonomics and may make content harder to inspect.

4. Preserve invisible resize hit targets while minimizing visible separators.

   Rationale: users still need to drag split/sidebar handles reliably. Keep the transparent drag target but make visible rules thin and avoid extra pane padding around them.

   Alternative considered: shrink both visible rule and hit target. That would improve density but make resizing frustrating.

## Risks / Trade-offs

- [Risk] GPUI scrollbar visibility may depend on platform/theme behavior -> Mitigation: verify large editor and preview content manually after implementation and keep `overflow_y_scroll` plus explicit scrollbar width.
- [Risk] Over-tightening spacing can make controls feel cramped -> Mitigation: reduce pane outer padding aggressively but retain minimal inner padding around text.
- [Risk] Hidden/visible pane logic in Edit/Read modes can interact with flex basis -> Mitigation: preserve current full-width mode logic and test switching after density changes.
- [Risk] Smaller visible separators can be harder to notice -> Mitigation: retain border color contrast and transparent drag targets.
