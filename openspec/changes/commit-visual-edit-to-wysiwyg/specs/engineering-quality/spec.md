## MODIFIED Requirements

### Requirement: Visual Edit invariant evidence
Every Visual Edit presentation or mutation strategy SHALL have executable evidence for each affected ownership layer: exact UTF-8 source ranges and WYSIWYG coverage classification in pure tests, canonical edit/version/id/selection behavior in document tests, rendered input/navigation/IME/history behavior in GPUI tests, and parser/export compatibility in workspace tests when semantics cross crate boundaries. A new visual block editor, a new WYSIWYG rendering for a previously gapped construct, or any change to the WYSIWYG coverage matrix MUST update the maintained coverage matrix and the `WYSIWYG coverage roadmap` (in the `markdown-editing` capability) before its change can be archived.

#### Scenario: A visual editor strategy is added or changed
- **WHEN** a change introduces or modifies a rendered editor, a progressive-reveal editor, or moves a construct out of the WYSIWYG coverage gap class
- **THEN** its proposal identifies source ownership and the resulting WYSIWYG coverage class
- **AND** its implementation updates the coverage matrix (and, if a gap was closed, the roadmap) and adds tests at every affected ownership layer

#### Scenario: A stale widget event arrives
- **WHEN** a direct-widget event targets an old document version, block identity, or field range
- **THEN** executable tests prove that the event is rejected before canonical source mutation

### Requirement: Markdown parser ownership
`pulldown-cmark` SHALL remain the root application's semantic Markdown parser and canonical preview-block classifier. Visual Edit boundary helpers MAY recognize exact field, payload, delimiter, and table-cell subranges only within an already-classified semantic block; they MUST round-trip the relevant authored source, MUST reuse the shared table implementation where applicable, and MUST NOT mutate an inferred rendered tree. When an exact range proof fails, the construct SHALL be classified as a WYSIWYG coverage gap under the `markdown-editing` capability's `WYSIWYG coverage roadmap` and shown as raw source only as a transitional affordance — the editor MUST NOT guess a rendered-tree mutation. Workspace member parsers and exporters MUST NOT create an independent Visual Edit mutation model.

#### Scenario: Exact boundary proof succeeds
- **WHEN** an already-classified block matches a supported byte-exact direct-editor form
- **THEN** the boundary helper returns typed UTF-8 ranges contained by that block
- **AND** the canonical semantic block and authored delimiters remain owned by the root parser/document model

#### Scenario: Boundary proof is ambiguous
- **WHEN** malformed, multiline, reference, nested, unclosed, or otherwise unsupported syntax prevents exact range proof
- **THEN** the helper returns no direct editor metadata
- **AND** Visual Edit classifies the construct as a WYSIWYG coverage gap and shows raw source as a transitional affordance
- **AND** the gap is tracked on the `WYSIWYG coverage roadmap` for closure by a future change
