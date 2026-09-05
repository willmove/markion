## ADDED Requirements

### Requirement: Generated YAML front matter is structurally valid and lossless
Whenever Markion renders parsed metadata back to Markdown or constructs transient Markdown for a pandoc export, it SHALL serialize the complete front-matter mapping with one YAML-aware implementation. Recognized scalar fields, tags, and custom scalar, sequence, mapping, null, boolean, and numeric values MUST produce structurally valid YAML that parses back to the same `YamlFrontMatter` values. The implementation MUST NOT assemble YAML container values or title overrides by concatenating independently escaped line fragments.

#### Scenario: Single-element custom containers round trip
- **WHEN** custom metadata contains a one-element sequence or a one-entry mapping
- **THEN** rendered front matter parses successfully as one value beneath the original custom key
- **AND** the reparsed value equals the original sequence or mapping

#### Scenario: Nested and multiline metadata round trips
- **WHEN** metadata contains nested containers, multiline strings, Unicode line separators, YAML-looking scalars, quotes, backslashes, control characters, or text equal to `---` on its own line
- **THEN** the rendered front-matter block remains structurally valid
- **AND** parsing it returns values equal to the original metadata

#### Scenario: DOCX and PDF title overrides use canonical YAML serialization
- **WHEN** a pandoc DOCX or PDF export applies a title override containing newlines, carriage returns, quotes, backslashes, control characters, or a standalone `---` line
- **THEN** the transient Markdown input contains valid front matter whose parsed title equals the complete override
- **AND** the export input does not create an unintended YAML document boundary

#### Scenario: Export override does not rewrite the editor source
- **WHEN** a title override is applied while building transient pandoc input
- **THEN** only the typed metadata used for that export invocation changes
- **AND** the open document's canonical Markdown source, version, dirty state, and undo history remain unchanged

