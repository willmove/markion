## ADDED Requirements

### Requirement: Git failure handling SHALL NOT depend on locale or message language
Markion SHALL classify every failed Git operation it runs into one of these categories:
- cancelled; timed out; offline or unreachable before connecting; connection interrupted during transfer;
- authentication needed;
- SSH host identity unknown; SSH host identity changed; encrypted key needs a passphrase;
- sync location not found or inaccessible;
- missing sync branch;
- upload rejected because the sync location has newer history;
- upload refused by the sync location;
- repository locked by another program;
- other.

Classification SHALL use signals that are independent of the user's locale and interface language: exit status, machine-readable output, and Git messages pinned to a fixed language.

Every decision that depends on a failure SHALL produce the same result for every user locale. This covers:
- the single automatic refresh-and-retry after a rejected upload;
- the authentication retry offer;
- quiet backoff of background checks when offline;
- pausing background checks for authentication;
- reporting a missing sync branch.

Pinning Git's message language SHALL NOT alter path or content bytes.

#### Scenario: Another device pushed first under a non-English locale
- **WHEN** the user's locale is `zh_CN.UTF-8`, Git translations are installed, and the upload is rejected because another device updated the sync branch
- **THEN** Markion performs the same single refresh, safe integration, and retry it performs under an English locale

#### Scenario: Offline background check under a non-English locale
- **WHEN** an enabled background check cannot reach the sync location while the user's locale is not English
- **THEN** the check is treated as offline, stays quiet, and backs off instead of reporting an error

#### Scenario: Sync branch was deleted remotely
- **WHEN** the bound sync branch no longer exists at the sync location and the user's locale is not English
- **THEN** Markion reports that the sync branch is missing instead of a generic failure, and does not recreate it

#### Scenario: Sign-in failure under a non-English locale
- **WHEN** a foreground sync fails because credentials were refused and the user's locale is not English
- **THEN** Markion reports that sign-in is needed, keeps local work, and offers to retry the requested operation

### Requirement: Upload failures SHALL distinguish definite non-delivery from uncertain delivery
When an upload fails before the sync location could have accepted the captured commit, Markion SHALL report the commit as saved on this computer and waiting to upload. It SHALL NOT report it as uncertain delivery, and SHALL NOT make an additional network attempt only to verify delivery. This applies when:
- the location is unreachable;
- sign-in is refused;
- SSH host identity verification fails;
- the location is not found or inaccessible;
- the location explicitly rejects the update.

Uncertain delivery SHALL be reserved for failures where the transfer may have reached the sync location: interruption, timeout, cancellation during upload, or an unclassified failure. These SHALL keep the existing verification against remote history before delivery is declared or the upload is repeated.

#### Scenario: Upload attempted while offline
- **WHEN** Sync Now has created a local commit and the upload fails because the sync location's host cannot be resolved
- **THEN** the commit is reported as waiting to upload with an offline explanation, not as uncertain delivery

#### Scenario: Server refuses the upload
- **WHEN** the sync location rejects the update for a reason other than newer remote history
- **THEN** the commit is reported as waiting to upload with an explanation that the sync location refused it, and the local commit is retained

#### Scenario: Connection drops during upload
- **WHEN** the upload process is interrupted after transfer may have started
- **THEN** delivery is treated as uncertain and Markion checks remote history before declaring delivery or uploading again

### Requirement: Ordinary failure feedback SHALL describe the classified cause
Ordinary status and Backup and Sync feedback for a failed Git operation SHALL be a message for its failure category in the interface language. The message SHALL state whether local work is safe and give a next step. Raw Git output SHALL appear only in Advanced Git details, redacted and bounded. Ordinary surfaces SHALL NOT display untranslated Git output or internal identifier names.

#### Scenario: Offline in a non-English interface
- **WHEN** the interface language is Japanese and a sync fails because the sync location is unreachable
- **THEN** the status shows a Japanese message saying the notes are saved on this computer and the upload is waiting
- **AND** the underlying Git error text is available only in Advanced Git details

#### Scenario: Background check finds rewritten remote history
- **WHEN** a background check detects that the sync branch history was rewritten
- **THEN** the status shows a localized explanation that the sync location's history needs review, not an internal state name
