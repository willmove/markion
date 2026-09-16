## Why

The root documentation no longer reflects the latest user-visible behavior added after v0.3.9, and Chinese readers must currently leave the root README to see the equivalent overview. Before publishing the next patch release, the project needs one current, bilingual landing page whose English and Simplified Chinese halves stay aligned with the implemented product and release artifacts.

## What Changes

- Audit the current implementation, stable OpenSpec requirements, completed changes since v0.3.9, and packaging configuration, then refresh user-facing README claims.
- Place the complete Simplified Chinese edition in the lower half of `README.md`, after the English edition, with clear in-document language navigation.
- Keep `README.zh-CN.md` updated to the same Chinese content for existing links and direct-language entry points.
- Synchronize Markion-controlled version metadata for the default next patch version, v0.3.10.
- Run the repository's full verification and canonical release workflow, including bilingual GitHub release notes and final asset, updater, mirror, and metadata checks.
- Non-goals: changing application behavior, persisted formats, rendering architecture, or packaging targets.

This documentation-and-release change does not touch the cached-per-document Markdown state, syntax-highlighting memoization, cached text handles, bounded file-tree rendering, or the GPUI-free workspace-member boundary.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-documentation`: Require the root README itself to contain current, equivalent English and Simplified Chinese editions while retaining a synchronized standalone Chinese entry point.

## Impact

- Documentation: `README.md` and `README.zh-CN.md`.
- Specification: the bilingual project-documentation contract.
- Release metadata: `Cargo.toml`, `Cargo.lock`, and `packager.toml`.
- External systems: the `main` branch, annotated `v0.3.10` tag, GitHub Actions release workflow, GitHub Release, and Aliyun OSS mirror.
- Tooling prerequisite: GitHub CLI must be available and authenticated before publication; it was not present during initial preflight.
