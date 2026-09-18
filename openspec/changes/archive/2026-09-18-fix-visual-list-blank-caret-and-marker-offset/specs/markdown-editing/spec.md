## ADDED Requirements

### Requirement: Visual Edit renders unquoted list trailing blank lines as unindented whitespace rows
When an unquoted list item is followed by authored blank lines (at document end, before following content, or between loose-list items), Visual Edit SHALL present those blank lines as full-width `Whitespace` rows at the document's left margin — the same presentation blank lines already use between paragraphs and after quoted lists — instead of painting them inside the indented list-item row. Each blank line SHALL keep its own source-backed, individually addressable position, and a caret on any of those rows SHALL paint at the row's left edge, aligned with the first content column of the surrounding document flow rather than with the list item's marker or text column. The caret at end-of-file after such a list SHALL keep its own empty insertion row when the last list line ends with a newline.

#### Scenario: Caret on a blank line below a nested list sits at the left margin
- **WHEN** the document contains a nested list whose innermost item is followed by several blank lines and the user places the caret on any of those blank lines
- **THEN** the caret is painted at the blank row's left edge, not indented to the inner list level nor aligned with the item's text column
- **AND** clicking the row, typing, and Undo keep using that line's canonical source offset

#### Scenario: Loose-list separators and mid-document tails use the same geometry
- **WHEN** a blank line separates two items of a loose list or follows a list before subsequent content
- **THEN** the blank line renders as an unindented whitespace row whose painted line count matches the authored blank lines, without duplicating or dropping the final separator before the next block

### Requirement: Revealed list structural prefixes align with the unfocused marker edge
While a list row's structural prefix is revealed for editing (caret on the marker, its following spaces, or an empty item), Visual Edit SHALL start the row's revealed source text at the same left edge the marker glyph occupies while the row is unfocused, instead of reserving the marker column width and shifting the revealed text to the right. The unfocused marker presentation (bullet, number, or task checkbox in the marker column) SHALL remain unchanged, and hiding the marker column SHALL NOT affect task-checkbox hit targets, which exist only while the prefix is hidden.

#### Scenario: Caret on a top-level bullet marker
- **WHEN** the caret sits on the `-` marker or the space following it in a top-level list row
- **THEN** the revealed `-` is painted at essentially the same left edge where the bullet glyph renders while the row is unfocused, without the reserved marker-column offset

#### Scenario: Empty and nested items keep working text flow
- **WHEN** an empty list item (`- ` with no content) or a nested item has its prefix revealed
- **THEN** the revealed source text still flows from the row's left edge, and pointer placement, text input, and IME caret geometry follow the same painted positions
