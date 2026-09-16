## MODIFIED Requirements

### Requirement: Bilingual project overview
The repository SHALL provide a root README whose English edition is followed by a complete Simplified Chinese edition, and SHALL retain a synchronized Simplified Chinese README for existing direct-language entry points. The English and Chinese editions SHALL present equivalent, current overviews of Markion's installation, implemented workflows, limitations, configuration, export behavior, Visual Edit WYSIWYG coverage (including the WYSIWYG coverage roadmap of known gaps), and contributor verification commands. The root README SHALL provide visible in-document language navigation, the standalone Chinese README SHALL link back to the root README, and both files SHALL link to the Visual Edit WYSIWYG coverage matrix. Stable capability purposes and project context metadata SHALL describe the current implemented architecture and MUST NOT characterize an archived capability as only future work.

#### Scenario: English reader opens the repository
- **WHEN** a reader opens `README.md`
- **THEN** the upper edition describes the current implemented application in English
- **AND** it provides visible navigation to the Simplified Chinese edition in the lower half of the same document
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

#### Scenario: README claims are checked against the project
- **WHEN** either README describes a feature, release package, configuration option, limitation, development command, or a Visual Edit WYSIWYG coverage class or gap
- **THEN** the claim matches the current stable OpenSpec requirements and implemented repository state

#### Scenario: Capability metadata is reviewed after archival
- **WHEN** an archived change makes a previously future capability part of the stable system
- **THEN** affected capability purposes and OpenSpec project context describe the implemented state
- **AND** no stable metadata contradicts the archived requirements
