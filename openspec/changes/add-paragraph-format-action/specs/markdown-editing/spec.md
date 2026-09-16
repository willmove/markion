## ADDED Requirements

### Requirement: Paragraph formatting removes ATX heading syntax
The editor SHALL provide a Paragraph formatting action in the Format menu. When invoked with a caret or selection, the action SHALL remove the complete ATX heading prefix (`#` through `######` plus its following separator) from every intersected heading line, preserve each line's content, leave intersected non-heading lines unchanged, and keep the resulting caret or selection anchored to the same content. The mutation SHALL use the same source-backed document lifecycle as other formatting actions. If no intersected line is an ATX heading, the action SHALL leave the document text, version, dirty state, undo/redo history, and derived Markdown caches unchanged and SHALL report that no formatting change occurred.

#### Scenario: Caret heading becomes a paragraph
- **WHEN** the caret is inside an ATX heading and the user invokes Format → Paragraph
- **THEN** the heading prefix is removed and the heading content becomes an ordinary paragraph
- **AND** the caret remains at the equivalent position in that content

#### Scenario: Selected heading lines become paragraphs
- **WHEN** a selection intersects multiple lines containing ATX headings and ordinary paragraphs
- **THEN** the heading prefixes are removed from the intersected ATX heading lines
- **AND** the ordinary paragraph lines and all line content remain unchanged
- **AND** the resulting selection remains anchored to the same textual content

#### Scenario: Paragraph action is idempotent on non-headings
- **WHEN** the caret or selection intersects no ATX heading line and the user invokes Paragraph
- **THEN** the canonical Markdown source and document version remain unchanged
- **AND** dirty state, undo/redo history, and derived Markdown caches remain unchanged
- **AND** the editor reports that no formatting change occurred

### Requirement: Paragraph formatting has a platform-aware shortcut
The Paragraph action SHALL use `Ctrl+0` by default on Windows and Linux and `Cmd+0` by default on macOS. It SHALL participate in the existing customizable shortcut system and SHALL expose its effective binding beside the Paragraph item in both supported Format-menu presentations and in the localized Editing section of the in-app shortcut reference.

#### Scenario: Default shortcut invokes Paragraph
- **WHEN** the editor has no Paragraph shortcut override and the user presses `Ctrl+0` on Windows/Linux or `Cmd+0` on macOS
- **THEN** the editor invokes the same Paragraph transformation as the Format-menu item

#### Scenario: Paragraph shortcut is discoverable
- **WHEN** the user opens the Format menu, shortcut preferences, or the Editing section of the shortcut reference
- **THEN** Paragraph appears with its effective platform-specific shortcut

#### Scenario: Paragraph shortcut can be customized
- **WHEN** the user assigns a valid shortcut override to Paragraph
- **THEN** the override replaces the default for dispatch and every displayed shortcut label
- **AND** resetting the override restores `Ctrl+0` on Windows/Linux or `Cmd+0` on macOS
