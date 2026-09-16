## ADDED Requirements

### Requirement: In-document table of contents from a TOC token
The editor SHALL treat a standalone paragraph whose trimmed plaintext is exactly `[TOC]` (ASCII, case-insensitive) as an in-document table of contents. The canonical source SHALL remain that token. Read mode, Split Preview, and unfocused Visual Edit SHALL render the current document outline as a nested, clickable heading list. Activating a TOC entry SHALL navigate to that heading the same way the outline panel does, without mutating document text, dirty state, undo history, or document version. When the Visual Edit caret belongs to the TOC block, the editor SHALL reveal the authored `[TOC]` token so the user can edit or delete it. A `[TOC]` token inside a list item, fenced code, or other non-paragraph construct SHALL remain ordinary text. Missing headings SHALL render an empty TOC without inventing source.

#### Scenario: TOC token renders a live outline
- **WHEN** the document contains a paragraph that is only `[TOC]` or `[toc]` and the document has headings
- **THEN** Read, Split Preview, and unfocused Visual Edit show those headings as a nested clickable list
- **AND** the canonical source still contains the `[TOC]` token rather than an expanded Markdown list

#### Scenario: Clicking a TOC entry jumps without mutating
- **WHEN** the user activates a TOC heading entry
- **THEN** the editor navigates to that heading’s source position and brings it into view for the active mode
- **AND** document text, version, dirty state, and undo history are unchanged

#### Scenario: Focused Visual Edit TOC reveals the token
- **WHEN** the Visual Edit caret belongs to the TOC block
- **THEN** the authored `[TOC]` token is visible for editing
- **AND** deleting that block removes the token through the existing exact block-delete path

#### Scenario: List-item TOC text is not a widget
- **WHEN** a list item’s text is `[TOC]`
- **THEN** Visual Edit and preview keep it as list-item text
- **AND** they do not replace the item with a generated outline

### Requirement: Footnote reference hover preview
The editor SHALL show a presentation-only tooltip with the matching footnote definition text when the pointer hovers a resolved footnote reference in Read, Split Preview, or Visual Edit. Hovering SHALL NOT mutate document text, dirty state, undo history, or document version. Unresolved footnote labels SHALL omit the tooltip. Clicking an existing Visual Edit footnote navigation icon SHALL continue to jump to the definition.

#### Scenario: Hovering a resolved footnote shows its definition
- **WHEN** the document defines `[^note]` and the pointer hovers a `[^note]` reference in Read, Split Preview, or Visual Edit
- **THEN** a tooltip shows that definition’s text
- **AND** document version and derived caches are unchanged

#### Scenario: Unresolved footnote markers have no tooltip
- **WHEN** the pointer hovers a footnote-like marker whose label has no definition
- **THEN** the editor does not show a footnote tooltip

### Requirement: In-document heading anchor navigation
The editor SHALL resolve bare Markdown link destinations of the form `#fragment` to a document heading. A successful match SHALL jump to that heading the same way the outline panel does, without calling the platform URL opener and without mutating source. Visual Edit SHALL attach a heading navigation target (and icon) to such links when the heading exists. Links that are not a bare `#fragment`, or whose fragment matches no heading, SHALL keep existing URL-open behavior. Heading ids SHALL prefer an authored heading attribute `{#id}` when present; otherwise they SHALL slugify the heading title keeping Unicode letters and digits, and duplicate ids SHALL uniquify with a numeric suffix (`hello`, then `hello-1`).

#### Scenario: Hash link jumps to the heading
- **WHEN** the document contains `## Hello` and a link `[go](#hello)`
- **THEN** activating that link in Read, Split Preview, or Visual Edit navigates to the Hello heading
- **AND** the platform URL opener is not invoked
- **AND** document text and version are unchanged

#### Scenario: Authored heading id wins
- **WHEN** the document contains `## Hello {#custom}` and a link `[go](#custom)`
- **THEN** activating that link jumps to that heading

#### Scenario: Duplicate titles uniquify
- **WHEN** the document contains two headings titled `Hello`
- **THEN** the first heading’s id is `hello` and the second is `hello-1`
- **AND** `[go](#hello-1)` jumps to the second heading

#### Scenario: CJK titles remain jumpable
- **WHEN** the document contains `## 中文标题` and a link `[go](#中文标题)`
- **THEN** activating that link jumps to that heading
