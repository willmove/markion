## Context

Visual Edit represents whitespace-only source ranges as `VisualBlockKind::Whitespace` rows. This preserves complete source-offset coverage and gives a caret somewhere to render when structural Enter, keyboard movement, or reveal logic moves the selection into an empty line. The renderer currently assigns every such row an I-beam cursor and mouse-down handler, so ordinary inter-block spacing also behaves like an editor surface before the caret has entered it.

The data flow is:

`Document text/version -> cached visual blocks -> current source selection -> rendered row interactivity`

The first two stages remain unchanged. Only the final rendering decision will distinguish passive whitespace from the whitespace block that owns the current caret.

## Goals / Non-Goals

**Goals:**

- Prevent pointer clicks on passive whitespace between rendered blocks from moving the source caret.
- Keep whitespace editable once its source range owns the caret, and keep a heading-created insertion line editable even when the parser retains its newline in the heading block.
- Preserve exact source coverage, structural Enter behavior, reveal behavior, and derived-state cache identity.
- Cover heading-to-heading and heading-to-paragraph gaps with interaction tests.

**Non-Goals:**

- Remove whitespace blocks from the visual model.
- Change Markdown parsing, visual spacing, source-mode editing, or block-level Enter semantics.
- Add a persistent activation flag to document or cached derived state.

## Decisions

### Derive whitespace interactivity from caret ownership

The renderer will use the existing source-range caret ownership check for each whitespace block. A passive row remains a layout element with its current height, but has no I-beam cursor or mouse-down handler. When that row owns the caret, it retains the visible caret and editing affordance.

This avoids a second source of truth. An alternative was to store a mutable `active` flag on whitespace blocks, but that would couple ephemeral selection state to cached Markdown-derived data and require invalidation on every selection movement.

### Preserve whitespace blocks and their source ranges

The visual block builder will remain unchanged. Removing gap blocks would prevent valid source offsets from mapping to a visual row and would regress keyboard navigation, search reveal, trailing whitespace, and structural Enter into an empty line.

### Test behavior through rendered pointer and keyboard paths

Regression tests will click an identifiable whitespace row and assert that passive gaps do not change selection or document state. A complementary test will press Enter from a heading, verify that the resulting source-backed insertion line owns the caret, and type through the normal Visual Edit input path. The test does not require a particular visual block kind because the Markdown parser may retain a single trailing newline in the heading source range.

## Risks / Trade-offs

- [An active multi-line whitespace range still maps pointer positioning to the range start] -> Keep the existing offset behavior in this scoped fix; precise vertical mapping inside multiple blank lines can be addressed independently.
- [Adding a stable element identifier solely for interaction tests could leak into production semantics] -> Use an identifier only as a UI inspection hook; do not use it for application state or behavior.
- [Removing the passive handler could expose an ancestor click handler] -> Exercise the real rendered tree in GPUI tests and assert selection remains unchanged.

## Migration Plan

No data migration is required. The change is confined to Visual Edit rendering and tests. Rollback consists of restoring unconditional whitespace pointer handling.

## Open Questions

None.
