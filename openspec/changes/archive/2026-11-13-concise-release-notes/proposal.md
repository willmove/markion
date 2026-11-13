## Why

The maintainer found the v0.5.3 release notes too long. Future releases need a concise user-facing summary with detailed evidence available through links.

## What Changes

- Require brief English-first/Chinese release notes with 3–5 combined highlights/fixes per language and compact compatibility, downloads and verification lines.
- Move implementation history, benchmark tables and detailed test/retry records to linked documentation; preserve essential upgrade and first-launch instructions.
- Align the release runbook template and AGENTS.md with the same policy.

Non-goals: rewriting existing published notes, changing release gates, or application behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-packaging`: make concise curated release information the default while retaining required coverage and accurate verification.

## Impact

AGENTS.md, docs/release-process.md and the release-packaging specification through archival. No runtime, dependency, version or cached-per-document architecture changes.
