## ADDED Requirements

### Requirement: Bounded preview image cache
Markion SHALL own decoded Markdown preview images (ordinary `![]()` images and HTML `<img>` sources presented in Split Preview, Read mode, and Visual Edit) in an application-level cache with a finite completed-byte budget and a finite entry capacity. Ready entries SHALL be reusable across tabs and document versions when the source identity matches. The cache SHALL distinguish pending, ready, and error states. Completed entries SHALL be evictable under memory pressure; pending entries MUST NOT be evicted solely to make room for new work.

#### Scenario: Identical source reuses a ready entry
- **WHEN** two preview presentations request the same local path or remote image URL without an intervening eviction
- **THEN** both use the same ready decoded image rather than decoding twice

#### Scenario: Budget pressure evicts oldest ready images
- **WHEN** inserting or completing an image would exceed the completed-byte budget
- **THEN** the cache evicts one or more least-recently-used ready entries until the new entry fits or the single image is rejected as too large
- **AND** pending entries are not chosen for eviction

#### Scenario: Oversized source is display-limited
- **WHEN** a preview image decodes to a bitmap whose longer edge exceeds the configured display maximum
- **THEN** the retained `RenderImage` is downsampled to that maximum edge with aspect ratio preserved before it is stored as ready

### Requirement: Preview images are released with their claimants
Each open tab SHALL claim the preview image sources referenced by its currently retained preview or visual blocks. Closing a tab, replacing its document, or clearing those block lists SHALL release that tab's claims. A ready cache entry with no remaining claims SHALL become eligible for immediate eviction. Markion SHALL drop the corresponding GPUI image resources when an entry is removed so decoded bitmaps do not remain retained solely by the UI framework.

#### Scenario: Closing a tab releases its image claims
- **WHEN** a tab that uniquely referenced a set of preview images is closed
- **THEN** those images have no remaining claimants and are removed from the Markion cache
- **AND** subsequent memory accounting no longer attributes their raster bytes as owned ready entries

#### Scenario: Shared image across two tabs survives one close
- **WHEN** two tabs claim the same image source and one of them closes
- **THEN** the ready entry remains available to the remaining tab

### Requirement: Preview image loading stays off the frame path
Image fetch and decode for Markdown preview SHALL run outside the synchronous GPUI frame and typing path. A missing cache entry SHALL present a non-blocking pending or fallback state until the background result arrives. Late completions SHALL only update a still-matching cache entry and request repaint; they MUST NOT mutate document text, document version, or derived Markdown caches.

#### Scenario: Decode completes after the tab closed
- **WHEN** a background image decode finishes after its requesting tab has been closed and no other tab claims that source
- **THEN** the result does not recreate a retained ready entry for an unclaimed source
- **AND** document state is unchanged
