## Context

See proposal.md. This release follows v0.6.3 and ships application changes already planned and completed under their own changes. The release runbook requires scoped, explicit acceptance exceptions for unavailable native environments.

## Goals / Non-Goals

Keep all language entry points equivalent and all verification claims tied to their actual build. No changes to Markdown models, persisted document formats, render caches, update trust keys or public tags.

## Decisions

- Use v0.6.4, the required patch increment from the highest stable remote tag. Commit the formatting-only repair separately, then the documentation, then create the dedicated version commit.
- The README download table must name the version being published; the highlights section is refreshed to the release being published rather than relabeled history. Retain the dated historical benchmark and existing screenshot captions; do not relabel historical evidence.
- Use the maintained acceptance suite and preserve missing-platform/manual results. Request a release-specific exception only after the candidate, notes and source-verification record are reviewable. Automated packaging, signatures, assets and public mirror checks remain mandatory.
- Preserve annotated public tags. Workflow failures require investigation, safe retries or a forward fix; never move the tag without explicit authorization.

## Risks / Trade-offs

- Unavailable native environments cannot be inferred from unit tests or CI packaging → disclose concrete gaps and retain NOT_RUN/BLOCKED results.
- Formatting repair is verified by a whitespace-insensitive review plus the complete workspace suite on the exact release tree.
- Signing mode depends on CI credentials → inspect both macOS jobs and publish actual status and necessary first-launch instructions.
- Main and tag workflows run separately → use the tag run as publication authority, retaining final asset hashes and channel-specific updater checks.

## Migration Plan

Visual Edit code folding becomes opt-in in this release; the preference and its default are disclosed in the release notes. Markdown files remain unchanged and no manual migration is required otherwise. Public publication is not rolled back by tag deletion; fix forward if necessary.
