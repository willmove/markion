## ADDED Requirements

### Requirement: Synchronization and source-layout menus are consolidated
The application SHALL label its top-level synchronization category as “Sync” and SHALL expose Source and Split Preview as one “Source/Split Preview” command in both the native View menu and the in-window View menu. The combined command SHALL replace the separate Source and Split Preview rows without removing the distinct Visual Edit, Read, or all-mode cycle controls.

#### Scenario: Top-level synchronization category uses the shorter title
- **WHEN** the native or in-window top-level menu is rendered
- **THEN** its synchronization category is labeled “Sync” in the active interface language
- **AND** Backup and Sync setup, status, settings, and sync-center content retain their existing descriptive wording

#### Scenario: View menus show one source-layout command
- **WHEN** the user opens the native or in-window View menu
- **THEN** exactly one “Source/Split Preview” command is shown in place of the separate Source and Split Preview commands
- **AND** the Visual Edit and Read commands remain available

#### Scenario: Combined menu command follows the source-layout toggle
- **WHEN** the user invokes “Source/Split Preview” from either View menu
- **THEN** Source changes to Split Preview, Split Preview changes to Source, and Visual Edit or Read changes to Source

#### Scenario: Existing all-mode cycle remains available
- **WHEN** the user invokes the existing all-mode cycle control
- **THEN** it continues to cycle through the complete existing view-mode order
