## RENAMED Requirements

### FROM: Source-backed Visual Edit mode
### TO: WYSIWYG Visual Edit mode

## MODIFIED Requirements

### Requirement: Editor view modes
The editor SHALL provide four mutually exclusive view modes: Edit (also surfaced as "Source"), Visual Edit, Split Preview, and Read. Source mode SHALL show the Markdown source editing surface without the rendered preview pane. Visual Edit mode SHALL show a single WYSIWYG editing surface where Markdown constructs are presented as close to their rendered result as the editor can edit through an exact, lossless source mutation, with constructs that cannot yet be rendered tracked as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement. Split Preview mode SHALL show the Markdown source editing surface and rendered preview pane together, preserving the current live-preview workflow. Read mode SHALL show the rendered Markdown preview without the source editing pane and SHALL NOT allow editing through the rendered preview.

#### Scenario: Edit mode shows only source editing
- **WHEN** the active view mode is Edit (also surfaced as "Source")
- **THEN** the source editing surface is visible and accepts normal editing operations
- **AND** the rendered preview pane is not visible

#### Scenario: Visual Edit mode shows one editable visual surface
- **WHEN** the active view mode is Visual Edit
- **THEN** the editor shows a single WYSIWYG editing surface where Markdown constructs render close to their preview appearance while remaining editable
- **AND** constructs that cannot yet be rendered are tracked as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement

#### Scenario: Split Preview mode shows both panes
- **WHEN** the active view mode is Split Preview
- **THEN** the source editing surface and rendered preview pane are both visible
- **AND** edits in the source surface continue to update the preview through the existing derived Markdown state

#### Scenario: Read mode shows only rendered Markdown
- **WHEN** the active view mode is Read
- **THEN** the rendered preview pane is visible without a source editing pane
- **AND** editing through the rendered preview is not permitted

#### Scenario: Mode switching preserves document state
- **WHEN** the user switches between Edit, Visual Edit, Split Preview, and Read for an open document
- **THEN** the document text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and tab identity are preserved
- **AND** derived preview blocks, outline, stats, syntax highlighting, visual edit blocks, and cached text handles continue to follow the existing per-document-version cache rules

### Requirement: Source-backed Visual Edit mode
The editor SHALL provide a Visual Edit mode whose default presentation contract is WYSIWYG (what you see is what you get): every Markdown construct SHALL be presented as close to its rendered result as the editor can edit through an exact, lossless source mutation. `MarkdownDocument.text` SHALL remain the single canonical editable representation — Visual Edit is a presentation and editing contract over that text, not a parallel rendered document model. Every Visual Edit mutation SHALL flow through the existing source-mutation path (dirty-state, undo/redo, autosave, recovery, per-tab isolation), and SHALL NOT edit an inferred rendered tree. Constructs that the editor currently cannot present in rendered form are classified as **WYSIWYG coverage gaps** under the `WYSIWYG coverage roadmap` requirement, not as accepted end states; each gap SHALL show raw source only as a transitional measure until a future change closes it. Math SHALL be rendered while unfocused and SHALL reveal its complete authored delimiter group when focused; it SHALL NOT be mutated through an inferred rendered formula tree.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Visual formatting actions remain source-backed
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

#### Scenario: Focused display math uses a source edit island
- **WHEN** the user focuses `$$...$$` or fenced `math` content in Visual Edit
- **THEN** that formula presents an editable payload containing its exact authored syntax alongside the rendered formula
- **AND** moving focus away restores formula rendering without changing the document version

#### Scenario: Complex constructs use conservative edit islands
- **WHEN** the user focuses a construct that the `WYSIWYG coverage roadmap` classifies as an open gap (for example a front-matter region, an indented code block, an unclosed code fence, or a paragraph containing decoded HTML entities)
- **THEN** the editor SHALL show the authored source as a transitional editing affordance and SHALL classify the construct against the roadmap
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit
- **AND** the gap SHALL be tracked for closure by a future change that moves the construct into rendered or progressive-reveal WYSIWYG

