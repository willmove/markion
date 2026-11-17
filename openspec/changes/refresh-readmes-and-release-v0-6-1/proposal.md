## Why

The READMEs still download v0.6.0 and describe Korean as source-only. Publish the completed post-v0.6.0 improvements as the next patch release with current, equivalent documentation and reproducible verification.

## What Changes

- Refresh all eight README editions for v0.6.1, covering Korean UI, configurable document spacing, background updates and content-preserving block insertions.
- Keep historical screenshots and performance measurements accurately labeled; link concise bilingual release notes and verification records.
- Synchronize workspace, application, lockfile and packager versions; run the canonical preflight and publish an annotated new tag after required acceptance or explicitly authorized exceptions.
- Verify all native builds, packages, updater signatures/manifests, OSS mirror and final GitHub metadata.

Non-goals: new application features, new performance claims, rewriting existing public tags, or silently waiving manual acceptance. The existing Markdown version/cache invariants are unaffected by documentation and version metadata changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is documentation and release execution under the existing project-documentation and release-packaging contracts; `skip_specs: true` applies. Application behavior is covered by its existing changes.

## Impact

Eight root READMEs, linked current guidance, release notes, verification records, Cargo.toml, Cargo.lock and packager.toml; GitHub Actions, Release assets and the existing OSS mirror. Existing completed block-edit changes will be reviewed and committed separately before the dedicated release commit.
