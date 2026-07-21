## MODIFIED Requirements

### Requirement: Visual Edit inline formatting fidelity
Visual Edit SHALL render byte-exact supported inline formatting in prose blocks without exposing its Markdown delimiters while the construct is unfocused. Supported formatting SHALL include emphasis, strong emphasis, safely nested strong/emphasis combinations, strikethrough, inline code, links, highlight, superscript, and subscript. Supported links SHALL include reference-style links (full `[text][label]`, collapsed `[label][]`, and shortcut `[label]` forms) whose definitions appear elsewhere in the document: Visual Edit SHALL resolve them against the document's link reference definitions, while definitions inside fenced code blocks SHALL NOT create links. Resolving document-scoped definitions SHALL preserve exact in-block source ranges — rendering and reveal mappings for the block's own content remain byte-identical to a full-document parse. Moving the caret or a selection endpoint into a supported formatted construct SHALL reveal one safe containing source group for precise editing without converting unrelated inline content in the same block to raw Markdown. Constructs whose source/display mapping is malformed, crossing, escaped, or otherwise ambiguous SHALL retain the conservative source-editing fallback.

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

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** a prose block contains escaped, malformed, crossing, or byte-inexact inline syntax that cannot be mapped safely
- **THEN** Visual Edit preserves a source-backed conservative editing affordance
- **AND** the editor does not guess a rendered-tree mutation for that construct
