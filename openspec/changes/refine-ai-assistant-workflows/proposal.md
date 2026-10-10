## Why

The AI feature exposes complete functionality in a long stack of equally weighted controls, leaving messages in a fixed-height viewport and setup without a reliable discovery/test/save/enable sequence. Users need a focused conversation surface, understandable document actions and settings that complete the first request without contradictory feedback.

## What Changes

- Allow model discovery before choosing a model; save and explicitly enable the tested draft while retaining capabilities only for the identical configuration.
- Give the panel a fixed header/composer and a flexible message viewport; use compact icon actions, explicit conversation/workspace modes and contextual writing options.
- Wrap multiline AI input with correct selection, caret, IME positioning and vertical movement.
- Make context origins, actual history policy, request state and unavailable actions clear; protect destructive history actions.
- Group settings by service, writing, history/privacy and advanced limits; expose key state, meaningful units and a fixed save footer.
- Provide a visible change summary for writing review while retaining the checked application boundary.

Non-goals: new providers, automatic document/workspace transmission, automatic indexing, new credential formats, arbitrary tools, a full searchable conversation archive, server model identity certification, and a full side-by-side diff editor.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ai-configuration`: First-run discovery and explicit save/enable completion, grouped settings and accurate verification/key state.
- `ai-conversation`: Flexible panel/composer layout, wrapped input, task mode and context/history feedback.
- `ai-writing`: Contextual parameter validation and useful review summaries.

These AI capabilities currently exist in the unarchived `add-configurable-ai-agent` change, with refinements in `improve-ai-agent-usability`. This change adds requirements at the same capability paths; archive those prerequisites first.

## Impact

Root GPUI AI panel/input, AI localization, embedded Lucide icons, guides and focused integration/layout tests. No new dependencies or persisted configuration schema are needed. Document-version-derived Arc caches, syntax highlighting, cached document text handles and GPUI-free member crates remain unchanged; wrapping caches are local to the AI input entity.
