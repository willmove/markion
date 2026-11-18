## ADDED Requirements

### Requirement: Visual Edit block grips SHALL identify the current block format

When automatic grip-click popups are enabled, each visible Visual Edit block grip SHALL show a compact, theme-aware format symbol to the left of its drag dots and provide the localized format name on hover. The symbol SHALL reflect the block's current Markdown type, including heading levels H1–H6 and the distinction between bulleted, numbered and task lists. The symbol and dots SHALL share the existing grip gestures. The grip SHALL fit in the leading gutter and its visibility SHALL leave content width, row height and wrapping unchanged.

#### Scenario: Visible grips identify different types

- **WHEN** the user hovers eligible paragraphs, headings, list items, or other supported blocks in Visual Edit
- **THEN** each grip displays the corresponding format next to the dots
- **AND** hovering the grip shows the format name in the active language

#### Scenario: The current format follows block edits

- **WHEN** the user clicks the format symbol and converts the block using the quick-settings popup
- **THEN** the next time its grip is visible, the symbol identifies the new format
- **AND** Undo restores the previous format symbol

#### Scenario: The entire grip accepts block gestures

- **WHEN** the user Shift-clicks or drags the format side of the grip
- **THEN** the gesture extends or moves the block selection through the existing grip behavior
- **AND** the gesture leaves the automatic popup closed
