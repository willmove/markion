# MarkNice workspace release evidence

Complete this matrix for every release that includes the local publishing
workspace. Automated rows are enforced by the quality and release workflows;
manual rows must record the version, browser/application version, result, and
tester before archive or publication.

| Evidence | Windows | macOS Apple Silicon | Linux X11 | Linux Wayland |
| --- | --- | --- | --- | --- |
| Packaged bundle extraction and digest verification | CI | CI | CI | CI |
| Default-browser dispatch | Pending | Pending | Pending | Pending |
| Offline shell, themes, typography, and math fonts | Pending | Pending | Pending | Pending |
| Remote and managed-image preview; explicit omission copy | Pending | Pending | Pending | Pending |
| Clipboard-denied feedback, expiry, and repeated sessions | Pending | Pending | Pending | Pending |

## Source-tree browser evidence

On 2026-08-20 the checked-in bundle was exercised on Windows with Microsoft
Edge 151 through the Playwright maintainer harness. The 15-theme normalized
golden corpus passed, and the browser loaded every application script, style,
font, claim, document, heartbeat, and protected-image request from loopback.
Theme, typography, desktop/phone, math, tab-local editing, rich HTML/plain-text
copy, clipboard denial, local-image cancel/explicit omission, safe copied-HTML
URL assertions, one-time fragment removal, and localized expiry guidance all
passed. This source-tree evidence does not replace packaged default-browser,
macOS/Linux, installer-size, or real WeChat paste verification below.

WeChat paste verification is also pending. Paste the compatibility corpus into
the WeChat editor and record headings, paragraphs and breaks, nested lists,
tables, code, math, links, remote images, plain-text fallback, and the reported
local-image omission count. Record installer/package size deltas for all four
native distributions alongside the results.
