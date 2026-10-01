# project-documentation Specification

## Purpose
Covers current bilingual project documentation, contributor commands, implemented capability claims, and links to detailed engineering contracts.

## Requirements

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

### Requirement: Local MarkNice workspace docs describe editor-skin parity, session-local Word import, and themed print-to-PDF
The bilingual READMEs and the local MarkNice workspace guide SHALL describe the workspace editor chrome as tracking the pinned MarkNice editor section, SHALL document Import Word as a browser-session replacement that does not write back to Markion and that requires Copy Markdown or another explicit Markdown save to recover into Markion, and SHALL document Save as PDF as printing the current themed sanitized preview via the browser print dialog, distinct from Markion's native or Pandoc PDF export. Those documents SHALL continue to state that Markdown-file import, PDF import, image import, and sample-document actions are out of this workspace scope.

#### Scenario: English README names the new workspace behaviors
- **WHEN** a reader opens `README.md`
- **THEN** the WeChat publishing workspace description includes editor-skin closeness to MarkNice, session-local Word import with a copy-or-save-Markdown recovery path, and themed print-to-PDF
- **AND** it still distinguishes browser Word/PDF from Markion native/Pandoc exporters

#### Scenario: Chinese README stays equivalent
- **WHEN** a reader opens `README.zh-CN.md`
- **THEN** it presents the same workspace editor-skin, Word-import, and print-to-PDF coverage in Simplified Chinese

#### Scenario: Workspace guide states the recovery and print semantics
- **WHEN** a reader opens the local MarkNice workspace guide
- **THEN** it states that Word import never mutates the Markion document
- **AND** it states that Save as PDF prints the themed preview and cannot prove a PDF was written
- **AND** it does not claim Markdown, PDF, or image import, or a sample-document action, as available workspace features

### Requirement: User docs SHALL name the Appearance preferences tab
English FAQ and bilingual READMEs that tell users where to pick a theme or change document typography SHALL name **Preferences → Appearance** (not Theme as a sibling tab, and not an undifferentiated Preferences panel for those controls).

#### Scenario: FAQ points at Appearance
- **WHEN** a reader opens the Themes section of `docs/faq.md`
- **THEN** it directs them to Preferences → Appearance

#### Scenario: README grouping matches the panel
- **WHEN** a reader opens `README.md` or `README.zh-CN.md`
- **THEN** theme and document typography are described as Appearance preferences rather than as Theme-tab-only or General-tab typography
