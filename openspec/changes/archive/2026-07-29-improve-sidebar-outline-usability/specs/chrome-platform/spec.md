## MODIFIED Requirements

### Requirement: View modes and application chrome
The editor SHALL provide source, split, and preview view modes, a toggleable sidebar (file tree / outline), a visible in-window menu bar (File, Edit, View, Format, Export, Help) with click-outside-to-close behavior, and a status bar. When the sidebar is visible, its column SHALL begin directly below the menu bar and extend through both the document-tab band and main content region, while document-tab controls and document panes remain in the adjacent document-workspace column.

#### Scenario: View modes are switchable
- **WHEN** the user switches between source, split, and preview modes
- **THEN** the editor pane layout updates accordingly

#### Scenario: In-window menu bar and status bar
- **WHEN** the editor is running
- **THEN** a visible in-window menu bar and a status bar are present, and open menus close on outside click

#### Scenario: Visible sidebar occupies the workspace from its top edge
- **WHEN** the Files or Outline sidebar is visible
- **THEN** the sidebar begins immediately below the menu bar and its tab controls occupy the top of that column
- **AND** no empty document-tab-band spacer is rendered above the sidebar
- **AND** any visible document-tab controls begin in the adjacent document-workspace column and remain aligned when the sidebar is resized

#### Scenario: Hidden sidebar returns the full workspace width
- **WHEN** the sidebar is hidden
- **THEN** the document-tab controls and document panes use the full available workspace width below the menu bar
