## ADDED Requirements

### Requirement: PDF viewing SHALL be an independently packaged and size-gated official plugin
Every supported release SHALL build one target-specific `dev.markion.pdf`
archive containing the official worker, exactly one digest-pinned non-V8/non-XFA
PDFium runtime for that target, the signed canonical manifest, and PDF-specific
license notices. The archive SHALL exclude other-target runtimes, JavaScript,
V8, XFA, headers, import libraries, debug symbols, fixtures, samples, and font
packs. The worker SHALL open/render the packaged fixture offline through the
framed capability without an external PDF tool or runtime download.

The Windows x86_64, macOS arm64, and Linux x86_64 plugin archives SHALL each be
at most 6 MiB compressed and 10 MiB extracted. CI SHALL publish exact archive
and installed byte counts, SHA-256, signature, notices, and provenance, and
SHALL fail rather than increase a budget implicitly.

#### Scenario: Target plugin contains one matching runtime
- **WHEN** native CI packages the official PDF plugin
- **THEN** its verified manifest closes exactly the worker, matching PDFium runtime, and notices for that target
- **AND** native protocol smoke opens and renders a fixture from the extracted package

#### Scenario: Optional PDF package exceeds a budget
- **WHEN** its archive exceeds 6 MiB or its extracted members exceed 10 MiB
- **THEN** the native job fails and the catalog is not published
- **AND** no release references the rejected artifact

### Requirement: Core installers SHALL exclude optional PDF payloads
The core NSIS, DMG/app, DEB, and AppImage SHALL contain the reusable plugin host
and signed bootstrap catalog but SHALL contain no PDF worker, `.markion-plugin`
archive, installed/rollback plugin tree, PDFium library, or PDF-only notice. The
root application SHALL not depend on the PDF viewer crate. The reusable host
SHALL remain within its separately measured 1 MiB compressed and 2 MiB installed
same-runner budgets.

#### Scenario: Core package closure is inspected
- **WHEN** native CI extracts or mounts a core package
- **THEN** it finds the signed bootstrap catalog and no optional PDF payload
- **AND** the absence check runs for NSIS, DMG, DEB, and AppImage

### Requirement: Catalog and PDF assets SHALL be signed and published separately
Tagged CI SHALL populate the catalog only from plugin archives that passed
signature, digest, target, protocol, launch, native fixture, and size checks.
It SHALL sign the canonical catalog with the dedicated plugin key, compile the
core against the matching public key and catalog, publish versioned plugin
assets plus catalog/checksum/size/notice/provenance artifacts to GitHub Releases,
and mirror them to OSS. The plugin key SHALL NOT reuse the updater key.

#### Scenario: A native plugin build fails
- **WHEN** any referenced target archive fails verification or smoke testing
- **THEN** catalog preparation and core release packaging do not proceed
- **AND** no dangling catalog URL is published
