## Why

The Visual Edit table grip offers generic formatting buttons that flatten table content and displays unavailable cell commands. Whole-table actions and insertion below the table are hard to distinguish at a glance.

## What Changes

- Give table grips a dedicated set of whole-table commands and a labeled table-to-quote action.
- Keep paragraph insertion directly accessible and move divider and new-table insertion into a labeled flyout.
- Reuse the existing checked command paths and menu navigation.

Non-goals include altering the cell context menu and changing Markdown syntax or history behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline` defines the table-specific block grip commands and insertion scope.

## Impact

Visual Edit menu state, rendering, localized labels, and rendered pointer and keyboard regressions. Source mutations retain their checked transactions and per-version derived caches.
