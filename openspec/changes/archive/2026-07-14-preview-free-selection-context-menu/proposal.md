## Why

Preview selection shipped in `preview-text-selectable` is limited to a single text run (one list item, one paragraph, one heading, etc.). Users cannot drag across a heading plus its body, or select several list items together, which feels broken next to mainstream Markdown editors. They also lack a preview-local way to copy the selection as Markdown or HTML—only plain text via the global Copy shortcut. Free-range selection plus a right-click copy menu closes that gap while keeping the preview non-editable.

Non-goals: this change does not make the preview editable (no cut/paste-into-document, no typing), does not add WYSIWYG editing, does not require pixel-perfect selection of decorative chrome (list bullets, code line numbers, table edit buttons), and does not redesign the source-editor selection model.

## What Changes

- Upgrade preview selection from per-text-run to contiguous multi-block selection: a single drag can span headings, paragraphs, list items, quotes, code, tables, and other textual preview content in document order, with highlight painted across all covered runs.
- Keep Edit→Copy / the Copy shortcut copying the selection as plain text (existing behavior), now over the free-range selection.
- Add a right-click context menu on the preview pane (when the preview is visible) with at least:
  - **Copy as Plain Text** (same payload as Copy)
  - **Copy as Markdown** (source Markdown for the selected region, reconstructed from document source ranges / block mapping)
  - **Copy as HTML** (an HTML fragment for the selected region)
- Recommended additional menu items (include if low-cost):
  - **Select All** (select all textual preview content for the active document)
  - **Copy Link Address** (enabled only when the click/selection resolves to a single link URL)
- Menu labels and status feedback are localized via `ui-i18n`.
- Selection and all copy actions MUST NOT mutate document text, dirty flag, undo history, or derived Markdown caches.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Extends the preview selection/copy requirement from per-run selection to free-range multi-block selection, and adds a preview context menu with multi-format copy actions.
- `ui-i18n`: Adds localized strings for the preview context menu items and related status feedback.

## Impact

- Affected code: `src/main.rs` (preview selection model, `SelectablePreviewText` / hit-testing, highlight paint across blocks, context menu UI patterned after the file-tree context menu, Copy routing), possibly small helpers in `src/lib.rs` / parse/export paths for Markdown/HTML fragment extraction from a block range; `src/i18n.rs` for new `Msg` variants.
- Reuses existing HTML/Markdown export building blocks where practical; no new crate dependencies expected.
- Invariants to preserve: derived Markdown state stays cached per document version and shared via `Arc`; syntax highlighting stays memoized; preview `ListState` virtualization/splice must not full-reset on every selection change; Read mode remains non-editable.
