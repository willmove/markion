## ADDED Requirements

### Requirement: Document overlay presentation uses active theme and viewport bounds
Document overlay surfaces introduced or repositioned by `anchor-visual-editing-overlays` SHALL use the active theme palette and GPUI anchored positioning with viewport clamping. Overlay positioning and visibility SHALL NOT change document layout, text, selection, version, history, dirty state, or derived caches.

#### Scenario: Dark theme and constrained viewport
- **WHEN** a document overlay opens in a dark theme or near a viewport edge
- **THEN** text, borders, backgrounds, and controls remain readable and the overlay stays within the usable viewport
- **AND** no document row is inserted, resized, or reparsed to accommodate the overlay
