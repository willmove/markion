## ADDED Requirements

### Requirement: Maintained Visual Edit support classification
The repository SHALL maintain a current Visual Edit support matrix that classifies every user-visible Markdown construct as rendered direct editing, rendered editing with progressive source reveal, a dedicated field/payload editor, a passive exact source position, or a complete conservative source island. The matrix SHALL identify the canonical editable range, uncertainty trigger, and required verification evidence for each classification, and SHALL agree with the stable requirements and implemented `VisualBlock`/`VisualBlockEditor` behavior.

#### Scenario: Contributor evaluates current WYSIWYG coverage
- **WHEN** a contributor reads the Visual Edit support matrix
- **THEN** it distinguishes directly editable prose, inline source reveal, code/math/image/table editors, whitespace/passive positions, and HTML/front-matter/diagram/ambiguous fallbacks
- **AND** it explains that canonical Markdown remains the single persisted representation

#### Scenario: A new visual block behavior is proposed
- **WHEN** a proposal changes how a Markdown construct is presented or edited in Visual Edit
- **THEN** the proposal selects one support classification and names its exact fallback trigger
- **AND** implementation and documentation cannot be considered complete until the matrix and invariant evidence are updated
