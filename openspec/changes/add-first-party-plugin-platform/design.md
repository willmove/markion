## Context

See `proposal.md` for motivation. The current PDF work is implemented but not merged or archived. Its native owner already lives in the GPUI-free `markion-pdf-viewer` member, while `src/app/pdf_viewer.rs`, `WorkspaceTab::Pdf`, the root PDF cache, file classification, and packaging are compiled into the application. The root has no runtime plugin discovery, package store, compatibility protocol, or dynamic command registration.

The completed PDF size reports provide a useful upper bound for what extraction can recover from the core distribution:

| Format | Current compressed PDF delta | Current installed PDF delta |
|---|---:|---:|
| Windows NSIS | 2,965,264 B (2.83 MiB) | 8,034,148 B (7.66 MiB) |
| macOS DMG/app | 3,609,082 B (3.44 MiB) | 8,452,256 B (8.06 MiB) |
| Linux DEB | 3,770,748 B (3.60 MiB) | 8,526,496 B (8.13 MiB) |
| Linux AppImage | 3,391,488 B (3.23 MiB) | 8,546,480 B (8.15 MiB) |

A Windows composition spike reused the existing 805,888-byte release smoke worker, the 7,211,520-byte PDFium runtime, and 2,446-byte notice file. The extracted payload was 8,019,854 bytes (7.65 MiB) and a Deflate ZIP was 3,921,972 bytes (3.74 MiB). This is feasibility evidence, not final acceptance: the real protocol worker, manifest, signatures, every target, and final archive must be measured in native CI.

The application already links Tokio, Serde/JSON, HTTP, ZIP-producing code, hashing, and updater-related machinery. A plugin host that reuses those facilities should add about 0.25–0.75 MiB compressed and 0.6–1.5 MiB installed; the specification deliberately sets stricter acceptance ceilings of 1 MiB and 2 MiB. Relative to the current PDF-bundled candidate, a no-plugin installation is therefore expected to become roughly 2.1–3.4 MiB smaller compressed and 6.2–7.6 MiB smaller installed, depending on platform. Only same-runner control/candidate reports are authoritative.

The existing `add-pdf-document-viewing` change currently requires PDFium in every core package. It must remain active until this change rewrites that distribution decision; archiving it unchanged would make the stable specs contradict the optional-plugin model.

## Goals / Non-Goals

**Goals:**

- Establish a deep plugin-management module whose small interface covers verified installation, activation, rollback, resolution, accounting, and removal.
- Establish a deep process-supervision module whose interface hides framing, child lifecycle, deadlines, backpressure, cancellation, and failure mapping.
- Prove both seams with production and in-memory adapters, and prove the platform with the real PDF provider rather than a pass-through sample alone.
- Keep GPUI rendering and mutable editor state inside the root application while moving optional native dependencies and provider logic outside its process and installer.
- Preserve all current PDF viewing, cache, safety, workspace, session, and localization behavior after extraction.
- Leave a capability-family mechanism that later changes can extend with a publisher-job interface without changing package installation or process supervision.
- Make core-host overhead and each plugin's download/installed footprint independently measurable and non-negotiable.

**Non-Goals:**

- Loading Rust `cdylib` files into the Markion process or promising a stable Rust ABI.
- Treating native plugin processes as an operating-system sandbox.
- A third-party trust store, marketplace submissions, developer-mode sideloading, arbitrary scripts, or community plugin compatibility in v1.
- Arbitrary plugin-provided GPUI trees, custom native windows, menu layouts, or direct document mutations.
- Defining `publisher.job` before one publishing change is ready to implement and test both a production and a fake adapter at that seam.
- Hot upgrade of an in-flight plugin session; updates activate after its current work reaches a safe stop.

## Decisions

### 1. Build one first-party package/lifecycle platform, then add narrow capability families

The external seam is a signed package plus a versioned process protocol, not a Rust trait exported across binaries. The root uses three internal deep modules:

```text
PluginCatalog
  resolve_file_handler(path) -> Known / Installed / Disabled / Unavailable
  describe(plugin_id)         -> signed identity, sizes, permissions, versions

PluginStore
  install_verified(staged_package)
  activate / rollback / disable / uninstall

PluginSupervisor
  acquire(plugin_id, capability) -> typed session adapter
  cancel / release / shutdown
```

Callers do not manage archive members, active-version pointers, processes, pipes, frame parsing, deadlines, or cleanup. Tests use an in-memory catalog/store and an in-memory process adapter at the same interfaces. The production adapters use filesystem storage and child processes. This makes the seams real while retaining locality.

Capability families are intentionally separate interfaces. The first is `paged-document/v1`; a later publishing change can add `publisher-job/v1` without enlarging the paged-document interface or exposing a universal `execute(any_json)` escape hatch.

Alternatives considered:

- *One universal plugin command interface.* Superficially flexible but shallow: every caller must understand plugin-specific schemas, errors, lifecycle, and security. Rejected.
- *Compile-time feature flags or separate Core/Full editions.* Lower initial engineering cost but fragment downloads and updates and do not support independent publisher evolution. Rejected as the strategic design; retained as a rollback option.
- *Download only PDFium while keeping PDF code built in.* Recovers most native bytes cheaply but leaves PDF-specific code and does not establish reusable plugin seams. Rejected as the target, though its size is a useful lower bound.

### 2. Use signed ZIP packages and a signed bootstrap/remote catalog

An official archive uses a `.markion-plugin` extension and ordinary Deflate ZIP encoding so extraction reuses a small, audited format already present in the workspace. Its root contains:

```text
plugin.json
plugin.json.minisig
bin/<target entry point>
resources/<target-only files>
THIRD_PARTY_NOTICES.md
```

`plugin.json` is canonical JSON and includes plugin ID/version, protocol range, target triple, entry point, capability declarations, localized identity records, permissions, unpacked/archive size declarations, and the length plus SHA-256 of every other member. The signature covers the canonical manifest; member digests close the archive. A dedicated plugin-signing key is separate from the Windows updater key and its public half is embedded in all tagged platform builds. Tests use a repository-controlled test key; production private material exists only in release secrets.

The app package carries a small signed bootstrap catalog with official identities, extensions, target assets, compatibility, and expected sizes, so `.pdf` remains discoverable while offline and uninstalled. An explicitly initiated plugin-manager refresh can fetch a newer signed catalog from GitHub with the existing OSS fallback. The last verified catalog is stored atomically; an invalid refresh never replaces it. Encountering a `.pdf` only opens the install surface and never fetches code.

Catalog and package signatures authenticate Markion's own distribution, but they are not Authenticode, Apple code signing, or notarization. Native CI must prove that an in-app-downloaded package can be extracted and launched on each supported runner. Markion must not silently strip OS quarantine metadata or advise bypassing an unknown publisher.

Alternatives considered:

- *Catalog signature only.* Does not authenticate manually supplied archives independently. Rejected.
- *Reuse the Windows updater key.* It is currently Windows/tag scoped and couples key rotation/blast radius. Rejected in favor of a dedicated cross-platform plugin key.
- *Custom package format.* Adds parser and tooling risk with no useful leverage. Rejected.

### 3. Install versions atomically under the per-user data root

The production store uses the platform application-data directory:

```text
plugins/
  catalog.cache              # atomically replaced canonical JSON + signature pair
  staging/<random>/...
  <plugin-id>/
    active.json
    versions/<semver>/<verified files>
```

Download goes to a unique staging directory. Validation checks archive path containment, duplicate/case-colliding names, member count, compressed and expanded limits, exact manifest closure, signature/digests, target, compatibility, and executable path before extraction. Activation writes a new `active.json` atomically only after a worker handshake/health check. At most the active version and one verified rollback version are retained by default, and both are included in displayed disk usage. Cleanup is idempotent after success, failure, interruption, or uninstall.

Plugin configuration is separate from immutable package versions. No credential is stored in the package tree. Uninstall stops the worker, removes versions/staging/configuration that the user confirms, updates registrations, and never traverses a path taken from the plugin itself.

### 4. Use an out-of-process, length-framed binary protocol over inherited pipes

The host starts the exact verified executable directly, with no shell, a plugin-version working directory, a minimal explicit environment, and inherited stdin/stdout pipes. Stderr is captured into a bounded diagnostic ring and never displayed verbatim. The cross-platform frame is conceptually:

```text
magic | protocol-major/minor | frame-kind | request-id
header-length (u32) | body-length (u64) | canonical JSON header | raw body
```

Control headers are small and versioned; raster bodies remain raw RGBA rather than Base64. The decoder applies limits before allocation, validates integer conversions, and rejects trailing/unknown security-sensitive fields. One writer owns each pipe. A bounded request table provides cancellation and backpressure. Process generation plus request ID rejects late responses after tab replacement, plugin restart, disable, or update.

The handshake exchanges host/plugin protocol ranges, plugin ID/version/target, package digest, capability versions, and per-capability limits. A mismatch fails the whole activation; partial registration is forbidden. Normal shutdown is requested first, followed by bounded wait and forced termination. The host never interprets process exit as application exit.

Alternatives considered:

- *Native dynamic libraries.* Rust ABI instability, in-process crash exposure, unload/update difficulty, and unrestricted memory access make this unsuitable. Rejected.
- *JSON Lines with Base64 rasters.* Adds roughly 33% wire growth plus copies for pages up to 32 MiB. Rejected.
- *Temporary files for every raster.* Avoids pipe bodies but creates cleanup/race/antivirus complexity and conflicts with the memory-only PDF cache model. Rejected for v1.
- *WASM/WASI.* Attractive for later untrusted pure plugins, but a runtime would consume core size and does not cleanly host target PDFium. Deferred.

### 5. Keep UI and editor ownership in the host

Plugins provide signed metadata and typed capability results, not GPUI elements. The root owns a Plugin Preferences tab (or dedicated panel), install confirmations, progress, errors, permissions, disk accounting, enable/disable/update/rollback/uninstall actions, and missing-provider surfaces. A generic `RunPluginAction { plugin_id, action_id }` can be introduced later, but v1 does not let manifests inject arbitrary menus.

The root's exhaustive `Msg` catalog retains every host phrase. Signed catalog entries supply locale-complete plugin name/description text through a validator that permits plain text only and requires every Markion locale. This is the one intentional extension to the compile-time-only i18n model; release validation replaces compile-time exhaustiveness for dynamic identity text.

Permission declarations are shown before install and determine which paths, credentials, and host snapshots Markion supplies. They do not prevent a malicious native executable from using ambient OS access, so normal builds accept only first-party signatures. A later third-party design must add a stronger execution model instead of overstating these declarations.

### 6. Generalize hard-coded PDF state into a host-owned paged-document module

The PDF implementation is split at the existing natural seam:

```text
path / tab intent
  -> official handler resolution
  -> PluginSupervisor acquire(paged-document/v1)
  -> Open { request_id, path, limits }
  <- document token + Arc<[PageGeometry]>
  -> host page layout / scroll / zoom / cache scheduling
  -> Render { token, generation, page, width bucket }
  <- raw RGBA body + dimensions
  -> GPUI RenderImage + host byte-bounded LRU
```

`WorkspaceTab::Pdf` becomes content-independent plugin-document state containing provider/plugin identity, process/session generation, path, immutable geometry, scroll/zoom/current-page state, loading/error state, and host raster claims. `src/app/pdf_viewer.rs` is divided: generic layout, visible-range scheduling, cache, tab lifecycle, controls, and rendering stay in a root `paged_document` module; PDF-specific runtime discovery, PDFium ownership, native errors, and render implementation move behind the worker.

The current `markion-pdf-viewer` member stops being a dependency of the root package and gains the protocol worker entry point. Its existing single-owner queue can remain as internal implementation if useful, but a redundant second queue should be removed if process supervision already provides all required serialization/backpressure. Tests move to the paged-document interface: in-memory provider tests exercise host behavior, while protocol/native tests exercise the real worker. Tests below the replaced interface are deleted when their behavior is fully covered through it.

The worker locates PDFium relative to its verified plugin package in release mode. Development overrides and staged runtimes remain limited to non-release/test tooling. The core `packager.toml` and release payload no longer stage `assets/pdfium`.

### 7. Resolve file types through immutable registry snapshots

The catalog produces an immutable registry snapshot shared by workspace scans and open dispatch. Core Markdown/text/image handlers remain built in. Official plugin document handlers add extension, icon identity, capability version, and resolution state. A file scan reads one `Arc` snapshot and keeps existing row bounds; it does not query a process or network per entry. Catalog validation rejects equal-priority extension collisions before publishing or activation.

Known-but-uninstalled types remain visible and open into the install surface. Existing path-backed session records need no plugin-specific schema: restoration reclassifies each stored path through the current registry, then opens it or presents the missing-provider state. Disabling/uninstalling a provider invalidates its process generation and converts existing tabs to a closable provider-unavailable state.

### 8. Make size evidence the first implementation gate

Before broad integration, native CI builds a same-toolchain PDF-disabled control and a host-only candidate. It reports final NSIS, DMG, DEB, and AppImage artifact deltas plus installed/staged payload deltas and fails above 1 MiB compressed or 2 MiB installed. This captures the signature verifier, protocol types, ZIP extraction, catalog, localization, and UI rather than estimating them independently.

The PDF plugin matrix then builds the real worker archive and measures all members. The archive must stay at or below 6 MiB and extracted payload at or below 10 MiB. It launches the worker from the extracted archive, completes handshake, opens/renders the fixture through packaged PDFium, and verifies no core package contains the worker/runtime.

Expected no-plugin core change relative to the current PDF-bundled candidate is:

| Format | Expected compressed reduction | Expected installed reduction | Minimum reduction at hard host ceiling |
|---|---:|---:|---:|
| NSIS | 2.08–2.58 MiB | 6.16–7.06 MiB | 1.83 / 5.66 MiB |
| DMG | 2.69–3.19 MiB | 6.56–7.46 MiB | 2.44 / 6.06 MiB |
| DEB | 2.85–3.35 MiB | 6.63–7.53 MiB | 2.60 / 6.13 MiB |
| AppImage | 2.48–2.98 MiB | 6.65–7.55 MiB | 2.23 / 6.15 MiB |

The first range subtracts the expected 0.25–0.75 MiB / 0.6–1.5 MiB host overhead from measured PDF deltas. The final column subtracts the full allowed 1 MiB / 2 MiB and is the guaranteed minimum if all gates pass. Numbers are planning estimates until native reports exist.

### 9. Publish plugins on the existing release topology without coupling app and plugin versions

Native jobs build the core and supported official plugins separately. A tagged release publishes `<plugin-id>-<version>-<target>.markion-plugin`, signatures, per-target size reports, and one signed catalog to GitHub and mirrors them to OSS alongside existing application artifacts. Catalog publication happens only after every referenced artifact passes signature, extraction, launch, capability, native fixture, and size checks.

Plugin versions are independent semantic versions with a host-protocol/application compatibility range. The initial PDF plugin may release lockstep for operational simplicity, but the data model and filenames must not require equal versions. Core release completion includes the catalog only when that release claims plugin compatibility; a failed plugin build never creates a dangling catalog entry.

### 10. Defer publisher capabilities but preserve the correct seam

