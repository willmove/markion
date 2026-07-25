## ADDED Requirements

### Requirement: Open Recent in the File menu
The editor SHALL provide an Open Recent parent item in the in-window File menu that opens a nested submenu (next menu level). The submenu SHALL list the bounded recent Markdown files from the session/recent store (most recent first). Choosing a listed path SHALL open that document through the same open-document flow used by File → Open, including reuse of an already-open tab for the same path. The submenu SHALL also provide a Clear Recent Files action. When the recent list is empty, the Open Recent submenu SHALL show a localized empty-state placeholder and SHALL NOT invent fake paths. Recent file entries and Clear Recent Files SHALL NOT appear as siblings of New, Open, Save, or other primary File menu actions in the top-level File dropdown.

#### Scenario: Open Recent is a submenu parent in File
- **WHEN** the user opens the in-window File menu
- **THEN** Open Recent appears as a single parent item in the File dropdown
- **AND** recent file paths and Clear Recent Files are not listed as top-level File siblings

#### Scenario: Recent files appear under File → Open Recent submenu
- **WHEN** the recent-files list contains one or more Markdown paths and the user opens the Open Recent submenu
- **THEN** the submenu lists those paths with the most recent first

#### Scenario: Choosing a recent file opens it
- **WHEN** the user chooses a path from the Open Recent submenu
- **THEN** that Markdown document opens in the editor through the existing open-document flow
- **AND** an already-open tab for the same path is reused when present

#### Scenario: Empty recent list shows a placeholder
- **WHEN** the recent-files list is empty and the user opens the Open Recent submenu
- **THEN** a localized empty-state placeholder is shown instead of file entries

#### Scenario: Clear Recent Files is available from the submenu
- **WHEN** the user invokes Clear Recent Files from the Open Recent submenu
- **THEN** the recent-files list is cleared
- **AND** subsequent Open Recent submenu views show the empty-state placeholder

#### Scenario: Primary File actions stay reachable
- **WHEN** the recent-files list is at its configured capacity and the user opens the in-window File menu
- **THEN** Save, Save As, and later File actions remain visible in the top-level File dropdown without requiring the recent-files list to be scrolled as part of that same panel
