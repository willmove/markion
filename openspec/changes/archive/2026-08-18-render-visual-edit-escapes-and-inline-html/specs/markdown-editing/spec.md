## MODIFIED Requirements

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, subscript, backslash-escaped ASCII punctuation, and exactly recognized inline HTML in the supported subset. A backslash followed by an ASCII punctuation character SHALL render as the literal punctuation character with the backslash hidden as a marker. The supported inline-HTML subset SHALL consist of the exact unattributed style pairs `<em>`/`<i>`, `<strong>`/`<b>`, `<s>`/`<del>`/`<strike>`, `<code>`, `<mark>`, `<sub>`, and `<sup>`, plus the void line-break forms `<br>`, `<br/>`, and `<br />`; their tags SHALL be hidden markers whose styling composes with Markdown formatting, and `<br>` SHALL render as an authored line break inside the inline flow. Supported links SHALL include reference-style links (full `[text][label]`, collapsed `[label][]`, and shortcut `[label]` forms) whose definitions appear elsewhere in the document: Visual Edit SHALL resolve them against the document's link reference definitions, while definitions inside fenced code blocks SHALL NOT create links. Resolving document-scoped definitions SHALL preserve exact in-block source ranges — rendering and reveal mappings for the block's own content remain byte-identical to a full-document parse. Moving the caret or a selection endpoint into a supported formatted construct — including an escaped-character group or a supported inline-HTML element — SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown. Constructs whose source/display mapping is malformed, crossing, or otherwise ambiguous, backslash sequences or inline HTML outside the proven subset, and text whose parser-visible form cannot be reconstructed byte-exactly from the authored slice SHALL retain the conservative source-editing fallback.

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
- **THEN** the complete local delimiters are revealed for source-backed editing
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
- **THEN** Visual Edit preserves the whole-block source-backed conservative editing affordance
- **AND** the editor does not guess a rendered-tree mutation for that content
- **AND** inline `<img>` tags keep their existing image-atom rendering and mixed-path behavior

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** a prose block contains malformed, crossing, or byte-inexact inline syntax whose visible text cannot be reconstructed byte-exactly from the authored slice
- **THEN** Visual Edit preserves a source-backed conservative editing affordance
- **AND** the editor does not guess a rendered-tree mutation for that construct
