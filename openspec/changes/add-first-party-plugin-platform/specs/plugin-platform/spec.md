## Purpose

Defines how Markion discovers, verifies, installs, runs, updates, and removes optional first-party plugins without loading unstable native code into the application process.

## ADDED Requirements

### Requirement: Official plugin packages SHALL be authenticated and target-compatible
Each installable plugin SHALL have a stable plugin identifier, semantic version, supported host-protocol range, capability declarations, target operating system and architecture, entry point, file manifest, byte sizes, license notices, permission disclosures, and cryptographic digests in a signed manifest. Markion SHALL install only packages signed by a trusted first-party key whose target and host-protocol range match the running application. A mismatched, expired, malformed, incomplete, or tampered package SHALL be rejected before any packaged executable runs.

#### Scenario: Matching official package is accepted
- **WHEN** the user explicitly installs an official plugin whose signature, digests, target, and host-protocol range all validate
- **THEN** Markion stages and activates that exact plugin version
- **AND** records its identity, version, installed bytes, capabilities, and disclosed permissions

#### Scenario: Tampered package is rejected
- **WHEN** any signed manifest, package member, digest, target, or compatibility declaration fails validation
- **THEN** Markion does not execute or activate the package
- **AND** reports a localized safe verification failure without replacing the previously active version

#### Scenario: Foreign platform package is rejected
- **WHEN** a user attempts to install a package built for another operating system or architecture
- **THEN** Markion rejects it before extraction or execution
- **AND** keeps the current application and plugin set unchanged

### Requirement: Plugin installation and changes SHALL be explicit, atomic, and recoverable
Markion SHALL provide an in-app plugin manager that lists official plugins, installed and available versions, compatibility, download and installed sizes, source, disclosed permissions, and enabled state. Install, update, rollback, enable, disable, and uninstall actions SHALL require an explicit user action. Package download and verification SHALL complete in a staging directory before an atomic activation switch; a failed install or update SHALL preserve the last working version. Markion SHALL NOT silently download or enable plugin code at startup or merely because a matching file was encountered.

#### Scenario: User installs an available plugin
- **WHEN** the user confirms Install for a compatible official plugin
- **THEN** Markion downloads, verifies, stages, and atomically activates the package
- **AND** the plugin manager reports its installed version and exact disk usage

#### Scenario: Update fails before activation
- **WHEN** an update download, verification, extraction, compatibility handshake, or health check fails
- **THEN** Markion keeps the previously active plugin version usable
- **AND** removes or quarantines the failed staged version without treating it as installed

#### Scenario: User disables or uninstalls a plugin
- **WHEN** the user disables or uninstalls an installed plugin
- **THEN** Markion stops accepting new work for it, terminates its process after bounded cleanup, and removes its capability registrations
- **AND** uninstall removes the selected plugin payload while preserving unrelated plugins and user documents

### Requirement: Native plugins SHALL run out of process under lifecycle supervision
Markion SHALL launch a native plugin executable only when one of its capabilities is needed. Communication SHALL use a versioned framed protocol that distinguishes bounded control data from raw binary payloads without Base64 expansion. The host SHALL enforce handshake, message-size, request-count, deadline, cancellation, and shutdown limits. A plugin crash, malformed frame, unexpected exit, timeout, or failed shutdown SHALL terminate or quarantine that plugin session without crashing Markion or mutating an open document.

#### Scenario: Plugin starts on first use
- **WHEN** an enabled plugin capability is requested and no healthy process is running
- **THEN** Markion starts the declared executable without a shell, completes a version/capability handshake, and dispatches work only after validation

#### Scenario: Plugin crashes during work
- **WHEN** a plugin process exits while requests are pending
- **THEN** those requests fail with a stable contained category
- **AND** Markion remains responsive, retains all document state, releases transient payloads, and allows the affected surface to close or retry

#### Scenario: Plugin stops responding
- **WHEN** a plugin exceeds a protocol deadline or emits an oversized or malformed message
- **THEN** Markion cancels the session, terminates the process if necessary, and marks that version unhealthy for the current run
- **AND** no late response can populate a replacement tab or job

### Requirement: Plugins SHALL extend only declared capability interfaces
The host SHALL expose versioned capability interfaces rather than arbitrary application access. A plugin SHALL register only capabilities declared in its signed manifest and accepted during handshake. The host SHALL own menus, dialogs, GPUI rendering, localization chrome, tab identity, document state, credentials, and user confirmation; plugin requests and results SHALL use typed capability data. Unknown capability names or versions SHALL fail closed. Plugins SHALL NOT receive GPUI objects, mutable Markdown state, undo or recovery state, or an interface for injecting arbitrary native UI.

#### Scenario: Declared capability is registered
- **WHEN** an authenticated plugin handshakes with a supported version of every declared capability
- **THEN** Markion registers the corresponding host adapter for that plugin version
- **AND** tests can replace the process adapter with an in-memory adapter at the same capability seam

#### Scenario: Plugin declares an unsupported capability
- **WHEN** a plugin requires an unknown capability or an incompatible major version
- **THEN** Markion refuses activation and reports the incompatibility
- **AND** does not partially register the remaining capabilities

#### Scenario: Plugin output cannot mutate the document model
- **WHEN** a plugin returns data, progress, an error, or a binary payload
- **THEN** the host admits it only through the active typed capability request
- **AND** document versions, selections, undo history, dirty state, derived caches, syntax memoization, and cached text handles remain unchanged unless a future capability explicitly specifies an authorized document mutation

### Requirement: First-party trust and permission disclosure SHALL be explicit
The initial plugin platform SHALL accept only packages signed by the configured first-party trust root. Before installation, Markion SHALL display the plugin publisher, capabilities, declared filesystem/network/credential needs, download size, and installed size. Permission declarations SHALL limit what the host supplies to the plugin but SHALL NOT be represented as an operating-system sandbox for native code. Developer-mode loading and any future third-party trust root SHALL remain disabled unless introduced by a later specification.

#### Scenario: Installation confirmation shows trust and permissions
- **WHEN** the user starts installation of an official plugin
- **THEN** the confirmation identifies the trusted publisher and displays every declared capability and sensitive resource need
- **AND** cancellation leaves no active or installed plugin payload

#### Scenario: Package is not signed by the first-party trust root
- **WHEN** a package has no signature or is signed by an unknown key
- **THEN** normal Markion builds reject it without an override
- **AND** do not offer to weaken verification or run it anyway

### Requirement: Plugin storage SHALL be isolated and accountable
Plugin packages SHALL be installed beneath Markion's per-user application-data root in a plugin-ID/version hierarchy, separate from the core application installation, document workspaces, session state, recovery snapshots, and presentation caches. Markion SHALL report active and retained rollback bytes per plugin. Temporary downloads, extraction directories, IPC payloads, and obsolete versions SHALL be bounded and cleaned after success, failure, startup recovery, or uninstall. Credentials SHALL remain in the operating-system credential store or another separately specified broker and SHALL NOT be written to plugin package directories.

#### Scenario: Plugin installation leaves workspaces untouched
- **WHEN** a plugin is installed or updated while documents are open
- **THEN** all plugin payloads are written only under the plugin data root
- **AND** no document directory, session file, recovery file, or Markdown cache contains plugin executables or credentials

#### Scenario: Interrupted installation is recovered
- **WHEN** Markion starts after interruption during plugin download, extraction, or activation
- **THEN** it identifies and removes or quarantines incomplete staging data
- **AND** selects only a completely verified active version

