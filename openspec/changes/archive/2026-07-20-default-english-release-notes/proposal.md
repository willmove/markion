## Why

The maintainer has directed that curated GitHub release notes default to English. The current release-packaging spec, release runbook, and agent guidance all default to Simplified Chinese, so future releases would keep publishing Chinese notes against the maintainer's standing preference.

## What Changes

- Change the default language for curated GitHub Release notes from Simplified Chinese to English in the `release-packaging` spec; notes are still written in another language when the requester explicitly asks.
- Update the canonical runbook `docs/release-process.md`: the default-language rule and the release-notes template become English (section headers `Highlights`, `Fixes`, `Compatibility`, `Downloads`, `Verification`, `Full comparison`).
- Update the release workflow guidance in `AGENTS.md` to match the new English default.

## Capabilities

### Modified Capabilities

- `release-packaging`: The curated-release-information requirement now defaults the notes language to English instead of Simplified Chinese; all other publication requirements are unchanged.

## Impact

- Documentation and guidance only: `openspec/specs/release-packaging/spec.md` (via this change's delta), `docs/release-process.md`, and `AGENTS.md`.
- No code, build, packaging, or workflow behavior changes; historical releases keep their existing notes.
