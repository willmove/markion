## Context

Visual Edit paints every proven GFM table through `visual_table_view` in `src/app/preview.rs`. The bordered chrome always includes a header row: localized `Msg::LabelTable` plus six compact row/column buttons (`VISUAL_TABLE_TOOLBAR_ACTIONS`). Split Preview and Read mode already omit that header (`table_toolbar_actions_for_view_mode` returns empty outside Visual Edit).

Row/column actions resolve through `visual_table_toolbar_target` and `apply_table_edit_at`. Whole-table delete already exists on the block context menu via `delete_block` / `delete_visual_block`; it is not on the table header. Collapsible math/diagram chrome already uses tab-local hover (`hovered_visual_source_block`) that does not bump document version or derived caches.

### Data flow

```text
pointer enter/leave table container  -> tab-local hovered VisualBlockId  -> notify (no version bump)
caret in a cell of this table        -> visual_table_toolbar_target is Some
                                      -> header visible iff hover OR caret-owned
row/column button                    -> existing revalidate + apply_table_edit_at
delete-table button                  -> BlockTarget::from_block -> delete_visual_block
                                      -> one BlockEdit + after_document_changed
```

Hover and caret-owned visibility are presentation-only. They MUST NOT recompute per-document-version preview/visual/outline Arc caches. Successful delete is a normal canonical source mutation and invalidates those caches once through the existing path.

## Goals / Non-Goals

**Goals:**

- Hide the Visual Edit table-editing header until the user is interacting with that table (pointer over its chrome, or caret in one of its cells).
- Add a whole-table delete control on that same header that reuses exact block delete (one undo, existing status/error reporting).
- Keep row/column targeting, disabled-boundary semantics, compact button metrics, and preview/read toolbar absence unchanged.
- Cover idle-hidden, hover-shown, caret-shown, multi-table isolation, and delete/undo in tests.

**Non-Goals:**

- Overlay-only chrome that never shifts layout, drag handles, or a confirmation dialog.
- Localizing the existing compact `+Row` / `-Row` / `Up` / `Down` / `+Col` / `-Col` labels.
- Extending `TableEdit` with a seventh structural variant, or adding a Format-menu / shortcut command for delete-table.
- Restoring table-editing chrome in Split Preview or Read mode.
- Changing nested-table / ambiguous-ownership delete rules beyond what `delete_block` already enforces.

## Decisions

### 1. Visibility is hover OR caret ownership, per table

Show the header when `hovered_visual_table_block == this block id` or `visual_table_toolbar_target` is `Some` for this block. Hide it otherwise.

Rationale: the request is “click inside or linger over the table.” Caret-in-cell covers click/keyboard focus so the bar stays up while editing after the pointer leaves. Hover covers browse-without-focus. Each table decides independently, matching current per-table toolbar targeting.

Alternative considered: hover-only. Rejected because moving the pointer to type would hide the bar, and click-to-edit would not keep it visible. Alternative: caret-only. Rejected because lingering over a table without focusing a cell would not reveal controls.

### 2. Tab-local hover, modeled on collapsible source chrome

Add `hovered_visual_table_block: Option<VisualBlockId>` on the editor tab (or reuse a shared hovered-block field if wiring both through one helper is smaller). `on_hover` on the table container updates it and `cx.notify()`. Clear it on mode/tab changes the same way `hovered_visual_source_block` is cleared.

Rationale: presentation-only, no `MarkdownDocument` version change. Reusing the math/diagram hover field would conflate unrelated chrome.

Alternative considered: derive hover purely from GPUI style `.hover()` without tab state. Rejected because caret-owned visibility still needs an explicit show path, and tests need a stable, inspectable condition.

### 3. Omit the in-flow header from the tree when hidden

When hidden, do not reserve header height; the table is a bordered grid like preview. When shown, insert the existing header as the first in-flow child so hovering the buttons remains hovering the table.

Rationale: idle tables should not keep an empty strip. Overlaying the header on the first grid row would hide cell content; floating it into the previous block’s margin would steal hits from neighboring prose.

Trade-off: showing the header grows the table by one chrome row. Accepted; overlay can be a follow-up if the shift proves noisy.

### 4. Whole-table delete reuses `delete_visual_block`, not `TableEdit`

Append a seventh control after `-Col`. It builds `BlockTarget::from_block(version, block)`, revalidates version and `VisualBlockId` on activation, then calls `delete_visual_block`. Availability is `block_can_reorder_at` (same gate as context-menu delete). Disabled when nested/ambiguous. Success uses the existing `P1Msg::BlockDeleted` status and one history entry.

Rationale: `TableEdit` rewrites pipe-table internals and returns a surviving cell selection; deleting the table is a block-source-unit removal. Duplicating that in `edit_table_at` would fork whitespace/separator handling.

Alternative considered: `TableEdit::DeleteTable` that replaces `table_range` with empty. Rejected because `delete_block` already owns exact source-unit and separator whitespace.

The new label is a user-visible string and MUST go through `src/i18n.rs` (compact, distinct from `-Row` / `-Col`). Existing six labels stay hardcoded compact English to keep this change scoped. Debug selector: `visual-table-delete-table` / `-disabled`.

### 5. Hovering a table does not enable another table’s row/column edits

Row/column buttons still require a caret-owned target in that table. A hovered table whose caret is elsewhere shows chrome with those controls disabled; its delete control may still be enabled if `delete_block` would succeed for that block. Activating delete on table B must not mutate table A.

## Risks / Trade-offs

- [Risk] In-flow header show/hide shifts later Visual Edit rows. → Mitigation: omit chrome only when idle; keep grid cell metrics unchanged; accept overlay as a follow-up if tests or review flag the jump.
- [Risk] Pointer moving from grid to header briefly reports `hovered=false` and hides the bar before the next enter. → Mitigation: attach `on_hover` to the outer table container that includes both header and grid; do not attach leave-to-hide on inner cells alone.
- [Risk] Existing rendered tests query `visual-table-add-row` without hover. → Mitigation: place the caret in a cell (already true for targeting tests) or drive hover in idle-visibility tests; update the six-action length assertion to include delete.
- [Risk] Accidental whole-table delete from hover-then-click. → Mitigation: place delete last, reuse the same immediate-delete-plus-undo contract as the block menu; no extra dialog in this change.
- [Risk] Nested list tables: `delete_block` returns `Unsupported`. → Mitigation: disable the control; do not guess a source range.

## Migration Plan

No persisted data or schema migration. Roll forward by shipping the UI + tests; roll back by reverting the change folder and `preview.rs` / tab-state / i18n / test edits. Documents are unaffected.

## Open Questions

None. Overlay-vs-in-flow is decided (in-flow, omit when hidden). Confirmation dialog is out of scope.
