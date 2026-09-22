## ADDED Requirements

### Requirement: Automatic sync commit messages SHALL carry date time and file context compactly
The default message for automatic sync commits SHALL be a single compact line combining a stable prefix, the commit's local date and time at minute precision, and a bounded list of the changed files' names, in the form `Sync notes 2026-09-18 14:32: a.md, journal.md, +3 more`.

The file list SHALL use file names rather than full paths, SHALL list at most three names, and SHALL summarize any remainder as `+N more`; when the combined line would exceed a compact subject length, names SHALL be dropped in favor of the summary tail so the line stays short. When the local time zone cannot be determined, the message SHALL use a well-defined fallback time instead of failing the sync. Existing persisted policies with an empty message template SHALL produce the new default format without any migration step.

User-editable message templates SHALL support `{date}`, `{time}`, `{files}`, and `{count}` placeholders; `{files}` SHALL render the same bounded file-name list. Manually authored version-history draft messages SHALL NOT be reformatted or auto-filled by this behavior.

#### Scenario: Single changed file
- **WHEN** an automatic sync commits one changed file named `a.md`
- **THEN** the commit message is one line like `Sync notes 2026-09-18 14:32: a.md` with the user's local date and time

#### Scenario: Many changed files stay compact
- **WHEN** an automatic sync commits seven changed files
- **THEN** the message lists at most three file names followed by a `+N more` tail for the rest
- **AND** the whole line remains within a compact subject length

#### Scenario: Message template placeholders
- **WHEN** a policy's message template contains `{date}`, `{time}`, `{files}`, or `{count}`
- **THEN** the committed message renders the local date, local time, bounded file-name list, and changed-file count in their places

#### Scenario: Existing policies keep working
- **WHEN** a policy saved before this change has an empty message template
- **THEN** automatic syncs produce the new date-time-and-files default without requiring the user to change any setting

#### Scenario: Version-history drafts stay manual
- **WHEN** the user creates a local version with a self-written message
- **THEN** that message is committed verbatim and is not rewritten into the automatic format
