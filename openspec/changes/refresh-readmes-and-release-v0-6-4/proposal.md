## Why

The READMEs still link v0.6.3 downloads and present v0.6.1 as the latest highlights, while 75 fixes and two Visual Edit improvements have landed on `main` since the `v0.6.3` tag. The repository quality gate has also been failing at its formatting step, so its workspace tests have not run on CI. Publish the completed work as the next patch release with current, equivalent documentation and reproducible verification.

## What Changes

- Restore `cargo fmt --all -- --check` by formatting the six drifted source files. Formatting only; no behavior change.
- Refresh all eight README editions for v0.6.4: the download table and pacman command, a highlights section that reflects this release's user-visible improvements, and any statement the documentation audit found outdated. Historical screenshots and performance measurements keep their labels.
- Write concise English-first/Chinese release notes and a preflight record linked to current evidence; explicitly record unavailable acceptance and obtain the release-specific scope decision before tagging.
- Synchronize workspace, application, lockfile and packager versions; run the canonical preflight and publish an annotated new tag after required acceptance or explicitly authorized exceptions.
- Verify all native builds, packages, updater signatures/manifests, OSS mirror and final GitHub metadata; then update project status and linked guidance.

Non-goals: new application features, new performance claims, rewriting existing public tags, or silently waiving manual acceptance. The existing Markdown version/cache invariants are unaffected by formatting, documentation and version metadata changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is documentation and release execution under the existing project-documentation and release-packaging contracts; `skip_specs: true` applies. Application behavior is covered by its existing changes.

## Impact

Six source files (formatting only), eight root READMEs, linked current guidance, release notes, verification records, Cargo.toml, Cargo.lock and packager.toml; GitHub Actions, Release assets and the existing OSS mirror.
