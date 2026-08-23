## 1. Workspace Service Foundation

- [x] 1.1 Create a GPUI-free `wechat-workspace` Cargo member with snapshot, resource-descriptor, session-limit, clock, error, and service-handle types, and wire only the minimal Tokio/HTTP/cryptographic-random dependencies through the root workspace.
- [x] 1.2 Implement runtime workspace-asset discovery for development and packaged resource layouts, returning a typed non-fatal error when the bundle or manifest is unavailable.
- [x] 1.3 Add deterministic test clocks, token sources, and service configuration so expiry, eviction, and authentication behavior can be tested without elapsed-time sleeps or production randomness.

## 2. Capability Sessions and Loopback HTTP

- [x] 2.1 Implement 256-bit one-time claim capabilities, claim replay rejection, independent session bearer tokens, authenticated touch timestamps, two-hour idle expiry, and an eight-session least-recently-used bound.
- [x] 2.2 Implement lazy binding to `127.0.0.1:0`, fixed static/claim/document/heartbeat/resource routes, graceful cancellation, and listener shutdown when the owning Markion process/service handle is dropped.
- [x] 2.3 Add the claim exchange contract used by the URL fragment, including one-time invalidation, uniform authentication denial, tab-scoped session-token response data, and revocation of an unused launch session.
- [x] 2.4 Add MIME-safe static responses and protected-response headers, including no-store dynamic caching, no-referrer policy, `nosniff`, and the restrictive workspace CSP defined by the design.
- [x] 2.5 Add unit and loopback integration tests for lazy startup, loopback/ephemeral binding, claim success/replay/invalid tokens, concurrent-session isolation, expiry, LRU eviction, fixed-route denial, response headers, and shutdown.

## 3. Canonical Document Resource Boundary

- [x] 3.1 Add a root/domain helper that obtains authored image references through Markion's canonical `MarkdownDocument`/`pulldown-cmark` ownership and builds a publishing snapshot without mutating text, version, selection, dirty state, or derived-cache identity.
- [x] 3.2 Resolve only supported regular images inside the named document's associated asset directory into opaque publishing descriptors, with untitled documents and out-of-scope references producing no widened filesystem authority.
- [x] 3.3 Recheck canonical containment immediately before serving each resource and reject missing files, unsupported types, absolute-path injection, encoded traversal, and symlink escape without revealing filesystem paths or path existence.
- [x] 3.4 Add cross-platform resource tests for valid managed images, untitled documents, missing files, mixed separators, percent encoding, `..`, absolute paths, symlink/junction escape where supported, MIME selection, and no-store responses.

## 4. Pinned MarkNice Publishing Bundle

- [x] 4.1 Add a maintainer-only sync/import command and check in an editor-only MarkNice workspace under `assets/marknice-workspace`, pinned to the selected MarkNice commit and excluding landing/guide pages, DOCX/PDF import, duplicate file export, Node proxy, OCR, OSS, analytics, and hosted-update code.
- [x] 4.2 Vendor pinned local `marked`, KaTeX JavaScript/CSS, and required KaTeX fonts; rewrite the workspace shell so every runtime script, style, font, and application fetch resolves locally with external networking disabled.
- [x] 4.3 Create the bundle provenance manifest with source repository/commit, import format version, third-party versions/licenses, required-file list, and SHA-256 digests, and update `THIRD_PARTY_NOTICES.md` plus applicable license files.
- [x] 4.4 Implement the browser bridge that reads and immediately removes the fragment claim, exchanges it once, stores the session token in `sessionStorage`, fetches the immutable snapshot, initializes MarkNice, sends heartbeats, and shows localized relaunch guidance after expiry.
- [x] 4.5 Add the persistent session-local-edit/privacy disclosure and ensure browser edits rerender and copy within the tab without any endpoint or callback that writes to Markion.
- [x] 4.6 Implement protected local-image resolution by mapping authored URLs to opaque descriptors, fetching bytes with authorization, using/revoking blob URLs for preview, and showing unresolved warnings without absolute paths.
- [x] 4.7 Implement the local-image copy gate with cancel and explicit copy-without-images choices; remove protected local image elements from the cloned payload and assert that successful partial output contains no loopback, blob, or filesystem URL while reporting the omitted count.
- [x] 4.8 Preserve the pinned MarkNice theme catalog, font-size/spacing controls, desktop/phone previews, math rendering, sanitizer, dual HTML/plain rich-copy paths, and localized success/clipboard-denied statuses in the editor-only shell.
- [x] 4.9 Add the construct/theme compatibility corpus, normalized golden outputs, and a browser-run self-test page covering headings, soft/hard breaks, lists, tables, code, math, links, remote images, managed images, and all bundled themes.
- [x] 4.10 Add a bundle verifier and automated tests for required files, manifest digests, provenance/licenses, local dependency closure, CSP-compatible references, and absence of remote runtime script/style/font/application URLs.

