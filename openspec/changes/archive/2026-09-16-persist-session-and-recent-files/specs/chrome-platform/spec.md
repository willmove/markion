## ADDED Requirements

### Requirement: Open Recent in the File menu
The editor SHALL provide an Open Recent entry in the in-window File menu that lists the bounded recent Markdown files from the session/recent store. Choosing a listed path SHALL open that document through the same open-document flow used by File → Open, including reuse of an already-open tab for the same path. The menu SHALL also provide a Clear Recent Files action. When the recent list is empty, the Open Recent surface SHALL show a localized empty-state placeholder and SHALL NOT invent fake paths.

#### Scenario: Recent files appear under File → Open Recent
- **WHEN** the recent-files list contains one or more Markdown paths and the user opens the in-window File menu
- **THEN** Open Recent lists those paths with the most recent first

#### Scenario: Choosing a recent file opens it
- **WHEN** the user chooses a path from Open Recent
- **THEN** that Markdown document opens in the editor through the existing open-document flow
- **AND** an already-open tab for the same path is reused when present

#### Scenario: Empty recent list shows a placeholder
- **WHEN** the recent-files list is empty and the user opens Open Recent
- **THEN** a localized empty-state placeholder is shown instead of file entries

#### Scenario: Clear Recent Files is available from the menu
- **WHEN** the user invokes Clear Recent Files from the Open Recent surface
- **THEN** the recent-files list is cleared
- **AND** subsequent Open Recent views show the empty-state placeholder
