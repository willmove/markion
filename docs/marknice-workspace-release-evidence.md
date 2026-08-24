# MarkNice workspace release evidence

Complete this matrix for every release that includes the local publishing
workspace. Automated rows are enforced by the quality and release workflows;
manual rows must record the version, browser/application version, result, and
tester before archive or publication.

| Evidence | Windows | macOS Apple Silicon | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| Packaged bundle extraction and digest verification | CI | CI | CI | CI |
| Default-browser dispatch | v0.2.1 / Edge 151.0.4129.101 / Pass / willmove | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer |
| Offline shell, themes, typography, and math rendering | v0.2.1 / Edge 151.0.4129.101 / Pass / willmove | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer |
| Remote and managed-image preview; explicit omission copy | v0.2.1 / Edge 151.0.4129.101 / Pass / willmove | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer |
| Clipboard-denied feedback, expiry, and repeated sessions | v0.2.1 / Edge 151.0.4129.101 / Pass / willmove | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer | Deferred for v0.2.1 by maintainer |

## Source-tree browser evidence

On 2026-08-20 the checked-in bundle was exercised on Windows with Microsoft
Edge 151 through the Playwright maintainer harness. The 15-theme normalized
golden corpus passed, and the browser loaded every application script, style,
font, claim, document, heartbeat, and protected-image request from loopback.
Theme, typography, desktop/phone, math, tab-local editing, rich HTML/plain-text
copy, clipboard denial, local-image cancel/explicit omission, safe copied-HTML
URL assertions, one-time fragment removal, and localized expiry guidance all
passed. For v0.2.1, the maintainer additionally verified the packaged Windows
default-browser workflow in the same Edge version and accepted the Windows
result as the manual browser release gate. macOS and Linux manual browser
coverage is explicitly deferred; their native package extraction, bundle
digest verification, and build results remain mandatory in the tag workflow.

## WeChat paste evidence

On 2026-08-23, maintainer `willmove` verified the representative rich-copy
payload in WeChat 4.1.13.7 on Windows. Headings, paragraphs and breaks, nested
lists, tables, code, math, links, remote images, plain-text fallback, and the
explicit local-image omission behavior passed.

## v0.2.1 release decision

On 2026-08-23, maintainer `willmove` explicitly authorized publishing v0.2.1
with the completed Windows Edge and WeChat evidence above. The unperformed
macOS and Linux manual browser checks are deferred rather than represented as
passes. Publication still requires successful Windows, macOS, and Linux native
build/package verification and all tag-only signing, release, and mirror jobs.

## v0.2.2 carry-over

v0.2.1..v0.2.2 contains no changes under `assets/`, so v0.2.2 packages the
byte-identical MarkNice workspace bundle. `verify-bundle` passes and reports
the same bundle revision `c009c1ec7e7c92f89afa5a32edcb126b5296bda7` pinned in
`bundle-manifest.json` for v0.2.1. The Windows Edge default-browser rows, the
WeChat paste evidence, and the maintainer-authorized macOS/Linux
manual-browser deferral therefore carry over to v0.2.2 unchanged. The automated
rows (packaged bundle extraction and digest verification on all three native
platforms) re-ran in the v0.2.2 tag workflow.
