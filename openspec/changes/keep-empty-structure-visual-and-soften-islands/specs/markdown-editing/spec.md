## ADDED Requirements

### Requirement: Empty ATX headings and empty list items stay rendered
The derived Markdown model SHALL retain empty ATX headings (1–6 opening hashes, with or without following spaces and with no visible title text) as heading blocks with byte-exact source ranges, and SHALL retain empty unordered and ordered list items as list-item blocks with byte-exact source ranges. Visual Edit, Read, and Split Preview SHALL present each such block in its heading or list typography and SHALL reserve the same heading- or list-row height when the payload is empty, rather than omitting the row or replacing it with an Unsupported source island. When the Visual Edit caret or a selection endpoint owns an empty heading or empty list row, Visual Edit SHALL reveal the structural prefix (`#`–`######` plus any following spaces, or the list or task marker) through the existing prefix projection, keep heading or list typography, paint a caret in that row, and accept typed text into the canonical source. Visual Edit SHALL NOT wrap that row in source-island chrome (heavy padding, bordered card, monospace poster background). Empty paragraphs that are not headings or lists remain whitespace or gaps. Caret, focus, and prefix reveal SHALL NOT invalidate per-document-version derived Markdown caches.

#### Scenario: Empty ATX heading remains a heading block
- **WHEN** the document contains a line that is only an ATX marker such as `##`, `###`, or `###     ` (hashes plus optional spaces, any level 1–6)
- **THEN** derivation emits a heading block whose source range covers that line
- **AND** the corresponding Visual Edit block is `Heading` with no Unsupported source island
- **AND** Read and Split Preview reserve heading-row height for that line instead of omitting it

#### Scenario: Empty list item remains a list row
- **WHEN** the document contains a line that is only an unordered or ordered list marker such as `- ` or `1. `
- **THEN** derivation emits a list-item block whose source range covers that line
- **AND** the corresponding Visual Edit block is a list item with no Unsupported source island
- **AND** Read and Split Preview reserve list-row height including the list marker

#### Scenario: Focused empty heading reveals its marker
- **WHEN** the Visual Edit caret owns an empty ATX heading row
- **THEN** the structural prefix (`## ` or equivalent) is visible in heading typography
- **AND** the row is not replaced by a whole-block source island
- **AND** typed text inserts into the canonical Markdown source after the prefix

#### Scenario: Focused empty list item reveals its marker
- **WHEN** the Visual Edit caret owns an empty unordered, ordered, or task-list item whose payload text is empty
- **THEN** the list or task marker remains visible in list typography
- **AND** the row is not replaced by a whole-block source island

#### Scenario: Unfocused empty heading keeps placeholder height
- **WHEN** an empty ATX heading is visible in Visual Edit and does not own the caret
- **THEN** the row keeps heading-sized layout height with no source-island border, padding, or monospace poster chrome
- **AND** switching to Read or Split Preview keeps a heading-sized placeholder for that line

#### Scenario: Format or slash heading on an empty line stays visual
- **WHEN** the user turns an empty Visual Edit row into a heading (Format menu, block menu, or slash command), producing source such as `## `
- **THEN** the row renders as an empty heading with revealed prefix while it owns the caret
- **AND** it does not become an Unsupported source island

#### Scenario: Empty-structure presentation does not reparse
- **WHEN** the user moves the caret onto or off an empty heading or empty list row without changing document text
- **THEN** document version, dirty state, undo history, and derived Markdown caches remain unchanged

### Requirement: Remaining Visual Edit source islands use lightweight chrome
Visual Edit SHALL keep an exact source-backed editing affordance for constructs that still have no rendered form (YAML front matter, indented or unclosed fenced code without a payload editor, and residual unsupported gaps that are not empty headings or empty list items). That affordance SHALL use lightweight chrome: code-slot font, a faint distinct background or left accent, and tight padding that does not insert a large bordered card into the document flow. Visual Edit SHALL NOT use heavy poster chrome (large uniform padding, rounded bordered box, and a sudden height jump) for those remaining islands. Empty ATX headings and empty list items are not remaining islands; they follow `Empty ATX headings and empty list items stay rendered`.

#### Scenario: Residual unsupported gap is not a padded source card
- **WHEN** Visual Edit must present a residual unsupported gap that is not an empty heading or empty list item
- **THEN** the gap remains source-backed and editable
- **AND** it does not use the previous heavy padded bordered source-island card

