## Context

`MarkdownDocument::derive_preview_and_outline` is a single `pulldown-cmark` event pass. It keeps one eager `ListItemDraft`, a top-level `blocks` vector, and the active blockquote's `quote_children`. A parent list draft can still be open when an inner list item starts. The current implementation flushes that old draft according to the *current* `quote_depth`, so an item opened before a blockquote is incorrectly appended to `quote_children` after the parser enters the quote.

For `- outer\n\n  > - inner`, the quote consequently owns an outer item whose source starts before the quote group. `quoted_leaf_source_range` raises that start to the quote start while retaining an end before the quote start, producing a reversed byte range. `block_prefix` indexes the canonical string with it; the Rust panic unwinds into the GPUI Win32 callback, where the FFI boundary aborts the process and Windows reports `0xC0000409`.

The stable specifications require CommonMark nesting, document-ordered disjoint blocks, exact UTF-8 Visual Edit ranges, source-backed fallbacks when proof fails, and per-version cached derivation.

Current faulty data flow:

```text
outer Item start (document)
  -> BlockQuote start
  -> inner Item start (quote)
  -> flush outer draft using current quote_depth
  -> outer item becomes quote child
  -> reversed quoted visual range
  -> Rust string slice panic in GPUI callback
```

Target data flow:

```text
Item start -> draft captures destination + nested boundary state
  -> later Item/ItemEnd flushes draft to captured destination
  -> list-nested blockquote truncates the parent item at quote start
  -> ordered preview blocks with valid ranges
  -> Visual Edit validates every leaf range before slicing
  -> invalid future derivation becomes source-backed gap coverage
  -> result is cached once per document version as before
```

## Goals / Non-Goals

**Goals:**

- Preserve the container destination selected when a list item starts.
- Keep nested-block truncation metadata owned by the corresponding list draft.
- Treat a blockquote nested in a list item like the already-supported nested code, table, and HTML blocks for document-order range partitioning.
- Ensure Visual Edit never indexes canonical text with reversed, out-of-bounds, or non-UTF-8-boundary leaf ranges.
- Cover the minimal topology and realistic UTF-8/CRLF/sibling variants with pure deterministic tests.

**Non-Goals:**

- Replace the streaming preview model with a general recursive Markdown DOM.
- Change rendering, editing semantics, persisted Markdown, session formats, or WYSIWYG classification.
- Add render-frame parsing, new dependencies, or GPUI types to workspace member crates.

## Decisions

### 1. Capture the list item's destination in `ListItemDraft`

Add a small parser-only destination enum (`Document` or `BlockQuote`) to each item draft. A single flush helper receives both output vectors and routes the completed item by this captured destination. Both eager flush on a nested `Item` start and ordinary `ItemEnd` use that helper.

This represents authorship at item creation instead of inferring ownership from unrelated later parser state. A boolean would work for today's two destinations, but a named enum makes illegal/ambiguous routing harder and documents the ownership seam.

Alternative: flush the outer item immediately on `BlockQuote` start. Rejected because it scatters list finalization across container handlers and still leaves later nested container combinations dependent on event timing.

Alternative: introduce a full stack of recursive list drafts. Rejected for this bug because the existing eager flattening model deliberately emits nested list items as separate preview blocks; captured destination restores correct ownership without redesigning that model.

### 2. Store the earliest nested-block boundary on its owning draft

Move `item_nested_block_start` from one global parser variable into `ListItemDraft`. Code, table, HTML, and blockquote handlers update the open draft's earliest boundary. Before flush, the item truncates its source range at that boundary when it lies strictly inside the item event range.

This prevents a boundary discovered for one item from being consumed by a subsequently opened item. Recording `BlockQuote` start also fulfills the stable document-order requirement that explicitly includes blockquotes nested in list items.

Alternative: sort overlapping blocks after derivation. Rejected because sorting changes order but cannot make two source owners disjoint.

### 3. Validate derived leaves before any Visual Edit string slicing

At the projection boundary, accept a preview/quote/visual range only when `start <= end <= text.len()` and both endpoints are UTF-8 character boundaries. Invalid leaves are omitted from semantic projection; the existing coverage loop then represents their canonical bytes through the ordinary `gap_block` source-backed fallback. Valid overlapping ranges retain the existing unsupported-overlap behavior.

This is a safety backstop, not the semantic fix. It avoids clamping a malformed range into a plausible but incorrectly owned visual row and preserves all source bytes through conservative fallback.

Alternative: change only `block_prefix` to use `str::get`. Rejected because other projection helpers also slice ranges and a late `None` would not restore complete source coverage.

Alternative: saturating-swap or clamp inverted endpoints. Rejected because that invents ownership and can hide parser corruption.

### 4. Test ownership and the FFI-crash seam independently

Parser tests assert the outer item remains top-level, the inner item remains a quote child, block ranges are in source order, and the outer item ends no later than the nested quote. Visual tests call `build_visual_blocks` on the minimal and UTF-8 variants and assert complete, monotonic, valid coverage with no panic. A deliberately malformed preview range verifies the fallback independently of the parser fix.

The original user document is used only for an end-to-end startup verification; private document contents are not copied into repository fixtures.

## Risks / Trade-offs

- [The eager single-item model may have another untested container transition] → Centralize all flushes through the captured-destination helper and add sibling, CRLF, UTF-8, ordered, task, and nested-depth variants.
- [Skipping an invalid leaf could hide rendered semantics] → Preserve every canonical byte through the existing source-backed gap fallback and keep strict invariant tests so normal parser output never takes that path.
- [Adding blockquote as a nested-block boundary could shorten a list item too aggressively] → Apply the boundary only when an item is open and the boundary lies strictly inside that item's pulldown range; retain regressions for plain and quoted list topologies.
- [Parser changes could invalidate incremental caches] → Keep changes inside the existing full derivation result and run incremental/full-equivalence plus workspace tests; no cache key or version behavior changes.

## Migration Plan

No data migration is required. Implement parser ownership first, then the projection guard, and validate against focused and workspace suites. Rollback consists of reverting the internal parser/projection change; Markdown and persisted application data are unchanged.

## Open Questions

None required before implementation. A general recursive container stack remains a separate architectural change if future CommonMark combinations outgrow the current eager flattened model.
