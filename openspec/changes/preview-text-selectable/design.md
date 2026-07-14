## Context

The preview pane (`preview_block_view` in `src/main.rs`) renders Markdown as GPUI `StyledText` / `InteractiveText` / plain string children inside a virtualized `ListState`. Those text elements paint glyphs but do not expose drag-selection or clipboard integration. `MarkionApp::copy` only reads the source editor's `selected_range`, so even if the user could highlight preview text, Copy would ignore it.

GPUI 0.2's `TextLayout` already provides `index_for_position` / `position_for_index` (used by `InteractiveText` for link clicks). There is no stock "selectable label" element in this GPUI version, so selection must be owned by the app.

## Goals / Non-Goals

**Goals:**

- Let users drag-select rendered preview text in Split Preview and Read modes.
- Copy the selected plain text to the system clipboard via Edit→Copy / the existing Copy shortcut when a preview selection is active.
- Keep the preview non-editable: selection/copy never mutates document text, dirty flag, or undo.
- Preserve link clicks, scrolling, list virtualization/splice, and per-version derived-state caches.

**Non-Goals:**

- Editing, cut, or paste-into-document from the preview.
- Rich clipboard formats (HTML/RTF/Markdown source).
- Perfect cross-block contiguous selection spanning many list items in one gesture if that requires a heavyweight document-wide text model (v1 may be per-text-run / per-block, with a clear upgrade path).
- Changing parse output, highlight memoization, or preview debounce.

## Decisions

### 1. App-owned preview selection state (not a new GPUI element crate)

**Choice:** Keep a small selection model on `MarkionApp` / the active tab, e.g.:

```text
PreviewSelection {
  block_index: usize,
  // identity of the selectable text run within the block (paragraph body, code body, cell, …)
  run_id: PreviewTextRunId,
  range: Range<usize>, // byte offsets into that run's plain string
}
```

Mouse down/move/up on selectable preview text updates this state; paint draws a highlight quad over the selected glyph range using `TextLayout::position_for_index` (same pattern as the source editor's selection quads).

**Why not** wrapping every string in a custom `Element` crate: orphan-rule / workspace rules keep GPUI types in the root crate; an in-app helper around `InteractiveText` + layout hit-testing is enough.

**Alternatives considered:**

| Approach | Pros | Cons |
|---|---|---|
| Platform native selectable widget | True OS selection | Not available for GPUI `StyledText` |
| Invisible `InputHandler` / fake editor over preview | Reuses editor copy path | High risk of edit mutations; fights Read-mode non-edit guarantee |
| **App selection + highlight paint** | Fits current architecture; cache-safe | Must implement drag + copy routing ourselves |

### 2. Build on `InteractiveText` hit-testing for rich runs; extend for code/plain runs

`rich_text_element` already wraps linked text in `InteractiveText`, which resolves character indices via `TextLayout::index_for_position`. Extend that path (or a thin `selectable_preview_text` helper) so:

- Mouse drag updates `PreviewSelection` for that element id / run.
- Click without drag on a link still opens the URL (click = down+up in same link range with empty/near-empty selection), matching current behavior.
- Code blocks (`StyledText` / per-line `StyledText`), math source lines, table cells, and image captions get the same helper even when they are not link-interactive.

### 3. Copy routing: preview selection wins when present

Extend `MarkionApp::copy`:

1. If `preview_selection` is non-empty → write that plain substring to the clipboard; status = copied.
2. Else fall through to the existing editor `selected_range` path.

Cut/paste/typing remain editor-only. In Read mode, Copy with a preview selection works; Copy with no preview selection keeps the existing "nothing to copy" (or editor selection if Split and the editor still has one — prefer the most recently interacted surface; document the rule as: non-empty preview selection takes precedence).

Clear preview selection when:

- the user starts a new selection in the source editor, or
- preview blocks for that tab are fully reset / the selected block index becomes invalid after a splice that removes it, or
- the active tab changes.

Selection updates MUST NOT bump document version or touch derived caches.

### 4. Selection scope for v1: per text run (within one preview list item)

**Choice:** v1 selects within a single selectable text run (one heading/paragraph/list-item body, one code block body, one table cell, etc.). Dragging outside that run ends or clamps the selection at the run boundary.

**Why:** Preview content is a virtualized list of heterogeneous blocks, not one shaped buffer. Cross-block selection needs a flattened plain-text index space and multi-element highlight painting — valuable later, but not required to unblock "copy this paragraph / code fence."

Decorative chrome (list markers, code line numbers, table edit buttons) is excluded from the selectable string so copied text stays clean.

### 5. Data flow (selection does not touch derived Markdown state)

```text
pointer down on preview text run
  -> resolve (block_index, run_id, char_index) via TextLayout
  -> set PreviewSelection anchor
pointer move (buttons down)
  -> update PreviewSelection head; cx.notify() for highlight repaint only
pointer up
  -> finalize range; link click only if range empty/near-empty and hit a link

Copy action
  -> if PreviewSelection non-empty: clipboard ← run_plain[range]
  -> else: existing editor copy

document edit / preview splice
  -> invalidate PreviewSelection if block_index/run no longer valid
  -> never reparse because of selection alone
```

### 6. Testing strategy

- Pure helpers: clamp/normalize selection ranges; plain-text extraction for a run; "preview selection takes copy precedence" decision helper; invalidation when block index is out of range.
- No full GPUI integration test required (consistent with the rest of the suite). Manual check: Split + Read, select paragraph/code, Ctrl/Cmd+C, paste elsewhere; confirm Read mode still cannot edit; confirm link click still works.

## Risks / Trade-offs

- **[Risk] Link click vs drag-select conflict** → Treat as click only when the selection range is empty or below a small index epsilon; otherwise prefer selection and skip `open_url`.
- **[Risk] Virtualized list recycles items; stale element state** → Store selection in tab/app state keyed by `block_index` + `run_id`, not only in ephemeral element local state; clear on invalid indices after splice.
- **[Risk] Per-run selection feels limited for multi-paragraph copy** → Accept for v1; document as non-goal; users can still copy from the source editor for large spans.
- **[Risk] Highlight painting cost** → Paint only the active selection's quads; do not rebuild `preview_list` or re-highlight code on selection change.
- **[Trade-off] Plain text only on clipboard** → Simplest correct behavior; matches "copy the words I see" for most cases (code fences copy code text without fences unless we later choose otherwise — v1 copies the visible code body).

## Migration Plan

- No config, file-format, or preference migration.
- Behavior is additive; rollback is revert of the change.

## Open Questions

- None blocking proposal/apply. If during implementation GPUI hit-testing on nested list rows proves unreliable, fall back to a coarser "click block → select all text in block + Copy" affordance for that block type and note it in the task notes — still satisfies the user-visible goal of getting preview text onto the clipboard.
