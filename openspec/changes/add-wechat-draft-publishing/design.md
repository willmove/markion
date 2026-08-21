## Context

See `proposal.md` for motivation and `specs/wechat-draft-publishing/spec.md` for the behavioral contract. This change extends the browser workspace introduced by `add-local-marknice-publishing-workspace`; that change must be completed and archived before implementation begins.

The current workspace is a static MarkNice-derived bundle served by the GPUI-free `crates/wechat-workspace` loopback service. A one-time URL-fragment claim becomes a tab-scoped session bearer used only on fixed same-origin routes. The browser owns session-local edits and the final themed DOM, while Markion owns an immutable snapshot and allowlisted local resources. The CSP intentionally has `connect-src 'self'`, and no browser-to-document mutation path exists.

WeChat draft creation has different trust and operational requirements. It needs a server-reachable Official Account AppID/AppSecret, token caching and IP allowlisting, media upload and URL rewriting, and a final draft-add side effect that cannot always be retried safely. Those responsibilities must not enter the desktop binary or browser page. The desktop remains native on Windows, macOS, and Linux; connector setup must fail closed when a platform credential service is unavailable.

## Goals / Non-Goals

**Goals:**

- Keep the connector protocol usable by non-Markion implementations and keep the reference connector independently buildable, deployable, upgradeable, and auditable.
- Add one deliberate draft-save workflow to the existing browser workspace while retaining the one-way document boundary.
- Keep all WeChat secrets, tokens, upstream behavior, remote-image fetching, and irreversible retry decisions inside the connector.
- Transfer local images without revealing filesystem paths and convert every retained image to a WeChat-hosted URL.
- Make duplicate prevention, ambiguous results, content retention, and operator diagnostics explicit.
- Preserve deterministic tests by placing GPUI-free package/client logic behind narrow traits and using fake connector and WeChat servers.

**Non-Goals:**

- A generic cloud account platform, multi-tenant connector, or multiple Official Accounts per connector deployment in v1.
- Browser possession of the connector credential or direct browser cross-origin access to a connector.
- Automatic publication, scheduling, mass send, draft editing/deletion, or draft-box browsing.
- Exactly-once guarantees when WeChat accepts `draft/add` but its response is lost; the design provides idempotency up to that boundary and an explicit unknown state after it.
- Persisting article packages in Markion, changing the source Markdown, or introducing work on the typing/render path.

## Decisions

### 1. Treat the connector as a versioned external boundary

The canonical interoperability artifacts will live under `contracts/wechat-draft-connector/v1/` as an OpenAPI document, JSON schemas, examples, and conformance fixtures. Markion supports an explicit set of major contract versions and refuses silent downgrade. The initial HTTP surface is:

```text
GET    /v1/capabilities
POST   /v1/draft-jobs
GET    /v1/draft-jobs/{job_id}
DELETE /v1/draft-jobs/{job_id}
```

All operations require `Authorization: Bearer <connector-token>`. Capability discovery returns the protocol version, configured account label, feature flags, default-cover availability, accepted image types, field/content limits, and retention policy. Job creation uses `multipart/form-data`: one JSON manifest plus zero or more binary parts named by opaque asset ID. The request also carries `Idempotency-Key` and the manifest contains a canonical package SHA-256. Creation returns `202 Accepted` and a job representation/location; an identical replay returns the same job. Status results contain structured stable error categories with optional safe detail, never raw upstream bodies.

The checked-in contract, not a shared Rust crate, is the connector's source of truth. A new GPUI-free Markion member such as `crates/wechat-publishing` owns its client-side DTOs, validation, package builder, client, and fake implementation. The reference connector defines its server-side types independently and must pass the same fixtures. This small duplication proves that compatibility does not depend on linking Markion code.

Alternatives considered:

- **Put WeChat calls in Markion:** simplest deployment, but it distributes AppSecret material, makes desktop IP allowlisting brittle, and couples releases to upstream API changes. Rejected.
- **Let the browser call the connector:** avoids a loopback forwarding route, but exposes the long-lived connector credential to JavaScript and adds CORS/origin complexity. Rejected.
- **Share one Rust protocol crate with the connector:** reduces duplicated DTOs, but makes the reference implementation appear compatible by construction and raises coupling. Rejected in favor of schema conformance.
- **Synchronous request until draft completion:** fewer endpoints, but browser/desktop timeouts create duplicate-prone retries and poor progress reporting. Rejected.

### 2. Keep the reference connector outside the Markion Cargo workspace and packages

