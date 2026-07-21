## Why

The maintainer has directed that curated GitHub release notes default to a bilingual format: English first, followed by the corresponding Simplified Chinese version. The release-packaging spec, release runbook, and agent guidance currently default to English only, so future releases would publish English-only notes against the maintainer's standing preference.

## What Changes

- Change the default language format for curated GitHub Release notes in the `release-packaging` spec from English-only to bilingual: the English summary first, then the corresponding Simplified Chinese summary; another language arrangement still applies when the requester explicitly asks.
- Update the canonical runbook `docs/release-process.md`: the default-language rule and the release-notes template become bilingual (English sections followed by their Chinese counterparts).
- Update the release workflow guidance in `AGENTS.md` to match the bilingual default.

## Capabilities

### Modified Capabilities

- `release-packaging`: The curated-release-information requirement now defaults the notes to a bilingual English-then-Simplified-Chinese summary instead of English only; all other publication requirements are unchanged.

## Impact

- Documentation and guidance only: `openspec/specs/release-packaging/spec.md` (via this change's delta), `docs/release-process.md`, and `AGENTS.md`.
- No code, build, packaging, or workflow behavior changes; historical releases keep their existing notes.
