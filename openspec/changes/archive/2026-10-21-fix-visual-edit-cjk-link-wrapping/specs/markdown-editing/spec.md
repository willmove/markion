## ADDED Requirements

### Requirement: Visual Edit shapes ordinary linked prose
Visual Edit SHALL lay out ordinary URL and heading link prose through the source-backed shaped text surface when the block has no rendered atom, script alignment, authored HTML break, footnote navigation tooltip, or stable-mode link edit affordance requiring mixed layout. The shaped label SHALL retain source-mapped caret placement, editing, and secondary-modifier navigation.

#### Scenario: Japanese linked list wraps with its marker
- **WHEN** a Visual Edit list item contains Japanese prose and an ordinary inline URL or heading link
- **THEN** the text wraps at the available width on shaped text lines aligned with the list marker
- **AND** clicking the label places the caret in its mapped source range
- **AND** secondary-modifier clicking the label activates its destination
