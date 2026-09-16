## MODIFIED Requirements

### Requirement: View mode switching shortcuts
The editor SHALL provide one platform-appropriate keyboard shortcut that alternates between Source and Split Preview, direct keyboard shortcuts for Visual Edit and Read, and MAY retain an existing shortcut that cycles through all view modes. The Source/Split Preview shortcut SHALL be `Ctrl+/` on Windows and Linux and `Cmd+/` on macOS. When invoked from Visual Edit or Read, it SHALL enter Source first.

#### Scenario: Direct shortcut enters Split Preview mode
- **WHEN** the active view mode is Source and the user presses the Source/Split Preview shortcut
- **THEN** the active view mode becomes Split Preview
- **AND** status feedback identifies Split Preview mode

#### Scenario: Combined shortcut returns to Source from Split Preview
- **WHEN** the active view mode is Split Preview and the user presses the Source/Split Preview shortcut
- **THEN** the active view mode becomes Source
- **AND** status feedback identifies Source mode

#### Scenario: Direct shortcut enters Edit mode
- **WHEN** the active view mode is Visual Edit or Read and the user presses the Source/Split Preview shortcut
- **THEN** the active view mode becomes Source
- **AND** a subsequent invocation enters Split Preview

#### Scenario: Direct shortcut enters Visual Edit mode
- **WHEN** the user presses the Visual Edit mode shortcut
- **THEN** the active view mode becomes Visual Edit
- **AND** status feedback identifies Visual Edit mode

#### Scenario: Direct shortcut enters Read mode
- **WHEN** the user presses the Read mode shortcut
- **THEN** the active view mode becomes Read
- **AND** status feedback identifies Read mode

#### Scenario: Mode shortcuts follow platform conventions
- **WHEN** the editor runs on macOS versus Windows/Linux
- **THEN** the view mode shortcuts use the same `secondary` modifier convention as other application shortcuts

#### Scenario: Source layout toggle preserves document state and caches
- **WHEN** the user alternates between Source and Split Preview with the combined shortcut
- **THEN** document text, dirty state, cursor and selection, undo and redo history, scroll positions, and tab identity remain unchanged
- **AND** derived Markdown state continues to follow the existing per-document-version cache rules
