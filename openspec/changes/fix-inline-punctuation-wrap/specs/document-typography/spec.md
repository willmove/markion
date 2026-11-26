## ADDED Requirements

### Requirement: Closing punctuation stays with rendered prose

Visual Edit, Split Preview, and Read SHALL keep closing punctuation with the immediately preceding text during automatic prose wrapping, including when a change in inline formatting separates them. When both fit on a line together, the renderer SHALL wrap at an earlier valid position as needed. Surrounding CJK prose SHALL wrap at available text boundaries even when the paragraph uses mixed inline layout. Authored breaks SHALL continue to start a new line.

#### Scenario: Closing punctuation after strong emphasis with inline math

- **WHEN** a Chinese paragraph has a bold phrase followed directly by a colon or comma and also contains inline math
- **THEN** the punctuation stays on the same line as the end of the bold phrase in Visual Edit, Split Preview, and Read
- **AND** the following text can use the remaining room on that line

#### Scenario: Tight wrap in ordinary shaped prose

- **WHEN** a rendered paragraph without inline widgets has room for the preceding glyph at the end of a line but lacks room for its closing punctuation
- **THEN** automatic wrapping keeps the glyph and punctuation together when the pair can fit on a line
