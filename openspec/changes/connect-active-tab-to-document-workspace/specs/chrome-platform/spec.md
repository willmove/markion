## ADDED Requirements

### Requirement: Active document tab connects to the document workspace
When more than one document tab is open, the application chrome SHALL render the document-tab controls over the document workspace rather than over the sidebar. Each document tab SHALL have rounded upper corners and square lower corners. The active tab SHALL use the active theme's document-surface fill, SHALL have no visible lower boundary separating it from the workspace, and SHALL visually connect to the shared source, visual-edit, split-preview, or read workspace below. Inactive tabs SHALL remain visually separated from that workspace. The treatment SHALL use existing active-theme palette values and SHALL preserve all existing tab actions and state indicators.

#### Scenario: Tabs align with the workspace when the sidebar is visible
- **WHEN** multiple document tabs and the sidebar are visible
- **THEN** the document-tab controls begin at the document-workspace boundary rather than above the sidebar
- **AND** resizing the sidebar keeps the tab controls aligned with that boundary

#### Scenario: Active tab opens into every document view mode
- **WHEN** a tab is active in Edit, Visual Edit, Split Preview, or Read mode
- **THEN** the tab has rounded upper corners and square lower corners
- **AND** its background continues into the shared document workspace without a lower border or accent line separating them
- **AND** in Split Preview the connection identifies both source and preview panes as content of the same active tab

#### Scenario: Inactive tabs remain distinct
- **WHEN** multiple tabs are visible
- **THEN** every inactive tab retains a visible boundary and subdued theme styling
- **AND** only the active tab appears connected to the document workspace

#### Scenario: Tab chrome follows the active theme
- **WHEN** the user switches among light, dark, or custom themes while multiple tabs are open
- **THEN** the active tab, inactive tabs, tab-band segments, borders, text, hover states, and workspace connection use the corresponding existing theme palette values

#### Scenario: Single-tab layout remains unchanged
- **WHEN** only one document tab is open
- **THEN** neither the document tab bar nor its sidebar-alignment segment is rendered
- **AND** no tab-band height or spacing is added to the workspace

#### Scenario: Existing tab interactions are preserved
- **WHEN** the user switches or closes a tab, creates a new tab, or views a dirty document tab
- **THEN** the existing click targets, close behavior, new-tab action, dirty marker, keyboard navigation, and per-tab document state behave as before

