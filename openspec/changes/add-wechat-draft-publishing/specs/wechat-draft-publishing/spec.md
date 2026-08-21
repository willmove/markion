## Purpose

Defines a secure, connector-based workflow for saving finalized Markion publishing content to a WeChat Official Account draft box while keeping WeChat credentials, media handling, and upstream API behavior outside the desktop application.

## ADDED Requirements

### Requirement: Publishing uses an explicitly configured self-hosted connector
Markion SHALL communicate with WeChat only through a user-configured connector that implements the supported draft-publishing contract. Markion SHALL persist the connector origin as non-secret configuration and SHALL store the scoped connector credential only through the operating system's credential facility, with no plaintext fallback. Markion SHALL reject non-HTTPS connector origins except an explicitly identified loopback development origin, SHALL NOT follow connector redirects to another origin, and SHALL provide test and clear-connection actions. Markion SHALL NOT request, store, log, or transmit a WeChat AppID, AppSecret, WeChat access token, or authorizer access token.

#### Scenario: User connects to a compatible connector
- **WHEN** the user enters an HTTPS connector origin and scoped connector credential and invokes connection testing
- **THEN** Markion authenticates directly to that origin and validates its supported protocol version and capabilities
- **AND** displays the connector's configured Official Account label without exposing either the connector credential or WeChat credentials

#### Scenario: Credential cannot be stored safely
- **WHEN** the operating-system credential facility cannot securely persist the connector credential
- **THEN** Markion does not persist the credential in application configuration or another plaintext file
- **AND** reports that publishing cannot be configured until secure credential storage is available

#### Scenario: Insecure remote origin is rejected
- **WHEN** a non-loopback connector origin uses plain HTTP or a connector response redirects to another origin
- **THEN** Markion refuses the connection without forwarding the connector credential to the insecure or redirected origin

#### Scenario: Connection is cleared
- **WHEN** the user clears the publishing connection
- **THEN** Markion removes the stored connector credential and account capability state
- **AND** draft submission remains unavailable until another connector is configured successfully

### Requirement: Draft submission starts from the finalized browser session without mutating Markion
The local publishing workspace SHALL provide a localized **Save to WeChat draft box** action after a connector is configured. Submission SHALL use a clone of the browser session's current sanitized publishing output rather than rerendering the launch-time Markdown in Markion. Starting, completing, failing, canceling, or retrying a submission SHALL NOT mutate or save the open Markion document, change its version or selection, synchronize browser edits back to it, or invalidate its per-version derived caches.

#### Scenario: Browser edits are submitted
- **WHEN** the user changes content or presentation settings in the browser workspace and starts draft submission
- **THEN** the submitted article reflects the current browser-session output
- **AND** the active Markion document remains byte-identical to the state before submission

#### Scenario: No connector is available
- **WHEN** the user opens a publishing workspace without a valid configured connector
- **THEN** the workspace does not claim that draft submission is available
- **AND** provides a localized route to connector setup while preserving rich copy

#### Scenario: Submission is canceled before the upstream draft call
- **WHEN** the user cancels while the job is still in a connector-declared cancellable state
- **THEN** no WeChat draft is created
- **AND** neither the browser-session content nor the Markion document is changed

### Requirement: The user confirms account, metadata, and cover before submission
Before a draft job is accepted, the workspace SHALL show the target connector account label and SHALL require a valid title and cover together with the finalized article preview. It SHALL let the user review and edit supported draft metadata including title, author, digest, source URL, and comment setting. Markion MAY propose defaults from the browser content and document display name, but the user SHALL confirm the values and target account. Values SHALL be validated against limits advertised by the connector before article bytes are sent.

#### Scenario: Valid metadata and cover are confirmed
- **WHEN** the user reviews the target account, accepts or edits valid metadata, chooses an article image or connector-provided default cover, and confirms
- **THEN** the workspace submits the confirmed values and cover choice as one draft job

#### Scenario: Required cover is unavailable
- **WHEN** the article has no eligible cover image and the connector advertises no default cover
- **THEN** the workspace blocks submission and explains how to choose or add a supported cover

#### Scenario: Advertised limit is exceeded
- **WHEN** confirmed metadata or packaged content exceeds a limit advertised by the connector
- **THEN** Markion rejects the draft job before transfer
- **AND** identifies the field or content limit that must be corrected

### Requirement: Markion transfers a path-free, self-contained article package
Markion SHALL convert the cloned publishing output into a versioned article package containing sanitized HTML, confirmed metadata, declared remote image URLs, opaque references for allowlisted local images, and the corresponding local image bytes. The browser SHALL submit this package only to its authenticated loopback session; it SHALL never receive or call the remote connector with the connector credential. Markion SHALL resolve local image references exclusively through the immutable resource allowlist created for that publishing session and SHALL reject browser-supplied filesystem paths or newly introduced local paths. The package sent to the connector SHALL contain no absolute filesystem path, `file:` URL, loopback URL, browser blob URL, script, executable event handler, form, frame, or active embedded object.

