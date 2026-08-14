## Context

See `proposal.md` for the motivation. The outline row handler currently captures a heading source offset and calls the existing source-navigation method. That method updates the canonical selection and cursor-driven outline highlight, prepares Visual Edit cursor reveal, and scrolls the source editor. In Read mode the source editor is hidden, while the visible preview is a virtualized GPUI list backed by a persistent `ListState` and the tab's cached `Arc<Vec<PreviewBlock>>`.

The outline headings and preview blocks are derived from the same Markdown parse when preview content is active. Both carry source positions, so a heading's outline offset can be matched to the `source_range.start` of its `PreviewBlock::Heading`. Read rendering synchronizes those blocks into the preview list before the outline's interactive element tree is built.

```text
outline row click (heading source offset)
                  |
                  v
      preserve canonical source navigation
                  |
          +-------+----------------+
          |                        |
     Read mode               all other modes
          |                        |
match cached Heading block   existing editable-surface
by source_range.start        reveal behavior
          |
scroll preview ListState to the matching item
```

## Goals / Non-Goals

**Goals:**

- Make a Read-mode outline click visibly navigate the preview.
- Preserve canonical cursor movement, active-outline highlighting, and later mode-switch position.
- Use the current tab's already-derived preview blocks and persistent list state without synchronous reparsing or a new cache.
- Leave existing navigation paths in Edit, Visual Edit, and Split Preview modes unchanged.

**Non-Goals:**

- Deriving the active outline item from manual preview scrolling.
- Changing proportional or future source-mapped Sync scroll behavior in Split Preview.
- Adding animation, preview-heading keyboard focus, or outline folding.

## Decisions

### Use an outline-specific navigation entry point

The outline click handler will call one application method that preserves the existing source-position navigation for every mode and adds preview-list navigation only when `ViewMode::Read` is active. Keeping the source cursor update in Read mode ensures the clicked row becomes active and that switching back to an editable mode retains the same document position.

An alternative was to replace source navigation with preview-only scrolling in Read mode. That would leave the active outline highlight tied to an unrelated hidden cursor and lose the user's navigated position when changing modes, so it is rejected.

### Match by exact source offset in the cached preview blocks

A pure helper will scan the current `PreviewBlock` slice for a `Heading` whose `source_range.start` equals the outline heading offset and return its list index. Exact source identity handles duplicate titles and duplicate generated anchors without relying on display text. The scan occurs only on an explicit click, so an O(number of preview blocks) lookup avoids adding or maintaining another derived index.

Matching by title or slug was rejected because both can be duplicated. Recomputing preview blocks on click was rejected because the Read render path already owns a versioned cached snapshot and the project invariant forbids unnecessary derived-state recomputation.

### Scroll to the logical list item without requiring prior measurement

Read-mode navigation will drive the persistent preview `ListState` by logical item index with zero in-item offset. GPUI can accept that target for virtualized, unmeasured rows; the heading is aligned at the top when scroll bounds permit, producing an unambiguous jump rather than merely revealing a partially visible item.

Using pixel estimates or whole-document scroll percentages was rejected because rendered block heights diverge from source layout. `scroll_to_reveal_item` was also not selected because it may leave an already-partially-visible target in place rather than presenting it as the navigation focus.

### Fail safely when the rendered target is unavailable

The navigation method will always preserve the canonical source-position update. If the current preview snapshot has no exact rendered heading match, it will not guess another block or trigger a synchronous parse; the preview list remains unchanged. This protects document/cache invariants for malformed, empty, or transiently stale content while normal Read-mode renders provide matching outline and preview data from the same version.

## Risks / Trade-offs

- [A transient or unusual heading has no matching rendered block] -> Keep the canonical navigation result, avoid guessed scrolling, and cover normal headings, front matter, formatting, and duplicate titles in lookup tests.
- [A linear block scan is noticeable in very large documents] -> The scan runs only on an outline click and allocates nothing; introduce a cached index only if profiling shows a real interaction delay.
- [Future source-mapped Sync scroll also needs offset-to-block lookup] -> Keep the helper pure and narrowly named so it can be reused or replaced without coupling this Read-only behavior to Split Preview reconciliation.

## Migration Plan

No stored data, preferences, file formats, or public APIs change. Implement the helper and mode-aware route behind the existing outline interaction, verify all four view modes, and roll back by restoring the original outline handler if necessary.
