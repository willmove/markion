## Context

See proposal.md for motivation. `BlockMenuState::root_items` exposes Divider/Table transforms in both grip and ordinary editing catalogs; `serialize_transform` substitutes `---` or an empty table regardless of the original body. Table-cell editing menus already omit these controls. Existing `insert_block_below` and empty-paragraph insertion helpers provide checked canonical edits and keep complete source units intact. A separate existing worktree change compacts the Insert flyout; this change preserves that catalog and layout.

## Goals / Non-Goals

**Goals:** Share insertion behavior between compact root shortcuts and the existing flyout, keep whole-block target ownership distinct from selection, and make a divider immediately usable for continued typing.

**Non-Goals:** New editing models, reparsing on menu interaction, modifying row/column or image menus, removing slash commands, or implementing insertion above.

## Decisions

### Give root insertion shortcuts their own menu identity

Add explicit root Insert Divider Below / Insert Table Below items to the shared whole-block operations group immediately after Add Below. Remove the two conversion items from both relevant root catalogs. Use existing compact buttons, icons, viewport scrolling, disabled-state handling and captured-target dispatch. Reuse scoped labels through `ContextMenuMsg` in every language. Keep ordinary Insert as its final root item and preserve its complete compact insertion catalog.

Keep whole-block Copy/Cut in their own compact clipboard group so Duplicate, Move Up/Down and the three insertion controls share one six-button operations row. Include the captured block type in the grip menu heading, preserving table identity after its old conversion/current-type button is removed.

Changing only the tooltip would retain destructive behavior; exposing only a flyout would remove the direct shortcut requested for table grips. Explicit root identities keep their operation grouping separate from the existing flyout and avoid duplicate element ids while it is open.

### Reuse checked source insertion and protect the transformation boundary

Route both new shortcuts to the same insertion branch used by Insert. Capture and validate document identity/version, complete source-unit range and blank-line offset exactly as before. Preserve table column-width comments and nested-unit boundaries through the existing source-unit helper. Reject Divider/Table transformation of a non-empty body at `transform_block`, so future direct callers cannot discard content accidentally. Slash query replacement continues through its separate exact-query path.

Data flow:

`captured menu target -> shared item availability -> checked insertion helper -> exact BlockEdit -> canonical mutation/history -> updated selection and per-version derived caches`

Menu lifecycle changes only ephemeral presentation state. No new Markdown derivation, highlighting or text-handle construction happens while opening, hovering, navigating, scrolling or canceling.

### A divider includes a blank continuation paragraph

Extend block insertion (below a source unit or at an empty paragraph) to reserve a source-backed blank paragraph below an inserted divider. Use the document's LF/CRLF sequence and enough separators to prevent the caret from entering the following block or a GFM table. Table insertion retains its first-cell caret. This applies equally to the existing Insert flyout's divider command, while slash query semantics remain unchanged.

Placing the caret at the end of `---` would expose the marker to accidental text edits; placing it directly before the next paragraph would unexpectedly edit that paragraph. The continuation line and divider are one mutation, so Undo/Redo remain atomic and preserve the previous selection, including structured grip selection.

## Risks / Trade-offs

- [An extra shortcut wraps an operations row] -> Use shared row classification and keyboard scroll calculation; test real pointer and keyboard activation in normal and short viewports.
- [Continuation whitespace consumes or merges a neighbor] -> Assert exact bytes and parsed block boundaries for LF/CRLF, EOF, pipe-less following prose and column-width metadata.
- [Catalog changes invalidate old transform tests] -> Update their assertions to the new insertion contract while retaining unrelated heading, table-cell, flyout, slash, and selection coverage.
- [Concurrent compact-flyout work shares menu files] -> Preserve the existing diff and validate both behavior sets together.

## Migration Plan

No persisted data migration or dependency change is required. Apply mutation safeguards and source-edit tests, then menu/localization/dispatch changes and rendered regressions. Run formatting, focused block-edit/menu/grip tests and the root test suite, plus strict OpenSpec validation. Record whether live manual inspection is available. Leave this change unarchived for review; stable specs are not edited during implementation.
