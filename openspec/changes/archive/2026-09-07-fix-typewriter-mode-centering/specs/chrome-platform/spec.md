## MODIFIED Requirements

### Requirement: Focus mode and typewriter mode
The editor SHALL provide a focus mode that dims text outside the current paragraph and a typewriter mode that keeps the active caret's visual row at the vertical center of the currently edited surface. In Edit mode and the source pane of Split Preview mode, typewriter positioning SHALL use the current measured wrapped source layout and actual source viewport height. In Visual Edit mode, it SHALL use the current measured visual caret and virtual-list viewport geometry. The editor SHALL reconcile typewriter positioning after every typing action, caret navigation, enabling the mode, entering or switching an editable view or document, and layout-affecting width or typography change; it SHALL defer positioning until geometry for the current document version is available rather than applying stale geometry. Presentation-only boundary space SHALL make both the first and final editable rows centerable. Each mode SHALL be independently toggleable and persisted.

#### Scenario: Focus mode dims non-current paragraphs
- **WHEN** focus mode is enabled and the cursor is in a paragraph
- **THEN** text outside the current paragraph is rendered dimmed

#### Scenario: Typewriter mode centers measured source rows
- **WHEN** typewriter mode is enabled in Edit mode or the source pane of Split Preview mode
- **AND** the user types or moves the caret between hard lines or soft-wrapped visual rows
- **THEN** the active caret row is positioned within one current source line height of the source viewport's vertical center, except where the valid scroll range clamps the target
- **AND** the position is derived from current wrapped-layout geometry rather than hard-newline counts

#### Scenario: Typewriter mode centers the Visual Edit caret
- **WHEN** typewriter mode is enabled in Visual Edit mode
- **AND** the user types or moves the caret to another rendered row
- **THEN** the visible Visual Edit caret is positioned within one current caret-row height of the virtual-list viewport's vertical center, except where the valid scroll range clamps the target
- **AND** an initially unmeasured target row is revealed and then refined after its current caret geometry is measured

#### Scenario: Typewriter mode follows current layout geometry
- **WHEN** typewriter mode is enabled and a tab switch, view switch, pane or window resize, font change, or document edit invalidates the active editing surface's layout
- **THEN** centering waits for geometry that represents the active document's current version and current viewport
- **AND** stale geometry from a prior version, tab, view, width, or font does not change the visible scroll position
- **AND** the caret is recentered once the current geometry is available

#### Scenario: Typewriter mode supports the document tail without document mutation
- **WHEN** typewriter mode is enabled and the caret is on the final editable row
- **THEN** presentation-only trailing scroll room allows that row to approach the viewport center as closely as the valid scroll range permits
- **AND** centering does not change document text, dirty state, undo history, document version, or per-version derived Markdown cache identity

#### Scenario: Typewriter mode centers consecutive typing at document boundaries
- **WHEN** typewriter mode is enabled and the caret is on the first editable row, the final editable row, or any row between them
- **AND** the user types consecutive characters
- **THEN** each completed typing update leaves the caret row within one current caret-row height of the viewport's vertical center
- **AND** presentation-only leading and trailing space prevents natural scroll bounds from pinning the caret to the viewport top or bottom
- **AND** ordinary coarse reveal does not displace an already measured centered caret before refinement

#### Scenario: Typewriter mode leaves manual scrolling alone until caret activity
- **WHEN** typewriter mode is enabled and the user scrolls without typing or moving the caret
- **THEN** the editor does not continuously snap back to the caret
- **AND** the next typing or caret-navigation action recenters the caret

#### Scenario: Disabled typewriter mode preserves ordinary reveal behavior
- **WHEN** typewriter mode is disabled
- **THEN** source and Visual Edit surfaces do not force the active caret to the vertical center
- **AND** their existing minimum-distance caret reveal and manual scrolling behavior remain available

#### Scenario: Split Preview sync follows a typewriter source recenter
- **WHEN** typewriter mode and Sync scroll are enabled in Split Preview mode
- **AND** source caret activity recenters the source pane
- **THEN** the source pane is treated as the sync-scroll driver and the preview follows through the existing source-backed mapping
- **AND** follower reconciliation does not displace the centered source caret row

#### Scenario: Both modes persist
- **WHEN** the user toggles focus or typewriter mode
- **THEN** the choice is applied and persists across launches
