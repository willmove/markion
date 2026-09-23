## REMOVED Requirements

### Requirement: Bilingual project overview
**Reason**: The application ships seven interface languages, and the project overview is expanding from two editions (English, Simplified Chinese) to seven; a bilingual-only requirement no longer describes the documentation set.
**Migration**: Replaced by the "Multilingual project overview" requirement. The existing combined root README (English + 简体中文) and the standalone `README.zh-CN.md` remain valid entry points unchanged in structure; five new standalone editions are added alongside them.

## ADDED Requirements

### Requirement: Multilingual project overview
The repository SHALL provide a root README whose English edition is followed by a complete Simplified Chinese edition, SHALL retain a synchronized standalone Simplified Chinese README for existing direct-language entry points, and SHALL provide a standalone README edition for each remaining interface language: Traditional Chinese (`README.zh-TW.md`), Japanese (`README.ja.md`), French (`README.fr.md`), German (`README.de.md`), and Spanish (`README.es.md`). All editions SHALL present equivalent, current overviews of Markion's installation, implemented workflows, limitations, configuration, export behavior, Visual Edit WYSIWYG coverage (including the WYSIWYG coverage roadmap of known gaps), and contributor verification commands. The root README SHALL provide visible language navigation to every edition, each standalone edition SHALL provide visible language navigation to the root README and to every other standalone edition with its own language marked as active, and every edition SHALL link to the Visual Edit WYSIWYG coverage matrix. Stable capability purposes and project context metadata SHALL describe the current implemented architecture and MUST NOT characterize an archived capability as only future work.

#### Scenario: English reader opens the repository
- **WHEN** a reader opens `README.md`
- **THEN** the upper edition describes the current implemented application in English
- **AND** it provides visible navigation to the Simplified Chinese edition in the lower half of the same document and to the standalone Traditional Chinese, Japanese, French, German, and Spanish editions
- **AND** it links to the Visual Edit WYSIWYG coverage matrix

#### Scenario: Chinese reader selects the Chinese edition
- **WHEN** a reader selects Simplified Chinese from the language navigation in `README.md`
- **THEN** the reader reaches a complete Simplified Chinese edition in the lower half of the same document
- **AND** that edition presents the same capability and limitation coverage as the English edition

#### Scenario: Chinese reader uses the standalone entry point
- **WHEN** a reader opens `README.zh-CN.md` directly or follows an existing link to it
- **THEN** the document contains the same current Simplified Chinese edition as the root README
- **AND** it links back to the English edition in `README.md`
- **AND** it links to the Visual Edit WYSIWYG coverage matrix

#### Scenario: Reader opens any other standalone language edition
- **WHEN** a reader opens `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, or `README.es.md`
- **THEN** the document presents the same capability and limitation coverage as the English edition in Traditional Chinese, Japanese, French, German, or Spanish respectively
- **AND** its language navigation links to the root README and to every other standalone edition, marking its own language as the active one
- **AND** it links to the Visual Edit WYSIWYG coverage matrix

#### Scenario: README claims are checked against the project
- **WHEN** any README edition describes a feature, release package, configuration option, limitation, development command, or a Visual Edit WYSIWYG coverage class or gap
- **THEN** the claim matches the current stable OpenSpec requirements and implemented repository state

#### Scenario: Capability metadata is reviewed after archival
- **WHEN** an archived change makes a previously future capability part of the stable system
- **THEN** affected capability purposes and OpenSpec project context describe the implemented state
- **AND** no stable metadata contradicts the archived requirements
