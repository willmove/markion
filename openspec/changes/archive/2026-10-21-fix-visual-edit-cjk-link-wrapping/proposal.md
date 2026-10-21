## Why

Visual Edit currently puts ordinary linked prose into a flex row of word fragments. Japanese text has no spaces at which those fragments can wrap, so linked list items can overflow or misalign against their marker. The same source-backed shaped text used by ordinary prose already handles line wrapping and editing.

## What Changes

- Route ordinary URL and heading link prose through the shaped `VisualEditableText` path when no rendered atom, script alignment, authored HTML break, or stable-mode link edit icon requires the mixed layout.
- Preserve footnote tooltips, stable-mode link edit icons, secondary-click navigation, and source-backed editing.
- Add visual regressions for linked Japanese list wrapping and source behavior.

Non-goals: Change link editing commands, stable inline preference behavior, or Markdown parsing and caching.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Specify natural shaped wrapping for ordinary linked prose in Visual Edit while preserving navigation and source editing.

## Impact

- `src/app/preview.rs`: choose the shaped text path for eligible link-only prose.
- `src/app/tests.rs`: replace mixed-layout assumptions and add linked-list rendering and interaction coverage.
- Derived Markdown remains cached by document version; this changes only the presentation element selected for an existing block.
