# diagram-rendering Specification

## Purpose
TBD - created by archiving change add-mermaid-diagrams. Update Purpose after archive.
## Requirements
### Requirement: Extensible diagram backend registry
The system SHALL provide a GUI-free diagram backend contract and registry in the `markion-diagram` workspace crate. Each backend SHALL expose a stable identifier, one or more fenced-language aliases, and static rendering through owned request/result/error types. Registry dispatch SHALL normalize the first fenced info-string token using ASCII case-insensitive matching, reject duplicate identifiers or aliases, and allow a new compile-time backend to be added by implementing the backend trait and registering an instance without changing Markdown parsing or GPUI types.

#### Scenario: Registered alias resolves to its backend
- **WHEN** a backend registers the alias `mermaid` and the registry receives a fenced info string whose first token is `Mermaid`
- **THEN** the registry resolves that fence to the registered backend using ASCII case-insensitive matching

#### Scenario: Duplicate registration is rejected
- **WHEN** two backends attempt to register the same normalized identifier or fenced-language alias
- **THEN** registry construction fails with a typed duplicate-registration error

#### Scenario: Member crate stays headless
- **WHEN** `cargo test -p markion-diagram` runs without GPUI or platform GUI libraries
- **THEN** the backend contract, registry, sanitization, and enabled backend tests compile and run successfully

### Requirement: Mermaid fenced diagrams render as static preview images
Markion SHALL register an in-process Mermaid backend and SHALL present a fenced code block as a Mermaid diagram when the first info-string token is `mermaid`, matched ASCII case-insensitively. Successful results SHALL be sanitized passive SVG rasterized by Markion and presented through GPUI's static image path in Split Preview and Read mode. The presented diagram SHALL reproduce the color channels the backend authored, SHALL include every text label the sanitized SVG declares, SHALL be rasterized above one device pixel per SVG unit so text and strokes remain sharp, and SHALL preserve the sanitized intrinsic aspect ratio at every preview width. Markion SHALL NOT require Node.js, Chromium, a WebView, a subprocess, network access, or temporary files for Mermaid preview.

#### Scenario: Valid Mermaid fence renders in Split Preview
- **WHEN** Split Preview contains a valid fenced block beginning with ` ```mermaid `
- **THEN** the preview displays the Mermaid backend's static diagram instead of syntax-highlighted source

#### Scenario: Valid Mermaid fence renders in Read mode
- **WHEN** Read mode contains a valid Mermaid fenced block
- **THEN** the same static diagram semantics are used without making the preview editable

#### Scenario: Rendered diagram preserves authored colors
- **WHEN** a sanitized diagram SVG declares a fill or stroke color
- **THEN** the pixels presented to GPUI carry that color's channels in the byte order GPUI renders as BGRA
- **AND** a red-dominant authored color is not presented as a blue-dominant one

#### Scenario: Rendered diagram retains its text labels
- **WHEN** a sanitized diagram SVG contains `<text>` nodes, including nodes whose labels are CJK
- **THEN** the rasterizer resolves fonts through a populated font database and the presented diagram contains those labels

#### Scenario: Rendered diagram is rasterized for display sharpness
- **WHEN** a diagram with a known sanitized intrinsic size is rasterized for preview
- **THEN** the resulting image is supersampled above one device pixel per SVG unit
- **AND** it is presented at the sanitized intrinsic size rather than at its raw pixel dimensions

#### Scenario: Diagram is presented at its intrinsic size, not its raster size
- **WHEN** a supersampled diagram raster is presented in preview
- **THEN** the element is sized from the sanitized intrinsic size rather than the raster's pixel count
- **AND** a diagram that fits the preview column occupies exactly its intrinsic width and height

#### Scenario: Wide diagram scales down without distortion
- **WHEN** a diagram's intrinsic width exceeds the available preview width
- **THEN** the diagram scales down to the available width with its aspect ratio preserved and remains fully visible
- **AND** the diagram is never stretched or cropped

#### Scenario: Unknown diagram-like language remains code
- **WHEN** a fenced block uses an identifier for which no diagram backend alias is registered
- **THEN** it follows the existing ordinary code-block highlighting/fallback behavior

### Requirement: Diagram rendering is asynchronous, theme-aware, and memoized
Preview diagram rendering and rasterization SHALL execute outside the GPUI frame path and SHALL use a bounded application-level cache keyed by backend identifier, authored source, and light/dark diagram theme. Entries SHALL distinguish pending, ready, and error states; a ready entry SHALL carry the rasterized image together with the presentation size used to display it. Concurrent requests for the same key SHALL share one render; completed results MAY be reused across tabs and document versions. Diagram rendering, rasterization, and theme switching SHALL NOT mutate document text, increment the document version, invalidate Markdown-derived caches, or reparse the document.

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

### Requirement: Diagram failures preserve authored source
The diagram layer SHALL return typed, non-localized errors for unsupported backends, excessive input, invalid source, unsafe output, and backend failures. Preview SHALL map those categories through Markion localization and display an actionable error together with the authored diagram source. While a result is pending, preview SHALL display a localized loading placeholder. Failure or pending presentation SHALL preserve the block's source range and SHALL NOT modify the document.

#### Scenario: Invalid Mermaid source shows fallback
- **WHEN** the Mermaid backend rejects a fenced block as invalid
- **THEN** preview displays a localized render error and the exact authored diagram source instead of a blank or stale diagram

#### Scenario: Oversized diagram is rejected before backend execution
- **WHEN** a diagram source exceeds the configured registry source-size limit
- **THEN** the registry returns an input-too-large error without invoking the backend

#### Scenario: Pending diagram shows loading state
- **WHEN** a diagram render has been scheduled but has not completed
- **THEN** preview displays a localized loading state without blocking the GPUI frame

### Requirement: Diagram SVG output is passive and bounded
The diagram registry SHALL sanitize every backend SVG result before returning it to any consumer. It SHALL reject malformed output, active content, external resources, unsupported media types, and output exceeding the configured byte limit. Preview and HTML export SHALL consume the same sanitized bytes so a backend cannot bypass the safety boundary through a different presentation path.

#### Scenario: Active SVG content is removed or rejected
- **WHEN** a backend result contains scripts, event-handler attributes, interactive hyperlinks, or external resource references
- **THEN** the registry removes the unsafe content or rejects the result before preview or export can consume it

#### Scenario: Excessive output is rejected
- **WHEN** sanitized backend output exceeds the configured output-size limit
- **THEN** the registry returns a typed unsafe-output or output-too-large error and does not expose the bytes to consumers

### Requirement: Diagram blocks remain source-backed in Visual Edit
Visual Edit SHALL continue to treat a recognized diagram fence as a source-backed code island. Diagram preview state, pointer interaction, theme changes, and backend completion SHALL NOT rewrite the fenced Markdown source or add an ambiguous rendered-tree editing path.

#### Scenario: Mermaid fence uses a Visual Edit source island
- **WHEN** the user focuses a Mermaid fenced block in Visual Edit mode
- **THEN** the existing fenced-code source editing affordance is used and edits mutate the canonical Markdown source through normal document mutation paths

#### Scenario: Diagram completion does not create an edit
- **WHEN** a pending diagram finishes rendering while Visual Edit is active
- **THEN** the document text, dirty flag, undo history, and document version remain unchanged

