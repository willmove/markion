## ADDED Requirements

### Requirement: Visual Edit SHALL edit indented and unclosed code blocks as code
Visual Edit SHALL present an indented code block as a rendered code block with a payload editor, the same as a fenced code block, instead of a raw-source island. Each body line's code-block indentation (four spaces or one tab) SHALL remain in the canonical source and SHALL NOT be displayed. Pressing Enter in, or pasting multi-line text into, an indented code payload SHALL prefix each new line with the block's indentation unit (a tab when the block already indents with tabs, otherwise four spaces), so that the new lines remain in the same code block. Indented code SHALL NOT offer an editable language label, and transforming an indented code block into another block type SHALL use its body without the indentation. A fenced code block that has no valid closing fence before its block ends (an unclosed or malformed fence) SHALL keep its payload editor and language label: the payload SHALL extend to the end of the block, and typing a closing fence SHALL be ordinary payload editing. Each edit SHALL be one source mutation through the existing history and dirty-state path.

#### Scenario: Indented code renders with hidden indentation
- **WHEN** a document contains a paragraph followed by an indented code block whose lines are indented four spaces, and Visual Edit is active
- **THEN** the block renders as a highlighted code block whose text shows the code lines without their leading four spaces
- **AND** the canonical source still contains the indentation

#### Scenario: Enter keeps new lines in the indented code block
- **WHEN** the caret is at the end of an indented code line and the user presses Enter and types text
- **THEN** the new source line starts with the block's indentation followed by the typed text
- **AND** the document still contains a single code block containing the new line

#### Scenario: Pasting multiple lines into indented code
- **WHEN** the user pastes text containing line breaks into an indented code payload
- **THEN** every pasted line after the first starts with the block's indentation in the source

#### Scenario: Unclosed fence keeps a payload editor
- **WHEN** a document ends with a fenced code block that has an opening fence such as ```` ```rust ```` and no closing fence
- **THEN** Visual Edit renders it as a code block with the `rust` language label and an editable payload that runs to the end of the block
- **AND** it does not render as a raw-source island

#### Scenario: Malformed closing line stays payload
- **WHEN** a fenced code block's would-be closing line carries extra text after the fence (for example ```` ``` not-a-close ````)
- **THEN** that line is part of the editable payload, matching CommonMark, and the block keeps its payload editor

## MODIFIED Requirements

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, YAML front matter, HTML blocks, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior. Empty ATX headings and empty list items SHALL be classified as rendered WYSIWYG with progressive-reveal structural prefixes, not as coverage gaps.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, headings including empty ATX headings, code including indented code blocks and unclosed or malformed fences, math including Pending/Error payload editors, diagrams, images including reference-style block images, tables including ragged grids, YAML front matter, lists and task lists including empty list items and clickable Visual Edit checkboxes, footnote definitions and references, blockquotes, alerts, rules, HTML blocks), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, decoded HTML entities, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (multiline or otherwise unprovable images, definition lists, residual unsupported gaps)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. After this change the remaining primary gap SHALL be multiline or otherwise unprovable images. The roadmap SHALL also track secondary gaps including GFM definition lists and residual unsupported gap bytes. YAML front matter, reference-style block images, ragged tables, math Pending/Error states, indented code blocks, and unclosed or malformed fenced code SHALL NOT remain on the roadmap. Task-list checkbox click interaction SHALL NOT remain on the roadmap once Visual Edit checkboxes are clickable.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gap (multiline or otherwise unprovable images) is listed with priority, effort, target class, and implementation seam
- **AND** YAML front matter, reference-style block images, ragged tables, math render-failure states, indented code blocks, and unclosed or malformed fenced code are absent from the open-gap list

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example indented code blocks, unclosed or malformed fenced code, YAML front matter, reference-style block images, empty ATX headings and empty list items, decoded HTML entities in the proven set, angle-bracket autolinks, ragged tables, math Pending/Error payload editors, escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, GFM alerts, or Visual Edit task-list checkbox click)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example GFM definition lists)
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
