## ADDED Requirements

### Requirement: Bundled PDF viewing SHALL be target-scoped and size-gated
Every supported release package SHALL contain exactly one runtime PDF rendering library for its own operating system and architecture, pinned by version and cryptographic digest, and SHALL exclude runtime libraries for other targets plus PDF JavaScript, V8, XFA, development headers, import libraries, debug symbols, sample documents, and bundled font packs. The Windows x86_64 NSIS installer, macOS arm64 DMG/app payload, Linux amd64 DEB, and Linux x86_64 AppImage SHALL load that bundled runtime without a separately installed PDF tool or network fetch. Third-party license notices SHALL be shipped with the runtime.

For this change, package and installed-payload sizes SHALL be compared with a PDF-disabled control built from the pre-change revision on the same native runner, with the same locked Rust toolchain, dependency lockfile, release profile, assets, and cargo-packager version. No compressed installer/package artifact may grow by more than 6 MiB, and no installed or staged application payload may grow by more than 10 MiB (1 MiB = 1,048,576 bytes). CI SHALL emit baseline, candidate, and delta byte counts for every format and fail when a budget is exceeded; increasing a budget requires an explicit later OpenSpec change.

#### Scenario: Native packages contain one matching runtime
- **WHEN** the release matrix packages Markion for Windows x86_64, macOS arm64, or Linux x86_64
- **THEN** the installed payload contains exactly the PDF runtime for that target and no runtime for another target
- **AND** packaged verification launches or probes PDF loading through the path the installed application will use

#### Scenario: Package-size delta stays within budget
- **WHEN** a candidate NSIS, DMG, DEB, or AppImage is compared with its same-environment PDF-disabled control
- **THEN** the candidate artifact is no more than 6 MiB larger
- **AND** CI records both byte counts and the computed delta in a retained size report

#### Scenario: Installed-payload delta stays within budget
- **WHEN** the candidate installed or staged application tree is compared with its same-environment PDF-disabled control
- **THEN** the candidate payload is no more than 10 MiB larger
- **AND** the report identifies the bundled PDF runtime and every other file contributing materially to the delta

#### Scenario: Size budget regression blocks packaging
- **WHEN** either the 6 MiB packaged-artifact budget or 10 MiB installed-payload budget is exceeded
- **THEN** the native packaging job fails and does not publish that candidate as a completed release
- **AND** the limit is not automatically raised or bypassed

#### Scenario: Runtime acquisition is integrity checked
- **WHEN** a native build stages the pinned PDF runtime
- **THEN** it verifies the downloaded bytes against the repository-controlled digest before packaging
- **AND** a missing or mismatched runtime fails the build rather than falling back to a network download at application startup