The reference service will live under `connectors/wechat-draft/` with its own `Cargo.toml`, `[workspace]`, `Cargo.lock`, test command, container build, sample environment, and deployment documentation. It will be a small Rust/Axum service for operational consistency, but it will not be a root workspace member and will not depend on `src/` or `crates/*`. Root `cargo test --workspace` remains the Markion suite; connector CI runs its own locked build and tests. Desktop packager manifests and package inspections explicitly exclude `connectors/`.

One deployment represents one Official Account in v1. Operators provide the account label, WeChat AppID/AppSecret, connector bearer-token hash or secret file, state directory, public origin, and retention/size limits through environment variables, container secrets, or a supported secret manager. The AppSecret is never accepted through the connector HTTP API. The service can listen on a deployment network address, but Markion accepts its configured public origin only over HTTPS; the deployment guide supplies reverse-proxy examples and uses loopback HTTP only for local development.

Alternatives considered:

- **Ship a local companion daemon:** still places WeChat secrets on each desktop and complicates updates/lifecycle. Rejected.
- **Make the connector another root workspace member:** convenient for a single test command, but makes dependency drift and accidental packaging more likely. Rejected.
- **Require a separately maintained repository immediately:** maximizes organizational separation but makes the initial contract and reference deployment harder to review atomically. The standalone subtree can be split later without code dependency changes.

### 3. Markion owns connection configuration but never WeChat credentials

`AppPreferences` gains an optional normalized connector origin and no secret field. A narrow `ConnectorCredentialStore` abstraction stores one scoped bearer token per normalized origin using the operating-system credential facility; the production adapter uses the maintained cross-platform keyring implementation and tests use an in-memory fake. Changing or clearing the origin removes the old credential after the new choice is confirmed. If keyring access fails, setup fails rather than writing a fallback file.

The Preferences panel gains a Publishing section with connector URL, token entry, Test connection, account/capability summary, and Clear connection. Token input is write-only: reopening the panel shows only whether a token exists. Test connection builds a client with a redirect policy of `none`, HTTPS-only remote origins, bounded connect/request timeouts, response-size limits, and sanitized errors. Authorization values are represented by redacted secret wrappers and are excluded from `Debug`/`Display`.

The root GPUI application owns preference UI and status messages. `crates/wechat-publishing` receives the origin and a secret value only when constructing a client; `crates/wechat-workspace` receives an injected draft-publishing facade. The browser sees only sanitized capabilities and per-job public status returned through authenticated same-origin routes.

Alternatives considered:

- **Store the connector token in `config.toml`:** portable but plaintext and easily copied into diagnostics/backups. Rejected.
- **Ask for the token on every submission:** avoids persistence but undermines the requested one-action workflow and encourages unsafe manual storage. Rejected.
- **Store AppID/AppSecret in the same keyring:** safer than plaintext but still couples Markion to WeChat authentication and distributes a high-value credential. Rejected.

### 4. Extend the authenticated loopback bridge, not the document model

The existing workspace service gains fixed protected routes conceptually equivalent to:

```text
GET    /api/draft/capabilities
POST   /api/draft/preflight
POST   /api/draft/jobs
GET    /api/draft/jobs/{local_job_id}
DELETE /api/draft/jobs/{local_job_id}
```

They reuse the tab's session bearer, no-store/security headers, expiry, and uniform denial. `connect-src 'self'` remains unchanged. Browser code clones the current sanitized MarkNice publishing subtree, reverses protected blob URLs to the opaque resource IDs already associated with them, and posts the clone plus metadata/cover choice to preflight. The server reparses and independently validates the HTML; it never trusts browser assertions about sanitization or resource ownership.

Preflight returns counts, limits, cover eligibility, and a short-lived package handle bound to the publishing session and a content digest. Confirmation submits that handle; server-side code rereads only allowlisted resource descriptors, constructs the multipart package, and calls the connector. A handle cannot name a filesystem path or be replayed in another session. Package handles and connector-job mappings are bounded by the existing session lifetime and disappear at process exit. The connector job remains queryable by its opaque remote ID if the browser tab reloads while the Markion session is still live.

The data flow and cache boundary are:

```text
MarkdownDocument + per-version caches
        │ explicit workspace launch (existing one-time snapshot)
        ▼
immutable PublishingSnapshot + immutable resource allowlist
        │ authenticated loopback bootstrap
        ▼
MarkNice browser session ── edits/theme ──> cloned sanitized HTML
        │ same-origin preflight; blob URLs become opaque asset IDs
        ▼
bounded session package ── Markion connector client ── HTTPS ──> connector job
                                                               │
                               media upload/rewrite + draft/add ▼
                                                              WeChat

No arrow returns to MarkdownDocument. Submission does not change document
version, selection, undo state, derived-cache identity, or cached text handle.
```

Alternatives considered:

