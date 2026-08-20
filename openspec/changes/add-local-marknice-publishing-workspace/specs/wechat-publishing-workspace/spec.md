## Purpose

Defines the local browser workspace that turns an explicit snapshot of a Markion document into MarkNice-compatible WeChat rich text without embedding a WebView or making the hosted MarkNice service a runtime dependency.

## ADDED Requirements

### Requirement: Launching creates a one-way publishing snapshot
Markion SHALL let the user open a local WeChat publishing workspace for the active document. Each launch SHALL capture the active document's current in-memory Markdown, including unsaved edits, and initialize an independent browser session with that snapshot. Launching or using the workspace SHALL NOT mutate the Markion document, save it, increment its version, replace its selection, or invalidate its per-version derived caches. The browser workspace SHALL visibly disclose that content edits made there are session-local and are not synchronized back to Markion.

#### Scenario: Unsaved active content is handed off
- **WHEN** the active document contains unsaved Markdown and the user opens the WeChat publishing workspace
- **THEN** the browser workspace starts with that exact in-memory Markdown
- **AND** the document remains dirty and otherwise unchanged in Markion

#### Scenario: Untitled or empty document is supported
- **WHEN** the active document has no filesystem path or contains no text
- **THEN** the workspace still opens with an empty or untitled snapshot
- **AND** no save prompt is required solely to launch publishing

#### Scenario: Browser edits remain session-local
- **WHEN** the user edits the Markdown inside the publishing workspace
- **THEN** the preview and copied publishing output reflect the browser-session edit
- **AND** the active Markion document remains byte-identical to the launch snapshot

#### Scenario: Repeated launches are independent
- **WHEN** the user changes the Markion document after one publishing session was created and launches the workspace again
- **THEN** the new session receives the newer snapshot
- **AND** the earlier session does not silently change its Markdown or presentation settings

### Requirement: The publishing workspace is self-contained and offline-capable
The publishing workspace SHALL be loaded from static assets bundled with Markion. Its application shell, scripts, styles, fonts, Markdown renderer, math renderer, and other runtime dependencies SHALL load without contacting a CDN or the hosted MarkNice site. The workspace SHALL provide the pinned MarkNice publishing theme catalog, font-size and paragraph-spacing controls, desktop and phone previews, Markdown editing, and WeChat rich-copy action. Document-authored remote resources MAY require network access, but the workspace itself SHALL make no telemetry, update, OCR, temporary-image-upload, or hosted-service request.

#### Scenario: Workspace loads with external network unavailable
- **WHEN** Markion and the default browser are running without external network access
- **THEN** the workspace UI, theme catalog, Markdown rendering, math rendering, preview controls, and copy controls load from the local Markion origin
- **AND** no application runtime script, style, font, or renderer is requested from a remote host

#### Scenario: Pinned MarkNice compatibility corpus renders
- **WHEN** the workspace renders the maintained compatibility corpus for headings, lists, tables, code, math, links, and images under each bundled publishing theme
- **THEN** its publishing HTML matches the normalized golden output recorded for the pinned MarkNice bundle

#### Scenario: Remote document resources remain distinguishable from app dependencies
- **WHEN** the document contains an HTTP or HTTPS image
- **THEN** the workspace MAY request that authored URL for preview
- **AND** the request is not treated as an application dependency or hidden service call

### Requirement: Loopback sessions protect document content and local files
The local publishing origin SHALL listen only on a loopback interface and an operating-system-selected ephemeral port. A new session SHALL require an unguessable capability that is removed from the visible browser URL after a successful claim. Document and resource responses SHALL use no-store caching, and requests without a valid, live session SHALL disclose neither document bytes nor whether a candidate local path exists. Local resource routes SHALL serve only supported regular image files within the active document's associated asset directory after canonical containment checks; traversal, absolute-path injection, symlink escape, and unrelated local-file access SHALL be rejected.

#### Scenario: Non-loopback clients cannot connect
- **WHEN** the workspace service starts
- **THEN** it binds only to a loopback address rather than an all-interface or LAN-visible address
- **AND** the operating system chooses the listening port

#### Scenario: Session capability is claimed and removed from the URL
- **WHEN** the browser first opens a valid publishing-session URL
- **THEN** the workspace claims the capability before retrieving document content
- **AND** browser history and the subsequently visible location do not retain the capability

