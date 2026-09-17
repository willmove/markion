## Why

Markion is beginning to accumulate valuable but optional capabilities whose native runtimes, platform SDKs, credentials, and release cadence should not be imposed on every user. PDF viewing already adds roughly 3.2–3.6 MiB to compressed non-Windows packages and about 8 MiB to installed payloads, while planned publishing integrations have similarly specialized operational and trust requirements.

## What Changes

- Add an official first-party plugin platform with signed target-specific packages, a versioned manifest and protocol, per-user installation, atomic update/rollback, enable/disable/uninstall controls, compatibility checks, and an in-app plugin manager.
- Run native plugins out of process and on demand behind a supervised, length-framed control/binary protocol. Plugin crashes, hangs, protocol violations, and shutdown failures remain contained from the Markion process.
- Keep GPUI and all application chrome in the host. Plugins register narrow capability families and return typed data; they cannot inject arbitrary GPUI elements, access the Markdown model directly, or extend the application through an unstable Rust dynamic-library ABI.
- Introduce the first capability family for paged read-only documents, with host-owned scrolling, zoom, page navigation, raster caching, localization, and tab behavior.
- Convert the current PDF viewer into the first official plugin: remove `pdfium-render`, PDFium, and PDF-specific native ownership from the core application package; package the worker and exactly one target PDFium runtime as an independently downloadable signed plugin; preserve the implemented PDF behavior and limits.
- When a supported path needs an absent or disabled official plugin, keep the source file untouched and show localized install/enable guidance. Downloads require explicit user action; an installed PDF plugin continues to work offline.
- Publish a signed official plugin catalog and target artifacts alongside releases, with byte-exact package and installed-size reports. The plugin-host addition is capped at 1 MiB compressed and 2 MiB installed, and the PDF plugin remains capped at 6 MiB compressed and 10 MiB installed.
- Reconcile the still-active `add-pdf-document-viewing` change before either change is archived: retain its viewing, safety, cache, and interaction requirements while replacing its mandatory bundled-runtime requirements with optional-plugin distribution.

Non-goals: a public third-party marketplace, arbitrary native DLL loading, arbitrary plugin-provided UI, a security sandbox for untrusted native code, WASM/JavaScript/Lua runtimes, hot reload, plugins that mutate the live Markdown document, or implementing a Xiaohongshu/Toutiao/WeChat publisher in this change. A later change can add a publisher-job capability on the same package, supervision, and compatibility foundation.

## Capabilities

### New Capabilities

- `plugin-platform`: Signed first-party plugin discovery, installation, compatibility, lifecycle supervision, capability registration, failure containment, storage accounting, and the host/plugin protocol.
- `paged-document-plugins`: Host-rendered paged-document behavior supplied by an out-of-process provider, including the optional PDF plugin and missing-plugin recovery flow.

### Modified Capabilities

- `release-packaging`: Keep optional plugins out of the core NSIS, DMG, DEB, and AppImage; publish and verify signed per-target plugin artifacts and enforce separate core-host and plugin payload budgets.
- `workspace`: Resolve plugin-provided supported file types through the installed/official catalog while preserving bounded scanning, normal tab/path behavior, and a non-destructive missing-plugin flow.
- `ui-i18n`: Localize plugin management, compatibility, permission disclosure, install/update/rollback, missing-plugin, crash, timeout, and recovery chrome in every supported language.

## Impact

- Adds a small GPUI-free protocol/package module and root-crate plugin catalog, installer, supervisor, capability registry, and host-rendered paged-document adapter.
- Replaces the root application's hard-coded `WorkspaceTab::Pdf` and PDF service ownership with content-independent plugin-document state while retaining document-only guards and cached Markdown invariants.
- Removes the root dependency on `markion-pdf-viewer`; the existing renderer code becomes a separately built first-party PDF worker and remains GPUI-free.
- Changes native CI and release publication to build, sign, inspect, size-test, and publish the core application and official plugin artifacts independently for Windows x86_64, macOS arm64, and Linux x86_64.
- Adds per-user plugin data beneath the platform application-data directory; plugin binaries never enter document workspaces, session snapshots, recovery storage, or the core installation tree.
- Preserves per-version Markdown caches, syntax-highlight memoization, cached text handles, bounded file-tree rendering, and the current PDF raster/cache limits.
