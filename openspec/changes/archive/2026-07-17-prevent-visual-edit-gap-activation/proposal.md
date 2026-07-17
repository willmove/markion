## Why

Visual Edit currently treats every source-backed whitespace gap between rendered blocks as a pointer-editable surface. Clicking the visual spacing between headings, or between a heading and paragraph, therefore moves the caret into an otherwise passive gap and exposes an unexpected typing area, which conflicts with the intended WYSIWYG interaction.

## What Changes

- Make whitespace gaps between rendered Visual Edit blocks passive to pointer input while they do not own the source caret.
- Preserve whitespace blocks as source-backed layout rows so keyboard navigation, structural Enter, search reveal, and exact source coverage continue to work.
- Activate a whitespace editing affordance only after the source caret intentionally enters that whitespace, while preserving the source-backed insertion line created by Enter from a heading.
- Add regression coverage for heading-to-heading and heading-to-paragraph gap clicks, plus heading Enter followed by typing.
- Non-goal: change Markdown parsing, block spacing, or the cached derived-state architecture.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Clarify when source-backed whitespace ranges in Visual Edit are passive layout versus active editing affordances.

## Impact

- `src/app/preview.rs`: condition whitespace pointer behavior on current caret ownership.
- `src/app/tests.rs`: add interaction regressions for passive clicks and keyboard activation.
- No public API or dependency changes.
- Per-document derived Markdown caches remain unchanged; whitespace interactivity is derived from the current selection during rendering rather than stored in cached document state.
