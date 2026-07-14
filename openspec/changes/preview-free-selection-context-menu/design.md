## Context

`preview-text-selectable` added app-owned preview selection, but only within a single text run (`block_index` + `PreviewTextRunId` + offsets into that run's plain string). Users cannot select across a heading and the following paragraph, or across multiple list items. Copy is plain-text-only via the global Copy action; there is no preview context menu.

The preview remains a virtualized `ListState` of `PreviewBlock`s. GPUI still has no stock multi-element selectable document, so selection stays app-owned. The file-tree already has a right-click menu pattern (`FileTreeContextMenu` + absolute overlay + `.occlude()`) to reuse.

Only `PreviewBlock::Table` currently carries a `source_range`. Free-range **Copy as Markdown** needs source mapping for other blocks too.

## Goals / Non-Goals

**Goals:**

- Contiguous multi-block drag selection in document order across textual preview content.
- Highlight painted on every covered run (partial first/last, full middle).
- Copy shortcut / Edit→Copy still copies plain text for the free-range selection.
- Preview right-click menu: Copy as Plain Text, Copy as Markdown, Copy as HTML; plus Select All and Copy Link Address when applicable.
- Localized menu/status strings; preview stays non-editable; no derived-cache invalidation from selection alone.

**Non-Goals:**

- Editing, cut, or paste-into-document from the preview.
- Selecting decorative chrome (list markers, code line numbers, table toolbar buttons) as first-class content.
- Bidirectional selection sync with the source editor caret.
- Rich clipboard with multiple MIME types in one paste (v1 writes one format per menu action).
- Perfect HTML fidelity for partial inline spans inside a block (block-granular HTML for middle blocks is acceptable; first/last may be plain-text-trimmed or whole-block HTML — see Decisions).

## Decisions

### 1. Selection model: ordered anchor/head carets across runs

Replace the single-run `PreviewSelection` with:

```text
PreviewCaret {
  block_index: usize,
  run_id: PreviewTextRunId,
  offset: usize,  // UTF-8 byte offset into that run's plain text
}

PreviewSelection {
  anchor: PreviewCaret,
  head: PreviewCaret,
}
```

Document order is `(block_index, run_rank(run_id), offset)` where `run_rank` is a stable ordering of runs inside a block (Body < CodeBody < CodeLine(i) < …).

**Normalize** so `start = min(anchor, head)`, `end = max(anchor, head)` in that order.

**Plain-text extract:** walk runs from `start` to `end` in document order; take a suffix of the first run, full middle runs, prefix of the last; join with `\n` between block boundaries (and within a code block keep internal newlines from the run text).

**Why not** a single flattened string of the whole preview: would duplicate large documents and fight virtualization. Walking blocks on demand is enough for copy and highlight.

### 2. Hit-testing and drag across list items

Keep per-run `SelectablePreviewText` for local index mapping via `TextLayout`, but on mouse move/up:

- If the pointer is over another run, resolve that run's caret and update `head` (do not clamp to the anchor run).
- If the pointer is in preview chrome / gutter between items, keep the last resolved textual caret or clamp to the nearest run edge in the hovered block.

Register a preview-pane-level mouse-move handler (in addition to per-run handlers) so drags that leave the original hitbox still update `head` when another run is hovered. Pattern: `MarkionApp` stores `preview_is_selecting`; any selectable run that sees a drag while that flag is set may update `head` for its own caret.

Auto-scroll while dragging near the top/bottom of the preview viewport is desirable; implement if the existing `ListState` scroll APIs make it cheap, otherwise defer (document as follow-up).

### 3. Highlight paint across many runs

Each `SelectablePreviewText` paints a highlight if its `(block_index, run_id)` intersects `[start, end]`:

- Fully inside → highlight entire run text.
- Partial first/last → highlight the intersecting byte range.
- Outside → no highlight.

No `preview_list` rebuild; only `cx.notify()` for repaint. Same invariant as v1.

### 4. Source ranges for Copy as Markdown

**Choice:** During `derive_preview_and_outline` / preview parse, attach a `source_range: Range<usize>` (byte offsets into the document) to every `PreviewBlock` variant that represents document content (extend the model; tables already have it). Cache stays per document version via the existing `Arc<Vec<PreviewBlock>>`.

**Copy as Markdown:**

1. Map selection carets → covered block indices (and for partial first/last, prefer slicing the block's `source_range` when the selection covers only part of the block's plain text — approximate by proportional slice or by including the whole block source when partial inline mapping is ambiguous).
2. Prefer **whole-block source slices** for any block that is not the sole selected block, and for the sole block use the best-effort substring of `source_range` when the selection is a proper subset of that block's plain text; if mapping is unreliable, fall back to whole-block source for that block.
3. Concatenate the sliced source in document order (preserve original Markdown, including markers).

**Alternatives considered:** Reconstruct Markdown from `PreviewBlock` AST (lossy for links/images/tables). Rejected for Copy as Markdown fidelity.

### 5. Copy as HTML

**Choice:** Build a temporary Markdown string from the same source slice used for Copy as Markdown, then run the existing HTML fragment path (`MarkdownDocument::from_text(slice).render_html_fragment()` or equivalent). One code path, consistent with export.

Partial-block HTML then matches whatever Markdown slice we chose (whole-block fallback included).

### 6. Context menu UI

Mirror `FileTreeContextMenu`:

```text
PreviewContextMenu { position: Point<Pixels>, link_url: Option<String> }
```

- Right-click on preview (Split/Read): open menu at pointer; if click hits a link run, stash `link_url`.
- If there is no selection, right-click may still open the menu with Select All / Copy Link Address enabled as appropriate; copy-format items disabled until a non-empty selection exists (or right-click on a word could select-word first — optional; v1: require existing selection for copy items, except Copy Link Address).
- **Decision for empty selection:** Copy as * items disabled; Select All enabled; Copy Link Address enabled when `link_url` is set.
- Dismiss on outside click / Escape, same as file-tree menu (`.occlude()` on the menu panel).
- Items call into `MarkionApp` methods that write `ClipboardItem::new_string(...)` and set status via i18n.

### 7. Select All / Copy Link Address

- **Select All:** set anchor to first textual run at offset 0, head to last textual run at its text len.
- **Copy Link Address:** clipboard ← `link_url` from the menu open event (not from selection), when present.

### 8. Data flow (selection does not touch derived caches)

```text
pointer down on run A
  -> PreviewSelection { anchor = head = caret(A) }; preview_is_selecting = true
pointer move / up over run B
  -> head = caret(B); notify (highlight only)
Copy / menu Copy as Plain Text
  -> plain = extract_plain(selection); clipboard
Copy as Markdown / HTML
  -> md = extract_source_markdown(selection, blocks, document)
  -> clipboard md  OR  html = render_html_fragment(md)
document edit / splice
  -> invalidate selection if carets' block indices invalid (same as v1)
```

### 9. Testing

- Pure helpers: caret ordering, range normalize, plain extract across 2–3 blocks, source-slice join, menu enablement predicates.
- Unit tests for `source_range` presence on common block types after parse.
- No full GPUI integration test required.

## Risks / Trade-offs

- **[Risk] Partial-block Markdown/HTML is imperfect without inline source maps** → Prefer whole-block source for ambiguous partials; document in status if needed; plain-text copy remains precise to the glyph selection.
- **[Risk] Drag across virtualized off-screen items** → Head updates only for mounted/hit-tested runs; auto-scroll follow-up if needed. Selecting “through” unloaded rows may stall at the last visible run until scroll reveals more.
- **[Risk] Adding `source_range` to all blocks grows parse output** → One `Range<usize>` per block is cheap; still Arc-shared per version; no per-keystroke recompute beyond existing parse.
- **[Risk] Context menu vs link click / selection** → Right-button does not start a text drag; left-button selection unchanged; link open remains left-click with empty selection.
- **[Trade-off] One clipboard format per action** → Simpler than multi-MIME; matches user-requested menu items.

## Migration Plan

- No preferences or file-format migration.
- Additive UI; rollback is revert.

## Open Questions

- None blocking. Auto-scroll during drag is a polish item: implement in the same change if straightforward, otherwise leave a task note and ship without it.
