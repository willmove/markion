## ADDED Requirements

### Requirement: Format menu heading depth follows preference
The editor SHALL expose heading formatting entries in the Format menu (in-window dropdown and native OS menu) from H1 through the configured maximum level. The default maximum level SHALL be 5 so H4 and H5 are visible without extra setup. When the maximum level is 6, H6 SHALL also appear alongside H1–H5 with the same behavior as existing heading actions.

#### Scenario: Default menus show H1 through H5
- **WHEN** Heading menu depth is H1–H5 (default)
- **THEN** the Format menu lists heading actions for H1, H2, H3, H4, and H5

#### Scenario: Extended menus show H1 through H6
- **WHEN** Heading menu depth is H1–H6
- **THEN** the Format menu lists heading actions for H1, H2, H3, H4, H5, and H6

#### Scenario: Heading actions apply the selected level
- **WHEN** the user triggers a heading action for level N from the Format menu
- **THEN** the editor applies `MarkdownFormat::Heading(N)` to the current selection or line

### Requirement: Heading keyboard shortcuts respect configured depth
The editor SHALL bind `Ctrl+4` and `Ctrl+5` (platform `secondary-4/5`) to Heading 4 and 5 by default, in addition to the existing H1–H3 shortcuts. When Heading menu depth is H1–H6, the editor SHALL also bind `Ctrl+6` to Heading 6. The keyboard shortcut reference SHALL list H4 and H5 by default and H6 only when Heading menu depth is H1–H6.

#### Scenario: Default shortcuts apply H4 and H5
- **WHEN** Heading menu depth is H1–H5 and the user presses the Heading 4 shortcut
- **THEN** the editor applies a level-4 heading to the current selection or line

#### Scenario: Shortcut reference documents extended headings conditionally
- **WHEN** the user opens the keyboard shortcut reference and Heading menu depth is H1–H6
- **THEN** the reference includes Heading 4, 5, and 6 shortcuts

## MODIFIED Requirements

### Requirement: Source-text editing with formatting actions
The editor SHALL provide a source-text editing surface where the user types raw Markdown, plus a set of formatting actions (bold, italic, inline code, links, images, headings, lists, task lists, blockquotes, fenced code blocks) that wrap or transform the selected text into the corresponding Markdown syntax. Heading actions exposed in the Format menu and via keyboard shortcuts SHALL include every level from H1 through the configured Heading menu depth maximum (default H1–H5, optionally H1–H6).

#### Scenario: Formatting actions wrap the selection
- **WHEN** the user triggers a formatting action (e.g. bold) with a selection
- **THEN** the selected text is wrapped in the corresponding Markdown markers and the document updates

#### Scenario: Common editing operations
- **WHEN** the user performs insert/delete, selection, copy/cut/paste, undo/redo, or select all
- **THEN** the editor applies the operation and reports status feedback (including for empty-clipboard or no-selection cases)

#### Scenario: Heading level switching
- **WHEN** the user switches the heading level of a line via the Format menu
- **THEN** the line's heading markers are updated without crashing
