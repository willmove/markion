## Context

See proposal.md. This release follows v0.6.0 and includes completed application changes already planned independently. The release runbook requires scoped, explicit acceptance exceptions for unavailable native environments.

## Goals / Non-Goals

Keep all language entry points equivalent and all verification claims tied to their actual build. No changes to Markdown models, persisted document formats, render caches, update trust keys or public tags.

## Decisions

- Use v0.6.1, the required patch increment from the highest stable remote tag. Commit existing completed block-edit work separately, then create the dedicated version commit.
- Retain v0.6.0 screenshot captions and the dated historical benchmark. Remove stale download sizes until actual assets exist; do not relabel historical evidence.
- Use the maintained acceptance suite and preserve missing-platform/manual results. Request a release-specific exception only after the candidate, notes and source-verification report are reviewable. Automated packaging, signatures, assets and public mirror checks remain mandatory.
- Preserve annotated public tags. Workflow failures require investigation, safe retries or a forward fix; never move the tag without explicit authorization.

## Risks / Trade-offs

- Unavailable native environments cannot be inferred from unit tests or CI packaging → disclose concrete gaps and retain NOT_RUN/BLOCKED results.
- Signing mode depends on CI credentials → inspect both macOS jobs and publish actual status and necessary first-launch instructions.
- Main and tag workflows run separately → use the tag run as publication authority, retaining final asset hashes and channel-specific updater checks.

## Migration Plan

Existing settings load backward-compatible defaults for new spacing and update options; Markdown files remain unchanged. No manual migration is required. Public publication is not rolled back by tag deletion; fix forward if necessary.