#### Scenario: Front matter and unclosed fences stay source-backed
- **WHEN** the document contains YAML front matter or an unclosed fenced code block
- **THEN** Visual Edit still presents an exact source-backed editing affordance for that construct
- **AND** the affordance uses the lightweight island chrome rather than a large padded bordered card
- **AND** the construct remains a WYSIWYG coverage-roadmap gap until a later change closes it

## MODIFIED Requirements

### Requirement: Progressive Markdown marker reveal in Visual Edit
Visual Edit SHALL keep supported paragraph, heading, list-item, and blockquote content visually rendered while it is focused. When precise editing requires Markdown syntax, the editor SHALL reveal only the smallest complete inline syntax group whose source mapping is proven exact, while `MarkdownDocument.text` remains the canonical representation. Structural prefixes of headings and list items SHALL be revealed when the caret or a selection endpoint is inside the prefix range or at the prefix end, and SHALL be revealed for the whole prefix when a heading or list item with no visible content runs owns the caret. Display-to-source and source-to-display mappings SHALL remain UTF-8-safe and monotonic for pointer placement, selection, keyboard navigation, platform text input, and IME caret geometry. Syntax whose mapping is nested, overlapping, byte-inexact, or otherwise ambiguous MUST use a conservative source-backed edit island.

#### Scenario: Focusing plain prose preserves visual rendering
- **WHEN** the user places the caret in plain text inside a supported visual paragraph, heading, list item, or blockquote
- **THEN** the block remains in its rendered visual style
- **AND** the entire block is not replaced by raw Markdown source

#### Scenario: Active inline syntax is revealed locally
- **WHEN** the caret enters exactly mapped strong, emphasis, strikethrough, or inline-code content in a supported visual block
- **THEN** the complete markers for that active inline construct are revealed together with its content
- **AND** other supported content in the same block remains visually rendered

#### Scenario: Active link exposes its destination
- **WHEN** the caret enters an exactly mapped inline link label or its hidden source syntax
- **THEN** the local link syntax, including its destination and optional title, becomes visible and editable
- **AND** editing it mutates the corresponding canonical Markdown source range

#### Scenario: Leaving a reveal group hides its markers without mutation
- **WHEN** the caret or selection endpoints leave a locally revealed syntax group without editing document text
- **THEN** that group returns to its rendered representation
- **AND** the document version, dirty state, undo history, and derived Markdown caches remain unchanged

#### Scenario: Selection remains source-accurate across hidden markers
- **WHEN** a Visual Edit selection crosses rendered runs separated by hidden Markdown markers
- **THEN** the visual highlight represents the selected canonical source content across projected segments
- **AND** replacement, copy, cut, and formatting actions operate on the exact source selection

#### Scenario: Keyboard navigation into a hidden marker reveals it
- **WHEN** source-based keyboard navigation moves the caret into a currently hidden marker range
- **THEN** the next Visual Edit render reveals the owning syntax group
- **AND** subsequent caret geometry and input use an identity-mapped visible source position

#### Scenario: Caret at heading prefix end reveals the marker
- **WHEN** the Visual Edit caret is at the end of an ATX heading prefix (the first title position, including on an empty heading)
- **THEN** the heading prefix is revealed together with any visible title text
- **AND** the heading is not replaced by a whole-block source island

#### Scenario: Title-interior caret still hides the heading prefix
- **WHEN** the Visual Edit caret is inside the visible title of a non-empty ATX heading and not inside the prefix range
- **THEN** the heading prefix remains hidden
- **AND** the heading stays in heading typography

#### Scenario: Ambiguous inline syntax remains conservative
- **WHEN** an inline construct is nested, overlapping, escaped, transformed, or otherwise lacks a proven byte-exact mapping
- **THEN** Visual Edit uses a source-backed edit island for the affected block or construct
- **AND** it does not guess a rendered-tree mutation

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit WYSIWYG coverage matrix that classifies every user-visible Markdown construct into exactly one of three classes: **rendered WYSIWYG** (the construct is shown in its rendered form, including dedicated field/payload editors for code, math, diagrams, images, and tables whose editors ARE the rendered form), **progressive-reveal WYSIWYG** (the construct is rendered by default and reveals its smallest complete source syntax group when the caret enters it — inline formatting, links, inline math, structural prefixes), or **WYSIWYG coverage gap** (the construct currently shows raw source and is tracked under the `WYSIWYG coverage roadmap` for closure by a future change). The matrix SHALL name the canonical editable range and the verification evidence for each rendered/reveal class, and SHALL name the roadmap priority and implementation seam for each gap. The matrix SHALL agree with the stable requirements and the implemented `VisualBlock`/`VisualBlockEditor` behavior. Empty ATX headings and empty list items SHALL be classified as rendered WYSIWYG with progressive-reveal structural prefixes, not as coverage gaps.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit WYSIWYG coverage matrix
- **THEN** it distinguishes rendered WYSIWYG constructs (prose, headings including empty ATX headings, lists and task items including empty items, blockquotes, alerts, rules, HTML blocks, code, math, diagrams, images, tables, footnote definitions and references), progressive-reveal WYSIWYG constructs (inline formatting, links, inline math, escaped punctuation, supported inline HTML, structural prefixes, heading attributes), and open WYSIWYG gaps (decoded entities, front matter, indented code, unclosed fences, reference-style images, malformed tables, unsupported inline-HTML forms, autolinks, task-list checkbox interaction, definition lists)
- **AND** it explains that canonical Markdown remains the single persisted representation and that no construct is edited through a parallel rendered tree

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one of the three coverage classes for the construct
- **AND** if the proposal moves a construct out of the gap class, it updates the matrix and the `WYSIWYG coverage roadmap`
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated

### Requirement: WYSIWYG coverage roadmap
The repository SHALL maintain, as part of the Visual Edit WYSIWYG coverage matrix, a prioritized roadmap of every Markdown construct that is currently classified as a WYSIWYG coverage gap. The roadmap SHALL name, for each gap, the construct, its current rendering (transitional source view), its target WYSIWYG class (rendered or progressive-reveal), its priority, its rough implementation effort, and the implementation seam in the existing code. The roadmap SHALL be closed incrementally by future changes, each of which SHALL move one or more constructs out of the gap class and update this roadmap. The initial roadmap SHALL include at minimum the following primary gaps in priority order: (1) decoded HTML entities in prose blocks (for example `&amp;`), (2) front matter (an editing form for YAML `---` regions, and detection of TOML/JSON forms), and (3) indented code blocks. The roadmap SHALL also track secondary gaps including unclosed or malformed fenced code, reference-style and malformed inline images, malformed tables, unsupported inline-HTML forms and angle-bracket autolinks in prose, task-list checkbox click interaction, GFM definition lists, and math render-failure states. Empty ATX headings and empty list items SHALL NOT appear on the roadmap after this change.

#### Scenario: Primary gaps are tracked with priority and effort
- **WHEN** a contributor reads the WYSIWYG coverage roadmap
- **THEN** the current primary gaps (decoded HTML entities, front matter, indented code blocks) are listed with priority, effort, target class, and implementation seam
- **AND** each primary gap points at the source location of the current transitional source-view rendering

#### Scenario: Closing a gap updates the roadmap
- **WHEN** a future change implements WYSIWYG rendering for a construct that the roadmap tracks as a gap
- **THEN** that change's spec delta moves the construct out of the gap class in the `Maintained Visual Edit support classification` matrix and removes it from this roadmap
- **AND** the change's proposal cites this roadmap requirement as its motivation

#### Scenario: Closed gaps do not regress
- **WHEN** a construct previously tracked as a gap has been implemented as rendered or progressive-reveal WYSIWYG (for example escaped punctuation, the supported inline-HTML subset, standalone HTML blocks, reference-style links, inline-dollar math, footnote and link-reference definitions, heading attributes, GFM alerts, empty ATX headings, or empty list items)
- **THEN** the coverage matrix classifies the construct in its implemented class and the construct does not reappear on the roadmap

#### Scenario: Secondary gaps are visible but lower priority
- **WHEN** a contributor evaluates whether to pick up a secondary gap (for example task-list checkbox interaction or angle-bracket autolinks)
- **THEN** the roadmap lists the secondary gap with its effort and implementation seam
- **AND** the contributor can open a change that closes it without re-litigating whether it is a gap

#### Scenario: New gaps discovered in implementation are added to the roadmap
- **WHEN** implementation or testing reveals a Markdown construct that renders as raw source in Visual Edit and is not yet on the roadmap
- **THEN** the discovering change SHALL add the construct to this roadmap with its class, priority, effort, and seam before completing
- **AND** the change SHALL NOT close the gap in the same change unless the gap is trivial
