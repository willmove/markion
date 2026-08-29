## ADDED Requirements

### Requirement: Shared HTML preview honors image dimensions
Standalone HTML blocks and Visual Edit inline HTML image atoms SHALL honor authored `width` and `height` attributes on `<img>` tags. Numeric values and `px` lengths SHALL be treated as CSS pixels; percentage values SHALL be applied to the loaded image's display width. Layout SHALL keep `max-width: 100%` so oversized hints still fit the pane. Read, Split Preview, and unfocused Visual Edit SHALL use the same sized image.

#### Scenario: README logo uses authored pixel size
- **WHEN** an HTML block contains `<img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">`
- **THEN** the preview and Visual Edit image present at 128 by 128 logical pixels (subject to max-width of the pane)
- **AND** pending or failed loads still show the existing alt/URL placeholder

#### Scenario: Width-only hint keeps aspect ratio
- **WHEN** an `<img>` has `width="200"` and no height
- **THEN** the rendered image is 200 CSS pixels wide and the height follows the decoded aspect ratio

### Requirement: Visual Edit HTML blocks keep rendered view when focused
Focusing a `VisualBlockKind::Html` row SHALL keep the shared HTML-parts rendering visible. The block SHALL expose a collapsible exact source payload editor over the complete authored HTML range (the same render-plus-payload pattern used for diagrams and display math). Expanding, collapsing, or hovering the source control SHALL NOT change document version or invalidate per-version derived caches. The block SHALL NOT be replaced by a bordered whole-block source island solely because it owns the caret.

#### Scenario: Focused HTML table stays rendered
- **WHEN** the caret enters a Visual Edit HTML block that renders as a table or image
- **THEN** the table or image remains visible
- **AND** an exact source payload for the authored HTML is available without replacing the rendered view

#### Scenario: HTML source payload is source-backed
- **WHEN** the user edits the HTML block's source payload
- **THEN** the mutation applies to `MarkdownDocument.text` through the existing dirty-state and undo path
- **AND** the next derived version re-renders the HTML parts from the updated source

### Requirement: HTML tables inside wrappers still render as grids
`html_preview_parts` SHALL resolve a `<table>` that appears anywhere inside a raw HTML block, not only when the block's trimmed text starts with `<table`. Prefix and suffix HTML SHALL remain as text/image parts. Nested tables that cannot be gridded SHALL flatten rather than panic.

#### Scenario: Centered wrapper around a table
- **WHEN** a raw HTML block is `<div align="center"><table><tr><td>A</td></tr></table></div>`
- **THEN** the preview and Visual Edit show a visual table grid containing `A`
- **AND** the table is not flattened to a single text run

#### Scenario: Caption after a table is kept
- **WHEN** a raw HTML block is `<table><tr><td>A</td></tr></table><p>caption</p>`
- **THEN** the parts include a table grid followed by a text part containing `caption`

### Requirement: Nested list and quote HTML uses the HTML-parts pipeline
HTML-only regions inside blockquotes and list items SHALL be emitted as `PreviewBlock::Html` in document order with source ranges that do not overlap the parent item's direct text, matching nested fenced-code partitioning. Those nested HTML blocks SHALL render through `html_preview_parts` (including images) in Read, Split Preview, and Visual Edit.

#### Scenario: HTML image inside a list item renders
- **WHEN** a list item contains a nested raw-HTML image such as `<p align="center"><img src="x.png" alt="X"></p>`
- **THEN** Read mode and Visual Edit show the image
- **AND** the list item's source range does not swallow the HTML block's source range

#### Scenario: HTML image inside a blockquote renders
- **WHEN** a blockquote contains an HTML-only paragraph with an `<img>`
- **THEN** the quote renders the image through the shared HTML-parts pipeline rather than flattened alt/URL text

### Requirement: Shared HTML preview preserves common document structure
HTML preview parts SHALL present `<h1>`–`<h6>` with heading typography derived from the rendered body size, `<ul>`/`<ol>`/`<li>` with bullet or decimal markers, and `<pre>` with preserved whitespace and code-slot font. Alignment SHALL honor `left`/`right`/`center` (attribute or `text-align`). Inline `color` from allowlisted hex/`rgb()` values and underline from `<u>` SHALL paint on text spans. These presentations apply in Read, Split Preview, and Visual Edit HTML blocks.

#### Scenario: HTML heading uses heading size
- **WHEN** a raw HTML block contains `<h1>Title</h1>`
- **THEN** the text `Title` renders at the document's H1 size, not body size

#### Scenario: HTML list shows markers
- **WHEN** a raw HTML block contains `<ul><li>one</li><li>two</li></ul>`
- **THEN** the preview shows two marked list items rather than unmarked wrapped lines

