## ADDED Requirements

### Requirement: Visual Edit footnote and link-definition fidelity
Visual Edit SHALL resolve footnote references against document-scoped footnote definitions during per-block inline parsing, rendering an unfocused footnote reference as superscript visible text without exposing the surrounding `[^` `]` markers as literal runs. Visual Edit SHALL present each footnote definition as a single source-backed block whose range covers the authored `[^label]:` marker and its definition body, and SHALL NOT split the marker into an Unsupported source island while emitting the body as an ordinary paragraph. Gaps that contain only link reference definition lines (`[label]: url`) SHALL remain source-backed and editable, and SHALL NOT be classified as Unsupported source islands with island chrome; fenced-code lines shaped like definitions remain excluded from definition collection.

#### Scenario: Notes sample footnote reference renders as superscript
- **WHEN** a Visual Edit document contains `text.[^links]` together with a later `[^links]: …` definition
- **THEN** the prose block's editable runs include a superscript footnote label and do not emit literal `[` / `^links` / `]` runs for that reference

#### Scenario: Notes sample footnote definition stays one block
- **WHEN** a document contains `[^links]: Links can point to project pages, files, and useful references.`
- **THEN** Visual Edit exposes one footnote-definition block whose source range covers both the `[^links]:` marker and the definition body
- **AND** that block is not an Unsupported source island
- **AND** the definition body is not also emitted as a separate ordinary paragraph block

#### Scenario: Link reference definition gap is not a source island
- **WHEN** a document ends with a standalone `[markion-repo]: https://github.com/willmove/markion` definition line that is not inside a fenced code block
- **THEN** Visual Edit covers that source with a non-island reference-definition block
- **AND** the corresponding reference-style link elsewhere in the document continues to resolve as a link

#### Scenario: Footnote stub collection does not shift in-block ranges
- **WHEN** Visual Edit appends document footnote definition stubs so a prose block's footnote references resolve
- **THEN** every editable run and reveal group for that prose block remains inside the block's own source range
