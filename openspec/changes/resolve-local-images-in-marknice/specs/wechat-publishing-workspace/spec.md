## MODIFIED Requirements

### Requirement: Loopback sessions protect document content and local files
The local publishing origin SHALL listen only on a loopback interface and an operating-system-selected ephemeral port. A new session SHALL require an unguessable capability that is removed from the visible browser URL after a successful claim. Document and resource responses SHALL use no-store caching, and requests without a valid, live session SHALL disclose neither document bytes nor whether a candidate local path exists. Local resource routes SHALL serve only supported regular image files that the snapshot enumerated from the document's own image references and that resolve within the active document's publishing image scope — the canonical parent directory of the saved document's directory, which covers the document's own directory tree at any depth and exactly one directory level above it — after canonical containment checks; traversal beyond that scope, absolute-path injection, symlink escape, and unrelated local-file access SHALL be rejected.

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

#### Scenario: Only referenced images inside the scope are servable
- **WHEN** a supported image file exists under the publishing image scope but the document's Markdown does not reference it
- **THEN** no resource route serves that file and no identifier for it is disclosed

#### Scenario: Traversal and symlink escape are denied
- **WHEN** a protected resource request resolves outside the canonical publishing image scope through traversal above the parent level, an absolute path, encoding, or a symlink
- **THEN** the service denies the request and returns no bytes from the target

#### Scenario: Dynamic content is not cached
- **WHEN** the service returns a document snapshot, session metadata, or a protected local resource
- **THEN** the response instructs the browser and intermediaries not to store it

### Requirement: Local images preview safely and copy with an explicit limitation
For a named document, supported images referenced by the document and resolving within its publishing image scope — the document's own directory tree at any depth and up to exactly one directory level above it, including the document-associated asset directory — SHALL preview through protected session resource URLs without exposing filesystem paths to the page. A local reference that is missing, unsupported, absolute, or escapes above that scope SHALL render as unresolved and SHALL produce a visible warning. Because this change provides no remote image publication backend, the rich-copy action SHALL NOT silently represent a loopback-served image as publishable: when such images are present, the user SHALL be able to cancel or explicitly copy the remaining article without those local image elements, and the result SHALL report the omitted count.

#### Scenario: Managed local image previews
- **WHEN** the snapshot references a supported image inside the document-associated asset directory
- **THEN** the workspace preview displays the bytes through the protected local session
- **AND** neither the generated DOM nor user-visible status reveals the absolute filesystem path

#### Scenario: Sibling, nested, and parent-level local images preview
- **WHEN** a saved document references supported images located beside the document, in subdirectories at any depth below it, or up to one directory level above the document's directory
- **THEN** each referenced image previews through the protected local session like a managed image
- **AND** references escaping above the parent level or using absolute paths render as unresolved with a visible warning

#### Scenario: Out-of-scope local image is unresolved
- **WHEN** the Markdown references a local image outside the publishing image scope, an absolute path, or a file that cannot be read safely
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