#### Scenario: HTML pre keeps spaces and newlines
- **WHEN** a raw HTML block contains `<pre>  a\n    b</pre>`
- **THEN** the preview shows the leading spaces and the line break instead of collapsing them to a single space

#### Scenario: Right alignment and underline
- **WHEN** a raw HTML block contains `<p align="right"><u>note</u></p>`
- **THEN** the text is right-aligned and underlined

### Requirement: HTML table cells render images and links
HTML `<td>`/`<th>` cells SHALL render complete `<img>` tags through the shared image pipeline and SHALL apply `<a href>` as link spans. Empty cells SHALL not result solely because the cell contained only an image tag.

#### Scenario: Image-only table cell
- **WHEN** a raw HTML table cell is `<td><img src="a.png" alt="A"></td>`
- **THEN** the cell shows the image `a.png`
- **AND** the table remains a grid

#### Scenario: Linked text in a table cell
- **WHEN** a cell contains `<a href="https://example.com">ex</a>`
- **THEN** the cell text `ex` is a link to that URL

### Requirement: Attributed supported inline HTML still renders
The Visual Edit supported inline-HTML subset SHALL still render when a recognized tag carries ignorable attributes `class`, `id`, or `clear`. The complete authored tags remain the reveal group. Attributes other than that ignorable set on a style tag SHALL keep the conservative-run path for that tag.

#### Scenario: Classed emphasis stays visual
- **WHEN** an unfocused prose block contains `text <em class="x">em</em> more`
- **THEN** `em` renders as emphasis with tags hidden
- **AND** the block does not collapse into a whole-block source island

#### Scenario: Classed br still breaks
- **WHEN** a prose block contains `a<br class="clear">b`
- **THEN** Visual Edit stacks `b` on the next visual line

### Requirement: Unsupported inline HTML is an inert atom
Unknown, stray, or unpaired inline HTML in a prose block SHALL appear as byte-exact source fragments (inert atoms) in the mixed layout and SHALL reveal that fragment's source range when the caret enters it. Visual Edit SHALL NOT promote the whole paragraph to a source island solely because it contains such tags. The editor SHALL NOT guess a rendered-tree mutation for those tags.

#### Scenario: Unknown tag stays in mixed layout
- **WHEN** an unfocused paragraph contains `Hello <span>x</span> world`
- **THEN** `Hello` and `world` remain rendered prose
- **AND** the `<span>` / `</span>` source is visible as fragments rather than a whole-block island
- **AND** focusing the paragraph does not replace it with a bordered source island

### Requirement: Angle-bracket autolinks are progressive-reveal
Visual Edit SHALL render pulldown-recognized angle-bracket autolinks (`<https://…>`, `<user@example.com>`) as links while unfocused and SHALL reveal the complete `<…>` source group when the caret or a selection endpoint enters that range.

#### Scenario: URL autolink renders
- **WHEN** a paragraph contains `<https://example.com>`
- **THEN** Visual Edit shows a link, not a whole-paragraph source island
- **AND** moving the caret into the construct reveals the authored `<https://example.com>` group

## MODIFIED Requirements

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, subscript, backslash-escaped ASCII punctuation, decoded HTML entity references that reconstruct against the parser, angle-bracket autolinks, and exactly recognized inline HTML in the supported subset. A backslash followed by an ASCII punctuation character SHALL render as the literal punctuation character with the backslash hidden as a marker. The supported inline-HTML subset SHALL consist of the style pairs `<em>`/`<i>`, `<strong>`/`<b>`, `<s>`/`<del>`/`<strike>`, `<code>`, `<mark>`, `<sub>`, and `<sup>`, plus the void line-break forms `<br>`, `<br/>`, and `<br />`, including when those tags carry only ignorable attributes `class`, `id`, or `clear`; their tags SHALL be hidden markers whose styling composes with Markdown formatting, and `<br>` SHALL render as an authored line break inside the inline flow. Supported links SHALL include reference-style links (full `[text][label]`, collapsed `[label][]`, and shortcut `[label]` forms) whose definitions appear elsewhere in the document, and pulldown-recognized angle-bracket autolinks: Visual Edit SHALL resolve reference-style links against the document's link reference definitions, while definitions inside fenced code blocks SHALL NOT create links. Resolving document-scoped definitions SHALL preserve exact in-block source ranges — rendering and reveal mappings for the block's own content remain byte-identical to a full-document parse. Moving the caret or a selection endpoint into a supported formatted construct — including an escaped-character group, an entity token, an autolink, or a supported inline-HTML element — SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown. Unknown, stray, or unpaired inline HTML SHALL present as inert source atoms in the mixed layout. Constructs whose source/display mapping is malformed, crossing, or otherwise ambiguous — including backslash sequences outside the proven subset and entity references that cannot be reconstructed — SHALL show conservative source runs for the affected slice and SHALL NOT guess a rendered-tree mutation.

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
- **WHEN** a prose block contains inline HTML outside the supported subset — an unknown tag, a tag carrying non-ignorable attributes, or an unpaired or crossing tag pair
- **THEN** Visual Edit shows those tags as byte-exact inert source fragments in the mixed layout
- **AND** the editor does not guess a rendered-tree mutation for that content
- **AND** the paragraph is not replaced by a whole-block source island
- **AND** inline `<img>` tags keep their existing image-atom rendering and mixed-path behavior

