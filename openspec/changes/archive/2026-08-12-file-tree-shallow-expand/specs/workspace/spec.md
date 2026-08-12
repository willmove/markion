## ADDED Requirements

### Requirement: Folder expansion reveals one level

When a folder is expanded interactively (by clicking it while collapsed), the file tree SHALL reveal only that folder's immediate children — its direct subfolders and the Markdown files it directly contains. Deeper subfolders SHALL remain collapsed until individually expanded, so each click drills down exactly one further level rather than opening the whole subtree at once. Collapsing a folder SHALL hide its entire subtree. This requirement governs interactive expansion only; the initial workspace-open collapse policy and filename filtering are unaffected.

#### Scenario: Expanding a collapsed folder reveals only its immediate children
- **WHEN** the user clicks a collapsed folder that contains nested subfolders and Markdown files
- **THEN** only that folder's direct children (immediate subfolders and the Markdown files it directly contains) become visible
- **AND** every deeper subfolder remains collapsed and its contents stay hidden

#### Scenario: Each click drills down exactly one more level
- **WHEN** the user clicks a now-visible collapsed subfolder that was revealed by the previous expand
- **THEN** only that subfolder's immediate children become visible
- **AND** levels deeper than it remain collapsed

#### Scenario: Collapsing a folder hides its entire subtree
- **WHEN** the user clicks an expanded folder
- **THEN** the folder's entire subtree is hidden, regardless of how deep individual descendants had been expanded

#### Scenario: Expanding a folder that contains only direct files
- **WHEN** the user clicks a collapsed folder that contains only Markdown files and no subfolders
- **THEN** those direct files become visible and no deeper structure is revealed
