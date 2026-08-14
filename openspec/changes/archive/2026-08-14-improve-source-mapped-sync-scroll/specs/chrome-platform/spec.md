## MODIFIED Requirements

### Requirement: Sync scroll preference
The editor SHALL provide a persisted "Sync scroll" preference, disabled by default, that when enabled and the active view mode is Split Preview SHALL couple the source editor and rendered preview by document location rather than by whole-document scroll percentage. Scrolling either pane by mouse wheel, trackpad, scrollbar drag, or an existing editor navigation action SHALL establish a source-backed viewport anchor at that pane's top content edge; the other pane SHALL align the corresponding source location at its own top content edge, except where clamping at the start or end of a scrollable range prevents exact top alignment. The mapping SHALL use rendered blocks' source ranges, SHALL interpolate relative progress within a source-backed block, and SHALL deterministically bridge source gaps that have no rendered content. The preference SHALL have no effect in Edit, Visual Edit, or Read mode, where both panes are not visible.

Synchronization SHALL be a no-op in a direction whose driving pane has no scrollable range or whose current preview mapping cannot identify a valid source location. When the preview list contains blocks for an older document version, synchronization SHALL NOT use those stale source ranges or force a Markdown parse; it SHALL retain the latest driving-pane intent and reconcile once the normal debounced preview update supplies a current mapping. A source-to-preview jump whose target virtual row has not been measured SHALL first reveal that row and then refine the within-row offset after layout, without falling back to whole-document percentage coupling or entering a feedback loop. Synchronization SHALL NOT reset the preview list, mutate the document, force a Markdown reparse, or disturb per-version derived-state caches.

#### Scenario: Sync scroll defaults to off
- **WHEN** the editor starts with no `sync_scroll` value in the preferences file
- **THEN** Sync scroll is disabled and the source editor and preview panes scroll independently as before

#### Scenario: Scrolling the editor aligns the corresponding preview content
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the source editor pane
- **THEN** the rendered preview aligns the block and relative block position corresponding to the source location at the editor viewport anchor
- **AND** the result is independent of unrelated differences between the panes' total scrollable heights

#### Scenario: Scrolling the preview aligns the corresponding source content
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the rendered preview pane
- **THEN** the source editor aligns the source location and relative block position represented at the preview viewport anchor
- **AND** the follower movement does not become a new preview-driving scroll on the next frame

#### Scenario: Non-uniform rendered blocks do not accumulate drift
- **WHEN** a Split Preview document contains blocks whose rendered heights differ substantially from their source heights, such as images, tables, wrapped prose, or code fences
- **AND** the user scrolls through multiple such blocks with Sync scroll enabled
- **THEN** each pane continues to show the same source-backed block near its viewport anchor instead of drifting according to total document percentage

#### Scenario: Source positions without rendered content bridge deterministically
- **WHEN** the editor viewport anchor falls in blank lines, link definitions, or another source interval with no independently rendered preview block
- **THEN** the preview target is derived from the adjacent source-backed block anchors
- **AND** continued scrolling across that interval does not jump to an unrelated document region

#### Scenario: An unmeasured preview target is refined after layout
- **WHEN** an editor-driven scrollbar jump targets a virtualized preview row that has not yet been measured
- **THEN** the preview first reveals the source-matched row
- **AND** after that row is measured, the preview refines its within-row offset to the source-mapped position without oscillating back to the editor pane

#### Scenario: Stale preview blocks defer source-mapped reconciliation
- **WHEN** the document version is newer than the source ranges represented by the debounced preview list
- **AND** the user scrolls either pane with Sync scroll enabled
- **THEN** the editor does not use the stale ranges and does not synchronously reparse Markdown
- **AND** once current preview blocks arrive through the normal debounce path, the latest driving pane reconciles the other pane by source location

#### Scenario: Sync scroll is inactive outside Split Preview
- **WHEN** Sync scroll is enabled but the active view mode is Edit, Visual Edit, or Read
- **THEN** scrolling the visible pane does not affect any other pane and the preference persists without error

#### Scenario: A pane with no scrollable range does not drive the other
- **WHEN** Sync scroll is enabled, the view mode is Split Preview, and one pane's content fits within its viewport
- **THEN** that pane does not move the other pane, and the other pane may still scroll independently

#### Scenario: Document boundaries remain clamped
- **WHEN** a source-mapped target would place either pane before its start or beyond its maximum scroll offset
- **THEN** that pane is clamped to the corresponding document boundary
- **AND** reaching the document start or end in the driving pane reaches the same boundary in the follower pane

