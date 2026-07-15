## Why

The root README no longer matches the implemented application: it omits Visual Edit and folder-opening workflows, still describes a preview table toolbar that has been removed, and has no Chinese edition for the project's Simplified Chinese audience. The documentation should reflect the current v0.1.2 codebase and provide equivalent entry points in English and Chinese.

## What Changes

- Refresh `README.md` against the current OpenSpec specifications, active implementation, package metadata, and release workflow.
- Add language navigation between the English README and a new `README.zh-CN.md`.
- Document the four current view modes, source-backed Visual Edit behavior and limits, workspace/file-tree workflows, configuration, export fallbacks, platform packaging, and development commands.
- Remove or correct stale claims, especially the removed preview table toolbar and the claim that single-surface visual editing is unimplemented.
- Keep both language editions structurally aligned and factually equivalent.

Non-goals: changing application behavior, release packaging, user-visible application strings, or stable capability requirements.

## Capabilities

### New Capabilities

- `project-documentation`: Maintain discoverable, factually aligned English and Simplified Chinese project overviews for users and contributors.

### Modified Capabilities

None. No OpenSpec requirement changes are needed.

## Impact

- Documentation: `README.md` and new `README.zh-CN.md`.
- Planning artifacts: this change folder records and validates the documentation refresh.
- No Rust code, APIs, dependencies, persisted formats, or performance invariants are affected.