#### Scenario: Visual-only interaction does not reparse unnecessarily
- **WHEN** the user moves the cursor, changes selection, hovers text, or focuses a rendered editor or transitional source view without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, subscript, backslash-escaped ASCII punctuation, and exactly recognized inline HTML in the supported subset. A backslash followed by an ASCII punctuation character SHALL render as the literal punctuation character with the backslash hidden as a marker. The supported inline-HTML subset SHALL consist of the exact unattributed style pairs `<em>`/`<i>`, `<strong>`/`<b>`, `<s>`/`<del>`/`<strike>`, `<code>`, `<mark>`, `<sub>`, and `<sup>`, plus the void line-break forms `<br>`, `<br/>`, and `<br />`; their tags SHALL be hidden markers whose styling composes with Markdown formatting, and `<br>` SHALL render as an authored line break inside the inline flow. Supported links SHALL include reference-style links (full `[text][label]`, collapsed `[label][]`, and shortcut `[label]` forms) whose definitions appear elsewhere in the document: Visual Edit SHALL resolve them against the document's link reference definitions, while definitions inside fenced code blocks SHALL NOT create links. Resolving document-scoped definitions SHALL preserve exact in-block source ranges — rendering and reveal mappings for the block's own content remain byte-identical to a full-document parse. Moving the caret or a selection endpoint into a supported formatted construct — including an escaped-character group or a supported inline-HTML element — SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown. Constructs whose source/display mapping is malformed, crossing, or otherwise ambiguous — including backslash sequences or inline HTML outside the proven subset, decoded HTML entities, and angle-bracket autolink sources the link reveal validator cannot yet classify — SHALL be classified as WYSIWYG coverage gaps under the `WYSIWYG coverage roadmap` requirement and SHALL show raw source only as a transitional editing affordance until a future change closes the gap with a byte-exact projection.

#### Scenario: Default inline formatting paragraph stays visual
- **WHEN** the default welcome document is opened in Visual Edit mode and its Inline formatting paragraph is not focused
- **THEN** supported Markdown delimiters in that paragraph are hidden
- **AND** italic, bold, combined bold-and-italic, strikethrough, inline code, link, highlight, superscript, and subscript content is rendered with its corresponding visual style

#### Scenario: Reference-style link resolves against a document-level definition
- **WHEN** a prose block contains a reference-style link whose definition line appears in a different block of the same document
- **THEN** Visual Edit renders the link label with link styling and hides the reference brackets while unfocused, exactly as Split Preview and Read modes do
- **AND** moving the caret into the link reveals the complete local `[text][label]` source group for editing
- **AND** all in-block source ranges (runs, reveal groups, markers) are identical to those of a full-document parse

#### Scenario: Reference-style link forms all resolve
- **WHEN** a document defines a link reference and uses it via the full `[text][label]`, collapsed `[label][]`, or shortcut `[label]` form in Visual Edit
- **THEN** each form renders as a link rather than literal bracketed text

#### Scenario: Bracketed text inside fenced code does not become a link
- **WHEN** a fenced code block contains a line shaped like a link reference definition
- **THEN** that line does not register as a definition
- **AND** matching `[text][label]` prose elsewhere in the document remains literal text in Visual Edit

#### Scenario: Undefined reference remains literal
- **WHEN** a prose block contains `[text][label]` with no matching definition anywhere in the document
- **THEN** Visual Edit renders it as literal text, matching CommonMark behavior

#### Scenario: Nested formatting reveals one safe containing group
- **WHEN** the caret or a selection endpoint enters byte-exact nested strong/emphasis content in Visual Edit
- **THEN** the editor reveals one outermost containing Markdown source range without duplicating text
- **AND** source/display mappings remain monotonic and UTF-8 safe
- **AND** unrelated inline content in the same block remains rendered

#### Scenario: Extended inline markers remain source-backed
- **WHEN** the caret enters a valid highlight, superscript, or subscript construct in Visual Edit
- **THEN** the complete local delimiters are revealed for editing
- **AND** moving the caret away hides those delimiters and restores the visual style
- **AND** cursor-only reveal does not change the document version or invalidate cached visual blocks

#### Scenario: Escaped punctuation renders as literal text
- **WHEN** an unfocused prose block contains backslash-escaped ASCII punctuation such as `\*` or `\.`
- **THEN** the paragraph renders as normal prose showing the literal punctuation character, not a whole-block source island
- **AND** the backslash stays hidden while the rest of the paragraph remains rendered
- **AND** the rendering matches Split Preview and Read mode visible text

