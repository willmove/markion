## MODIFIED Requirements

### Requirement: Diagram rendering is asynchronous, theme-aware, and memoized
Preview diagram rendering and rasterization SHALL execute outside the GPUI frame path and SHALL use a bounded application-level cache keyed by backend identifier, authored source, and light/dark diagram theme. Entries SHALL distinguish pending, ready, and error states; a ready entry SHALL carry the rasterized image together with the presentation size used to display it. The cache SHALL bound both the number of entries and the total completed raster bytes; when completing a ready entry would exceed the byte budget, least-recently-completed ready entries SHALL be evicted until the new entry fits, or the result SHALL be rejected as too large if a single raster exceeds the budget. Concurrent requests for the same key SHALL share one render; completed results MAY be reused across tabs and document versions. Pending entries MUST NOT be evicted solely to admit new work. Diagram rendering, rasterization, and theme switching SHALL NOT mutate document text, increment the document version, invalidate Markdown-derived caches, or reparse the document.

#### Scenario: Repeated frame reuses completed diagram
- **WHEN** multiple frames present the same backend, source, and theme without a document edit
- **THEN** the cached result is reused and neither the backend nor the rasterizer is invoked again

#### Scenario: Duplicate pending request is deduplicated
- **WHEN** the same diagram key is requested while its background render is still pending
- **THEN** no second backend render is launched and both presentations observe the pending entry

#### Scenario: Rasterization stays off the frame path
- **WHEN** a diagram cache entry is missing and a render is scheduled
- **THEN** sanitization and rasterization both complete on a background executor before the entry becomes ready
- **AND** no frame decodes or rasterizes diagram SVG while painting

#### Scenario: Theme switch uses an independent cache key
- **WHEN** the active Markion theme changes between light and dark while the document text is unchanged
- **THEN** the appropriate diagram theme result is requested or reused without reparsing Markdown or changing the document version

#### Scenario: Stale completion cannot overwrite document state
- **WHEN** a background diagram render completes after the authored fence has changed or its tab has closed
- **THEN** the result can populate only its immutable cache key and cannot replace newer preview blocks or mutate any document cache

#### Scenario: Completed raster byte budget is enforced
- **WHEN** completing a diagram raster would push total completed raster bytes over the configured budget
- **THEN** older ready entries are evicted until the new raster fits, or the completion is rejected if the single raster exceeds the budget
- **AND** pending entries remain resident
