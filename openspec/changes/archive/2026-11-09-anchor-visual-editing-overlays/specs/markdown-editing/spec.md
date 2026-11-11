## ADDED Requirements

### Requirement: Anchored automatic Visual selection toolbar
When Visual Edit owns a non-empty selection accepted by the existing exact selection-format target, the editor SHALL offer a compact automatic toolbar for Bold, Italic, Inline Code, and Link near the selection-owning block. The toolbar SHALL prefer placement above that block and flip below when necessary. It SHALL be hidden while dragging, after a document mutation until selection changes, and whenever an incompatible overlay is active. Existing context-menu formatting actions SHALL remain available. Every toolbar action SHALL revalidate document version, exact selected range, and block identity before dispatch and SHALL use the existing canonical command path.

#### Scenario: Selection shows nearby formatting
- **WHEN** the user finishes selecting exactly mapped Visual prose
- **THEN** Bold, Italic, Inline Code, and Link appear near the selection-owning block without changing document layout or state
- **AND** invoking an action performs the same single source-backed mutation as the corresponding context-menu action

#### Scenario: Toolbar lifecycle is non-mutating and stale-safe
- **WHEN** the toolbar appears, is positioned, or is dismissed without a command
- **THEN** document text, selection, version, dirty state, history, and derived cache identity remain unchanged
- **WHEN** the captured selection becomes stale before activation
- **THEN** no formatting mutation occurs

### Requirement: Document overlays anchor to their source interaction
Slash commands and document link, inline-math, and raw-token editors SHALL resolve an anchor from painted caret bounds, the source-owning VisualBlock bounds, or Visual input bounds in that order. Slash and inline editors SHALL open near the lower-left of the resolved source anchor. Non-document preference editors MAY use a bounded fallback. Anchoring SHALL NOT insert layout space or modify canonical document state.

#### Scenario: Slash follows the active query
- **WHEN** the user opens or filters a slash query in a painted Visual row
- **THEN** the palette appears near that row rather than at a fixed unrelated window position
- **AND** keyboard and pointer commands retain their existing exact behavior

#### Scenario: Inline editor follows its source target
- **WHEN** the user opens a document link, inline-math, or raw-token editor
- **THEN** the editor appears near its source interaction and uses the active theme palette
- **AND** canceling or dismissing it leaves document source and selection unchanged

### Requirement: Input suppresses transient block chrome
A canonical document change SHALL suppress hover/focus Visual block grips until subsequent pointer movement over the Visual surface. Suppression and re-arming SHALL be presentation-only and SHALL NOT change source, selection, version, history, or derived caches.

#### Scenario: Continuous input reduces visual noise
- **WHEN** the user types in a Visual block
- **THEN** transient block grips remain hidden until the pointer moves over the Visual surface again
- **AND** all block operations remain available through existing keyboard and context-menu paths
