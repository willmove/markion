# Proposal

## Why

The document outline is fixed to the left of the editor. Users need a persistent preference to place it on either side of the document workspace.

## What Changes

- Add a localized Outline position control with Left and Right choices to General preferences, defaulting to Left.
- Persist the choice, restore it on startup, and return it to Left when preferences are reset.
- Place the shared sidebar on the chosen side while Outline is active; Files continues to appear on the left. Keep existing Files/Outline switching, visibility, navigation, folding, scrolling, and width persistence.
- Adapt the outline divider and resize direction to its chosen side without mutating documents or recomputing version-cached Markdown state.

Non-goals: simultaneous Files and Outline panels, independent sidebar widths, or changes to outline content and editing behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `theme-preferences`: expose and persist the outline-position preference and define its immediate layout behavior.

## Impact

Touches the preference model and TOML adapter, application initialization/reset/save paths, General preference controls, localized strings, workspace layout and sidebar resizing. No new dependencies or Markdown derivation changes.
