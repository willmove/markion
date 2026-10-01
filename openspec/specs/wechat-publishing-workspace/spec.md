# wechat-publishing-workspace

## Purpose
Defines the local browser workspace that turns an explicit snapshot of a Markion document into MarkNice-compatible WeChat rich text without embedding a WebView or making the hosted MarkNice service a runtime dependency.

## Requirements

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

### Requirement: Bundle digests are platform-independent and kept in sync
The publishing bundle manifest SHALL record an SHA-256 digest for every bundled file, computed over canonical LF-normalized bytes for text files, and bundle verification SHALL apply the same normalization before comparing digests, so the checked-in workspace verifies identically on every platform regardless of checkout line endings. The manifest SHALL be regenerated whenever a bundled file's bytes change.

#### Scenario: A CRLF checkout still verifies
- **WHEN** a text file in the bundle is materialized with CRLF line endings, as Windows checkouts with `core.autocrlf` produce
- **THEN** bundle verification normalizes its line endings to LF and accepts the recorded digest
- **AND** a genuine content change to that file still fails verification

#### Scenario: Manifest digests match the checked-in bytes
- **WHEN** the maintainer regenerates the manifest with the sync command
- **THEN** every digest equals the LF-normalized digest the verifier computes for the same file
- **AND** the checked-in workspace passes bundle verification on every supported platform

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

### Requirement: Publishing launches after a minimal runtime gate

Launching the local WeChat publishing workspace SHALL require only a minimal runtime gate: the
bundle manifest is readable and parses, its provenance metadata is valid, and the entry shell
`index.html` exists and matches its manifest-recorded LF-normalized SHA-256 digest. The launch
path SHALL NOT require whole-tree bundle verification and SHALL NOT fail because files exist on
disk that the manifest does not list, because manifest-listed files other than the entry shell
fail their digest check, or because release-only scans (remote runtime dependencies, prohibited
export artifacts) would reject the directory. Full-bundle verification SHALL remain available and
SHALL stay exhaustive for release construction, pre-publication checks, and maintainer tooling.

#### Scenario: Upgrade leftovers do not block launching

- **WHEN** the installed workspace directory still contains files that a newer package no longer
  ships (for example, files removed from the bundle between two released versions, left behind by
  an in-place package upgrade), and the user opens the WeChat publishing workspace
- **THEN** the workspace session is created and the default browser opens to it
- **AND** the launch does not attempt to delete or modify the leftover files

#### Scenario: Missing manifest or invalid provenance blocks launching

- **WHEN** the workspace directory's manifest is missing, unparseable, or fails provenance
  validation
- **THEN** workspace setup fails with a setup error status and no browser session is created

#### Scenario: Tampered or missing entry shell blocks launching

- **WHEN** `index.html` is absent from disk, absent from the manifest, or its bytes do not match
  the manifest digest after LF normalization
- **THEN** workspace setup fails with a setup error status naming the file and no browser session
  is created

#### Scenario: Release verification stays exhaustive

- **WHEN** the release pipeline or the `verify-bundle` maintainer CLI verifies a workspace
  directory that contains an unlisted file, a missing file, or a digest mismatch on any listed
  file
- **THEN** full-bundle verification still fails before publication
- **AND** the strictness of release-time checks is not reduced by the relaxed runtime gate

### Requirement: The publishing editor chrome tracks the pinned MarkNice editor skin
The local publishing workspace SHALL present its Markdown editor and WeChat preview using an editor skin derived from the pinned MarkNice editor section: dual rounded cards, traffic-light panel headers, the MarkNice CSS-variable palette including indigo accent `#6366f1`, Inter plus PingFang SC and Microsoft YaHei chrome fonts, an SF Mono / Fira Code / Menlo / Consolas editor stack, SVG formatting-toolbar icons in pill buttons, grouped font-size and paragraph-spacing steppers, SVG desktop/phone preview toggles, and a phone preview framed at 375px CSS pixels with a notch. The skin SHALL remain as close as the local workspace allows to that editor section. The workspace SHALL keep its session-local and privacy disclosures and SHALL NOT restore MarkNice marketing chrome (site navbar, hero, features, guide, or footer). Dark appearance SHALL use the same `data-mode='dark'` token set as the pinned MarkNice editor.

#### Scenario: Editor cards and typography match the MarkNice editor section
- **WHEN** the authenticated workspace shell is shown
- **THEN** the Markdown input and WeChat preview occupy separate rounded cards with traffic-light headers
- **AND** chrome fonts include Inter, PingFang SC, and Microsoft YaHei rather than a Segoe-only stack
- **AND** the Markdown textarea uses a monospace stack that includes SF Mono or Fira Code before Consolas

#### Scenario: Formatting and preview controls use MarkNice icon chrome
- **WHEN** the formatting toolbar and preview toolbar are shown
- **THEN** formatting actions other than heading labels use SVG stroke icons in pill buttons rather than Unicode glyphs
- **AND** font-size and paragraph-spacing offsets are grouped stepper controls
- **AND** desktop and phone modes are SVG toggles that share a `.mode-btn` active-state contract so only one mode appears selected

#### Scenario: Phone preview uses a device frame
- **WHEN** the user selects phone preview
- **THEN** the preview article is shown inside a 375px-wide framed phone chrome with a notch
- **AND** the frame is not merely a max-width constraint on an otherwise undressed preview card

#### Scenario: Marketing chrome stays omitted
- **WHEN** the workspace loads
- **THEN** it does not render the MarkNice site navbar, hero, features grid, guide link farm, or footer
- **AND** the session-local editing disclosure remains visible

### Requirement: Word import replaces only the browser-session Markdown
The publishing workspace SHALL provide a user-initiated Import Word action that reads a `.docx` file entirely in the browser with the pinned JSZip and MarkNice Word-import runtime, converts it to Markdown, and replaces the current workspace textarea and preview. The action SHALL NOT send the file or resulting Markdown to Markion, SHALL NOT mutate or save the Markion document, and SHALL NOT increment its version or invalidate derived caches. After a successful import, the workspace SHALL tell the user that the result is session-local and that bringing it into Markion requires Copy Markdown or another explicit save of the session Markdown. Application scripts required for import SHALL load from the verified local bundle with no CDN.

#### Scenario: A Word document becomes session Markdown
- **WHEN** the user selects a supported `.docx` file in the workspace Import Word control
- **THEN** the workspace textarea is replaced with the converted Markdown
- **AND** the preview rerenders from that session Markdown
- **AND** the Markion document text, dirty state, version, selection, undo history, and derived-cache identities remain unchanged

#### Scenario: Successful import discloses the recovery path
- **WHEN** Word import completes
- **THEN** the workspace shows a localized success status
- **AND** that status states that the import is session-local and that the user must copy or otherwise save the session Markdown to keep it in Markion

#### Scenario: Invalid or oversized Word input fails closed
- **WHEN** the selected file is not a readable `.docx`, exceeds the documented import size bound, or the parser throws
- **THEN** the workspace does not replace the current Markdown with a partial conversion
- **AND** it shows a localized actionable error
- **AND** the Markion document remains unchanged

#### Scenario: Word import works offline
- **WHEN** the workspace is used with external networking unavailable
- **THEN** Import Word still runs from bundled JSZip and the Word-import runtime
- **AND** no parser, zipper, or conversion script is fetched from a remote host

#### Scenario: Word-embedded images stay in the session as data URIs
- **WHEN** the imported document contains a supported embedded image
- **THEN** the session Markdown may reference that image as a data URI
- **AND** the preview does not expose a filesystem path for that image
- **AND** the image is not treated as a managed loopback resource that must be omitted solely because it originated from Word
