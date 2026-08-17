## ADDED Requirements

### Requirement: Visual Edit blockquote and GFM alert fidelity
Visual Edit SHALL render blockquote content without discarding authored structure. A blockquote that opens with a GFM alert marker line (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, or `[!CAUTION]`) SHALL present that line as a styled callout title row within the same visual quote group as the alert body, not as a raw-source island or other unstyled block. Quoted paragraphs written across consecutive lines (lazy continuation) SHALL preserve the authored line breaks in the rendered row. Every blockquote byte SHALL keep exactly one visual owner, and focusing a callout title row SHALL reveal its exact authored source line through the same progressive marker-reveal behavior as other hidden blockquote markers.

#### Scenario: GFM alert renders as a callout title plus body
- **WHEN** the document contains a GFM alert such as `> [!NOTE]` followed by one or more quoted body lines and Visual Edit is active
- **THEN** the marker line renders as a callout title row labeled for the alert kind inside the quote group's visual styling
- **AND** the body lines render as quoted content directly below the title row
- **AND** no raw-source island or unsupported block appears for the marker line

#### Scenario: Focusing the callout title reveals its source
- **WHEN** the caret enters the callout title row in Visual Edit
- **THEN** the row reveals its exact authored source line (for example `> [!NOTE]`) as an editable, byte-exact range
- **AND** leaving the row restores the rendered title without changing document text, version, or derived caches

#### Scenario: Alert with no body still shows its title
- **WHEN** a blockquote consists solely of a GFM alert marker line with no following content
- **THEN** Visual Edit renders the callout title row for that line without emitting a raw-source island

#### Scenario: Multi-line quoted paragraphs keep line breaks
- **WHEN** a quoted paragraph continues over consecutive source lines (for example `> line one` then `> line two` with no blank separator)
- **THEN** the rendered row shows the lines separated by a line break
- **AND** inline formatting, links, and the quote group's markers continue to map byte-exactly to the source

#### Scenario: Unknown alert markers stay literal text
- **WHEN** a blockquote contains a marker line with an unrecognized type such as `> [!CUSTOM]`
- **THEN** the marker text renders as literal paragraph text on its own line without callout styling
- **AND** any following quoted line renders below the line break

#### Scenario: Structural quote separator lines are unchanged
- **WHEN** a blockquote contains bare `>` separator lines between paragraphs
- **THEN** those lines keep their current whitespace-row rendering and the quote group remains visually contiguous
