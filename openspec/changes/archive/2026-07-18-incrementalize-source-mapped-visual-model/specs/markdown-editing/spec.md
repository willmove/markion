## MODIFIED Requirements

### Requirement: Markdown parsing via CommonMark + GFM
The parser SHALL parse Markdown using `pulldown-cmark` configured for CommonMark conformance plus the GitHub Flavored Markdown extensions in use (tables, task lists, strikethrough, footnotes, superscript/subscript, highlight, autolinks). Parsing SHALL produce structured data consumed by the preview, Visual Edit, outline, stats, and search subsystems. Source-mapped Visual Edit derivation SHALL incrementally reuse independently parseable top-level regions after a localized source edit and SHALL fall back to a full-document parse whenever global Markdown context, region boundaries, or exact source ranges are uncertain. Incremental and fallback output SHALL be semantically and byte-range equivalent to a full parse of the current canonical source.

#### Scenario: Local edit reparses affected safe regions
- **WHEN** a source edit is confined to an independently parseable top-level region
- **THEN** source-mapped derivation reparses that region and bounded boundary context
- **AND** text-identical unaffected regions are reused without reparsing

#### Scenario: Globally scoped syntax uses full fallback
- **WHEN** an edit can affect reference definitions, footnotes, front matter, an unclosed fence, HTML block boundaries, or another cross-region parse dependency
- **THEN** the editor derives the source-mapped model through the full-document parser
- **AND** it does not publish a speculative incremental mapping

#### Scenario: Incremental output equals full parse
- **WHEN** incremental derivation accepts an edit sequence containing insertions, deletions, replacements, UTF-8 text, block splits, or block merges
- **THEN** its block variants, content, ordering, outline, and every source range equal a full parse of the same canonical source after each edit

#### Scenario: Extended inline syntax is recognized
- **WHEN** the document contains `==highlight==`, `^superscript^`, `~subscript~`, task list items, or footnote references
- **THEN** the parser recognizes these constructs and the preview renders them with their respective styles

#### Scenario: Nested Markdown constructs are preserved
- **WHEN** a block construct contains inline or nested constructs (e.g. a list with nested code, a blockquote with a table)
- **THEN** the parser handles the nesting per CommonMark precedence rules

## ADDED Requirements

### Requirement: Stable source-mapped visual block identity
Every derived Visual Edit block SHALL carry an opaque, non-persisted identity that remains stable across document versions only when the block is proven to descend unchanged from the same source block. Identity SHALL be independent from the block's current byte range and SHALL NOT replace canonical source ranges for editing.

#### Scenario: Prefix edit preserves shifted suffix identity
- **WHEN** a localized edit changes one block and shifts later unchanged blocks by a byte delta
- **THEN** each proven unchanged suffix block retains its prior visual block identity
- **AND** its source ranges are shifted to the exact current canonical offsets

#### Scenario: Changed block receives new identity
- **WHEN** an edit changes, splits, merges, or ambiguously reparses a visual block
- **THEN** every affected resulting block receives a new identity
- **AND** stale row layout, navigation, or widget state is not attached to it

#### Scenario: Repeated equal blocks remain occurrence-safe
- **WHEN** a document contains multiple textually equal blocks and an edit affects only one occurrence
- **THEN** identity reuse follows source-edit lineage and occurrence order
- **AND** an unchanged occurrence is not confused with the edited occurrence solely because their text hashes match

#### Scenario: Local edit invalidates only affected visual rows
- **WHEN** stable identities prove that visual rows outside an edited region are unchanged
- **THEN** the virtualized Visual Edit list splices only the affected middle rows
- **AND** unchanged row height and scroll anchoring state remain reusable

#### Scenario: Identity and incremental cache remain ephemeral
- **WHEN** a document is saved, reopened, recovered, cloned for undo, or replaced wholesale
- **THEN** visual identities and incremental region caches are rebuilt rather than persisted
- **AND** Markdown file contents and undo snapshot formats remain unchanged
