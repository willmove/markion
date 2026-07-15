# project-documentation Specification

## Purpose
TBD - created by archiving change refresh-bilingual-readme. Update Purpose after archive.
## Requirements
### Requirement: Bilingual project overview
The repository SHALL provide an English root README and a Simplified Chinese README that present equivalent, current overviews of Markion's installation, implemented workflows, limitations, configuration, export behavior, and contributor commands. Each README SHALL provide a visible link to the other language edition.

#### Scenario: English reader opens the repository
- **WHEN** a reader opens `README.md`
- **THEN** the document describes the current implemented application in English
- **AND** it links to `README.zh-CN.md`

#### Scenario: Chinese reader selects the Chinese edition
- **WHEN** a reader opens `README.zh-CN.md`
- **THEN** the document presents the same capability and limitation coverage in Simplified Chinese
- **AND** it links back to `README.md`

#### Scenario: README claims are checked against the project
- **WHEN** either README describes a feature, release package, configuration option, limitation, or development command
- **THEN** the claim matches the current stable OpenSpec requirements and implemented repository state

