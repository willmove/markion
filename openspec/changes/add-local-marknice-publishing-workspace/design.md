## Context

See `proposal.md` for motivation. Markion is a Rust/GPUI application whose canonical document semantics and per-version derived caches are owned by `MarkdownDocument` and `pulldown-cmark`. MarkNice is a build-step-free browser application whose rendering, sanitization, themes, preview, and rich clipboard path are coupled to browser DOM APIs. Its current application page also loads renderer libraries from a CDN and conditionally calls a Node proxy for PDF OCR and temporary OSS image publication.

GPUI exposes platform URL opening but does not provide an HTML/CSS surface, and its public clipboard item represents strings and images rather than a portable `text/html` plus `text/plain` pair. A browser workspace therefore preserves MarkNice behavior and delegates rich clipboard negotiation to the browser without adding a native WebView to Markion. Markion releases must remain native on Windows, macOS, and Linux, and a release checkout cannot assume that `C:\Coding\EditorProjects\marknice`, Node.js, or the public internet exists.

The publishing path is explicit and off the typing path. It may snapshot the active document once, but it must not change document version, force repeated derivation, or disturb shared derived-cache identity.

## Goals / Non-Goals

**Goals:**

- Ship a self-contained, editor-only MarkNice publishing surface that opens in the default browser and works with external networking disabled.
- Make the active Markion in-memory Markdown the source of each immutable handoff while allowing session-local publishing edits.
- Protect snapshot text and managed local images with a narrow, capability-based loopback service.
- Preserve the pinned MarkNice themes, typography controls, desktop/phone preview, math rendering, WeChat sanitization, and rich-copy behavior.
- Keep bundle origin, third-party versions, digests, and licenses auditable and release-verifiable.
- Make the session/server implementation independent of GPUI and deterministic under tests.

**Non-Goals:**

- A WebView embedded in the Markion window or a second GPUI renderer for arbitrary MarkNice HTML/CSS.
- Live or bidirectional synchronization, saving browser edits into the document, or sharing undo history.
- Replacing the root parser, preview blocks, existing Markion themes, or existing HTML/PDF/DOCX export implementations.
- MarkNice's landing page, guide, DOCX/PDF import, browser-print export, Word export, Node proxy, OCR, OSS upload, analytics, or hosted update behavior.
- Making local images directly publishable to WeChat without a separately designed remote image backend.

## Decisions

### 1. Use a loopback browser workspace, not `file:` URLs or an embedded WebView

The action will lazily start a loopback HTTP service on `127.0.0.1:0`, create a publishing session, and dispatch a URL to the platform default browser. A local origin gives the page predictable relative asset loading, fetch, blob URL, CSP, and browser clipboard behavior. It also permits a dynamic document handoff without putting document text in the command line, URL, or generated temporary HTML file.

Alternatives considered:

- **Embedded Wry/WebView:** highest visual integration, but it adds WebView2/WebKitGTK lifecycle and packaging concerns, Linux X11/Wayland divergence, and native-child layering issues inside GPUI. Rejected for this change.
- **`file:` workspace plus generated JSON:** simpler server lifecycle, but exposes a durable plaintext file and creates inconsistent browser security/CORS/clipboard behavior. Rejected.
- **Hosted MarkNice URL:** smallest implementation but loses offline behavior and makes availability/privacy depend on a remote service. Rejected as the primary workflow.

### 2. Separate GPUI orchestration from a GPUI-free workspace service

A new workspace member under `crates/` will own session state, capability generation, HTTP routing, response headers, static bundle lookup, resource containment, expiry, and shutdown. It will take plain input types such as:

```text
PublishingSnapshot
├── markdown: Arc<str>
├── display_name: String
├── resource_map: authored URL -> opaque resource descriptor
└── created_at
```

The root application will own the Export action, localized status, browser dispatch, logging, and conversion from the active `MarkdownDocument` into `PublishingSnapshot`. The member crate will not depend on GPUI or access app globals.

Use Tokio, which is already a workspace dependency, plus a maintained HTTP implementation rather than a handwritten HTTP parser. The chosen HTTP dependency will be configured only for the small server surface needed here. The server exposes fixed routes and does not provide directory listing, proxying, arbitrary URL fetching, file upload, or document mutation endpoints.

The browser-dispatch call will be wrapped by a small adapter with a test double and an immediate `Result`. A dispatch failure revokes the newly created unused session. This is preferable to directly scattering platform commands or relying on a void-returning call where tests cannot distinguish accepted from rejected dispatch.

### 3. Vendor a purpose-built, pinned MarkNice subset

The checked-in runtime will live under a stable Markion resource directory such as `assets/marknice-workspace/`. It will contain:

- a dedicated `index.html` with only the publishing editor, disclosure banner, preview, presentation controls, warning UI, and copy action;
- the MarkNice theme/sanitization/rendering code needed by that surface;
- a Markion bridge for bootstrap, session-local state, protected resource fetches, expiry, and local-image copy policy;
- pinned local copies of `marked`, KaTeX JavaScript/CSS, and all KaTeX fonts required by the workspace;
- a manifest recording the MarkNice repository URL and commit, transformation/import version, third-party component versions/licenses, and SHA-256 identity of every bundled runtime file.

JSZip, html-docx-js, `docx-parser.js`, `pro-extras.js`, the Node server, landing-page content, and remote service code are excluded because their features are non-goals. The workspace must contain no runtime `<script>`, `<link>`, font, or application fetch to a remote host.

A maintainer-only sync/check command may use the sibling MarkNice checkout and development tooling to refresh the vendored subset, but refreshed files and the manifest are committed. Normal Cargo build, test, packaging, and application runtime consume only the committed bundle and do not execute the sync command or require Node.js.

Alternatives considered:

- **Git submodule:** preserves upstream history but makes clean checkouts, source archives, CI credentials, and package reproducibility more fragile. Rejected.
- **Copy the entire MarkNice repository:** easiest initial import but bundles unrelated pages, APIs, credentials documentation, and CDN references. Rejected.
- **Download a MarkNice release during packaging:** prevents reproducible offline packaging and weakens provenance. Rejected.

### 4. Exchange a URL-fragment claim for a protected session token

The launch URL will place a cryptographically random one-time claim capability in the URL fragment. Fragments are not included in the HTTP request or referrer. The static shell reads it, immediately removes it with `history.replaceState`, and sends it in an authorization header to a fixed claim endpoint. A successful claim invalidates that capability and returns a second cryptographically random session token in a no-store response. The page retains the session token in tab-scoped `sessionStorage` and supplies it in authorization headers for document, heartbeat, and resource requests.

Protected image bytes are fetched by JavaScript with the authorization header and converted to browser blob URLs; raw protected resource URLs are never assigned directly to `<img>`. This avoids putting a reusable token in image URLs, cookies, browser history, or remote referrers and allows independent sessions in concurrent tabs on the same loopback origin.

The static shell is not sensitive and may be served before authentication. Dynamic responses use `Cache-Control: no-store`, `Referrer-Policy: no-referrer`, `X-Content-Type-Options: nosniff`, and a restrictive CSP. The CSP permits packaged scripts/styles, inline style attributes required by MarkNice's output, blob/data/local images, and document-authored HTTP(S) images, while rejecting frames, plugins, remote scripts, remote fonts, form submission, and cross-origin connections.

Production tokens use operating-system cryptographic randomness and at least 256 bits of entropy. Authentication failures use uniform responses that do not reveal session or filesystem existence.

### 5. Resolve local resources in Markion's canonical document boundary

The service crate will not parse Markdown or infer document ownership. At launch, the root/domain layer will obtain image references through Markion's canonical `MarkdownDocument`/`pulldown-cmark` semantics and construct an immutable allowlist. Only supported regular images whose canonical targets remain inside the named document's associated asset directory become opaque resource descriptors. Untitled documents have no associated asset directory and therefore no local resource descriptors.

The browser bridge keeps the authored Markdown text unchanged. After each MarkNice render, it resolves recognized authored local image URLs against the immutable descriptor map, fetches allowed bytes with authorization, and supplies blob URLs to the preview. A browser-session edit that introduces another local path cannot widen the allowlist; it remains unresolved until the user returns to Markion and launches a new snapshot.

Canonical containment is checked both while creating the descriptor and immediately before serving bytes, so replacement or symlink changes after launch fail closed. Absolute filesystem paths are never returned to JavaScript or user-facing status.

### 6. Keep the handoff immutable and one-way

The service offers no endpoint that changes the Markion document or session snapshot. `setMarkdown` initializes the browser editor once; subsequent browser edits remain in that tab. A persistent banner explains the boundary, and unload/reload behavior does not imply that Markion saved browser edits.

The data and cache flow is:

```text
explicit Export action
        │
        ├─ read active in-memory Markdown once
        ├─ reuse canonical image-reference/resource resolution
        │
        ▼
immutable PublishingSnapshot (Arc-backed, bounded session store)
        │
        ▼
authenticated loopback JSON/resource fetch
        │
        ▼
MarkNice browser editor → themed preview → rich clipboard

No arrow returns to MarkdownDocument.
No document version or derived-cache invalidation occurs.
```

Snapshot creation is user-triggered and outside the synchronous typing/render path. Static bundle bytes can be process-shared; document text and resource maps are shared by `Arc` within their session rather than copied per request.

