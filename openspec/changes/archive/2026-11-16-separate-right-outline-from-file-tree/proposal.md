# Proposal

## Why

The outline-position implementation relocates the shared Files/Outline sidebar. The requested Right setting must instead show an independent outline beside the document while the file tree remains visible on the left.

## What Changes

- Render Files on the left and Outline on the right simultaneously when Outline position is Right, with separate headers and no Files/Outline tab switch on either side.
- Give the detached outline its own persisted visibility and width; Files and Outline commands affect their respective panes. The sidebar master toggle controls both panes.
- Preserve the existing left-side Files/Outline tabs when Outline position is Left.
- Keep document navigation, folding, selection, history, and per-version derived caches intact.

Non-goals: a redesigned file tree or outline, additional docking locations, or Markdown editing changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `theme-preferences`: correct Right placement to independent simultaneous panes, define separate visibility controls, migration, and pane-width persistence.

## Impact

Workspace rendering, sidebar actions and preferences, preference/session adapters, localization, and GPUI regression tests. No new dependencies or document-cache changes.
