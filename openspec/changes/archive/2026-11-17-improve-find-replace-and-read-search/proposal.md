## Why

The current Find / Replace overlay renders localized prefixes as if they were part of each field while routing text through an append-only buffer, so ordinary caret movement, selection, deletion, and Enter behavior do not work like a text field. Find is also ineffective in Read mode because it scans and selects hidden Markdown source without highlighting or revealing the corresponding rendered result.

## What Changes

- Replace the append-only Find and Replace buffers with real single-line field state supporting caret placement, selection, keyboard editing, paste, and IME composition.
- Remove the fixed `Find:` / `查找：` and `Replace:` / `替换：` prefixes from inside the field values; identify fields with chrome outside the editable text, such as an adjacent label or icon with a localized tooltip/accessibility label.
- Define mode-aware find targets: Edit, Split Preview, and Visual Edit retain canonical-source search and replacement, while Read mode searches only user-visible rendered text and keeps replacement unavailable.
- Highlight every match with a subdued theme-aware treatment, distinguish the current match, initialize a valid current match as the query changes, and reveal the current result in the visible editor or preview pane.
- Make Enter / Shift+Enter navigate next / previous, make Tab move through Find / Replace controls, keep Escape dismissal, and ensure field-focused navigation/editing shortcuts operate on the field rather than the document.
- Present invalid regular expressions and empty/no-match states within the overlay, disable unavailable replacement actions, and keep current/total counts synchronized after query, option, navigation, and replacement changes.
- Localize all added labels, tooltips, accessibility text, validation feedback, and Read-mode replacement guidance in every supported interface language; remove hard-coded English search status text.
- Preserve the existing compact floating placement, active-theme styling, query/replacement retention on close, regex and case-sensitive options, shortcuts, and single-undo-unit replacement behavior.
- Non-goals: workspace-wide or multi-document search, search history, fuzzy search, whole-word matching, replacement from Read mode, or a new persisted search preference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Strengthen Find / Replace requirements with true field editing semantics, visible all/current match feedback, correct navigation, rendered-text Find in Read mode, and explicit replacement availability rules.
- `ui-i18n`: Require every new search-field label, tooltip, accessibility label, validation message, and Read-mode guidance string to use the exhaustive localization catalog.

## Impact

- Affected application state and actions: `src/app/mod.rs`, `src/app/application.rs`, `src/app/search.rs`, `src/app/editing.rs`, and `src/app/workspace.rs`.
- Affected rendering and navigation: `src/app/root_view.rs`, `src/app/editor_element.rs`, and `src/app/preview.rs`, including the virtualized preview list and preview text-run identities.
- Affected localization and tests: `src/i18n.rs`, `src/app/tests.rs`, core search tests where shared matching helpers change, and `docs/keyboard-shortcuts.md` if interaction documentation is updated.
- No external dependency or persisted-data migration is expected. Read-mode search must consume the existing per-version cached preview blocks and virtualized run model; it must not force a new Markdown parse or recompute derived Markdown state on every render or keystroke.
