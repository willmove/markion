## MODIFIED Requirements

### Requirement: Visual Edit SHALL provide selection-contextual formatting controls
When Visual Edit owns a non-empty, exactly source-mapped text selection, Markion SHALL NOT automatically display a floating selection-formatting toolbar. Instead, invoking the existing Visual Edit context menu for the block that completely owns that selection SHALL expose a distinct formatting group containing Bold, Italic, Inline Code, and Link. The formatting group SHALL be available through both pointer and keyboard context-menu invocation, and eligible block operations SHALL remain reachable in the same menu. Invoking a formatting action SHALL revalidate the captured document version, exact selection, and safe source ownership before using the existing canonical Markdown mutation, semantic undo, selection, autosave, and exact UTF-8 source paths. Merely opening, positioning, navigating, or dismissing the menu SHALL NOT change document state or invalidate derived caches.

#### Scenario: Selecting visual text does not open a floating toolbar
- **WHEN** the user creates a non-empty, exactly mapped prose selection in Visual Edit
- **THEN** the selection remains visibly highlighted without automatically showing a floating formatting toolbar over the document
- **AND** the document content remains unobscured until the user explicitly invokes a context menu or another editor surface

#### Scenario: Right-click exposes all selection formatting actions
- **WHEN** the user right-clicks the Visual Edit block that completely owns an exactly mapped non-empty selection
- **THEN** the context menu shows Bold, Italic, Inline Code, and Link in a distinct formatting group
- **AND** block transforms and operations that are valid for the same target remain reachable in that menu

#### Scenario: Keyboard context invocation exposes the same actions
- **WHEN** an exactly mapped non-empty selection is owned by the keyboard context-menu target in Visual Edit
- **THEN** the keyboard-invoked context menu exposes the same selection formatting group as pointer invocation
- **AND** keyboard navigation can reach and invoke each enabled formatting action

#### Scenario: Context-menu action formats visual text atomically
- **WHEN** the user invokes Bold, Italic, or Inline Code from the selection formatting group
- **THEN** the corresponding canonical Markdown markers are changed through one semantic command
- **AND** one Undo restores the prior source and selection

#### Scenario: Context-menu link action opens the exact link editor
- **WHEN** the user invokes Link for an exactly mapped non-empty selection
- **THEN** the existing source-backed link editor opens with that selection as the proposed label
- **AND** canceling the link editor leaves source, version, history, dirty state, and derived-cache identity unchanged

#### Scenario: Unrelated or ambiguous selection does not expose unsafe actions
- **WHEN** the context-menu target does not completely own the active selection, or the selection crosses an ambiguous, conservative, math, or source-island boundary
- **THEN** the menu does not expose selection formatting actions for that range
- **AND** valid block operations and raw source editing remain available without collapsing or mutating the selection merely to open the menu

#### Scenario: Stale selection target is rejected
- **WHEN** the document version, exact selection, or safe source ownership changes after the context menu opens but before a formatting action is dispatched
- **THEN** the action is rejected and the stale menu closes without guessing a mutation
- **AND** no additional source, history, dirty-state, or derived-cache change is introduced

#### Scenario: Selection context-menu lifecycle is presentation-only
- **WHEN** the user opens, navigates, repositions, or dismisses a selection-aware Visual Edit context menu without confirming an action
- **THEN** canonical Markdown, document version, exact selection, history, dirty state, and derived-cache identity remain unchanged

