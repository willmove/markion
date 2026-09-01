## MODIFIED Requirements

### Requirement: Help menu Markdown Reference
The Help menu SHALL offer a "Markdown Reference" item on both menu surfaces the application renders: the in-window menu bar dropdown and the native OS menu bar. The item SHALL sit in the help/reference group immediately after Check for Updates and its following separator, and immediately before "Report an Issue". Invoking the item SHALL open an in-app Markdown syntax-reference overlay and SHALL dismiss the in-window Help dropdown when invoked from that surface. The overlay SHALL present localized syntax examples for the Markdown constructs Markion parses (headings; inline emphasis, strong emphasis, strikethrough, highlight, superscript, subscript, and inline code; links and images; block quotes and thematic breaks; unordered, ordered, nested, and task lists; tables; fenced code; inline and display math; footnotes and reference links), using the active theme palette. The overlay body SHALL provide a right-side vertical scrollbar when its content exceeds the visible area so users can scroll the full reference. The overlay SHALL NOT open a document tab, fetch remote content, embed a web view, or mutate document text, dirty state, or derived Markdown caches.

The factory-default shortcut for Markdown Reference SHALL be `F1` on every platform, using stable action id `show-markdown-reference` in the customizable-shortcut registry. The in-window Help item SHALL display that effective binding. The `ShowShortcuts` action SHALL NOT use `F1` as its factory default; Preferences → Shortcuts SHALL remain reachable from the Preferences panel, and `show-shortcuts` MAY receive a user-assigned override later.

#### Scenario: Help menu lists Markdown Reference before Report an Issue
- **WHEN** the user opens the Help menu
- **THEN** a Markdown Reference item is present after Check for Updates and before Report an Issue

#### Scenario: Markdown Reference opens as an overlay
- **WHEN** the user chooses Help → Markdown Reference or presses `F1` while no override occupies that key
- **THEN** the Markdown Reference overlay opens
- **AND** no new document tab is created

#### Scenario: Markdown Reference body shows a right-side scrollbar when overflowing
- **WHEN** the Markdown Reference overlay is open and the body content exceeds the visible body height
- **THEN** a vertical scrollbar is visible on the right side of the reference body
- **AND** scrolling moves through the remaining sections

#### Scenario: Markdown Reference does not mutate documents
- **WHEN** the user opens and then dismisses Markdown Reference
- **THEN** open tabs, document text, dirty flags, and derived Markdown caches are unchanged

#### Scenario: Escape dismisses Markdown Reference
- **WHEN** the Markdown Reference overlay is open and the user presses Escape or activates the close control
- **THEN** the overlay closes

#### Scenario: F1 defaults to Markdown Reference when show-shortcuts is unbound
- **WHEN** no override is stored for `show-shortcuts` or `show-markdown-reference`
- **THEN** `F1` opens Markdown Reference
