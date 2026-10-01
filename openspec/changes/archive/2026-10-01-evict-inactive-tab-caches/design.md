## Context

`switch_active_tab` today clears only ephemeral interaction flags (`preview_is_selecting`, caret affinity, IME mark, open undo capture). Every `EditorTab` keeps:

- `MarkdownDocument` derived caches once populated
- `preview_list_blocks` / `visual_list_blocks` Arc clones
- `last_lines` / line offset & height snapshots from Edit/Split shaping
- undo/redo stacks (intentionally retained)

Derived caches are already versioned and lazy. Evicting them for inactive tabs is therefore a pure memory optimization: reactivation calls the same accessors the first visit used.

## Goals / Non-Goals

**Goals:**
- Reduce accounted per-tab memory for inactive tabs to approximately text + undo (+ small UI scalars).
- Preserve editing correctness: text, dirty, path, selection, undo/redo, and scroll fractions survive dormancy.
- Make dormancy observable via `memory_report` for harness tests.
- Keep the active tab warm (no eviction of the tab the user is looking at).

**Non-Goals:**
- Evicting process-global diagram/math/highlight/image caches (owned by `bound-preview-raster-memory` / existing caps).
- Changing Visual Edit mutation semantics or source-mapped incremental algorithm.
- Unloading document `text` from memory (session lazy-load is a later change).

## Decisions

### Evict on deactivate, with a small warm-set exception

When switching away from a tab, immediately dormant that tab's heavy caches. Policy is simple and matches user expectation that background tabs are cheap.

Alternative considered: **time-based or N-tab LRU warm set** (keep last 2–3 tabs warm). Rejected for v1 because:
- Harder to test and tune
- Immediate deactivate already recovers the bulk of multi-tab cost
- Can add a `WARM_TAB_LIMIT` later without changing the dormancy primitive

If deactivate-every-time causes measurable reactivation jank on huge docs, a follow-up can keep the previous tab warm for one generation.

### What dormancy clears

**Cleared:**
- Document: `cached_preview_blocks`, `cached_visual_blocks`, `cached_outline`, `cached_stats`, `cached_line_count`, `source_mapped_cache`, `pending_source_edits = Full`
- Tab: `last_lines`, `line_offsets`, `line_heights`, `last_bounds`, display/line-offset/measured-height caches
- Tab list mirrors: `reset` preview/visual `ListState` item counts to 0 and replace block Arcs with empty Arcs (scroll fraction fields on the tab are kept separately for sync-scroll / restore)
- Visual navigation snapshots / expanded source-block sets (ephemeral; rebuild on interaction)

**Retained:**
- `document.text`, `path`, `dirty`, `text_version` (version must NOT bump — eviction is not an edit)
- `selected_range`, `selection_reversed`
- `undo_stack` / `redo_stack` / capture state finished before switch (already cleared by `finish_undo_capture`)
- `editor_scroll` handle and sync-scroll fractions
- `last_recovery_file`, autosave generation

### Document API: `evict_derived_caches(&self)` or `&mut self`

Prefer `&mut self` on `MarkdownDocument` that clears the `RefCell`/`Cell` caches without touching `text` or `text_version`. Callers on the app side invoke it from `EditorTab::enter_dormant()`.

Critical invariant: **do not bump `text_version`**. Editor layout caches keyed on version must invalidate via explicit tab-side clears, not a fake edit.

### Reactivation

No special path: the next render of the active tab in Visual Edit calls `visual_blocks_shared()`, Split/Read calls preview debounce paths, Edit reshapes on prepaint. Scroll restoration uses retained scroll handles / fractions; list state is re-`splice`d by existing `sync_*_list` helpers when blocks return.

### Interaction with image claims

If `bound-preview-raster-memory` is implemented first or second, dormancy MUST release the tab's preview-image claims when clearing block lists (same as `reset_preview_list`). Order independence: dormancy calls the same claim-release helper tab close uses.

### Diagnostics

Harness: open 2 tabs of `plain_long` with VisualEdit warmup, switch to tab 0, assert tab 1's document derived sites report unpopulated/zero and shaped_lines is 0, while undo (if any) and text bytes remain; switch back and assert visual_blocks becomes populated again without text/version change.

## Risks / Trade-offs

**Reactivation cost on huge documents** → Accept one-time reparse on focus; same cost as first open. Optional later warm-set.

**Scroll jump after list reset** → Preserve scroll handles/fractions; re-sync lists before paint when possible; add a GPUI test that selection/cursor offsets survive a dormant round-trip.

**Forgetting to clear a new cache field** → Dormancy method lives next to `EditorTab` fields; memory test fails if a populated inactive tab still reports large `document.visual_blocks` after switch.
