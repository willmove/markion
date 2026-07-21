## RENAMED Requirements

### FROM: Source-backed Visual Edit mode
### TO: WYSIWYG Visual Edit mode

## MODIFIED Requirements

### Requirement: Editor view modes
The editor SHALL provide four mutually exclusive view modes: Edit (also surfaced as "Source"), Visual Edit, Split Preview, and Read. Source mode SHALL show the Markdown source editing surface without the rendered preview pane. Visual Edit mode SHALL show a single WYSIWYG editing surface where Markdown constructs are presented as close to their rendered result as the editor can edit through an exact, lossless source mutation, with constructs that cannot yet be rendered tracked as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement. Split Preview mode SHALL show the Markdown source editing surface and rendered preview pane together, preserving the current live-preview workflow. Read mode SHALL show the rendered Markdown preview without the source editing pane and SHALL NOT allow editing through the rendered preview.

#### Scenario: Source mode shows only source editing
- **WHEN** the active view mode is Edit (Source)
- **THEN** the source editing surface is visible and accepts normal editing operations
- **AND** the rendered preview pane is not visible

#### Scenario: Visual Edit mode shows one editable WYSIWYG surface
- **WHEN** the active view mode is Visual Edit
- **THEN** a single editable WYSIWYG surface is shown where Markdown constructs render close to their preview appearance while remaining editable
- **AND** constructs that cannot yet be rendered are tracked as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement

#### Scenario: Split Preview shows source and rendered side by side
- **WHEN** the active view mode is Split Preview
- **THEN** the Markdown source editing surface and rendered preview pane are both visible
- **AND** live-preview editing in the source surface updates the rendered preview

#### Scenario: Read mode shows only the rendered preview
- **WHEN** the active view mode is Read
- **THEN** the rendered Markdown preview is visible without a source editing pane
- **AND** editing through the rendered preview is not permitted

### Requirement: WYSIWYG Visual Edit mode
The editor SHALL provide a Visual Edit mode whose default presentation contract is WYSIWYG (what you see is what you get): every Markdown construct SHALL be presented as close to its rendered result as the editor can edit through an exact, lossless source mutation. `MarkdownDocument.text` SHALL remain the single canonical editable representation — Visual Edit is a presentation and editing contract over that text, not a parallel rendered document model. Every Visual Edit mutation SHALL flow through the existing source-mutation path (dirty-state, undo/redo, autosave, recovery, per-tab isolation), and SHALL NOT edit an inferred rendered tree. Constructs that the editor currently cannot present in rendered form are classified as **WYSIWYG coverage gaps** under the `WYSIWYG coverage roadmap` requirement, not as accepted end states; each gap SHALL show raw source only as a transitional measure until a future change closes it. Math SHALL be rendered while unfocused and SHALL reveal its complete authored delimiter group when focused; it SHALL NOT be mutated through an inferred rendered formula tree.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Visual formatting actions edit Markdown source
- **WHEN** the user applies bold, italic, inline code, link, image, heading, list, task list, blockquote, or fenced-code formatting in Visual Edit mode
- **THEN** the editor updates the underlying Markdown markers in `MarkdownDocument.text`
- **AND** switching to Source mode shows Markdown source that represents the visual result

#### Scenario: Focused syntax can be exposed for editing
- **WHEN** the cursor enters visually formatted inline content whose hidden Markdown syntax is needed for precise editing
- **THEN** the editor SHALL reveal the smallest complete source syntax group for that focused content (progressive-reveal WYSIWYG)
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit

#### Scenario: Unfocused math is rendered in Visual Edit
- **WHEN** valid inline, display, or fenced math is visible in Visual Edit and neither its source range nor delimiter group is focused
- **THEN** inline math appears as a baseline-aligned formula atom and display math appears as a typeset block
- **AND** the authored Markdown remains the canonical content

#### Scenario: Focused inline math reveals one complete source group
- **WHEN** the caret or a selection endpoint enters an inline math source range in Visual Edit
- **THEN** the complete byte-exact delimiter group is revealed as one editable source range
- **AND** unrelated prose in the same block remains rendered