#### Scenario: Allowlisted local images are packaged
- **WHEN** the finalized article uses a protected local image from the publishing session's immutable resource map
- **THEN** Markion transfers an opaque image identifier and the validated image bytes to the connector
- **AND** neither the browser payload nor connector payload contains the image's filesystem path or loopback URL

#### Scenario: Browser edit introduces another local path
- **WHEN** a browser-session edit references a local file that was not allowlisted when the publishing session launched
- **THEN** Markion refuses to read or transfer that file
- **AND** submission remains blocked until the unresolved image is removed or a new Markion publishing session safely includes it

#### Scenario: Active content is present
- **WHEN** the finalized article contains a script, event handler, form, frame, active object, or unresolved local or blob URL
- **THEN** the unsafe or unresolved content is rejected before a connector job is created
- **AND** the workspace does not report a successful draft save

### Requirement: The connector contract is versioned, interoperable, and idempotent
Markion SHALL publish a machine-readable contract and conformance fixtures that allow an independently implemented connector to interoperate without linking to Markion code. The contract SHALL define authenticated capability discovery, creation of a draft job, retrieval of job status and result, and cancellation while still safe. Each creation request SHALL carry a unique idempotency key and content digest. A connector SHALL return the same job for a replay with the same key and digest, and SHALL reject reuse of the key with different content. Connector results SHALL expose only bounded account, status, validation, and opaque draft/job identifiers; they SHALL NOT expose WeChat secrets or raw upstream credentials.

#### Scenario: Independent connector is compatible
- **WHEN** a connector passes the published conformance fixtures for a protocol version supported by Markion
- **THEN** Markion can discover its capabilities, submit an article package, and observe the draft-job result without connector-specific application code

#### Scenario: Identical create request is replayed
- **WHEN** the connector receives the same idempotency key and content digest more than once
- **THEN** it returns the existing job and does not intentionally create another WeChat draft

#### Scenario: Idempotency key is reused for different content
- **WHEN** the connector receives an existing idempotency key with a different content digest
- **THEN** it rejects the request as a conflict and does not submit the changed content upstream

#### Scenario: Protocol version is unsupported
- **WHEN** capability discovery reports no protocol version compatible with Markion
- **THEN** Markion disables submission and reports an actionable connector-upgrade or application-upgrade error

### Requirement: The connector owns WeChat authentication and draft creation
The reference connector SHALL be deployable and operable independently from Markion and SHALL keep its WeChat application credentials server-side. It SHALL obtain and cache WeChat access tokens, upload the selected cover as a supported permanent material when necessary, upload inline article images through WeChat's image-upload interface, rewrite article image sources to the returned WeChat-hosted URLs, validate the final article, and invoke the WeChat draft-add interface exactly once for an unambiguous job attempt. A successful result SHALL mean only that the item was saved in the configured Official Account draft box; the connector SHALL NOT invoke free-publish, scheduling, mass-send, or other publication interfaces.

#### Scenario: Draft is saved successfully
- **WHEN** WeChat accepts the prepared article and returns a draft media identifier
- **THEN** the connector records a successful job with that opaque draft identifier and account label
- **AND** no publish, schedule, or mass-send API is called

#### Scenario: WeChat credentials are invalid or unauthorized
- **WHEN** token acquisition or the draft interface rejects the connector's WeChat credentials or account permissions
- **THEN** the job fails with a sanitized actionable category for the operator and user
- **AND** neither the AppSecret nor any WeChat token appears in the API result or logs

#### Scenario: Connector is absent from a Markion package
- **WHEN** a native Markion installer or staged application bundle is inspected
- **THEN** it contains the connector client and contract assets needed by Markion
- **AND** contains neither the deployable connector service nor any WeChat application credential

### Requirement: Every article image is converted to a WeChat-hosted resource
Before calling the draft-add interface, the connector SHALL resolve every retained article image to a WeChat-hosted URL. It SHALL upload attached local image bytes and SHALL fetch and re-upload authored HTTP or HTTPS images subject to strict URL, redirect, address, media-type, size, dimension, and timeout limits. It SHALL reject credentials in URLs, non-HTTP schemes, loopback, link-local, private, multicast, reserved, or otherwise non-public destinations, and SHALL revalidate DNS and every redirect destination. The connector SHALL fail the job rather than send a draft containing an unresolved, external, local, loopback, blob, or package-placeholder image source.