- **Render the launch Markdown again in Rust:** loses browser edits and may diverge from the pinned MarkNice DOM. Rejected.
- **Post connector credentials into browser memory:** violates the trust boundary. Rejected.
- **Persist packages to the Markion config directory:** would improve crash recovery but creates a durable copy of sensitive article content. Rejected for v1.

### 5. Use a canonical article manifest with opaque asset placeholders

The v1 manifest contains protocol version, package digest, article metadata, sanitized HTML, a list of remote authored image URLs, binary asset descriptors, and a cover selector. An attached local image is represented inside HTML as `asset:<opaque-id>` and has exactly one matching multipart part. The cover selector references one attached asset, one declared public remote image, or the connector's configured default cover. Unknown/duplicate asset IDs, unreferenced parts, digest mismatch, missing cover, unsupported MIME, unsafe metadata, or declared-limit violations fail before job acceptance.

The browser bridge is the only layer that knows which generated blob URL corresponds to which session resource. The loopback service is the authority that maps the opaque ID to a canonical allowlisted file. It reads with the existing containment and symlink recheck immediately before upload, enforces byte/MIME limits, and sends neither authored local paths nor absolute paths. HTML validation rejects scripts, event attributes, forms, frames, embedded active objects, `file:`, loopback, `blob:`, `data:` image payloads, and any unresolved scheme. The connector repeats safety validation after rewriting because it is the final upstream trust boundary.

Remote HTTP(S) sources remain URL declarations rather than being downloaded by Markion. This avoids giving desktop local-network context to the connector package and centralizes media policy and retry behavior in the self-hosted service.

Alternatives considered:

- **Inline every image as base64 JSON:** simple schema but high memory overhead and poor streaming behavior. Rejected.
- **Send loopback/blob URLs:** they are unreachable by the connector and can leak capabilities. Rejected.
- **Have Markion download remote images:** exposes the user's network, duplicates SSRF/media policy, and couples the desktop to publication rules. Rejected.

### 6. Make the connector a bounded, persistent job processor

The reference connector stores job identity, idempotency key, content digest, state, safe error category, timestamps, and result identifiers in SQLite. Accepted package files live in a permission-restricted per-job directory, are bounded on ingress, and are deleted shortly after a terminal result or configured expiry. The database does not store article HTML, metadata values, or image bytes. A bounded worker queue prevents request handlers from holding upstream operations open.

The state machine is:

```text
queued ──> preparing ──> committing ──> succeeded
  │            │              ├──────> failed (definitive upstream rejection)
  │            │              └──────> outcome_unknown (response may be lost)
  │            └──────> failed
  └───────────────────> canceled
```

Only `queued` and the safe portion of `preparing` are cancellable. `committing` begins immediately before the draft-add request is sent. Media upload failures before that boundary may use bounded, classified retries because they cannot create a draft. The draft-add call is made once per job attempt. On restart, queued/preparing jobs with complete packages can resume; a job found in committing becomes `outcome_unknown`; terminal jobs remain queryable until status retention expires. The connector never automatically creates a replacement job for unknown outcome.

Idempotency is scoped to the authenticated connector deployment. Reusing a key and digest returns the stored job; changing the digest returns conflict. SQLite constraints enforce this before enqueueing. This protects browser retries and lost connector responses, but cannot convert an ambiguous WeChat result into an exactly-once guarantee.

Alternatives considered:

- **No persistent state:** easy deployment, but process restart could duplicate an accepted request. Rejected.
- **Automatically retry `draft/add`:** improves apparent availability at the cost of duplicate drafts. Rejected.
- **Store full content in SQLite indefinitely:** simplifies recovery but conflicts with content-minimization goals. Rejected.

### 7. Isolate WeChat behavior behind a connector-side adapter

The connector has separate clients for WeChat APIs and for authored remote images. The WeChat adapter obtains and caches access tokens in memory, refreshes them before expiry, and performs a single classified refresh/retry only when an operation is known not to have created a draft. It uploads inline images through WeChat's article-image upload interface, resolves the cover to permanent material as required, rewrites HTML, validates the result, and finally invokes draft-add. Current WeChat limits are returned through capabilities so Markion can preflight them; upstream errors map to stable safe categories while exact codes remain in redacted operator diagnostics when safe.

The authored-image client disables ambient proxies and automatic redirects. For every initial URL and redirect it parses the host, rejects embedded credentials and non-HTTP(S) schemes, resolves DNS, rejects all non-public addresses, pins an accepted destination for the connection while preserving TLS hostname verification, and revalidates the next location. It enforces redirect count, total deadline, response bytes, decoded pixel dimensions, content type, and supported format. Image data is decoded and re-encoded where supported before upload to strip metadata and reject malformed/polyglot content. Operators are advised to enforce an egress firewall as defense in depth.