#### Scenario: Angle-bracket autolinks render as progressive-reveal links
- **WHEN** a prose block contains an angle-bracket autolink such as `<https://example.com>` or `<user@example.com>`
- **THEN** Visual Edit renders the autolink as a progressive-reveal link
- **AND** moving the caret into the construct reveals the complete authored `<…>` group

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** a prose block contains malformed, crossing, or byte-inexact inline syntax whose visible text cannot be reconstructed byte-exactly from the authored slice
- **THEN** Visual Edit preserves a source-backed transitional editing affordance for the affected slice
- **AND** the editor does not guess a rendered-tree mutation for that construct

### Requirement: Visual Edit whitespace activation
The system SHALL keep source-backed whitespace ranges available for exact caret mapping while treating whitespace between rendered blocks as passive layout until the source caret intentionally enters that range. When the source caret owns a whitespace row — whether because the user pressed Enter at the end of a paragraph (whose source range excludes the trailing newline) or because keyboard navigation moved the caret into a whitespace-only range — Visual Edit SHALL present the row as the same passive-height layout it uses when unfocused, plus a thin insertion caret line visually consistent with the caret in a paragraph or heading, and SHALL accept subsequent typed text at the exact source caret position. Visual Edit SHALL NOT wrap a whitespace row that owns the caret in a source-island box (border, padding, monospace styling, or differentiated background), because such chrome misrepresents ordinary inter-paragraph spacing as a code-like block. Source islands SHALL remain reserved for blocks whose source has no rendered visual form (frontmatter, unclosed or indented code, unsupported constructs) or for inline runs whose source/display mapping is ambiguous and therefore requires a conservative source-editing fallback. Rendered HTML blocks SHALL keep their HTML-parts view when focused and SHALL NOT use a whole-block source island as the default focused presentation.

#### Scenario: Clicking a passive gap between headings does not activate editing
- **WHEN** the Visual Edit caret belongs to a rendered heading and the user clicks the whitespace gap between that heading and another heading
- **THEN** the source selection and document content remain unchanged and the gap does not present an insertion caret

#### Scenario: Clicking a passive gap before a paragraph does not activate editing
- **WHEN** the Visual Edit caret belongs to a rendered block and the user clicks the whitespace gap between a heading and a paragraph
- **THEN** the source selection and document content remain unchanged and the gap does not become an editable typing area

#### Scenario: Structural Enter activates an insertion line
- **WHEN** the user presses Enter from a heading in Visual Edit and the structural edit creates a new source-backed insertion line
- **THEN** the owning visual row presents the caret and accepts subsequent typed text at the exact source position regardless of whether the parser retains the newline in the heading range

#### Scenario: Intentional source caret movement preserves whitespace editing
- **WHEN** keyboard navigation or reveal logic moves the source caret into an existing whitespace-only range
- **THEN** the owning whitespace row provides the source-backed editing affordance without recomputing the document's cached Markdown-derived state

#### Scenario: Whitespace row owning the caret renders a caret line, not a source island
- **WHEN** the source caret owns a whitespace row in Visual Edit — for example after creating a blank line by pressing Enter (so a second newline lands outside any paragraph range), or after pressing Down arrow across an existing blank line
- **THEN** the row is rendered as passive-height layout with a thin insertion caret line and no border, padding, monospace styling, or differentiated background
- **AND** typed text is inserted into the canonical Markdown source at the caret position through the same dirty-state, undo/redo, autosave, and per-tab isolation paths as any other edit

