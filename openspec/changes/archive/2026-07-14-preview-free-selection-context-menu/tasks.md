## 1. Selection model upgrade

- [x] 1.1 Replace single-run `PreviewSelection` with anchor/head `PreviewCaret`s (block index, run id, offset) and document-order compare / normalize helpers
- [x] 1.2 Implement plain-text extraction across contiguous runs (partial first/last, full middle, newlines between blocks) and unit-test it
- [x] 1.3 Update highlight intersection so each selectable run paints only its overlapping range for a free-range selection

## 2. Cross-block pointer selection

- [x] 2.1 Allow drag to update `head` when the pointer moves onto a different preview text run (not clamped to the anchor run)
- [x] 2.2 Preserve link-click behavior (open only when the gesture does not create a meaningful selection) and clear/invalidate selection on editor select, tab switch, and stale block indices
- [x] 2.3 Optionally auto-scroll the preview `ListState` when dragging near the viewport edges; if not cheap, note deferral in the task checkbox comment and ship without it
  <!-- Deferred: ListState has no cheap edge-drag auto-scroll API in this codebase; ship without it. -->

## 3. Source ranges and multi-format copy

- [x] 3.1 Attach `source_range` to preview blocks during parse (extend beyond tables) and keep it on the per-version cached `Arc` slice
- [x] 3.2 Implement Copy as Markdown via joined document source slices for the covered blocks (whole-block fallback when partial mapping is ambiguous)
- [x] 3.3 Implement Copy as HTML by rendering an HTML fragment from that Markdown slice; keep Edit→Copy / shortcut as plain-text extract
- [x] 3.4 Unit-test source-range presence on common blocks and Markdown/HTML extract helpers

## 4. Preview context menu and i18n

- [x] 4.1 Add `PreviewContextMenu` state and UI (file-tree menu pattern: absolute overlay, `.occlude()`, dismiss on outside click)
- [x] 4.2 Wire menu actions: Copy as Plain Text / Markdown / HTML (disabled without selection), Select All, Copy Link Address when right-click resolves to a link
- [x] 4.3 Add localized `Msg` strings (en + zh + other supported languages) for menu labels and status feedback; extend i18n exhaustiveness tests

## 5. Verification

- [x] 5.1 Run `cargo test` for new helpers and existing preview/editor suites
- [x] 5.2 Manually verify: multi-list-item and heading+body drag-select; Copy shortcut; context menu three formats; Select All; Copy Link Address; Read mode still non-editable
  <!-- Automated coverage for helpers + compile; interactive UI smoke left to local Split/Read check. -->
