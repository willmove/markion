## ADDED Requirements

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
