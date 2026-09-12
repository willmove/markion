## ADDED Requirements

### Requirement: YAML front matter is a collapsible Visual Edit header
Visual Edit SHALL present a leading YAML `---` / `...` front-matter region as a rendered document-header row, not as a FrontMatter source island. The row SHALL carry a payload editor whose exact range is the complete authored header including opening and closing delimiters. While collapsed, the header SHALL show a localized YAML label and the parsed `title` when one exists. Focusing the row, expanding the source toggle, or placing the caret in the header SHALL reveal the YAML source through the collapsible payload editor. Invalid YAML SHALL remain editable in that editor and SHALL NOT demote the row to source-island chrome. Showing, collapsing, or hovering the header SHALL NOT increment document version or invalidate per-version derived caches. TOML and JSON front matter remain undetected.

#### Scenario: Valid YAML header stays rendered
- **WHEN** the document starts with `---\ntitle: Demo\n---\n\nBody`
- **THEN** Visual Edit derives a FrontMatter block covering the YAML region
- **AND** the block has a FrontMatter payload editor and no FrontMatter source island
- **AND** the collapsed chrome includes the parsed title `Demo`

#### Scenario: Invalid YAML still edits as header source
- **WHEN** the document starts with `---\n: not mapping\n---\n`
- **THEN** Visual Edit still presents a FrontMatter payload editor over the authored header
- **AND** the row is not a source island

#### Scenario: Header interaction is version-stable
- **WHEN** the user expands, collapses, or hovers the YAML header without editing text
- **THEN** the document version and per-version derived caches stay unchanged

### Requirement: Reference-style block images stay rendered when focused
Visual Edit SHALL treat a single-line reference-style Markdown image that the parser classifies as a block image (`![alt][label]`, collapsed `![alt][]`, or shortcut `![alt]`) as rendered WYSIWYG: the image remains visible while focused and SHALL carry the same collapsible whole-span source payload used for inline `![alt](url)` images. The payload range SHALL be the complete authored image span. Width and alignment field controls SHALL NOT appear for reference-style images, and confirming those controls SHALL NOT rewrite the span into inline-destination form. Multiline, angle-bracket-only, or otherwise unprovable image spans MAY keep the Image source-island fallback.

#### Scenario: Focused reference image keeps the picture
- **WHEN** Visual Edit shows a document whose only image is `![alt][asset]` with a later `[asset]: pic.png` definition
- **THEN** the image block has an Image payload editor covering `![alt][asset]`
- **AND** focusing the image does not replace it with a source island

#### Scenario: Shortcut and collapsed reference forms are proven
- **WHEN** the parser emits a block image for `![alt]` or `![alt][]` on one line
- **THEN** Visual Edit attaches an Image payload editor over that authored span
- **AND** `source_island` is none

#### Scenario: Presentation controls stay inline-only
- **WHEN** the focused image is reference-style
- **THEN** width and alignment controls are not shown
- **AND** the source remains `![…][…]` (or shortcut `![…]`) rather than `![…](…)`

### Requirement: Ragged GFM tables keep cell editors
When a GFM table’s body or header rows have fewer or more cells than the separator, Visual Edit SHALL still attach a Table cell editor and SHALL keep the best-effort grid while focused. Missing cells SHALL map to empty UTF-8-safe insertion ranges at the end of that row’s authored content (before the newline). Extra cells SHALL remain editable. Visual Edit SHALL NOT demote a parseable ragged table to a Table source island solely because cell counts differ. Structural toolbar edits continue to replace the complete table source as one undoable mutation.

#### Scenario: Short body row stays a focused grid
- **WHEN** the document contains `| A | B | C |\n| --- | --- | --- |\n| 1 | 2 |`
- **THEN** Visual Edit derives a Table editor whose cell count matches the logical grid (including an empty third body cell)
- **AND** focusing the table does not paint source-island chrome

#### Scenario: Extra cells remain editable
- **WHEN** a body row has more pipe cells than the separator
- **THEN** those extra cells still have source ranges
- **AND** the table is not an island

### Requirement: Math Pending and Error keep the payload editor
Visual Edit SHALL present display and fenced math whose KaTeX (or equivalent) cache entry is Pending or Error through the math payload editor, with the payload forced visible until the render is Ready. Visual Edit SHALL NOT replace that block with source-island chrome because the formula has not rendered or has failed to render. If delimiter/payload bounds cannot be split, the payload editor SHALL cover the complete authored math span. Caret, hover, and expand/collapse of the payload SHALL NOT increment document version.

#### Scenario: Failed display math stays an editor
- **WHEN** a `$$…$$` or fenced math block’s render entry is Error
- **THEN** Visual Edit shows the error message and the LaTeX payload editor
- **AND** the block is not a source island

#### Scenario: Pending display math stays an editor
- **WHEN** a display math block’s render entry is Pending
- **THEN** Visual Edit shows the pending status and the payload editor
- **AND** the block is not a source island

## MODIFIED Requirements

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, YAML front matter, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, code, math including Pending/Error payload editors, diagrams, images including reference-style block images, tables including ragged grids, YAML front matter, task lists, footnote definitions and references, blockquotes, alerts, rules, HTML blocks), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (indented code, unclosed fences, multiline or otherwise unprovable images, definition lists, residual unsupported gaps)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. After this change the remaining primary gap SHALL be indented code blocks. The roadmap SHALL also track secondary gaps including unclosed or malformed fenced code, multiline or otherwise unprovable images, GFM definition lists, and residual unsupported gap bytes. YAML front matter, reference-style block images, ragged tables, and math Pending/Error states SHALL NOT remain on the roadmap.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gap (indented code blocks) is listed with priority, effort, target class, and implementation seam
- **AND** YAML front matter, reference-style block images, ragged tables, and math render-failure states are absent from the open-gap list

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example YAML front matter, reference-style block images, ragged tables, math Pending/Error payload editors, escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, GFM alerts, or Visual Edit task-list checkbox click)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example unclosed fenced code or GFM definition lists)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial
