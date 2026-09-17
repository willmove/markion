## 1. Feasibility, baselines, and budgets

- [x] 1.1 Record immutable control revisions and artifact manifests for the current core-only release and the built-in-PDF candidate, including compressed and installed byte counts for NSIS, DMG, DEB, and AppImage.
- [x] 1.2 Build a minimal signed fixture worker and verify direct launch, executable permissions, quarantine/Gatekeeper behavior, and child-process cleanup on packaged Windows, macOS, and Linux installations; stop the change if a reliable first-party launch path cannot be demonstrated.
- [x] 1.3 Add the smallest end-to-end plugin host skeleton and perform same-environment control/candidate builds for all four package formats, enforcing the 1 MiB compressed and 2 MiB installed host budgets before broader implementation.
- [x] 1.4 Compose target-specific PDF plugin prototype archives from the existing worker/runtime payload, produce exact compressed and extracted manifests, and confirm the 6 MiB compressed and 10 MiB installed plugin budgets on every supported target.

## 2. Protocol and signed package model

- [x] 2.1 Add a GPUI-free `markion-plugin-protocol` workspace crate defining versioned manifests, catalogs, capability declarations, compatibility ranges, permissions, lifecycle messages, error codes, and resource limits.
- [x] 2.2 Implement deterministic manifest/catalog serialization and dedicated first-party signature verification, with test-only keys and tests for tampering, wrong keys, digest mismatches, incompatible targets, and unsupported protocol versions.
- [x] 2.3 Implement defensive plugin archive inspection and extraction with path traversal, absolute path, symlink, case-collision, duplicate member, file-count, expanded-size, and manifest-closure checks.
- [x] 2.4 Implement the length-framed IPC codec using canonical JSON control headers and bounded raw binary bodies, with round-trip, truncation, oversize, malformed-frame, and adversarial-input tests.
- [x] 2.5 Add a deterministic fixture worker and in-memory adapter covering handshake, capability negotiation, request/response correlation, cancellation, graceful shutdown, and version rejection.

## 3. Plugin catalog and store

- [x] 3.1 Define per-user plugin paths and persisted store state for staged versions, the active version pointer, one rollback version, disabled state, quarantine metadata, and storage accounting.
- [x] 3.2 Implement explicit install, update, atomic activation, rollback, disable, enable, and uninstall transactions with staging cleanup and recovery tests for interrupted or partially written operations.
- [x] 3.3 Ship and verify a signed bootstrap catalog, then add signed remote-catalog refresh, cache, offline fallback, monotonic update handling, and deterministic plugin/capability/file-handler collision rejection.
- [x] 3.4 Expose authoritative compressed-download and installed-storage estimates before installation and exact per-plugin storage accounting after installation, including staged and rollback bytes.

## 4. Worker supervision and resource control

- [x] 4.1 Implement direct worker spawning from the verified active plugin directory with a sanitized environment, controlled working directory, inherited-handle restrictions, and bounded diagnostic output capture.
- [x] 4.2 Add a bounded request table, backpressure, deadlines, cooperative cancellation, worker-generation tracking, and stale-response rejection.
- [x] 4.3 Handle crash, hang, malformed output, protocol violation, startup failure, and forced shutdown with deterministic child/process-tree cleanup, temporary-file cleanup, and per-version quarantine.
- [x] 4.4 Add in-memory and real-process supervisor tests for concurrency limits, timeouts, cancellation races, restart boundaries, crash loops, and clean application shutdown.

## 5. Plugin manager and localization

- [x] 5.1 Add host-owned plugin manager state and UI for available, installed, disabled, update-available, incompatible, quarantined, and rollback-available states.
- [x] 5.2 Require explicit install/update confirmation showing publisher trust, requested capabilities/permissions, download size, installed size, target compatibility, and restart requirements.
- [x] 5.3 Add progress, cancellation, retry, offline, verification-failure, activation-failure, rollback, uninstall, and storage-reclamation flows without permitting silent installation during document open.
- [x] 5.4 Extend locale validation so every dynamic first-party plugin and capability label is complete in all supported locales before its catalog is accepted, and add translations for all host-owned plugin UI and errors.
- [x] 5.5 Add GPUI tests for plugin manager actions and state transitions, and verify plugin preferences remain isolated from document preferences and workspace content.

## 6. File handlers and host-owned paged-document UI

- [x] 6.1 Implement an immutable file-handler registry snapshot combining core handlers with verified catalog declarations, including known-but-uninstalled, disabled, incompatible, and collision states.
- [x] 6.2 Route file tree visibility, open-file dispatch, recent files, session restore, missing-provider prompts, and save-as/path-remap behavior through the generic registry while preserving path identity and isolation.
- [x] 6.3 Replace content-specific workspace tab state with generic plugin-document state, ensuring document-only commands, dirty tracking, backups, autosave, Markdown preview, and outline operations remain unavailable to read-only paged documents.
- [x] 6.4 Extract the reusable paged-document viewport, toolbar, scroll/zoom/page navigation, visible-page calculation, render scheduling, and bounded raster cache from the current PDF-specific application module.
- [x] 6.5 Add the `paged-document/v1` host adapter for open, metadata, page sizing, bounded render, cancellation, and close operations, including admission checks for dimensions, stride, pixel format, body length, and cache generation.
- [x] 6.6 Add regression tests for continuous scrolling, wheel and keyboard navigation, page jumps, zoom anchoring, resize, stale render rejection, worker restart, disable/uninstall during work, and preservation of derived-state/cache identity invariants.

## 7. Official PDF plugin extraction

- [x] 7.1 Add the official PDF worker entry point and signed manifest, mapping the existing GPUI-free PDF engine to `paged-document/v1` without exposing PDF-specific types to the host.
- [x] 7.2 Move PDFium discovery, loading, notices, and target runtime payloads into the plugin directory, prohibit runtime downloads, and test missing, corrupt, and incompatible runtime failures.
- [x] 7.3 Remove the core application's direct PDF engine dependency, PDFium packager resources, and hard-coded PDF tab/dispatch variants so a no-plugin installation contains no PDF renderer payload.
- [x] 7.4 Port existing fake-backend and native smoke coverage through the framed protocol, including encrypted, malformed, oversized, zero-page, text-only, and image-heavy documents.
- [x] 7.5 Verify the installed official plugin preserves file-tree discovery, open/session behavior, all paged controls, localized failures, cancellation, cleanup, and bounded-memory behavior from the built-in implementation.

## 8. Packaging, signing, and release gates

- [x] 8.1 Add reproducible target-specific plugin build, archive, manifest, digest, signature, and size-report jobs using the dedicated plugin signing key and deterministic file ordering.
- [x] 8.2 Publish versioned official plugin assets plus the signed catalog to GitHub Releases and OSS without embedding plugin payloads in core application packages.
- [x] 8.3 Add closure assertions that NSIS, DMG, DEB, and AppImage core packages contain the plugin host and bootstrap metadata but no plugin worker, PDF engine, PDFium binary, or plugin rollback payload.
- [ ] 8.4 Extend packaged smoke tests to install into paths containing spaces and non-ASCII characters, install the PDF plugin explicitly, launch it, render multiple pages, restart offline, disable it, roll it back, and uninstall it.
- [x] 8.5 Enforce exact same-environment size gates of at most 1 MiB compressed and 2 MiB installed for the core host, and at most 6 MiB compressed and 10 MiB installed for the official PDF plugin, on all four release formats.
- [x] 8.6 Generate and verify dependency, license, notice, provenance, checksum, and signature artifacts independently for the core application and every official plugin package.

## 9. Reconciliation, documentation, and final verification

- [x] 9.1 Reconcile the active `add-pdf-document-viewing` proposal, design, delta specs, and tasks so PDF viewing is delivered as the optional official plugin, then validate both changes without contradictory packaging requirements.
- [x] 9.2 Document the user-facing plugin installation/update/rollback/storage model and the internal first-party package/protocol architecture, explicitly stating that version 1 is not a third-party arbitrary-code ecosystem.
- [x] 9.3 Run formatting, clippy, root-package tests, workspace tests, targeted protocol/store/supervisor/UI tests, OpenSpec validation, and `openspec doctor`, resolving every failure.
- [ ] 9.4 Complete manual Windows, macOS, and Linux packaged smoke coverage for fresh install, offline restart, update, rollback, crash recovery, disabled/uninstalled providers, and multi-page PDF interaction.
- [ ] 9.5 Publish final per-format core-host and PDF-plugin compressed/installed byte reports against immutable controls, confirm every budget, and record any variance from the feasibility estimates.