After every upload, the connector parses the HTML again and requires all retained image sources to be WeChat-hosted results. It also reapplies an HTML/attribute/style allowlist compatible with the pinned MarkNice corpus. Any unresolved placeholder or external/local/active source fails before `draft/add`.

Alternatives considered:

- **Leave public remote images unchanged:** WeChat may filter them and the resulting draft is not self-contained. Rejected.
- **Allow arbitrary server-side URL fetches:** required for convenience but unsafe without SSRF controls. Rejected; the fetcher is image-only and policy-bound.
- **Trust MarkNice sanitization alone:** browser content and bridges can be modified, and connector callers need not be Markion. Rejected.

### 8. Use precise user states and never conflate draft save with publication

The browser workspace adds the draft action, metadata/cover review, final account confirmation, progress, cancellability indicator, and terminal result. Defaults may derive title from the first heading or document display name, but required values remain editable and confirmed. Capability discovery provides the connector account label and default-cover availability; it does not promise that WeChat permissions will remain valid.

The UI maps structured categories rather than displaying raw connector/WeChat messages. Success says “Saved to the draft box” and shows the account label and opaque result reference; it never says published. `outcome_unknown` has a distinct persistent warning telling the user to inspect the draft box before manually trying again. A connector setup link is shown when publishing is unavailable, while the existing rich-copy path remains usable.

All strings are added exhaustively to `src/i18n.rs` for English, Japanese, French, German, Spanish, Simplified Chinese, and Traditional Chinese. Security distinctions—especially failed versus unknown and saved versus published—must remain semantically equivalent in every locale.

## Risks / Trade-offs

- **[The self-hosted connector makes setup more involved]** → Ship a container image definition, reverse-proxy examples, secret-generation command, health check, connection-test guide, and an operator checklist for WeChat credentials/IP allowlisting.
- **[OS keyring behavior varies across desktop environments]** → Hide it behind a testable adapter, fail closed with actionable setup text, and verify Windows Credential Manager, macOS Keychain, and Linux Secret Service in release testing.
- **[Contract and two independent Rust models can drift]** → Make OpenAPI/schema fixtures canonical and run both Markion-client and connector-server conformance suites against the same committed examples.
- **[Remote-image fetching creates SSRF and decompression risk]** → Use a dedicated no-proxy fetcher, public-address pinning, redirect revalidation, strict byte/pixel/type/time limits, decode/re-encode, and recommended egress filtering.
- **[WeChat API limits or error semantics change]** → Keep upstream behavior connector-side, advertise limits through capabilities, classify unknown responses conservatively, and allow connector upgrades independent of Markion.
- **[A lost `draft/add` response can still yield a duplicate after a manual retry]** → Never retry automatically, retain the unknown job status, identify the target account, and require draft-box inspection.
- **[Browser HTML and server sanitization can differ and alter formatting]** → Maintain a MarkNice article/HTML corpus through browser extraction, package validation, connector rewriting, and final normalized golden output.
- **[Temporary connector packages contain sensitive drafts]** → Use bounded permission-restricted storage, short deletion windows, content-free logs/database rows, and documented state-directory security.
- **[Reference connector in the same repository may be mistaken as part of Markion]** → Give it a separate workspace/lockfile/CI/container/release path and fail desktop package inspection if any connector artifact is staged.

## Migration Plan

1. Complete, validate, and archive `add-local-marknice-publishing-workspace`; rebase this change's implementation on its final spec and routes.
2. Land the v1 contract, conformance fixtures, and GPUI-free Markion package/client with fake-connector tests while leaving the browser action unavailable.
3. Add optional connector-origin preference, OS credential adapter, test/clear UI, and localized connection states. Existing configurations deserialize with no connector and no migration prompt.
4. Add loopback preflight/job routes, package handles, browser metadata/cover UI, DOM extraction, progress, cancellation, and document/cache invariant tests.
5. Build the standalone reference connector, persistent state machine, WeChat adapter, safe image fetcher, sanitizer, container assets, and fake-WeChat integration suite.
6. Add operator/user documentation, contract validation, connector CI, package-exclusion checks, and cross-platform keyring/browser tests.
7. Enable release eligibility only after an authorized test Official Account receives a verified draft and all installer/bundle inspections pass.

Rollback is additive. A Markion release can hide/remove the draft action and ignore the optional connector origin while retaining rich copy and existing documents. Operators can stop or roll back the connector independently; queued work is canceled where safe and committing jobs retain their known/unknown status. Users should clear the connection before downgrade to remove the OS credential entry; no Markdown or workspace-session migration is required.