Package verification, installation, catalog resolution, supervision, framing, cancellation, progress events, and compatibility are capability-independent and can be reused later. A future publishing change should add a `publisher-job/v1` interface shaped around `capabilities`, `preflight`, `submit`, `status`, and `cancel`, with host-owned confirmation/credential handling. Platform-specific login, remote behavior, and irreversible outcomes remain in the provider or its external connector.

This change does not add placeholder publishing methods to `paged-document/v1` or a generic UI schema. The later interface is designed when at least one production publisher and an in-memory adapter can prove a real seam.

## Risks / Trade-offs

- **[Native plugin code is trusted but not sandboxed]** → Accept only the first-party signature in v1, disclose permissions accurately, minimize host-supplied data, and reserve third-party support for a later execution-security design.
- **[A downloaded child executable may hit Gatekeeper, SmartScreen, executable-bit, or antivirus behavior]** → Make in-app download/extraction/launch smoke tests an early native gate; never strip quarantine or bypass OS policy silently; stop the design if reliable launch cannot be achieved under documented unsigned-build constraints.
- **[Large RGBA frames add a process-boundary copy]** → Use raw bounded frames, width buckets, one serialized PDF renderer, backpressure, cancellation, and the existing 32 MiB page / 64 MiB host cache ceilings; measure scroll latency and peak memory.
- **[A generic platform can become a shallow collection of escape hatches]** → Add capability families only with typed requirements and real production/fake adapters; reject arbitrary JSON commands, UI injection, and direct application-state access.
- **[Catalog compromise could distribute executable code]** → Use a dedicated offline-protected signing key, verify canonical manifest/member digests again locally, support key rotation explicitly, and retain the last verified catalog/version for rollback.
- **[Plugin update and app update can become incompatible]** → Validate compatibility before activation, retain one rollback version, fail closed on unknown majors, and test N/N-1 combinations declared in the catalog.
- **[Plugin management code consumes more core space than expected]** → Run host-only size spikes first and fail at 1 MiB compressed / 2 MiB installed rather than weakening the budget.
- **[Moving PDF across a process regresses current behavior]** → Reuse the existing provider and host tests through the new seam, retain all limits and packaged fixtures, and require parity smoke tests before removing the built-in path.
- **[Two active OpenSpec changes describe conflicting PDF distribution]** → Update `add-pdf-document-viewing` proposal/design/release delta/tasks before archive, validate both strictly, and archive only after their combined final behavior is consistent.

## Migration Plan

1. Record an immutable control revision and run the native host-only and extracted-PDF-plugin launch/size spike. Stop if any host or plugin budget, platform execution policy, or protocol memory limit fails.
2. Add manifest/catalog types, signature verification, safe ZIP validation, atomic store, and in-memory adapters without enabling dynamic file handlers.
3. Add the framed protocol, supervised process adapter, deterministic fixture worker, crash/timeout/backpressure tests, and cross-platform packaged launch checks.
4. Add the host plugin manager, signed bootstrap/remote catalog, localized install/update/rollback/disable/uninstall flows, and exact storage accounting.
5. Extract the generic paged-document host module, replace hard-coded PDF tab/classifier state with provider identity, and prove document/cache invariants with an in-memory provider.
6. Convert `markion-pdf-viewer` into the official PDF worker, remove it and PDFium from the root package, and rerun every existing PDF behavior/native fixture through the installed plugin path.
7. Publish signed native plugin archives/catalog and size reports in CI, mirror them, and verify core packages contain no optional payload.
8. Reconcile `add-pdf-document-viewing` artifacts with the optional distribution, run both strict validations plus the repository quality gate, then complete native manual smoke testing before either change is archived.

Rollback keeps user documents untouched. A build may disable plugin discovery and return to the prior built-in PDF implementation until extraction is released. After release, rolling back the app leaves plugin payloads inert when their host range is incompatible; users can remove them from the compatible app's plugin manager or documented data directory. Catalog rollback re-points to the last verified compatible plugin version without modifying PDFs or Markdown files.