#### Scenario: Focused display math reveals its payload editor
- **WHEN** the user focuses `$$...$$` or fenced `math` content in Visual Edit
- **THEN** that formula presents an editable payload containing its exact authored syntax alongside the rendered formula
- **AND** moving focus away restores formula rendering without changing the document version

#### Scenario: Constructs in the WYSIWYG coverage roadmap show source as a transitional affordance
- **WHEN** the user focuses a construct that the `WYSIWYG coverage roadmap` classifies as an open gap (for example an HTML block, a front-matter region, or a block containing escaped punctuation or decoded entities)
- **THEN** the editor SHALL show the authored source as a transitional editing affordance and SHALL classify the construct against the roadmap
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit
- **AND** the gap SHALL be tracked for closure by a future change that moves the construct into rendered or progressive-reveal WYSIWYG

#### Scenario: Visual-only interaction does not reparse unnecessarily
- **WHEN** the user moves the cursor, changes selection, hovers text, or focuses a rendered editor or transitional source view without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, and subscript. Moving the caret or a selection endpoint into a supported formatted construct SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown (progressive-reveal WYSIWYG). Constructs whose source/display mapping is currently malformed, crossing, escaped, or otherwise ambiguous SHALL be classified as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement and SHALL show raw source only as a transitional affordance until a future change closes the gap with a proper bidirectional projection.

#### Scenario: Default inline formatting paragraph stays visual
- **WHEN** the default welcome document is opened in Visual Edit mode and its Inline formatting paragraph is not focused
- **THEN** supported Markdown delimiters in that paragraph are hidden
- **AND** italic, bold, combined bold-and-italic, strikethrough, inline code, link, highlight, superscript, and subscript content is rendered with its corresponding visual style

#### Scenario: Nested formatting reveals one safe containing group
- **WHEN** the caret or a selection endpoint enters byte-exact nested strong/emphasis content in Visual Edit
- **THEN** the editor reveals one outermost containing Markdown source range without duplicating text
- **AND** source/display mappings remain monotonic and UTF-8 safe
- **AND** unrelated inline content in the same block remains rendered

#### Scenario: Extended inline markers reveal on focus
- **WHEN** the caret enters a valid highlight, superscript, or subscript construct in Visual Edit
- **THEN** the complete local delimiters are revealed for editing
- **AND** moving the caret away hides those delimiters and restores the visual style
- **AND** cursor-only reveal does not change the document version or invalidate cached visual blocks

#### Scenario: Escaped or decoded inline syntax is a tracked WYSIWYG gap
- **WHEN** a prose block contains escaped punctuation, decoded HTML entities, smart-punctuation substitution, or otherwise byte-inexact inline syntax that the current projection cannot safely render
- **THEN** Visual Edit shows the authored source as a transitional affordance and classifies the construct as a WYSIWYG coverage gap under the roadmap
- **AND** the editor SHALL NOT guess a rendered-tree mutation for that construct
- **AND** the gap SHALL be tracked for closure by a future change that introduces a bidirectional escaped/decoded projection

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, code, math, diagrams, images, tables, task lists, footnotes, blockquotes, rules), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, structural prefixes), and open WYSIWYG gaps (escaped punctuation, decoded entities, inline HTML, HTML blocks, frontmatter, malformed variants of otherwise-supported constructs, footnote definitions, heading attributes, definition lists, alerts)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

## ADDED Requirements

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. The initial roadmap SHALL include at minimum the following primary gaps in priority order: (1) escaped punctuation, (2) HTML entity decoding and smart-punctuation substitution, (3) inline HTML embedded in prose, (4) standalone HTML blocks, (5) frontmatter (YAML, and detection of TOML/JSON forms). The roadmap SHALL also track secondary gaps including indented and malformed fenced code, inline-dollar math at block position, reference-style and malformed images, malformed tables, footnote definitions, heading attributes, task-list checkbox click interaction, and any GFM extensions not yet enabled for rendering.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the five primary gaps (escaped punctuation, entity decoding, inline HTML, HTML blocks, frontmatter) are listed with priority, effort, target class, and implementation seam
- **AND** each primary gap points at the source location of the current transitional source-view rendering

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example footnote definitions or task-list checkbox interaction)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial
