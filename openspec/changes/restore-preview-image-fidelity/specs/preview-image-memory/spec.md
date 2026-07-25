## ADDED Requirements

### Requirement: Claimed image fidelity is never reduced by budget pressure
The preview image cache SHALL NOT reduce the resolution of a ready entry that has one or more claims. Budget enforcement SHALL prefer, in order: evicting unclaimed completed entries; retaining the incoming ready entry with bounded budget overshoot while the claimed set alone exceeds the budget; and only as a last resort downscaling the *incoming* raster (from its freshly decoded bitmap, never by resampling an already-resampled entry) before it is first shown.

#### Scenario: On-screen image stays sharp when later images load
- **WHEN** a claimed ready image is on screen and further images complete under byte-budget pressure
- **THEN** the existing entry's pixel dimensions are unchanged

#### Scenario: Decode resolution is independent of image count
- **WHEN** a document references many images
- **THEN** each image decodes toward the configured display edge cap, not a per-image share derived from the number of claimed keys

### Requirement: SVG preview sources render at HiDPI-adequate density
SVG preview images SHALL be rasterized with a 2× supersample of their intrinsic size (subject to the display edge clamp) and presented at intrinsic size, so 2×-scale displays receive full pixel density, matching the diagram pipeline's presentation approach.

#### Scenario: SVG on a HiDPI display
- **WHEN** an SVG image is presented in preview or Visual Edit on a 2×-scale display
- **THEN** its backing raster carries at least 2 device pixels per logical pixel of its presented size

## MODIFIED Requirements

### Requirement: Preview images are released with their claimants
Each open tab SHALL claim the preview image sources referenced by its currently retained preview or visual blocks. Closing a tab, replacing its document, entering dormancy, or clearing those block lists SHALL release that tab's claims. A ready entry whose claim count reaches zero SHALL be demoted to an unclaimed LRU entry — eligible for eviction under capacity or byte pressure — rather than removed immediately, so re-activating a tab reuses decoded images. GPUI image resources SHALL be dropped when an entry is actually evicted or removed.

#### Scenario: Tab switch reuses decoded images
- **WHEN** a tab is dormanted and re-activated without intervening budget pressure
- **THEN** its images present from cache without re-decoding or showing pending placeholders

#### Scenario: Unclaimed entries yield to budget pressure
- **WHEN** the byte budget is exceeded while unclaimed ready entries exist
- **THEN** unclaimed entries are evicted (oldest first) before any other measure
