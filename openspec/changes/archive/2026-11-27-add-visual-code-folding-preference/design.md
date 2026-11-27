## Context

Visual Edit stores folded block identities per tab and preserves source ranges for dormant tabs. Changing the availability of the fold control changes code row height.

## Goals / Non-Goals

Persist the feature switch in `config.toml`. Keep individual folds as session presentation state.

## Decisions

- Load the switch through the existing code display preferences. Use the same state update for the Appearance control and preference reset.
- Use the switch when rendering code rows and handling fold actions. Disabling it clears both live folds and dormant fold bookmarks while preserving structured selection bookmarks.
- Remeasure the Visual Edit list around its scroll anchor when the control appears or disappears. Use the existing presentation invalidation path to keep version-cached Markdown and highlighting available.

## Risks / Trade-offs

A dormant tab could otherwise restore a fold after the user disables and re-enables the preference. Clear its fold bookmarks when disabling so it returns expanded even in that sequence.
