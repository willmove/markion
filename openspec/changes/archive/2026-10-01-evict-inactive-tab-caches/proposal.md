## Why

Even without images, each visited tab retains full derived Markdown state (preview blocks, visual blocks, source-mapped partitions, and—if Edit/Split was used—whole-document shaped lines). Diagnostics measured ~3.9 MB of accounted per-tab state for a ~3k-line Visual Edit document, and that state is never released while the tab stays open. Inactive-tab eviction recovers that memory without changing the active editing experience: caches are already lazy and rebuild on next activation.

## What Changes

- Define an inactive-tab dormancy policy: when a tab is not active, Markion MAY drop expensive derived and layout caches while keeping canonical document text, path/dirty flags, selection, scroll fractions, and undo/redo history.
- Evict on tab deactivation and/or when the number of open tabs exceeds a threshold (whichever the design selects), always preserving the active tab's warm caches.
- Clear dormant tabs' `last_lines` / line layout snapshots, `preview_list_blocks` / `visual_list_blocks` Arc handles (resetting list state as needed), and `MarkdownDocument` derived caches (`preview`, `visual`, `outline`, `stats`, `line_count`, `source_mapped`) without bumping `text_version` or marking the document dirty.
- Ensure reactivation in Visual Edit / Split / Read / Edit rebuilds the needed caches through existing accessors with no user-visible loss of caret, selection, or scroll position beyond what virtualized lists already tolerate.
- Extend memory diagnostics / harness tests to prove per-tab accounted bytes fall after dormancy and rise again after reactivation, and that undo history remains intact.

Non-goals: image/diagram raster budgets (change `bound-preview-raster-memory`); trimming undo history; lazy session restore of file bytes; viewport-limited `shape_text` for the active editor.

## Capabilities

### New Capabilities
- `tab-memory-lifecycle`: dormancy and reactivation rules for per-tab derived Markdown and editor layout caches.

### Modified Capabilities
- (none) — workspace and markdown-editing requirements already allow lazy derived caches; this change adds an explicit lifecycle without altering editing semantics. Executable evidence lands under engineering-quality via tasks/tests rather than rewriting stable parser requirements.

## Impact

- `EditorTab` / `switch_active_tab` / close-tab paths in `src/app/state.rs`, `src/app/application.rs`, `src/app/documents.rs`.
- Optional `MarkdownDocument::evict_derived_caches()` (or equivalent) in `src/lib.rs` that clears derived `RefCell` caches without changing `text` or `text_version`.
- Memory harness tests and `docs/memory-retention.md`.
- Depends on memory accounting from `add-memory-diagnostics` for before/after assertions.
