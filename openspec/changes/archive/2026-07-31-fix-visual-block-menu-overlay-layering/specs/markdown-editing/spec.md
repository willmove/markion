## MODIFIED Requirements

### Requirement: Visual Edit SHALL support exact block transformations and operations
A supported focused Visual Edit block SHALL expose contextual operations to turn it into Text, Heading 1 through Heading 6, Bulleted List, Numbered List, Task List, Quote, or Code Block, and to Duplicate or Delete it. The contextual block-operation menu SHALL render in an overlay above all Visual Edit document rows and media, SHALL remain anchored near its invoking control within the usable viewport, and SHALL keep every command reachable when space is constrained. Showing, positioning, scrolling within, or dismissing the menu SHALL NOT change canonical source, document version, history, or derived-cache identity. Each operation SHALL validate current document version, block identity, and exact source ownership; it SHALL perform one canonical source mutation with one undo entry and preserve unrelated bytes, line endings, dirty state, autosave/recovery behavior, tab isolation, and cache invariants.

#### Scenario: Heading turns into a task item
- **WHEN** the user transforms an exactly mapped heading into a Task List block
- **THEN** only the proven structural marker is replaced with canonical unchecked-task Markdown
- **AND** inline authored content and UTF-8 text remain byte-identical

#### Scenario: Code block turns into text
- **WHEN** a closed exactly mapped fenced code block is transformed to Text
- **THEN** its payload becomes paragraph source and the fence metadata is removed in one edit
- **AND** an unclosed or ambiguous fence is not transformed speculatively

#### Scenario: Duplicate and delete are atomic
- **WHEN** the user duplicates or deletes a supported block
- **THEN** the complete exact block source and deterministic separator whitespace are duplicated or removed
- **AND** one Undo restores the prior source and selection

#### Scenario: Stale or ambiguous transform is rejected
- **WHEN** a block event carries a stale version/identity/range or the source ownership overlaps an ambiguous nested structure
- **THEN** no source, history, document version, or cache identity changes
- **AND** complete source editing remains available

#### Scenario: Block menu overlays later visual content
- **WHEN** the user opens a supported block's operation menu where its bounds overlap following headings, formatted prose, an image, or another Visual Edit row
- **THEN** the complete menu background and commands paint above the overlapping document content
- **AND** underlying document content cannot visually obscure the menu or receive pointer actions within its bounds

#### Scenario: Block menu stays reachable near viewport edges
- **WHEN** the user opens the block-operation menu with insufficient space below or beside its invoking control
- **THEN** the menu flips or is constrained within the usable viewport
- **AND** overflow commands remain reachable through menu-local scrolling without scrolling the document

#### Scenario: Block menu dismissal is presentation-only
- **WHEN** the user dismisses an open block-operation menu with Escape, an outside action, document scrolling, a tab or mode change, or stale-target invalidation
- **THEN** the menu closes without changing canonical Markdown, document version, selection, history, dirty state, or derived-cache identity

