## ADDED Requirements

### Requirement: Definition lists SHALL render in preview, Read, and Visual Edit
The Markdown parser SHALL recognize definition lists: a term line followed by one or more definitions, each opened by a `:` marker at the start of a line. Split Preview and Read mode SHALL render each term in bold and each definition indented below it. Inline formatting inside terms and definitions SHALL render as in paragraphs. A definition that spans several paragraphs SHALL render as one definition with a line break between its paragraphs. A non-paragraph block nested in a definition (for example a list or code block) SHALL end that definition's own text and render as an ordinary block after it. Visual Edit SHALL render terms and definitions the same way, with the `:` marker hidden as a block prefix that is revealed when the caret enters the definition. Terms and definitions SHALL edit as ordinary source-backed inline text with exact caret mapping and one source mutation per edit. Definition lists inside a blockquote SHALL render inside the quote. Every source byte of a definition list SHALL have exactly one visual owner.

#### Scenario: Term and definition render in preview
- **WHEN** a document contains `Apple` followed by `: Red fruit` and is shown in Split Preview or Read mode
- **THEN** "Apple" renders in bold and "Red fruit" renders indented below it
- **AND** the `:` marker is not shown

#### Scenario: Several definitions and terms
- **WHEN** a term has two `:` definitions and is followed by a second term with its own definition
- **THEN** each term renders once and each definition renders as its own indented row below its term

#### Scenario: Loose multi-paragraph definition
- **WHEN** a definition is followed by an indented continuation paragraph
- **THEN** both paragraphs render inside the same definition, separated by a line break

#### Scenario: Visual Edit hides and reveals the marker
- **WHEN** a definition list is shown in Visual Edit and the caret is outside the definition
- **THEN** the definition renders indented without its `:` marker
- **AND** moving the caret into the definition reveals the authored marker for editing
- **AND** typing inside the definition edits exactly that source text

#### Scenario: Nested block in a definition
- **WHEN** a definition contains a nested list
- **THEN** the definition row owns only the bytes before the list, and the list renders as an ordinary list after it

## MODIFIED Requirements

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, YAML front matter, HTML blocks, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior. Empty ATX headings and empty list items SHALL be classified as rendered WYSIWYG with progressive-reveal structural prefixes, not as coverage gaps.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, headings including empty ATX headings, code including indented code blocks and unclosed or malformed fences, math including Pending/Error payload editors, diagrams, images including reference-style block images, tables including ragged grids, YAML front matter, lists and task lists including empty list items and clickable Visual Edit checkboxes, footnote definitions and references, definition lists, blockquotes, alerts, rules, HTML blocks), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, decoded HTML entities, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (multiline or otherwise unprovable images, residual unsupported gaps)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. After this change the remaining primary gap SHALL be multiline or otherwise unprovable images. The roadmap SHALL also track residual unsupported gap bytes as a secondary gap. YAML front matter, reference-style block images, ragged tables, math Pending/Error states, indented code blocks, unclosed or malformed fenced code, and definition lists SHALL NOT remain on the roadmap. Task-list checkbox click interaction SHALL NOT remain on the roadmap once Visual Edit checkboxes are clickable.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gap (multiline or otherwise unprovable images) is listed with priority, effort, target class, and implementation seam
- **AND** YAML front matter, reference-style block images, ragged tables, math render-failure states, indented code blocks, and unclosed or malformed fenced code are absent from the open-gap list

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example definition lists, indented code blocks, unclosed or malformed fenced code, YAML front matter, reference-style block images, empty ATX headings and empty list items, decoded HTML entities in the proven set, angle-bracket autolinks, ragged tables, math Pending/Error payload editors, escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, GFM alerts, or Visual Edit task-list checkbox click)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example residual unsupported gap bytes)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial

#### Scenario: Task-list checkbox click is a closed gap
- **WHEN** a contributor reads the WYSIWYG coverage roadmap after this change
- **THEN** task-list checkbox click is absent from the open-gap list
- **AND** the coverage matrix records clickable Visual Edit task checkboxes as rendered WYSIWYG