#### Scenario: Missing or invalid session is denied
- **WHEN** a request for document content or a protected resource has no valid live session
- **THEN** the service returns a non-success response without document content, filesystem paths, or path-existence details

#### Scenario: Traversal and symlink escape are denied
- **WHEN** a protected resource request resolves outside the canonical document asset directory through `..`, an absolute path, encoding, or a symlink
- **THEN** the service denies the request and returns no bytes from the target

#### Scenario: Dynamic content is not cached
- **WHEN** the service returns a document snapshot, session metadata, or a protected local resource
- **THEN** the response instructs the browser and intermediaries not to store it

### Requirement: Local images preview safely and copy with an explicit limitation
For a named document, supported images inside its document-associated asset directory SHALL preview through protected session resource URLs without exposing filesystem paths to the page. A local reference that is missing, unsupported, or outside that directory SHALL render as unresolved and SHALL produce a visible warning. Because this change provides no remote image publication backend, the rich-copy action SHALL NOT silently represent a loopback-served image as publishable: when such images are present, the user SHALL be able to cancel or explicitly copy the remaining article without those local image elements, and the result SHALL report the omitted count.

#### Scenario: Managed local image previews
- **WHEN** the snapshot references a supported image inside the document-associated asset directory
- **THEN** the workspace preview displays the bytes through the protected local session
- **AND** neither the generated DOM nor user-visible status reveals the absolute filesystem path

#### Scenario: Out-of-scope local image is unresolved
- **WHEN** the Markdown references a local image outside the associated asset directory or a file that cannot be read safely
- **THEN** the workspace does not serve that file
- **AND** the preview and status identify the image as unresolved without revealing other local path information

#### Scenario: Copying with local images requires a choice
- **WHEN** the user invokes rich copy while one or more preview images use protected loopback resources
- **THEN** the workspace reports how many images cannot be published by this local-only workflow
- **AND** offers only cancel or an explicit copy-without-those-images path

#### Scenario: Partial copy never includes loopback image URLs
- **WHEN** the user confirms copy without local images
- **THEN** the copied HTML contains no loopback URL or local filesystem URL
- **AND** the success status reports the number of omitted image elements

### Requirement: Rich copy provides HTML and plain-text representations
On a user-initiated copy action, the workspace SHALL attempt to write both WeChat-compatible `text/html` and readable `text/plain` representations to the system clipboard using browser capabilities available on the local origin. It SHALL report success only after the copy operation succeeds, and SHALL provide a localized actionable error when clipboard permission or browser behavior prevents rich copy.

#### Scenario: Rich copy succeeds
- **WHEN** the browser permits a user-initiated rich clipboard write
- **THEN** the clipboard contains the themed publishing HTML and a readable plain-text representation
- **AND** the workspace shows a localized success status

#### Scenario: Browser denies clipboard access
- **WHEN** the preferred rich clipboard API and supported fallback cannot complete the copy
- **THEN** the workspace does not show a success status
- **AND** it tells the user that browser clipboard permission or compatibility prevented copying

### Requirement: Sessions have bounded process-owned lifetimes
The publishing service SHALL start lazily, SHALL keep concurrent sessions isolated, SHALL bound retained session snapshots, and SHALL expire inactive session capabilities after a documented finite interval. Closing Markion SHALL stop the listener and invalidate every session. An expired browser page SHALL present a localized instruction to relaunch from Markion when it next attempts a protected operation.

#### Scenario: Service starts only on demand
- **WHEN** Markion starts and the user never opens the publishing workspace
- **THEN** no publishing listener is created and no publishing snapshot is retained

#### Scenario: Concurrent sessions remain isolated
- **WHEN** publishing sessions exist for two different document snapshots
- **THEN** each claimed browser session can retrieve only its own Markdown, settings, and permitted resources

#### Scenario: Inactive session expires
- **WHEN** a session exceeds the documented inactivity limit
- **THEN** subsequent protected requests for that session are denied
- **AND** the browser workspace asks the user to relaunch it from Markion

#### Scenario: Application exit ends all sessions
- **WHEN** Markion exits
- **THEN** the loopback listener stops and all previously issued session capabilities cease to authorize requests

