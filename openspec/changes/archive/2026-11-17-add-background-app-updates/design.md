## Context

See proposal.md for motivation. `src/app/update.rs` already maps GitHub assets, fetches OSS/GitHub manifests, and supports authenticated NSIS and AppImage installation. It currently combines download and installation. Startup preference fields exist but no scheduler uses them. The updater's Windows `install` implementation terminates the process directly, so Markion must own the Windows handoff to flush state and propagate launch errors.

## Goals / Non-Goals

**Goals:** make discovery periodic, stage authenticated packages independently of installation, and keep the final installation gate on the UI thread.

**Non-Goals:** cross-process or cross-session package reuse; additional self-update formats; automatic restarting.

## Decisions

- A weak-entity timer starts after bootstrap, delays briefly for startup, and checks due state at short bounded intervals. Persist RFC 3339 timestamps using `time`; manual attempts also advance the interval. A single update-operation state prevents overlaps. Tests and native replay do not start network timers.
- Keep `automatically_download_updates` independent of the legacy startup option. Read it when discovery finishes, so a preference change during discovery takes effect. Toggle-off affects future downloads only.
- Use cargo-packager-updater's separate `download` API, which authenticates the payload. Save verified bytes in a private temporary file; retain its SHA-256 digest and updater metadata in memory. Recheck the digest off-thread before installation to reject modified staged content. This avoids retaining a large installer in RAM while the user chooses Later. Temporary ownership cleans files on normal shutdown. Version mismatch fails safely instead of silently installing a different release.
- State flow: Idle → Checking → Downloading → Ready → Installing. A single shared busy guard covers network work; Ready survives Later and dirty-document rejection. The menu reopens a ready prompt. Background failures log quietly; explicit operations retain actionable dialogs.
- Reopening Ready from a menu reserves the prompt slot and schedules it after the action callback returns, since that callback already owns the GPUI window and a nested window-handle update would fail.
- At restart confirmation recheck dirty tabs, then read/validate staged bytes off-thread. Recheck dirty state again when preparation returns, before the final installation handoff. While installation is active, document-mutating UI must be gated until replacement/handoff finishes so no edits can race installation.
- Windows launches the staged verified `.exe` directly with `/P /R`, avoiding the updater's internal process exit and PowerShell launcher. Linux retains the existing replacement implementation and relaunch handling. Session/layout state is flushed before handoff. Fallback platforms keep browser downloads.
- All user-facing strings use `Msg` localization in all eight supported languages. UI displays only update state; updater work never reads or invalidates Markdown caches.

## Risks / Trade-offs

- [No persistent package cache] → Later retains downloads for the running session; closing Markion removes temporary staging and a later check can download again.
- [Slow network or large packages] → bounded manifest/download timeouts; downloaded bytes are released after staging.
- [Manifest mirror lag] → require the GitHub-discovered version; retain manual recovery and retry at the next check.
- [Platform differences] → existing signed-update gating remains; macOS and package-manager installations explicitly explain manual fallback.

## Migration Plan

Missing settings default to automatic downloads enabled. Existing startup and timestamp keys stay readable. No document-data migration or release-pipeline change is needed. Rollback ignores the added preference key.