#### Scenario: Escaped construct reveals its authored group
- **WHEN** the caret or a selection endpoint moves into an escaped-character group such as `\*` (including the escaped-backslash form `\\`)
- **THEN** the complete authored backslash-plus-character source group is revealed for editing
- **AND** moving the caret away hides the backslash again and restores the literal rendering without changing the document version

#### Scenario: Escapes compose with Markdown formatting
- **WHEN** a prose block contains an escape inside other supported formatting, such as `**a \* b**`
- **THEN** the escaped character renders literally inside the styled construct with the backslash hidden
- **AND** entering the construct reveals one safe containing source group

#### Scenario: Inline HTML style pair renders with hidden tags
- **WHEN** an unfocused prose block contains an exact unattributed pair such as `text <em>em</em> more` or `a <strong>b</strong> c`
- **THEN** the paragraph renders as normal prose with the tagged content carrying the corresponding visual style
- **AND** the tags stay hidden and the block does not collapse into an HTML source island

#### Scenario: Inline HTML element reveals its complete source
- **WHEN** the caret or a selection endpoint moves into content between a supported inline-HTML tag pair
- **THEN** the complete element source — opening tag, content, and closing tag — is revealed as one group for editing
- **AND** moving the caret away hides the tags and restores the rendered form without changing the document version

#### Scenario: Inline `<br>` renders an authored line break
- **WHEN** an unfocused prose block contains a void `<br>`, `<br/>`, or `<br />` form
- **THEN** the paragraph renders the same stacked line-break layout it uses for authored hard breaks, without collapsing into an HTML source island
- **AND** caret activation of the tag reveals its authored source with pointer and keyboard resolution limited to the tag's safe source boundaries

#### Scenario: Unsupported inline HTML remains conservative
- **WHEN** a prose block contains inline HTML outside the supported subset — an unknown tag, a tag carrying attributes, an unpaired or crossing tag pair, or an HTML entity such as `&amp;`
- **THEN** Visual Edit preserves the whole-block source-backed transitional editing affordance and classifies the construct as a WYSIWYG coverage gap under the roadmap
- **AND** the editor does not guess a rendered-tree mutation for that content
- **AND** inline `<img>` tags keep their existing image-atom rendering and mixed-path behavior

#### Scenario: Angle-bracket autolinks are a tracked WYSIWYG gap
- **WHEN** a prose block contains an angle-bracket autolink such as `<https://example.com>` or `<user@example.com>`
- **THEN** Visual Edit keeps the paragraph on the source-backed transitional editing path because the link reveal validator only accepts bracketed link sources
- **AND** the construct is classified as a WYSIWYG coverage gap under the roadmap for closure by extending the link reveal validator

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** a prose block contains malformed, crossing, or byte-inexact inline syntax whose visible text cannot be reconstructed byte-exactly from the authored slice
- **THEN** Visual Edit preserves a source-backed transitional editing affordance and classifies the construct as a WYSIWYG coverage gap under the roadmap
- **AND** the editor does not guess a rendered-tree mutation for that construct

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, code, math, diagrams, images, tables, task lists, footnote definitions and references, blockquotes, alerts, rules, HTML blocks), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (decoded entities, front matter, indented code, unclosed fences, reference-style images, malformed tables, unsupported inline-HTML forms, autolinks, task-list checkbox interaction, definition lists, empty list items)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

## ADDED Requirements

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. The initial roadmap SHALL include at minimum the following primary gaps in priority order: (1) decoded HTML entities in prose blocks (for example `&amp;`), (2) front matter (an editing form for YAML `---` regions, and detection of TOML/JSON forms), and (3) indented code blocks. The roadmap SHALL also track secondary gaps including unclosed or malformed fenced code, reference-style and malformed inline images, malformed tables, unsupported inline-HTML forms and angle-bracket autolinks in prose, task-list checkbox click interaction, GFM definition lists, empty list items, and math render-failure states.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gaps (decoded HTML entities, front matter, indented code blocks) are listed with priority, effort, target class, and implementation seam
- **AND** each primary gap points at the source location of the current transitional source-view rendering

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, or GFM alerts)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example task-list checkbox interaction or angle-bracket autolinks)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial
