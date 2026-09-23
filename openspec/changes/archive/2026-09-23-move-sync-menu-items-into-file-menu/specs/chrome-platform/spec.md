## ADDED Requirements

### Requirement: Backup and Sync and Advanced Git Tools submenus live in the File menu
The File menu on both menu surfaces the application renders — the native OS menu bar and the in-window menu bar dropdown — SHALL expose the synchronization entries as two submenus placed after the save group: a "Backup and Sync" submenu containing the open-sync-center entry (labeled "View Sync Status"), the "Version History" entry for the active document, "Sync Now", "Resolve Conflicts", and "Sync Setup"; and an "Advanced Git Tools" submenu containing "Commit", "Fetch", "Pull", and "Push". Version History SHALL NOT remain a top-level File menu entry, and the in-window surface SHALL keep its existing rule of offering Version History only when a text document is active. No top-level synchronization category SHALL remain on either surface, and the in-window menu bar order SHALL stay File, Edit, View, Format, Export, Help. Every submenu entry SHALL dispatch the same action with the same effective shortcut as before the move, and all submenu titles and entry labels SHALL be routed through the localization layer like every other menu string.

#### Scenario: File menu hosts both submenus on both surfaces
- **WHEN** the user opens the File menu in the native OS menu bar or the in-window menu bar
- **THEN** a "Backup and Sync" submenu and an "Advanced Git Tools" submenu appear after the save group
- **AND** the "Backup and Sync" submenu lists View Sync Status, Version History, Sync Now, Resolve Conflicts, and Sync Setup
- **AND** the "Advanced Git Tools" submenu lists Commit, Fetch, Pull, and Push
- **AND** Version History no longer appears as a top-level File menu entry

#### Scenario: Version History stays tied to the active document
- **WHEN** a text document is active and the user invokes Version History from the "Backup and Sync" submenu on either surface
- **THEN** the sync center opens at the file-history page for that document, as the former top-level File entry did
- **WHEN** no text document is active in the in-window menu
- **THEN** the Version History entry is not offered, matching its pre-move availability

#### Scenario: No top-level synchronization category remains
- **WHEN** either menu surface is rendered
- **THEN** no top-level synchronization menu category exists
- **AND** the in-window menu bar shows File, Edit, View, Format, Export, Help in that order

#### Scenario: Moved entries dispatch unchanged actions
- **WHEN** the user invokes any entry from the two File-menu submenus on either surface
- **THEN** the same action runs as before the move with its existing effective shortcut, opening the sync center for View Sync Status, opening the active document's file-history page for Version History, running synchronization for Sync Now, opening conflict resolution for Resolve Conflicts, opening sync setup for Sync Setup, and running the corresponding Git operation for each Advanced Git Tools entry

#### Scenario: In-window submenus behave like the existing nested flyout
- **WHEN** the in-window File menu is open and the user hovers a "Backup and Sync" or "Advanced Git Tools" parent row
- **THEN** its flyout opens to the side of the File dropdown
- **AND** opening one flyout closes the other nested flyouts of the File menu
- **AND** an outside click or choosing an entry dismisses the flyout and the File menu together

#### Scenario: Submenu labels follow the active language
- **WHEN** the active interface language changes
- **THEN** the two submenu titles and all their entry labels re-render in the new language on both surfaces
- **AND** the "Backup and Sync" submenu title and the feature wording inside setup, status, and the sync center remain the same existing localized strings, with no duplicate top-level title left behind

## REMOVED Requirements

### Requirement: Synchronization and source-layout menus are consolidated
**Reason**: The top-level "Sync" menu category this requirement mandated was removed; its entries now live in the File menu's Backup and Sync and Advanced Git Tools submenus (see the added "Backup and Sync and Advanced Git Tools submenus live in the File menu" requirement). The surviving source-layout consolidation is restated below without the retired top-level scenario.
**Migration**: Readers looking for the synchronization menu entries should consult the File menu requirement; the Source/Split Preview consolidation moved to "Source and Split Preview are one View-menu command".

## ADDED Requirements

### Requirement: Source and Split Preview are one View-menu command
The application SHALL expose Source and Split Preview as one "Source/Split Preview" command in both the native View menu and the in-window View menu. The combined command SHALL replace the separate Source and Split Preview rows without removing the distinct Visual Edit, Read, or all-mode cycle controls.

#### Scenario: View menus show one source-layout command
- **WHEN** the user opens the native or in-window View menu
- **THEN** exactly one "Source/Split Preview" command is shown in place of the separate Source and Split Preview commands
- **AND** the Visual Edit and Read commands remain available

#### Scenario: Combined menu command follows the source-layout toggle
- **WHEN** the user invokes "Source/Split Preview" from either View menu
- **THEN** Source changes to Split Preview, Split Preview changes to Source, and Visual Edit or Read changes to Source

#### Scenario: Existing all-mode cycle remains available
- **WHEN** the user invokes the existing all-mode cycle control
- **THEN** it continues to cycle through the complete existing view-mode order
