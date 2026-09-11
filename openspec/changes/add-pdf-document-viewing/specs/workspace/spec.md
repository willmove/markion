## ADDED Requirements

### Requirement: PDF files SHALL participate in workspace file handling
The Files panel SHALL include existing local files with the case-insensitive `.pdf` extension alongside supported Markdown, curated text, and image files. PDF discovery SHALL use the existing background scan, filename filtering, folder hierarchy, ignore rules, bounded-row rendering, selection, context actions, and current-file marking. Clicking a PDF SHALL open or focus its read-only PDF tab under the application-wide default open-target rule, while Ctrl/Cmd+click SHALL force a new tab. Path-backed PDF tabs inside a workspace SHALL participate in that workspace's ordered session snapshot and active-path selection under the same rules as other supported path-backed tabs.

#### Scenario: Workspace scan includes PDFs
- **WHEN** a workspace contains `.pdf`, `.PDF`, or mixed-case PDF files outside ignored and hidden directories
- **THEN** the background scan returns them as PDF file entries in their containing folders
- **AND** bounded row rendering and filename filtering remain unchanged

#### Scenario: File-tree PDF opens with the default target rule
- **WHEN** the user clicks a PDF file-tree row
- **THEN** Markion opens it in the tab chosen by the default open-target rule, or focuses an existing tab for the same path
- **AND** Ctrl/Cmd+click always uses a new-tab intent

#### Scenario: PDF context actions preserve tab identity
- **WHEN** a clean open PDF is renamed or moved through an existing Files-panel action
- **THEN** its tab path is remapped through the existing path-remap pipeline and the current-file marker follows it
- **AND** deleting that file resets or closes the corresponding read-only tab under the existing deleted-path policy without a dirty-document prompt

#### Scenario: Workspace snapshot restores PDFs
- **WHEN** a workspace snapshot records supported PDF paths and a PDF as its active path
- **THEN** the still-existing PDF paths reopen in their recorded order and the recorded PDF becomes active
- **AND** missing PDF paths are skipped without preventing other paths from restoring
