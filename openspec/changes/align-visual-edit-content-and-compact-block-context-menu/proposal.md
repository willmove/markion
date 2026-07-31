## Why

Visual Edit currently reserves a fixed leading gutter for block controls on transformable paragraphs, headings, lists, and other prose blocks, even when those controls are not visible. This shifts their content and wrapping away from the Read-mode document axis while non-transformable media such as images and formulas remain unshifted, producing an inconsistent page silhouette and unnecessary editing chrome.

## What Changes

- Remove permanent flow-space reservation for Visual Edit block controls so equivalent top-level prose and media share the same document content axis and available line width as their Read-mode presentation.
- Make a compact right-click block context menu the primary pointer entry for exact block transforms, duplicate, move, and delete operations; provide the same menu through the keyboard context-menu path so right-click is not the only accessible entry.
- Retain source-safe drag reordering through a hover/focus-only grip positioned outside normal content flow, so its appearance never shifts text or changes wrapping.
- Reorganize the existing flat block menu into compact grouped and hierarchical sections with a current-type indicator, related transform groups, non-destructive block actions, and a separated destructive delete action.
- Define precedence for existing text selection, link/media-specific interactions, pointer targeting, stale-target rejection, overlay dismissal, and localized menu labels.
- Add rendered GPUI regression evidence for content-axis parity, unchanged line wrapping, right-click/keyboard targeting, compact menu reachability, drag behavior, and presentation-only menu lifecycle.
- **Non-goals:** changing Markdown serialization or exact block-operation semantics, redesigning the selection formatting toolbar or Read-mode preview context menu, adding a parallel document model, or changing Visual Edit support classifications for Markdown constructs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Require flow-neutral Visual Edit block chrome, consistent content alignment, and a compact right-click/keyboard block context menu while preserving exact source ownership, one-mutation/one-undo behavior, and derived-cache invariants.

## Impact

- Affected UI seams include Visual Edit row composition and block chrome in `src/app/preview.rs`, root contextual-overlay composition in `src/app/root_view.rs`, block-menu state and invocation in `src/app/mod.rs` and `src/app/editing.rs`, keyboard action routing, localization in `src/i18n.rs`, and rendered GPUI tests in `src/app/tests.rs`.
- Existing validated `BlockTarget`, block transform/reorder helpers, canonical `MarkdownDocument.text`, tab-local history, dirty/autosave/recovery paths, per-document-version derived `Arc` caches, memoized highlighting, and cached text handles remain unchanged.
- No external dependency, persistence migration, public API change, or workspace-member dependency is introduced.
