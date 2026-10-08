## Context

See proposal.md for motivation. The cached `VisualBlockEditor::Table` already carries logical row/column identities and exact source ranges, including zero-length insertion ranges. The view only registers the active cell's navigation layout once the table owns the caret. Generic vertical navigation therefore treats a cell edge as a block edge. Empty text has no projection segments or usable text hitbox.

## Goals / Non-Goals

**Goals:** use the existing cell identities and source-backed layouts for column-preserving movement, full-interior clicks, and visible empty-cell carets.

**Non-Goals:** change persisted Markdown, parser ownership, source editing, or general multi-field traversal semantics.

## Decisions

1. Register geometry for every painted table cell, refreshing table snapshots once per list paint to avoid accumulating stale or duplicate windows on unchanged-selection repaints. Scope the current table navigation snapshot to its active cell, so wrapped-line and Home/End movement cannot land in sibling columns. At a cell boundary, explicitly locate the adjacent logical row in the same column and resolve a target using that cell's geometry. Retain generic adjacent-block navigation at the outer table boundaries. Merely enabling every cell's layout without scoping was rejected because closest-X navigation can switch columns. GPUI assigns exact wrap-boundary positions to the preceding line's trailing edge: scoped cell windows use the next UTF-8 boundary when necessary to select an actually paintable position on the target line, and hit results are clamped to the window's display range.
2. Empty table cells use a display-only blank glyph with a zero-length source mapping. Its geometry supplies a line and caret while both display edges resolve to the existing insertion offset. A literal space mutation was rejected because pointer/navigation actions must not edit Markdown.
3. Add a padding-click fallback on the editable cell container. Precisely handled text clicks stop bubbling to that fallback; grips retain their existing event isolation. Padding placement focuses the editor and follows the ordinary selection/caret cleanup path.

Data flow:

```text
per-version Table cell ranges --> per-frame cell projections/layouts
                                     |
                                     v
canonical caret --> scoped cell snapshot --> wrapped line / same-column cell
                                     |
                                     v
                              canonical selection
```

All new geometry is ephemeral. Pointer/navigation operations reuse the document's per-version shared block cache and do not change document version or history. Text input continues to use the existing checked cell mutation and undo paths.

## Risks / Trade-offs

- [Focused source reveal changes wrapping] -> exercise formatted cells and repeated movement after repaint; clamp every target to the exact target cell.
- [Sibling or stale geometry] -> retain document-version/block-identity checks and scope windows to cell source ranges.
- [Padding handler overrides precise clicks or controls] -> stop propagation for handled table text clicks and test text, padding, grips, and resize behavior.
- [Empty geometry leaks placeholder text] -> keep it in the display projection only; assert byte-identical source and cache identity after clicks/navigation.

## Migration Plan

No migration. Revert these view/navigation changes to roll back; existing documents and preferences remain compatible.
