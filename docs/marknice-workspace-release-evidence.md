# MarkNice workspace release evidence

Last reconciled: 2026-11-13. This restores the evidence entry point referenced by the [release process](release-process.md). A supported package containing a verified bundle does not by itself establish browser, clipboard or Microsoft Word interoperability.

## Existing evidence

The [workspace guide](marknice-workspace.md) records Windows Copy Markdown and browser Word-export checks on 2026-08-23, plus editor-skin, browser Word-import fixture and themed print-clone checks added on 2026-09-02. It does not record exact browser/Word versions or a complete platform matrix. These partial records are retained as such; no new manual run is claimed here.

## Required manual matrix

| Platform | Browser/version | Required interactions | Recorded result |
| --- | --- | --- | --- |
| Windows x86_64 | Default browser; exact version pending | Launch from Markion, themed preview, Copy WeChat, Copy Markdown, HTML/DOCX download, Word import and PDF print | Partial historical Copy Markdown/Word-export evidence; full row pending |
| macOS Apple Silicon | Default browser; exact version pending | Same interactions and comparison with the pinned editor skin | Pending |
| macOS Intel | Default browser; exact version pending | Same interactions | Pending |
| Linux X11 | Default browser; exact version pending | Same interactions and clipboard interoperability | Pending |
| Linux Wayland | Default browser; exact version pending | Same interactions and clipboard interoperability | Pending |
| Microsoft Word | Version/platform pending | Open generated DOCX; inspect headings, lists, tables, images, formulas and typography | Pending documented viewer/version |
| Secondary DOCX viewer | Product/version pending | Best-effort fidelity check; record differences explicitly | Pending |

For each applicable browser, also record clipboard permission denial, cancelled downloads, repeated actions, expired sessions, local-image omission, large-image fallback, offline runtime availability and preservation of the native Markion document. Record the tested app commit/build, operating-system and browser versions, fixture, outcome and evidence path/date. Browser-generated Word/PDF must remain distinct from native/Pandoc export.

## Packaging and publication

Source and extracted-package verification checks the pinned manifest, local dependencies and digests through `verify-bundle` and the release workflow. Record the actual tag workflow and package checks in the version's release preflight report; do not fill the manual rows from CI success.

The [content-export checklist](../openspec/changes/add-marknice-workspace-content-exports/tasks.md) still has pending manual/browser/viewer tasks. Under the canonical release process, pending required rows block a release-readiness claim.

## v0.5.3 follow-up

The [release preflight](verification/release-v0.5.3-preflight.md) records the maintainer’s release-specific authorization to skip unavailable manual environments. The Codex in-app browser passed the 15-theme compatibility self-test and formatting/export/locale/skin/Word/print checks; actual Markdown copy and HTML/DOCX downloads also passed. See [raw results](verification/marknice-browser-self-test-v0.5.3.json), [screenshot](verification/marknice-browser-self-test-v0.5.3.png) and [download checks](verification/marknice-browser-downloads-v0.5.3.json). The interface did not expose an engine/version, and this is not the default-browser or WeChat/Word-client matrix. The remaining rows above stay unverified and are skipped for v0.5.3, without being marked as passed.