#### Scenario: Whitespace row not owning the caret remains passive
- **WHEN** a whitespace row does not own the source caret
- **THEN** it renders as passive layout without a caret, exactly as before, regardless of whether it owns the caret on other frames

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, HTML blocks, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links including angle-bracket autolinks, inline math, escaped punctuation, decoded entities, supported and inert inline HTML, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, code, math, diagrams, images, tables, task lists, footnote definitions and references, blockquotes, alerts, rules, HTML blocks with collapsible source), progressive-reveal WYSIWYG constructs (inline formatting, links, autolinks, inline math, escaped punctuation, decoded entities, supported and inert inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (front matter, indented code, unclosed fences, reference-style images, malformed tables, task-list checkbox interaction, definition lists, empty list items, math render-failure)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: Visual Edit renders HTML images
Visual Edit SHALL present raw-HTML images the same way Read mode does wherever Read mode renders them, and SHALL NOT collapse prose blocks into raw-source islands solely because they contain image tags. Standalone raw-HTML blocks containing `<img>` SHALL render through the shared HTML-parts pipeline (text, images, tables), honoring authored width and height, with a collapsible source payload when focused rather than a whole-block source island. Inline `<img>` tags inside paragraphs, headings, list items, blockquote leaves, and footnote text SHALL render as inline image atoms loaded through the same image pipeline as preview (workspace-relative paths, remote URLs, and data URIs), while the surrounding prose remains rendered and editable. Nested HTML-only regions inside lists and quotes SHALL render as HTML blocks through that same pipeline. Each inline image atom SHALL be source-backed: entering its byte-exact authored `<img>` tag range with the caret or a selection endpoint SHALL reveal the complete authored tag as one editable source run, and leaving the range SHALL restore the rendered atom without changing the document version. Prose blocks whose only inline HTML consists of complete `<img>` tags SHALL NOT use a whole-block HTML source island. When a prose block mixes `<img>` tags with other inline HTML, the image atoms SHALL still render and the non-image inline HTML SHALL appear as rendered supported tags or inert source fragments in the same mixed layout. Images inside HTML `<td>`/`<th>` cells SHALL render as images. Images inside GFM pipe-table cells SHALL present the flattened alt/URL text exactly as Read mode does.

#### Scenario: Standalone HTML image block renders
- **WHEN** an unfocused Visual Edit document contains a raw-HTML block such as `<p align="center"><img src="logo.svg" alt="Logo"></p>`
- **THEN** the block renders through the shared HTML-parts pipeline showing the image and honoring centering
- **AND** focusing the block keeps that rendered image visible and offers a collapsible exact HTML source payload

#### Scenario: Inline HTML image renders inside prose
- **WHEN** an unfocused Visual Edit paragraph, heading, list item, or blockquote line contains text and one or more complete `<img>` tags
- **THEN** each tag renders as an inline image atom between the surrounding rendered prose runs
- **AND** the block does not present a whole-block raw-source island

#### Scenario: Focused inline image reveals its exact source
- **WHEN** the caret or a selection endpoint enters the authored `<img …>` source range of an inline image atom
- **THEN** the complete byte-exact tag is revealed as one editable source run
- **AND** moving the caret out restores the rendered atom without a document-version change

#### Scenario: Mixed inline HTML renders images beside conservative source fragments
- **WHEN** a prose block mixes one or more `<img>` tags with other inline HTML such as `<a href=…>` wrappers, `<br>`, or `<em>…</em>`
- **THEN** the block renders each image atom in the mixed layout
- **AND** supported non-image inline HTML renders and unknown tags appear as byte-exact fragments alongside the atoms
- **AND** the block does not collapse into a whole-block source island

#### Scenario: Other inline HTML keeps the conservative fallback
- **WHEN** a prose block contains supported inline HTML but no `<img>` tag (for example only `<br>` or `<em>…</em>`)
- **THEN** the block renders that HTML with hidden tags
- **AND** no partial rendering mutates or misrepresents the authored source

#### Scenario: HTML image in a table cell matches Read mode
- **WHEN** a GFM table cell contains a complete `<img>` tag and the table contains no other inline HTML
- **THEN** the table renders with the cell showing the flattened alt/URL text as Read mode does
- **AND** the table does not collapse into a whole-table source island

#### Scenario: Inline HTML images share the preview image lifecycle
- **WHEN** an inline HTML image is visible in Visual Edit
- **THEN** its URL is claimed, preloaded, and evicted through the same preview image cache lifecycle as block-level images
- **AND** pending and failed loads present the same placeholders as Read mode

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. After this change the primary gaps SHALL be (1) front matter (an editing form for YAML `---` regions, and detection of TOML/JSON forms) and (2) indented code blocks. The roadmap SHALL also track secondary gaps including unclosed or malformed fenced code, reference-style and malformed inline images, malformed tables, task-list checkbox click interaction, GFM definition lists, empty list items, and math render-failure states. Decoded entities, unsupported inline-HTML forms, and angle-bracket autolinks SHALL NOT remain on the roadmap once this change's implementation is complete.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gaps (front matter, indented code blocks) are listed with priority, effort, target class, and implementation seam
- **AND** each primary gap points at the source location of the current transitional source-view rendering

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, decoded entities, unsupported inline HTML atoms, angle-bracket autolinks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, or GFM alerts)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example task-list checkbox interaction)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial
