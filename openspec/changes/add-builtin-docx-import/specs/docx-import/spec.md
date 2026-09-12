## ADDED Requirements

### Requirement: Word import SHALL be an explicit offline native workflow
Markion SHALL provide File → Import Word (.docx) in its native and in-app menus. The action SHALL accept one user-selected non-encrypted, non-macro DOCX package and convert it locally without requiring Word, LibreOffice, Pandoc, Node, a browser, or a network service. The importer SHALL validate the package type and required parts rather than trusting its extension. Existing File → Open, drag/drop, exports, and browser-session import behavior SHALL remain unchanged.

#### Scenario: Import works without external tools
- **WHEN** the user selects a supported DOCX while offline and no external converter is installed
- **THEN** native import produces a conversion report and offers to save the converted Markdown
- **AND** no external process, browser session, or network request is required

#### Scenario: Unsupported or disguised input is selected
- **WHEN** the input is legacy DOC, macro-enabled, encrypted, a non-DOCX archive, or lacks readable required DOCX parts even if named `.docx`
- **THEN** import reports the applicable unsupported-format or invalid-document error
- **AND** it creates no Markdown output or document tab

### Requirement: Conversion SHALL preserve supported text and list semantics
The importer SHALL preserve body text, paragraph order, explicit line breaks, heading levels 1–6, bold, italic, strikethrough, underline, superscript/subscript, safe hyperlinks, and supported bookmarks using Markion-compatible Markdown or limited safe HTML. Bookmark targets SHALL be standalone block HTML rather than inline heading content so Visual Edit does not expose anchor source as document text. Heading and numbering resolution SHALL honor direct properties and inherited styles with bounded cycle detection. Lists SHALL retain hierarchy, start/restart and continuation; significant numbering labels not representable as Markdown markers SHALL remain readable in item text with a normalization diagnostic. Markdown delimiters, URLs, table separators, and authored text SHALL be escaped by context without changing their visible meaning. Heading depths beyond six SHALL retain their text at level six with a simplification diagnostic.

#### Scenario: Styled multilingual manuscript is converted
- **WHEN** a document uses inherited heading styles, mixed Chinese/Latin text, literal Markdown delimiters, explicit line breaks, and adjacent differently formatted runs
- **THEN** the output preserves text, order, heading structure, supported emphasis, and explicit breaks
- **AND** literal characters do not become unintended Markdown structures

#### Scenario: Nested list restarts after a paragraph
- **WHEN** nested ordered and unordered lists include a non-one starting number, a continuation, and an explicit restart
- **THEN** the output preserves the intended nesting and numbering transitions
- **AND** any custom numbering normalization retains the significant label and is reported

#### Scenario: Malformed style inheritance is encountered
- **WHEN** styles contain a cycle or exceed the configured inheritance depth
- **THEN** conversion stops with a bounded structural-limit or invalid-document error rather than looping

#### Scenario: Word-generated bookmarks precede headings
- **WHEN** a paragraph contains authored or Word-generated `_Toc`/`_Ref` bookmark targets and layout-only leading breaks before heading text
- **THEN** the targets remain resolvable as standalone HTML blocks
- **AND** the heading contains its Markdown marker and visible title on the same line rather than inline anchor source or an empty heading

### Requirement: Table conversion SHALL retain content and cell structure
The importer SHALL convert supported rectangular tables to GFM tables without discarding body rows or inventing a semantic header from the first data row. Headerless tables SHALL use an empty GFM header. Supported horizontal/vertical merged cells SHALL be represented by one safe HTML table using colspan/rowspan that remains one CommonMark HTML block on LF and CRLF files. Unsupported nested or complex table structures SHALL preserve readable cell content where available and report structural simplification or content loss accurately.

#### Scenario: Headerless table contains literal pipes
- **WHEN** a rectangular source table has no header and its cells contain pipe characters and emphasis
- **THEN** the Markdown preserves every body row and the supported cell content with correct escaping
- **AND** the first body row is not relabeled as a source header

#### Scenario: Merged table is imported
- **WHEN** a supported source table has horizontal and vertical merged cells
- **THEN** the resulting HTML table preserves the cell spans and content
- **AND** Split Preview, Read, and Visual Edit recognize the output as one table according to existing HTML-table capabilities

### Requirement: Embedded images and footnotes SHALL remain usable after import
The importer SHALL return supported PNG/JPEG/GIF/WebP embedded images as separate bounded assets, preserving available alt text and document order. Floating placement SHALL be normalized and reported. Unsupported, absent, or corrupt image data SHALL produce a visible fallback and content-loss diagnostic instead of an unresolved hidden reference. Footnote references and definitions SHALL remain linked through deterministic Markdown labels, preserving repeated references, paragraphs, and supported inline content while excluding Word separator notes.

#### Scenario: The same embedded image is referenced twice
- **WHEN** two locations reference identical supported embedded image bytes
- **THEN** both locations remain present in the Markdown with their respective alt text
- **AND** the prepared import contains one reusable asset identity for those bytes

#### Scenario: A referenced image cannot be imported
- **WHEN** an embedded image is missing, corrupt, or uses an unsupported format
- **THEN** the report identifies the affected location and the content loss
- **AND** the proposed Markdown contains a visible fallback with available alt text

#### Scenario: A decorative DrawingML shape has no image relationship
- **WHEN** a run contains a non-picture DrawingML shape without image data or an image relationship
- **THEN** the importer does not count it as an embedded image
- **AND** it does not emit a missing-image placeholder or content-loss diagnostic for that decoration

#### Scenario: Multiple references share a footnote
- **WHEN** a footnote is referenced more than once and its definition contains multiple paragraphs
- **THEN** all references resolve to one Markdown definition with the preserved supported content
- **AND** separator notes do not become ordinary user footnotes

### Requirement: Equations and revisions SHALL have explicit conversion semantics
The importer SHALL convert a tested OMML subset covering text/symbol runs, fractions, radicals, sub/superscripts, delimiters, and sums/products/integrals with limits into validated inline or display LaTeX. Unsupported equation structures SHALL retain readable fallback text where available and produce content-loss diagnostics; the importer SHALL NOT fabricate an equivalent equation. Tracked changes SHALL use the accepted revision view: include inserted and move-to content, omit deleted and move-from content, and retain current formatting. This policy SHALL cover inline and paragraph/block boundaries and SHALL be disclosed in the report. Ambiguous revision semantics SHALL be reported as content loss.

#### Scenario: Supported inline and display equations are present
- **WHEN** an equation consists entirely of supported OMML constructs
- **THEN** the output preserves inline/display placement and yields LaTeX accepted by the existing math path

#### Scenario: An equation exceeds the supported subset
- **WHEN** a formula contains an unsupported structure that cannot be represented reliably
- **THEN** the report identifies its location as content loss
- **AND** continuation retains readable fallback text or a visible placeholder without claiming mathematical equivalence

#### Scenario: Revision acceptance includes moved text and paragraph boundaries
- **WHEN** the document includes insertions, deletions, moved content, and changed paragraph boundaries
- **THEN** the converted content matches the accepted revision view without duplicating move-from/move-to content
- **AND** the report states that tracked changes were resolved using that policy

### Requirement: Conversion fidelity SHALL be reviewed before saving
Every prepared import SHALL show a report with source name, recovered-content and image counts, revision policy, and diagnostics classified as informational normalization, formatting/structure loss, or content loss. Diagnostics SHALL identify the affected package part and a useful content location/context where available. The importer SHALL inventory nonempty unsupported content-bearing structures, including comments, endnotes, headers/footers, text boxes, embedded objects and `altChunk`, and SHALL NOT silently discard them. Fatal package errors and results with no recovered non-whitespace text, table content, image, or math SHALL disable saving; generated placeholders alone SHALL NOT count as recovered content.

The save action for content-loss results SHALL explicitly name continuation with the reported losses and SHALL require a deliberate action rather than default Enter-key acceptance. Cancel SHALL remain available. Reports with only informational or formatting diagnostics SHALL offer ordinary Save as Markdown without an additional loss confirmation.

#### Scenario: Unsupported content would otherwise disappear
- **WHEN** a readable document contains unsupported text boxes or an embedded object alongside supported paragraphs
- **THEN** the report lists the affected content and available fallback before saving
- **AND** the user must explicitly continue with those losses to select a destination

#### Scenario: Normal layout differences are summarized
- **WHEN** all document content is supported but fonts, colors, and page geometry are not preserved
- **THEN** the report summarizes the normalization and offers Save as Markdown
- **AND** it does not claim that the original page layout was preserved

#### Scenario: A document contains many cached Word fields
- **WHEN** TOC, HYPERLINK, PAGEREF, SEQ, or similar field instructions have readable cached results
- **THEN** the cached visible results are retained and executable field instructions are not emitted as document text
- **AND** repeated instructions are summarized by field kind instead of producing one report row per instruction

#### Scenario: No content is recoverable
- **WHEN** the reader produces only placeholders or no recoverable content
- **THEN** import reports an empty-result or unsupported-content failure
- **AND** it does not save an apparently successful empty document

