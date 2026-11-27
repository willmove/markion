## Why

GitHub issues #107–#140 document 34 reproducible defects across editing, navigation, workspace restoration, history, preview, and built-in exports. These defects violate behavior already covered by the current specifications and can corrupt edits, hide content, lose UI state, or produce incorrect exported documents.

## What Changes

- Fix issues in ascending numeric order, with focused regression coverage for each report:
  - #107–#109: keyboard activation of Replace All, ordered-list continuation after a tab, and complete emoji-shortcode replacement.
  - #110–#114: resilient math editing/rendering, consistent reference-link resolution and HTML copying, the documented Markdown export command, and preserved multi-paragraph drag selection.
  - #115–#118: source/visual page navigation and focused rename-field clipboard/undo routing, plus reference definitions during HTML copy.
  - #119–#122: PDF image alternative text, non-breaking-space preview text, per-image drag measurements, and unfolding the block containing a match's end.
  - #123–#126: Visual Edit word selection, definition-list Find, initial Read Find reveal, and source-exact clicks in multiline table cells.
  - #127–#129: recent-workspace tab persistence/restoration and rename-aware historical diffs.
  - #130–#134: faithful code copying, DOCX ordered-list starts and nesting, and UTF-8 BOM handling in preview/outline.
  - #135–#140: close dirty-state consistency, lossless image URL/title serialization, standalone slash-command insertion, backward Unicode word navigation, and PDF combining-mark offsets.
- Keep each fix scoped to the reported path, retaining existing undo, selection, source-mapping, and export behavior outside the failing cases.
- Add model-level regression tests and app-level tests where focus, layout, clipboard, pointer, or session coordination is part of the defect.
- Preserve per-document-version derived Markdown caches and `Arc` sharing, syntax-highlight memoization, cached text handles, bounded file-tree rendering, and GPUI-free `crates/*` members.

**Non-goals:** no new product capabilities, specification redesign, broad UI refactor, release work, issue closure/commenting, or changes to unrelated active OpenSpec changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This change restores behavior already required by the existing specifications; it does not alter requirements, so the change opts out of specification deltas.

## Impact

Affected areas include `src/app/` editing, preview, application/session, search, workspace, and Git-history coordination; `src/` Markdown, inline/block editing, emoji, math, navigation, export, and document-state helpers; `crates/git-sync/` rename-aware history metadata; and `crates/pdf/` text/image writing. Tests will use the existing Rust test suites and temporary local fixtures. No new dependency is anticipated; any apparently missing capability must be confirmed before implementation rather than silently adding one.
