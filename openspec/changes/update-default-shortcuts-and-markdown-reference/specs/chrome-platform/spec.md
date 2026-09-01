## ADDED Requirements

### Requirement: Help menu Markdown Reference
The Help menu SHALL offer a "Markdown Reference" item on both menu surfaces the application renders: the in-window menu bar dropdown and the native OS menu bar. The item SHALL sit in the help/reference group immediately after Check for Updates and its following separator, and immediately before "Report an Issue". Invoking the item SHALL open an in-app Markdown syntax-reference overlay and SHALL dismiss the in-window Help dropdown when invoked from that surface. The overlay SHALL present localized syntax examples for the Markdown constructs Markion parses (headings; inline emphasis, strong emphasis, strikethrough, highlight, superscript, subscript, and inline code; links and images; block quotes and thematic breaks; unordered, ordered, nested, and task lists; tables; fenced code; inline and display math; footnotes and reference links), using the active theme palette. The overlay SHALL NOT open a document tab, fetch remote content, embed a web view, or mutate document text, dirty state, undo history, or derived Markdown caches. Dismissal SHALL use an explicit close control and Escape, and SHALL leave the current document and view mode unchanged.

The factory-default shortcut for Markdown Reference SHALL be `F1` on every platform, using stable action id `show-markdown-reference` in the customizable-shortcut registry. The in-window Help item SHALL display that effective binding. The `ShowShortcuts` action SHALL NOT use `F1` as its factory default; Preferences → Shortcuts SHALL remain reachable from the Preferences panel, and `show-shortcuts` MAY receive a user-assigned override later.

#### Scenario: Help menu lists Markdown Reference before Report an Issue
- **WHEN** the user opens the Help menu (in-window dropdown or native menu bar)
- **THEN** a Markdown Reference item is present after Check for Updates and before Report an Issue
- **AND** About Markion remains last

#### Scenario: Menu or F1 opens the overlay
- **WHEN** the user chooses Help → Markdown Reference or presses `F1` while no override occupies that key
- **THEN** the Markdown Reference overlay opens
- **AND** the in-window Help dropdown is closed if it was open
- **AND** no new document tab is created

#### Scenario: Overlay shows Markion-supported syntax
- **WHEN** the Markdown Reference overlay is open
- **THEN** it includes syntax examples for headings, inline formatting, links and images, quotes, lists and task lists, tables, fenced code, math, and footnotes
- **AND** the overlay surface, text, border, and controls use the active theme palette

#### Scenario: Overlay does not touch document state
- **WHEN** the user opens and then dismisses Markdown Reference
- **THEN** the active tab's text, dirty flag, undo history, view mode, and derived Markdown caches are unchanged
- **AND** no network request is made for the overlay body

#### Scenario: Escape and close dismiss the overlay
- **WHEN** the Markdown Reference overlay is open and the user presses Escape or activates the close control
- **THEN** the overlay is hidden
- **AND** the editor continues running normally

#### Scenario: F1 is not the factory ShowShortcuts binding
- **WHEN** no override is stored for `show-shortcuts` or `show-markdown-reference`
- **THEN** `F1` opens Markdown Reference
- **AND** `ShowShortcuts` has no factory keystroke
- **AND** File → Preferences still exposes the Shortcuts tab