### 7. Treat local-image copy as an explicit partial operation

Protected local preview images use blob URLs and cannot be assumed to survive a paste into WeChat. Before rich copy, the bridge clones the sanitized publishing DOM and detects every image backed by a protected local descriptor. If any exist, it offers only cancel or “copy without local images.” On confirmation, those image elements are removed from the clone, the remaining HTML/plain text are copied, and the status reports the omitted count. The copy payload is checked to contain no loopback, blob, or filesystem URL.

Remote document-authored HTTP(S) image elements keep the pinned MarkNice behavior. Missing and out-of-scope local images remain visibly unresolved and are never presented as successfully published. A future image-publication capability can replace this policy without changing the session transport.

### 8. Bound sessions and fail closed

The service starts on first use and remains owned by the Markion process. Production defaults retain at most eight sessions and expire a session after two hours without an authenticated request; both limits are injectable for deterministic tests. Creating a ninth session evicts the least-recently-used inactive snapshot. Process shutdown cancels the listener and drops every snapshot/token. There is no daemon, child Node process, persistent database, or recovery file.

Missing/corrupt bundle assets, bind failure, snapshot setup failure, claim failure, and browser-dispatch failure all produce explicit errors. A feature failure never prevents Markion from starting or continuing normal editing. The browser page converts an expired/denied protected request into a localized relaunch instruction rather than continuing with a misleading stale status.

### 9. Verify the bundle at source, test, and package boundaries

Verification is layered:

- Unit tests cover token issue/claim/replay, token isolation, expiry/LRU behavior, fixed routing, MIME types, headers, path decoding, traversal, absolute paths, symlink escape, unsupported files, and shutdown.
- Loopback integration tests start the service on an ephemeral port and exercise shell load, claim, document fetch, protected image fetch, denial, no-store behavior, and concurrent isolation.
- Root/GPUI tests use a fake browser launcher to prove the action, localized status/error paths, untitled behavior, launch rollback, and unchanged document/version/selection/cache identities.
- A bundle verifier checks manifest digests, required files, provenance/licenses, local dependency closure, and absence of remote runtime script/style/font/application URLs.
- A browser-run compatibility page exercises the pinned theme/construct corpus and records normalized golden output. It is maintainer/release evidence; the normal application does not expose it.
- Release jobs inspect the staged resources for every native package before publication. Manual release verification covers default browsers on Windows, macOS, Linux X11/Wayland, rich paste into the WeChat editor, offline shell loading, math fonts, remote images, local-image omission, and session expiry.

## Risks / Trade-offs

- **[Browser behavior differs by vendor or clipboard permission]** → Keep the MarkNice preferred and fallback copy paths, report actual failure, and maintain a cross-browser/manual release matrix.
- **[Loopback HTTP expands the attack surface]** → Bind only to loopback, use high-entropy one-time claims and per-tab session tokens, fixed routes, no-store responses, CSP, uniform denial, bounded lifetime, and containment tests.
- **[Remote image previews disclose the user's IP to document-authored hosts]** → Make the distinction visible, send no referrer, and keep all application/runtime dependencies local; a future change may add a remote-image loading preference.
- **[Pinned MarkNice behavior drifts from the standalone repository]** → Record provenance and digests, provide an explicit sync/check process, and gate updates with the compatibility corpus rather than silently tracking upstream.
- **[MarkNice and Markion parse soft breaks or extensions differently]** → The browser workspace intentionally owns the pinned MarkNice publishing output while Markion retains canonical editing semantics; golden fixtures make the boundary explicit.
- **[Local images cannot be published in a purely local workflow]** → Preview only validated managed images and require an explicit copy-without-images choice; do not bundle credentials or imply upload success.
- **[Bundled renderer/font assets increase installer size]** → Import only the publishing subset and measure staged/package deltas in release verification.
- **[A browser tab can retain already-fetched content after Markion exits]** → Document that process shutdown revokes future requests but cannot erase browser memory; avoid persistent HTTP/browser caches and durable temporary files.

## Migration Plan

1. Add the GPUI-free session service and security tests behind an unexposed root integration point.
2. Import and verify the pinned MarkNice publishing subset, manifest, and license updates.
3. Add the browser bridge, compatibility corpus, protected resource preview, partial local-image copy, and expiry UI.
4. Wire the localized Export action and browser-launch adapter, then add root/GPUI state-invariant tests.
5. Add bundle verification to local/CI quality paths and stage the resources in every package format.
6. Perform offline, browser, WeChat paste, local-image, and packaged-artifact verification on every release platform before enabling the action for release.

Rollback is additive: remove or hide the Export action and stop packaging the workspace resources. Existing documents, preferences, sessions, and file formats require no migration; active loopback sessions disappear when the old process exits.
