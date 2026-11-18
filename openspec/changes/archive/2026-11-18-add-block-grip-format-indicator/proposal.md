## Why

The dotted grip in Visual Edit gives no indication of the current block format until its menu opens. A format symbol beside the dots makes that information available at the block start.

## What Changes

- Show the current block format beside the dots when the block-menu preference is enabled, with a localized name on hover.
- Make the format symbol and dots one grip for selecting, opening the menu, and dragging. Reserve a shared leading gutter in Visual Edit and Read so the grip can appear, disappear, or change format without shifting the content.
- Show the dots alone when the preference is disabled.

Non-goals include changes to the block menu catalog and new Markdown formatting operations.

## Capabilities

### New Capabilities

### Modified Capabilities

- `markdown-editing` gains a format indicator on existing Visual Edit block grips.
- `theme-preferences` uses the existing block-menu preference to control indicator visibility.

## Impact

Visual Edit row rendering, the block grip, and localized block type names are affected. The indicator reads the existing per-version Visual Edit block data and requires no new Markdown derivation or persisted setting.