### Requirement: Import work SHALL be bounded, cancellable, and free of external resource loading
All package reading, XML conversion, and media validation SHALL execute outside UI rendering and the typing path. Import SHALL enforce finite limits on source size, package entries, actual decompressed bytes, XML-part size/nesting, style depth, image counts/bytes/decoded dimensions, generated output, and elapsed conversion time. Limits SHALL be tested at their boundaries and checked during consumption rather than only against archive metadata. Invalid required XML, conflicting duplicate parts, exceeded limits, and conversion deadline expiration SHALL produce typed errors before output publication.

The importer SHALL expose cancellation before the publication boundary and check it between bounded work units. A canceled or superseded job SHALL NOT later publish files or create a tab. During the short commit boundary the UI SHALL show a saving state and disable cancellation until the outcome is known. Authored archive paths, external relationships, XML entities, macros, fields, and embedded objects SHALL NOT cause execution, archive traversal writes, or network/arbitrary host-file reads. Unsafe links SHALL retain readable text with diagnostics; safe user-clicked hyperlinks SHALL remain available. External images SHALL become diagnosed visible fallbacks, preventing import or subsequent preview from automatically loading them.

#### Scenario: Compressed size understates decompression work
- **WHEN** a small source expands past the decompressed-byte or structure limit
- **THEN** the importer stops while consuming bounded data and reports the exceeded limit
- **AND** existing tabs remain usable and no output is published

#### Scenario: Conversion is canceled while another tab is active
- **WHEN** the user switches tabs during conversion and then cancels the import
- **THEN** conversion stops at a bounded checkpoint and releases temporary data
- **AND** no late result writes files or changes either tab

#### Scenario: Source contains external and escaping relationships
- **WHEN** package relationships or entries target an external image, host file, traversal path, executable URL, or external entity
- **THEN** import performs no such fetch, read, execution, or traversal write
- **AND** it reports the rejected content or fatal package error as appropriate

### Requirement: A completed import SHALL create a separate saved Markdown document
After report acceptance, Markion SHALL request a new `.md` destination, suggest the source stem, and commit Markdown/resources through the managed import workflow. It SHALL reject destinations already open in a tab and existing Markdown/asset paths rather than overwrite them. The source Word file and every existing tab's content, version, dirty flag, selection, undo history, syntax memoization, and cached text/derived identities SHALL remain unchanged by import interactions. Following durable publication, Markion SHALL open and activate a new clean tab using its ordinary document, session, recent-file, and per-version caching lifecycle. If tab creation fails after publication, Markion SHALL identify the saved output path so the user can reopen it.

#### Scenario: Import completes while an existing tab has unsaved changes
- **WHEN** a reviewed import is saved to a new destination while another tab is dirty
- **THEN** a separate clean tab opens for the `.md` result
- **AND** the original DOCX and existing dirty tab retain their prior contents and state

#### Scenario: Destination selection is canceled
- **WHEN** the user cancels the destination picker
- **THEN** no Markdown, managed images, or new tab are created
- **AND** import-owned temporary data is released

#### Scenario: Saved output cannot be opened immediately
- **WHEN** durable publication succeeds but initializing the new tab fails
- **THEN** the status identifies the successfully saved Markdown path
- **AND** the application retains the committed files for normal reopening

### Requirement: Import interface strings SHALL use the application localization layer
Native and in-app menu labels, file filters, progress/cancellation, report sections and diagnostic templates, loss-continuation actions, errors, and saved-path statuses SHALL use the existing i18n entry point for every supported interface language. Diagnostic codes SHALL be independent of display language. Original document content and fallback excerpts SHALL NOT be translated.

#### Scenario: Interface language changes before import
- **WHEN** the user performs import in any supported interface language
- **THEN** every import control and diagnostic template is displayed in that language
- **AND** document text and source excerpts remain as authored

### Requirement: Claimed DOCX support SHALL have reproducible compatibility evidence
The project SHALL maintain a redistributable compatibility corpus with producer/version/provenance metadata and expected semantic content, revision views, diagnostics, and resource outcomes. It SHALL include representative Word, WPS, and LibreOffice files, native Markion export, MarkNice HTML-compatibility export, and synthetic corruption/limit/edge cases. Required content-preservation fixtures SHALL pass without silent loss before the feature is declared complete. Automated tests SHALL exercise the GUI-free converter, persistence failures, cancellation, and document/cache invariants; platform smoke evidence and diagnostic performance measurements SHALL record tested environments and outstanding gaps honestly.

#### Scenario: A reader change is evaluated
- **WHEN** the selected reader or conversion rules change
- **THEN** the same corpus asserts expected content/order/structure, diagnostics, and assets
- **AND** success cannot be inferred solely from parsing without an exception or round-tripping Markion-generated files

#### Scenario: Platform support is reported
- **WHEN** implementation completion or compatibility documentation is prepared
- **THEN** the evidence distinguishes automated results, manually verified producer/platform cases, and outstanding checks
- **AND** untested compatibility or timing is not reported as verified
