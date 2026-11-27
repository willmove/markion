## ADDED Requirements

### Requirement: Formula rendering SHALL prioritize the active viewport
The native renderer SHALL request formulas from the visible region of the active Read, Split Preview, or Visual Edit list. Initial or unmeasured list regions SHALL use bounded lookahead. Offscreen formulas SHALL NOT repeatedly displace visible results in a document whose total formula count exceeds the cache capacity.

#### Scenario: Viewport remains stable in a formula-heavy document
- **WHEN** a document contains more formulas than the cache can hold and the visible working set fits within the cache bounds
- **THEN** visible formulas settle on their rendered results
- **AND** scrolling to another part of the document requests the formulas in the new viewport