#### Scenario: Local and public remote images are prepared
- **WHEN** an article contains an attached supported local image and a permitted public HTTPS image
- **THEN** the connector uploads both through the supported WeChat media path
- **AND** the final draft HTML references only the corresponding WeChat-hosted image URLs

#### Scenario: Remote image targets a private address
- **WHEN** an authored image URL resolves or redirects to a loopback, private, link-local, reserved, or other blocked address
- **THEN** the connector refuses to fetch it and fails validation without making the protected request

#### Scenario: Remote image exceeds a safety limit
- **WHEN** an authored image exceeds an advertised byte, dimension, media-type, redirect, or timeout limit
- **THEN** the connector stops processing it and returns a sanitized image-validation failure

#### Scenario: Placeholder remains after rewriting
- **WHEN** final validation finds an external, local, loopback, blob, or unresolved package-placeholder image source
- **THEN** the connector does not call the draft-add interface
- **AND** reports that the article still contains an unpublishable image

### Requirement: Draft-job status distinguishes success, failure, cancellation, and uncertainty
The connector SHALL expose a bounded job lifecycle that distinguishes queued, preparing, cancellable, committing, succeeded, failed, canceled, and outcome-unknown states. Markion SHALL poll or otherwise retrieve that lifecycle and present it in localized terms. Once the upstream draft-add request may have reached WeChat, neither Markion nor the connector SHALL automatically retry it or claim it was canceled. If the upstream result is lost or times out after that point, the job SHALL become outcome-unknown and the user SHALL be told to inspect the WeChat draft box before choosing a new manual attempt.

#### Scenario: Failure occurs before commit
- **WHEN** validation, asset transfer, or media preparation fails before the draft-add request begins
- **THEN** the job enters failed state without a created draft
- **AND** the user may correct the issue and intentionally start a new job

#### Scenario: Upstream outcome is ambiguous
- **WHEN** the connector cannot determine whether WeChat accepted a draft-add request
- **THEN** the job enters outcome-unknown state and is not automatically retried
- **AND** Markion instructs the user to inspect the target draft box before starting another attempt

#### Scenario: Success is reported precisely
- **WHEN** the connector reports a succeeded job
- **THEN** the workspace states that the article was saved to the named account's draft box
- **AND** does not describe it as published, scheduled, or sent

### Requirement: Connector processing minimizes retained content and secret exposure
The connector SHALL authenticate every contract operation, apply bounded request and job-retention limits, redact authorization material and WeChat tokens, and avoid logging article HTML, metadata values, image bytes, or upstream response bodies that can contain sensitive content. It SHALL remove transient article and image payloads after the documented completion or expiry period while retaining only the minimum idempotency and audit metadata needed to prevent accidental duplicate submission. Authentication and lookup failures SHALL not disclose whether another tenant, token, job, file, or credential exists.

#### Scenario: Job payload retention expires
- **WHEN** a completed or abandoned job passes the documented payload-retention period
- **THEN** the connector removes its stored article HTML and image bytes
- **AND** retains at most bounded non-content metadata required for idempotency and operational audit

#### Scenario: Request is unauthenticated
- **WHEN** a caller omits or supplies an invalid connector credential
- **THEN** the connector returns a uniform denial without account, job, content, or credential-existence details

#### Scenario: Operational logs are inspected
- **WHEN** connector and Markion logs for a submission are reviewed
- **THEN** they contain correlation identifiers and sanitized status categories as needed
- **AND** contain no connector bearer token, AppSecret, WeChat access token, article HTML, metadata value, image bytes, or absolute local path

### Requirement: Publishing behavior is localized, testable, and release-verifiable
All user-facing connector setup, confirmation, progress, success, uncertainty, and error text SHALL be available in every Markion locale with equivalent semantics. Markion SHALL support deterministic testing against a fake conforming connector, and the reference connector SHALL support deterministic tests against a fake WeChat service without real credentials or network access. Release verification SHALL cover connector exclusion from desktop packages, immutable-document behavior, contract conformance, image rewriting and SSRF denial, idempotency, ambiguous outcomes, and successful saving to an authorized test account.

#### Scenario: User changes locale
- **WHEN** any connector or draft-job state is displayed in each supported Markion locale
- **THEN** action labels, account confirmation, progress, success, uncertainty, and remediation text are localized without changing their security meaning

#### Scenario: Automated suites run without real services
- **WHEN** the workspace and connector test suites run in continuous integration
- **THEN** fake connector and fake WeChat endpoints exercise success and failure paths deterministically
- **AND** no real WeChat credential or public network call is required

#### Scenario: End-to-end release evidence is collected
- **WHEN** the feature is prepared for release
- **THEN** an authorized test account receives a draft through the self-hosted connector and the result is manually verified in its draft box
- **AND** package inspection confirms that Markion contains no connector service or WeChat secret