## 5. Markion Application Integration

- [x] 5.1 Add a process-owned, lazily initialized publishing-service handle to application state and ensure startup, ordinary editing, and shutdown remain correct when the feature is never used or service startup fails.
- [x] 5.2 Add an injectable cross-platform default-browser launcher that returns immediate dispatch success/failure and can be replaced by a deterministic fake in root/GPUI tests.
- [x] 5.3 Register the WeChat publishing action in the in-window and native Export menus, create a snapshot for the active tab, launch the fragment-capability URL, revoke on dispatch failure, and report success/setup/launch errors through status feedback.
- [x] 5.4 Add every new Markion and browser-workspace string to the localization catalogs, covering the menu item, opened/setup/launch errors, session-local disclosure, privacy note, expiry, unresolved/local image counts, partial copy, clipboard success, and clipboard denial.
- [x] 5.5 Add root/GPUI tests for untitled, empty, saved, and dirty documents; repeated independent launches; localized action/status paths; launch rollback; and unchanged active tab, view mode, selection, text, dirty state, version, and pre-existing derived-cache identities.
- [x] 5.6 Add diagnostic logging for service startup/shutdown, session creation/revocation/expiry counts, bundle verification/setup failures, and browser dispatch failures without logging Markdown, tokens, absolute resource paths, or copied HTML.

## 6. Packaging and Release Verification

- [x] 6.1 Add the complete `assets/marknice-workspace` tree and provenance/license material to the cargo-packager resources while preserving the current Windows NSIS, macOS app/DMG, Linux DEB, and Linux AppImage layouts.
- [x] 6.2 Add pre-publication release checks that inspect each staged/package resource tree, run the bundle verifier, and fail for missing, unlisted, remotely referenced, or digest-mismatched workspace files.
- [x] 6.3 Verify a clean Markion checkout can build, test, and stage the workspace without the sibling MarkNice repository, Node.js, or network downloads, and document the separate maintainer refresh procedure.
- [x] 6.4 Update user documentation with how to launch the local publishing workspace, the one-way/session-local editing model, offline scope, remote-resource privacy note, local-image omission behavior, clipboard permissions, and relaunch guidance.

## 7. Quality and Platform Evidence

- [x] 7.1 Run Rust formatting and lint checks applicable to the repository, `cargo test -p wechat-workspace`, root-package tests, `cargo test --workspace`, and the bundle verifier; fix every deterministic failure.
- [x] 7.2 Run the browser compatibility self-test and manually verify offline workspace loading, theme/typography controls, math fonts, rich copy, clipboard-denied feedback, remote images, managed-image preview, copy-without-images, expiry, and repeated sessions in the Windows default browser; record the maintainer-authorized v0.2.1 deferral for macOS and Linux manual browser coverage.
- [ ] 7.3 Verify default-browser launch and packaged resource discovery on Windows, macOS Apple Silicon, Linux X11, and Linux Wayland, and record installer/package size deltas plus any platform-specific limitation in release evidence.
- [x] 7.4 Paste representative rich output into the WeChat editor and verify headings, paragraphs, lists, tables, code, math, links, remote images, plain-text fallback, and explicit local-image omission match the pinned compatibility expectations.
- [ ] 7.5 Run `openspec validate add-local-marknice-publishing-workspace` and the repository quality gate, then update every completed checkbox and any required manual-verification record before requesting archive.
- [x] 7.6 Fix the checked-in workspace digest mismatch: regenerate every manifest digest from the canonical LF bytes, normalize text-file line endings in both digest generation and verification, pin `assets/marknice-workspace/**` to `eol=lf` in `.gitattributes`, and add a CRLF-checkout regression test so Windows `autocrlf` checkouts and packaged copies verify on every platform.
