## ADDED Requirements

### Requirement: Core installers SHALL exclude optional plugin payloads and stay size-gated
The Windows NSIS, macOS app/DMG, Linux DEB, and Linux AppImage core distributions SHALL contain the plugin host and signed official catalog metadata but SHALL NOT contain an official plugin executable, PDFium runtime, plugin archive, plugin extraction cache, or another target's plugin assets. Against a same-environment PDF-disabled pre-plugin control, the complete plugin-host addition SHALL increase no compressed core artifact by more than 1 MiB and no installed/staged core payload by more than 2 MiB. CI SHALL retain byte-exact control, candidate, and delta reports and fail rather than relaxing either limit.

#### Scenario: Core package is inspected
- **WHEN** any supported core installer or application payload is assembled
- **THEN** it contains the plugin host/catalog support but no optional plugin worker or PDFium library
- **AND** packaged verification fails on any leaked plugin payload or foreign-target artifact

#### Scenario: Plugin-host size budget is measured
- **WHEN** the host-enabled core is compared with its same-toolchain PDF-disabled pre-plugin control
- **THEN** compressed growth is at most 1 MiB and installed/staged growth is at most 2 MiB
- **AND** the report identifies every materially changed core file

#### Scenario: Host budget is exceeded
- **WHEN** either plugin-host size delta exceeds its configured ceiling
- **THEN** native CI fails and does not publish the candidate as a completed core release
- **AND** increasing the budget requires a later explicit OpenSpec change

### Requirement: Official plugin artifacts SHALL be target-specific, signed, and independently downloadable
Every official native plugin release SHALL produce one package for each supported target it declares. Each package SHALL contain only its signed manifest, target entry point, required target-specific resources, and applicable notices. The release pipeline SHALL publish byte length and SHA-256 metadata in a signed catalog, verify installation and handshake from the final archive, and make the plugin independently downloadable without placing it in the core installers. A plugin release failure SHALL not produce a catalog entry that names a missing, unsigned, or unverified artifact.

#### Scenario: Tagged release publishes the PDF plugin
- **WHEN** a tagged Markion release includes a compatible official PDF plugin version
- **THEN** the release pipeline publishes verified Windows x86_64, macOS arm64, and Linux x86_64 plugin archives plus their signed catalog entries
- **AND** each archive contains exactly one matching PDFium runtime and no runtime for another target

#### Scenario: Catalog or artifact verification fails
- **WHEN** a plugin signature, digest, compatibility declaration, packaged smoke test, or expected release asset is invalid or missing
- **THEN** the invalid plugin version is omitted from the published catalog and the release is not reported complete
- **AND** an earlier valid catalog remains usable

### Requirement: The official PDF plugin SHALL retain independent size budgets
For every supported target, the final compressed PDF plugin archive SHALL be no larger than 6 MiB and its extracted installed payload SHALL be no larger than 10 MiB. Measurements SHALL include the worker executable, manifest, notices, PDFium runtime, and every shipped resource. CI SHALL report exact archive and extracted bytes plus the largest files and SHALL fail when either limit is exceeded.

#### Scenario: PDF plugin stays within budget
- **WHEN** a target-specific PDF plugin archive is built
- **THEN** its archive is at most 6 MiB and its extracted payload is at most 10 MiB
- **AND** the retained report identifies the worker and PDFium contributions

#### Scenario: PDF plugin exceeds its budget
- **WHEN** either PDF plugin size ceiling is exceeded
- **THEN** the plugin packaging job fails and no catalog entry references that artifact
- **AND** the core application package remains independently buildable without the plugin

