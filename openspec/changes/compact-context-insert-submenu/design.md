## Context

See `proposal.md` for motivation and the delta spec for visible behavior. `BlockMenuState::submenu_items(Insertion)` already supplies the complete 21-action catalog. Every insertion item currently belongs to `BlockMenuGroup::Insertion`, which is not compact, so `rows_for` emits 21 single-action rows and `block_menu_item_button` renders full-width labels. The shared renderer only gives transform headings an H-level caption; insertion headings currently use the generic Text icon.

The existing panel and flyout already provide theme styling, localized tooltips, viewport flipping, menu-local scrolling, captured-target dispatch, and presentation-only lifecycle behavior. The rendered test `context_top_level_block_controls_and_bottom_insertion_are_reachable` traverses all 21 insertion actions in regular and constrained viewports and checks source/selection/cache preservation.

## Goals / Non-Goals

**Goals:** Keep layout classification consistent between rendered rows, compact button styling, and keyboard scrolling. Reuse the existing command catalog and dispatch instead of creating a second palette.

**Non-Goals:** Changing selection/target capture, Markdown mutation code, keyboard traversal order, or the layout of table/image flyouts and the root menu.

## Decisions

### Classify compact insertion groups in the shared item model

Give non-table block insertion commands and callout insertion commands distinct compact groups. Leave Table and Link/Formula/Image in the labeled insertion group. Existing contiguous grouping and six-column row splitting then produce two block rows, a labeled Table row, a five-button callout row, and the labeled inline settings actions in the existing command order.

Use the same classification for `rows_for` and `block_menu_item_button`. Merely changing renderer widths would leave one button per row and produce incorrect scroll targets. A renderer-only custom Insert grid would duplicate item indexing and bypass the tested shared menu path.

Data flow remains:

`captured BlockMenuState -> unchanged submenu_items catalog -> compact/labeled groups -> rows_for -> shared panel/buttons -> existing activation -> existing canonical insertion edit`

Opening, hovering, keyboard navigation, scrolling and dismissal modify only ephemeral menu state. Canonical document versions and per-version derived caches remain untouched until command confirmation; no new document parsing, syntax highlighting, or text-handle construction is introduced.

### Render insertion heading captions using the transform heading presentation

Extend the shared heading-content match to insertion Heading commands, so H1–H5 are distinguishable with the same captions and sizing as root transforms. Reuse existing command labels for localized tooltips, existing callout-kind icons, and `palette_controls` for button dimensions, padding, selected/disabled styling, and theme colors.

Keeping the generic Text icon for every heading would make compact commands ambiguous. Adding new strings or a parallel component is unnecessary because labels, tooltips, and H-level captions already exist.

### Extend existing rendered coverage

Extend the current insertion reachability test to assert equal root/flyout compact button sizing, shared row positions for paragraph/H1–H5, and a separate callout row. Preserve its traversal of all 21 commands in both themes/viewports and its source/selection/history/cache assertions. Exercise an actual compact H2 pointer click plus Undo to catch coordinate-to-command mistakes; retain existing exhaustive block/callout insertion tests and LF/CRLF coverage.

## Risks / Trade-offs

- [Mixed compact and labeled items accidentally share a row] -> Use distinct group classification, retaining the existing command order and separator behavior.
- [Keyboard selection scrolls to an obsolete single-row position] -> Generate render rows and scroll positions from the same `rows_for` result and retain constrained-viewport traversal coverage.
- [Icons reduce immediate label visibility] -> Preserve localized tooltips, use explicit H1–H5 captions, and keep specialized Table/Link/Formula/Image actions labeled.
- [The flyout can still overflow a very short viewport] -> Retain the existing max-height clamp, flip logic, and local scrolling.

## Migration Plan

No persisted data or configuration migration is required. Apply shared grouping and heading rendering together, extend the existing rendered test, run formatting and focused context-menu regression checks, and inspect the resulting flyout in the application when available. Rollback only the presentation changes if needed; document content and insertion semantics remain compatible.
