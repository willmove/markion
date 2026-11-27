## ADDED Requirements

### Requirement: Adjacent HTML images wrap as an aligned group
In Read, Split Preview, and Visual Edit, adjacent images of the same alignment within the same flow of a standalone HTML block SHALL share a line when their display widths and spacing fit. Source whitespace and inline links SHALL preserve adjacency. When the pane narrows, whole images SHALL wrap in source order, and each line SHALL follow the resolved alignment. Each image SHALL retain its authored size hints, link, and pane-width limit.

#### Scenario: Centered README badges wrap with the pane
- **WHEN** a centered HTML paragraph contains linked Release, unlinked platform, and linked MIT image badges separated by source newlines
- **THEN** the three badges share a centered line at sufficient width
- **AND** they wrap in order into independently centered lines as the pane narrows

#### Scenario: Explicit HTML boundaries separate image lines
- **WHEN** consecutive images occur in different HTML paragraphs, or a `<br>` separates them
- **THEN** the authored boundary starts a new image line
