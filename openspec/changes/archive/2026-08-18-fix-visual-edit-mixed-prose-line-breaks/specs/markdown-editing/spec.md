## ADDED Requirements

### Requirement: Visual Edit mixed-prose line breaks
When Visual Edit lays out a prose row as mixed fragments (because the row contains a link or footnote navigation icon, an inline math atom, or an inline HTML image), authored soft breaks and hard breaks inside that row SHALL still start a new visual line, matching Read and Split Preview for the same source. Intra-line wrapping of long prose, progressive syntax reveal, and source-backed editing SHALL remain unchanged. Interaction-only layout grouping SHALL NOT change document version or invalidate per-version derived caches.

#### Scenario: Consecutive source lines with a link stay stacked
- **WHEN** a paragraph (or heading / list item / quoted leaf) is written as consecutive source lines with no blank separator, at least one of those lines contains a Markdown link, and Visual Edit is active
- **THEN** each authored line renders on its own visual row rather than joining into a single line
- **AND** the link keeps its rendered label and navigation icon on the line that owns the link

#### Scenario: Hard breaks still break in mixed layout
- **WHEN** a mixed-fragment prose row contains a Markdown hard break (two trailing spaces or a backslash before the newline)
- **THEN** the text after that break renders on the following visual row

#### Scenario: Single-line mixed prose is unchanged
- **WHEN** a mixed-fragment prose row contains no authored line break
- **THEN** its fragments continue to wrap as one flowing paragraph
- **AND** navigation icons and inline atoms stay on that same flow
