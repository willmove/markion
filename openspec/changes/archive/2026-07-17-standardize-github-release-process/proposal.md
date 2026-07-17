## Why

Markion's GitHub release pipeline builds the right artifacts, but the operator steps used to choose a version, verify the workspace, publish the tag, monitor CI, and replace sparse generated notes are not recorded. A repository-owned runbook and agent instruction are needed so future releases repeat the verified v0.1.5 process instead of relying on chat history.

## What Changes

- Define a canonical GitHub release runbook covering version selection, synchronized metadata updates, validation, commit and annotated tag creation, push, CI monitoring, release-note editing, and final verification.
- Require release notes to summarize user-visible changes, compatibility or migration information, available installers, verification results, and the full comparison link rather than leaving only auto-generated notes.
- Record the runbook in `AGENTS.md` as the default release procedure for future repository agents.
- Preserve the current native Windows, macOS, and Linux packaging workflow and unsigned-installer policy.
- Non-goals: changing artifact formats, adding code signing, adding an auto-update channel, or publishing crates to crates.io.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-packaging`: Add operator requirements for repeatable version publication, detailed GitHub Release notes, and post-publication verification.

## Impact

- Adds `docs/release-process.md` as the canonical operator runbook.
- Updates `AGENTS.md` so future automated work follows that runbook by default.
- Adds a delta to the `release-packaging` specification; no Rust code, file format, runtime dependency, or packaging format changes.
- Uses existing `cargo`, `git`, `gh`, and GitHub Actions tooling and preserves the native build matrix.
