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

## Runtime gate versus release gate

Two distinct verification levels apply to the workspace. **Release
verification** (`verify_bundle`, the `verify-bundle` CLI, and every
pre-publication check) stays exhaustive: every manifest-listed file must match
its LF-normalized digest, no unlisted file may exist, and remote-dependency
and prohibited-artifact scans must pass. **The runtime launch gate**
(`verify_launch_gate`, used by `discover_workspace_assets` and
`WorkspaceService::new`) checks only that the manifest parses with valid
provenance and the entry shell `index.html` matches its recorded digest. The
split is deliberate: the Windows NSIS installer and the in-app updater install
by overwriting and never remove files a newer package no longer ships, so an
upgraded install legitimately contains unlisted leftovers, and launch-time
full-bundle equality would hard-fail publishing for every upgrading user.

On 2026-08-27, maintainer `willmove` verified the launch gate on the affected
Windows machine whose Markion install was upgraded in place from v0.1.24
(KaTeX-era workspace) to v0.2.2 (MathJax-era workspace): the install directory
retains 63 orphaned KaTeX files beside the 21-file manifest. The
`preview-workspace` harness pinned to that directory via
`MARKION_MARKNICE_WORKSPACE_DIR` created a session and served the shell,
self-test page, and static assets over loopback. Before the gate split, the
same directory failed with
`the local publishing workspace contains an unlisted runtime file` on every
publish attempt.

## v0.2.7 source-tree evidence

v0.2.6..v0.2.7 reworks the workspace bundle itself (MarkNice editor skin,
session-local Word import, themed print-to-PDF, vendored JSZip), so the
v0.2.2 byte-identical carry-over no longer applies. On 2026-09-02, before
tagging, a ZCode agent session on Windows 10 x64 re-verified the changed
bundle from the source tree:

- `verify-bundle` passed on `assets/marknice-workspace`: 23 files,
  3,141,109 bytes, MarkNice revision `c009c1ec7e7c92f89afa5a32edcb126b5296bda7`.
- `preview-workspace` served a fresh session over loopback. The full
  `static/self-test.html` suite reported
  `PASS (15 themes + formatting + exports + skin + word + pdf)`: every theme
  digest matched the golden corpus, and the formatting/shortcut, export,
  locale, editor-skin, Word-import, and themed print-to-PDF checks (92
  assertions) all passed in the ZCode 3.10.2 in-app Chromium 146.0.7680.80
  (Electron 41.0.3) browser — not the OS default browser.
- The live session exposed and rendered the new chrome: formatting toolbar,
  Import Word control, 15-template selector, font-size/paragraph-spacing
  steppers, Desktop/Phone modes, Copy for WeChat / Copy MD / Save as HTML /
  Save as PDF / Save as Word actions, and the themed article (heading, list,
  table, managed image). A screenshot confirmed the dual rounded cards,
  traffic-light headers, indigo accent, and phone frame.

Carry-overs that still apply, with their reasons:

- The WeChat rich-paste payload path is unchanged in v0.2.7: the
  `navigator.clipboard` write logic in `bridge.js` and the rich-HTML
  generation in `export-runtime.js` received only label/locale changes, so the
  2026-08-23 WeChat 4.1.13.7 Windows paste evidence carries over.
- The default-browser dispatch and loopback/session service code are
  unchanged (`crates/wechat-workspace` gained only manifest allowlist entries
  for the new vendored files), so the v0.2.1 packaged Windows Edge dispatch
  evidence carries over. The packaged-bundle extraction and digest rows re-ran
  in the v0.2.7 tag workflow.
- macOS and Linux manual browser checks remain deferred (maintainer-authorized
  since v0.2.1), not represented as passes; their native packaging,
  extraction, and digest verification remain mandatory in the tag workflow.
