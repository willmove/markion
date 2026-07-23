## ADDED Requirements

### Requirement: Visual Edit link and footnote navigation icons
Visual Edit SHALL attach a resolved navigation target to each actionable inline link run (including reference-style links whose destination was resolved from document-scoped definitions) and each footnote reference run. Visual Edit SHALL render a compact clickable icon immediately after the construct's rendered label. Clicking the icon SHALL navigate without mutating document text: for a URL target the editor SHALL open the destination with the platform URL opener; for a footnote reference the editor SHALL move the source caret to the matching footnote definition block and scroll that block into view. Clicking the rendered label text SHALL continue to update the source selection for editing and SHALL NOT open the destination. Constructs without a resolvable destination SHALL omit the icon.

#### Scenario: Inline and reference-style links expose an open icon
- **WHEN** a Visual Edit prose block contains an inline link or a resolved reference-style link
- **THEN** a navigation icon is shown after the link label
- **AND** clicking the icon opens the resolved URL
- **AND** clicking the label places or extends the source selection without opening the URL

#### Scenario: Footnote reference jumps to its definition
- **WHEN** a Visual Edit prose block contains a footnote reference whose definition exists in the document
- **THEN** a navigation icon is shown after the superscript footnote label
- **AND** clicking the icon moves the caret to the footnote definition block and scrolls it into view

#### Scenario: Unresolved constructs stay non-navigable
- **WHEN** bracketed text does not resolve to a link or footnote reference
- **THEN** Visual Edit does not show a navigation icon for that text
