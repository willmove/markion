## Why

Markion users currently have to leave the editor, manually transfer Markdown into MarkNice, and depend on the hosted site to prepare WeChat-compatible rich text. A bundled local publishing workspace can preserve MarkNice's existing browser-based rendering and rich-clipboard behavior while keeping the active Markion document as the canonical source and avoiding a WebView dependency in the GPUI process.

## What Changes

- Add an Export-menu action that opens a dedicated MarkNice publishing workspace for a snapshot of the active Markion document in the user's default browser.
- Bundle a pinned, editor-only MarkNice web distribution and all of its runtime JavaScript/CSS/font dependencies with Markion so the workspace can load without CDN or hosted-site availability.
- Add a lazy Rust loopback session service that binds only to the local host on an ephemeral port, hands the document to the bundled page through an unguessable single-session URL, and exposes only validated resources belonging to that document.
- Keep synchronization one-way: launching copies the current document into the browser workspace, while subsequent workspace edits and presentation choices remain session-local and never mutate or save the Markion document.
- Preserve MarkNice's WeChat theme, typography, preview, and browser rich-copy workflow, while surfacing explicit warnings for local images that cannot be published safely without a configured remote image backend.
- Pin bundle provenance and third-party notices, and verify packaged Windows, macOS, and Linux artifacts contain the complete offline workspace.
- Add localized launch, status, error, privacy, session-local-edit, and unresolved-image messaging.
- Non-goals: embedding a WebView in the GPUI window, bidirectional live editing, importing DOCX/PDF, bundling the MarkNice Node proxy, shipping OCR or OSS credentials, or replacing Markion's canonical `pulldown-cmark` document model.

## Capabilities

### New Capabilities

- `wechat-publishing-workspace`: Covers launching, serving, securing, and using the bundled browser workspace, including one-way document handoff, offline assets, resource isolation, rich copy, lifecycle, and user-visible limitations.

### Modified Capabilities

- `chrome-platform`: Adds a localized Export-menu entry and status/error feedback for opening the publishing workspace without disturbing the active document or its versioned caches.
- `release-packaging`: Requires every supported native package to contain the pinned MarkNice workspace, local third-party runtime assets, provenance, and notices, with an offline bundle verification gate.

## Impact

- Affected Markion areas include application actions and menus, browser launching, localization, document/resource path handling, logging, packaging resources, release CI, and tests.
- A small GPUI-free loopback/session module or workspace member will own HTTP parsing, token validation, MIME responses, resource containment, and lifecycle; GPUI-specific action wiring remains in the root crate.
- The checked-in web bundle is derived from the sibling MarkNice repository but is self-contained in Markion builds, pinned to a source revision, and does not require that sibling checkout or Node.js at build or runtime.
- The service reads the active document snapshot and existing resource bytes without mutating the document, incrementing its version, or invalidating derived Markdown caches.
- Security and privacy boundaries expand to a loopback HTTP origin and browser history, requiring loopback-only binding, unguessable short-lived session capabilities, no-store document responses, restrictive asset routing, and no embedded service credentials.
