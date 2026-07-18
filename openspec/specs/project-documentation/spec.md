# project-documentation Specification

## Purpose
Covers current bilingual project documentation, contributor commands, implemented capability claims, and links to detailed engineering contracts.
## Requirements
### Requirement: Bilingual project overview
The repository SHALL provide an English root README and a Simplified Chinese README that present equivalent, current overviews of Markion's installation, implemented workflows, limitations, configuration, export behavior, Visual Edit support/fallback behavior, and contributor verification commands. Each README SHALL provide a visible link to the other language edition and to the detailed Visual Edit engineering/support matrix. Stable capability purposes and project context metadata SHALL describe the current implemented architecture and MUST NOT characterize an archived capability as only future work.

#### Scenario: English reader opens the repository
- **WHEN** a reader opens `README.md`
- **THEN** the document describes the current implemented application in English
- **AND** it links to `README.zh-CN.md` and the Visual Edit support matrix

#### Scenario: Chinese reader selects the Chinese edition
- **WHEN** a reader opens `README.zh-CN.md`
- **THEN** the document presents the same capability and limitation coverage in Simplified Chinese
- **AND** it links back to `README.md` and the Visual Edit support matrix

#### Scenario: README claims are checked against the project
- **WHEN** either README describes a feature, release package, configuration option, limitation, development command, or Visual Edit fallback
- **THEN** the claim matches the current stable OpenSpec requirements and implemented repository state

#### Scenario: Capability metadata is reviewed after archival
- **WHEN** an archived change makes a previously future capability part of the stable system
- **THEN** affected capability purposes and OpenSpec project context describe the implemented state
- **AND** no stable metadata contradicts the archived requirements

