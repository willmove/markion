## MODIFIED Requirements

### Requirement: Find and replace
The editor SHALL provide a compact floating Find / Replace workflow supporting case-sensitive and regular-expression search, next/previous navigation with wraparound, current/total counts, replace current, and replace all. Find and Replace fields SHALL behave as single-line text editors with caret placement, selection, keyboard editing, clipboard operations, and IME composition. Their editable content SHALL contain only the user-provided query or replacement value: field identity or guidance SHALL be rendered outside the editable value and SHALL NOT be inserted as a fixed prefix or placeholder inside the field.

In Edit, Split Preview, and Visual Edit modes, Find and Replace SHALL operate on canonical Markdown source. In Read mode, Find SHALL operate on user-visible selectable text in the rendered document rather than hidden Markdown syntax, and replacement SHALL be unavailable without changing the document or forcing a view-mode change. All matches that have a visible representation in the active search surface SHALL be highlighted with a subdued theme-aware treatment, the current match SHALL be visually distinct, and navigation SHALL reveal the current match in the visible source, Visual Edit, or Read pane.

The Find / Replace controls SHALL remain a compact floating overlay near the upper-right of the editor workspace, above the editor/preview panes, without consuming layout height or shifting the main workspace. The overlay SHALL provide an explicit close control that hides the overlay, clears active match highlighting and search focus, and preserves the current query and replacement text for a later reopen. The overlay, fields, buttons, disabled states, borders, hover states, match highlights, validation feedback, and summary text SHALL use the active theme palette rather than hard-coded light colors.

#### Scenario: Field values exclude fixed prefixes and placeholders
- **WHEN** the Find or Replace field is rendered with an empty or non-empty value
- **THEN** its editable text contains exactly the user-provided value
- **AND** localized field identity or guidance is presented outside the editable value
- **AND** no fixed `Find`, `查找`, `Replace`, `替换`, colon, or placeholder text is prepended or inserted into the field value

#### Scenario: Find and Replace fields provide normal single-line editing
- **WHEN** a Find or Replace field has focus
- **THEN** pointer placement, Left, Right, Home, End, shifted selection, Select All, Backspace, Delete, Cut, Copy, Paste, and IME composition operate on that field
- **AND** those commands do not move, select, delete, or insert text in the document
- **AND** typed line breaks are not inserted into either field or the document

#### Scenario: Search keyboard commands navigate and move focus
- **WHEN** a search field has focus and the user presses Enter or Shift+Enter
- **THEN** the editor selects the next or previous match respectively, with wraparound
- **WHEN** the user presses Tab or Shift+Tab within the overlay
- **THEN** focus moves through the currently available Find / Replace controls without entering the document

#### Scenario: Search with options initializes a current result
- **WHEN** the user enters a non-empty valid query or toggles case-sensitive or regular-expression search
- **THEN** matching is recomputed for the active mode's search domain
- **AND** the first match at or after the current source caret or visible Read-mode position becomes current, wrapping to the first match when necessary
- **AND** the overlay shows a current/total count that never reports `0/N` when matches exist

#### Scenario: All matches and the current match are distinguishable
- **WHEN** a valid query has one or more matches
- **THEN** every match with a visible representation in the active search surface is highlighted
- **AND** the current match uses a stronger, theme-aware treatment than the other matches
- **AND** changing the current match updates the count and visible highlight without changing the query

#### Scenario: Navigate, replace current, and replace all
- **WHEN** the user steps to next/previous in an editable mode
- **THEN** the current source-backed match is revealed and selected
- **WHEN** the user replaces the current match
- **THEN** the replacement is applied to that exact match and the next surviving match becomes current
- **WHEN** the user replaces all matches
- **THEN** all matching source ranges are replaced as one undoable operation and the match state is recomputed

#### Scenario: Read mode searches visible rendered text
- **WHEN** the active view mode is Read and the user enters a valid query
- **THEN** Find matches selectable text visible in rendered headings, prose, lists, quotations, code, tables, footnotes, and visible HTML text
- **AND** it does not match hidden Markdown punctuation, non-visible link destinations, image resource paths, or other source-only syntax
- **AND** styling boundaries within one rendered textual run do not prevent a visible phrase from matching

#### Scenario: Read mode navigation reveals and highlights the result
- **WHEN** the user selects a Read-mode match through query initialization, next, or previous
- **THEN** the virtualized preview scrolls to reveal the owning rendered block
- **AND** the exact visible text range is highlighted as the current match
- **AND** no hidden source editor scroll is used as the only navigation feedback

#### Scenario: Replace remains unavailable in Read mode
- **WHEN** the user invokes Replace while the active view mode is Read
- **THEN** the Find overlay remains usable in Find-only form
- **AND** localized guidance outside the editable field states that replacement is unavailable in Read mode
- **AND** replacement controls cannot mutate the document
- **AND** the stored replacement value is preserved for a later editable mode

#### Scenario: Changing view mode refreshes the search domain
- **WHEN** the Find / Replace overlay remains open while the user enters or leaves Read mode
- **THEN** matches and highlights are recomputed for the new mode's search domain
- **AND** the query, replacement value, case-sensitive option, regular-expression option, and requested Find/Replace form are preserved
- **AND** replacement controls become unavailable in Read mode and available again on return to an editable mode

#### Scenario: Invalid regular expression is actionable
- **WHEN** regular-expression search is enabled and the query is invalid
- **THEN** the overlay displays localized validation feedback and an invalid field treatment
- **AND** stale match highlights and the current match are cleared
- **AND** replace-current and replace-all actions are disabled
- **AND** the document remains unchanged

#### Scenario: Empty and no-match states clear stale actions
- **WHEN** the query is empty or a valid query has no matches
- **THEN** the overlay presents the corresponding localized state rather than a stale current/total count
- **AND** stale highlights and the current match are cleared
- **AND** replacement actions that require a match are disabled

#### Scenario: Find overlay does not shift workspace layout
- **WHEN** the user opens Find or Replace
- **THEN** the controls appear as a compact upper-right floating overlay above the editor/preview workspace
- **AND** the tab bar, editor pane, preview pane, and status bar keep their existing layout positions

#### Scenario: Closing the overlay clears active highlights
- **WHEN** the Find / Replace overlay is visible and the user activates its close control or presses Escape
- **THEN** the overlay is hidden
- **AND** active search focus is cleared
- **AND** active match highlighting is cleared
- **AND** the current find query and replacement text are preserved for the next time Find or Replace opens

#### Scenario: Find overlay follows active theme
- **WHEN** the active theme changes
- **THEN** the Find / Replace overlay surface, input fields, buttons, disabled states, borders, hover states, match highlights, validation feedback, and summary text render using the active theme palette
- **AND** the overlay does not use hard-coded light-only chrome colors

#### Scenario: Existing Find and Replace entry points are preserved
- **WHEN** the user invokes existing Find, Replace, Find Next, or Find Previous menu items, shortcuts, or actions
- **THEN** the overlay and navigation behavior follow this requirement without changing the configured shortcut bindings

