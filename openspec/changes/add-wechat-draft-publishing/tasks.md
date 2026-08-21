## 1. Prerequisite and connector contract

- [ ] 1.1 Confirm `add-local-marknice-publishing-workspace` is completed, strictly validated, and archived; reconcile this change with the archived workspace spec before touching implementation.
- [ ] 1.2 Add the v1 OpenAPI document and JSON schemas under `contracts/wechat-draft-connector/v1/` for capabilities, multipart draft-job creation, status/result, cancellation, structured errors, idempotency, and package digests.
- [ ] 1.3 Add valid and invalid conformance fixtures for every v1 operation, job state, metadata/cover variant, and asset-placeholder rule, plus a contract-validation command suitable for local and CI use.
- [ ] 1.4 Add a GPUI-free `crates/wechat-publishing` workspace member for client-side protocol models, package construction, validation, and connector access; assert that it has no `gpui` dependency and add an explicit dev-profile override if its test/validation path is compute-heavy.

## 2. Markion connector client and article package

- [ ] 2.1 Implement strict v1 capability and job DTO parsing in `crates/wechat-publishing`, including supported-version negotiation, bounded fields, stable error categories, and rejection of unknown security-sensitive values.
- [ ] 2.2 Implement redacted secret/header types and an HTTPS-only connector client with no cross-origin redirects, bounded timeouts/body sizes, exact-origin credential forwarding, and sanitized diagnostics.
- [ ] 2.3 Implement canonical manifest serialization, package SHA-256 calculation, unique idempotency keys, opaque `asset:` references, streamed multipart construction, and digest/idempotency replay tests.
- [ ] 2.4 Implement independent article/package validation for required metadata, advertised limits, cover choice, asset/part consistency, active HTML, and forbidden local, loopback, blob, data, or unresolved URLs.
- [ ] 2.5 Implement capability discovery, create/status/cancel operations and the complete queued-to-unknown client state model behind a mockable `DraftConnector` boundary.
- [ ] 2.6 Add a fake conforming connector and integration tests covering success, definitive failure, cancellation, unsupported version, redirect refusal, timeout, malformed/oversized response, idempotent replay, digest conflict, and outcome-unknown behavior.

## 3. Connector preferences and secure credential storage

- [ ] 3.1 Extend `AppPreferences` and its backward-compatible TOML conversion with an optional normalized connector origin only; verify old configs load unchanged and rendered configs never contain connector or WeChat secrets.
- [ ] 3.2 Add a `ConnectorCredentialStore` abstraction with an in-memory test double and production OS-keyring adapter for Windows Credential Manager, macOS Keychain, and Linux Secret Service, with no plaintext fallback and redacted errors.
- [ ] 3.3 Implement atomic configure/change/clear behavior so a tested origin and scoped token replace old state safely, clearing removes both capability state and the matching keyring entry, and failure cannot leave a partially active connection.
- [ ] 3.4 Add a Publishing section to Preferences for origin, write-only token entry, secure-storage availability, Test connection, sanitized account/capability summary, and Clear connection.
- [ ] 3.5 Add exhaustive English, Japanese, French, German, Spanish, Simplified Chinese, and Traditional Chinese strings for setup, connection testing, secure-storage failure, version mismatch, and clearing, with locale parity tests.
- [ ] 3.6 Add storage/UI tests proving tokens, AppID/AppSecret values, authorization headers, and account secrets never enter `config.toml`, snapshots, logs, `Debug` output, or browser-visible capability data.

## 4. Authenticated browser-to-Markion publishing bridge

- [ ] 4.1 Inject a mockable draft-publishing facade into `crates/wechat-workspace` and add fixed authenticated no-store capability, preflight, create, status, and cancel routes without weakening the existing claim/session model or `connect-src 'self'` CSP.
- [ ] 4.2 Add bounded short-lived preflight/package handles tied to one publishing session and content digest; enforce expiry, cross-session denial, replay rules, cleanup, and process-owned shutdown.
- [ ] 4.3 Extend the browser bridge to clone the current sanitized MarkNice output, reverse only known protected blob images to opaque `asset:` IDs, collect public remote image declarations, and reject newly introduced/unresolved local paths.
- [ ] 4.4 Resolve accepted opaque assets server-side through the immutable session allowlist, recheck containment/symlinks/MIME/size immediately before transfer, and prove that no filesystem or loopback path crosses the browser or connector boundary.
- [ ] 4.5 Add the localized **Save to WeChat draft box** action and metadata/cover review for title, author, digest, source URL, comments, article-image cover, and connector-default cover, including advertised-limit validation.
- [ ] 4.6 Add final target-account/content confirmation, progress, safe cancellation, success, structured failure, session expiry, and persistent outcome-unknown UI that says saved-to-draft rather than published and directs inspection before manual retry.
- [ ] 4.7 Add browser/loopback tests for route authentication, CSP/CORS behavior, active-content rejection, asset tampering, handle/session isolation, limit errors, connector unavailability, and preservation of the existing rich-copy workflow.
- [ ] 4.8 Add root/GPUI tests proving draft actions, polling, cancellation, failure, success, and unknown outcome never mutate/save the document or change version, selection, undo state, dirty state, derived-cache identity, syntax memoization, or cached text handle.

## 5. Standalone reference connector foundation

- [ ] 5.1 Scaffold `connectors/wechat-draft/` as an independent locked Rust workspace with no dependency on Markion packages, plus its own build/test commands, container definition, non-secret sample configuration, health/readiness endpoints, and desktop-package exclusion guard.
- [ ] 5.2 Implement startup configuration for one account label, WeChat AppID/AppSecret secret sources, hashed/scoped connector bearer token, public-origin and storage validation, safe limits, and fail-fast secret redaction.
- [ ] 5.3 Implement constant-time bearer authentication, uniform denial, bounded request middleware, correlation IDs, safe structured errors, and v1 capability discovery that passes the shared contract fixtures.
- [ ] 5.4 Implement streaming multipart ingestion into permission-restricted per-job storage with manifest/digest/part/limit validation, atomic acceptance, cleanup on rejection, and no article-content logging.
- [ ] 5.5 Implement SQLite schema/migrations and repository operations for job identity, idempotency constraints, content digest, lifecycle, safe error/result metadata, timestamps, and retention without storing article HTML, metadata values, or image bytes in the database.
- [ ] 5.6 Implement the bounded worker queue, safe cancellation boundary, restart recovery, committing-to-outcome-unknown recovery rule, terminal status retention, payload deletion, and scheduled cleanup with deterministic clock/crash tests.
- [ ] 5.7 Implement create/status/cancel handlers and idempotent replay/conflict behavior, then run the server side against every shared v1 conformance fixture independently of Markion client code.

## 6. Connector media and HTML safety

- [ ] 6.1 Implement an authored-image fetcher with no ambient proxy or automatic redirect, public-address resolution/pinning, TLS hostname verification, redirect revalidation, credential/scheme denial, and IPv4/IPv6 private/loopback/link-local/reserved/multicast SSRF tests.
- [ ] 6.2 Enforce remote-image redirect, deadline, encoded-byte, decoded-pixel, MIME, and format limits; decode/re-encode supported images to reject malformed/polyglot content and strip metadata before upload.
- [ ] 6.3 Implement streaming validation for attached local images with the same advertised type/byte/pixel policies and deterministic failures for MIME spoofing, decompression bombs, duplicate/missing assets, and invalid cover sources.
- [ ] 6.4 Implement connector-side HTML parsing/sanitization compatible with the pinned MarkNice corpus, asset/remote-source rewriting hooks, active-content removal/rejection, and final assertion that no external, local, loopback, blob, data, or placeholder image remains.
- [ ] 6.5 Add normalized golden fixtures that carry representative MarkNice themes, typography, code, math, tables, links, local images, and remote images through package validation and final connector sanitization without unsafe content or unintended formatting loss.

## 7. WeChat adapter and draft-job execution

- [ ] 7.1 Implement connector-only WeChat access-token acquisition, in-memory expiry caching, classified refresh, AppSecret/token redaction, and fake-upstream tests for invalid credentials, IP restrictions, permission denial, rate limits, and malformed responses.
- [ ] 7.2 Implement inline article-image upload and WeChat-URL replacement with bounded safe retries only before draft creation, including partial-upload failure reporting and cleanup/retention behavior.
- [ ] 7.3 Implement cover resolution and permanent-material upload for attached, remote, and connector-default covers, including required `thumb_media_id` validation and advertised default-cover capability.
- [ ] 7.4 Implement the final draft-add request with confirmed metadata and sanitized HTML, make exactly one call after entering `committing`, map definitive rejection to failed, map lost/ambiguous response to outcome-unknown, and never invoke publish/schedule/mass-send APIs.
- [ ] 7.5 Integrate media preparation and the WeChat adapter into the worker state machine, returning only safe account/job/draft identifiers and deleting transient content on the documented terminal/expiry schedule.
- [ ] 7.6 Add an end-to-end fake-WeChat suite covering token refresh, mixed local/remote images, cover paths, final URL validation, successful draft ID, pre-commit failures, definitive draft rejection, lost response, restart during commit, no automatic retry, and absence of real network credentials.

## 8. Documentation, CI, and release verification

- [ ] 8.1 Write user documentation for configuring/testing/clearing a connector, choosing metadata and cover, interpreting failed versus unknown results, and verifying that success means draft saved rather than published.
- [ ] 8.2 Write an operator guide for container deployment, HTTPS reverse proxy, connector-token generation/rotation, WeChat AppID/AppSecret secret injection, IP allowlisting, account/API eligibility, state backup/update, retention, egress filtering, health checks, and incident-safe logs.
- [ ] 8.3 Add CI jobs for contract validation, root `cargo test --workspace`, standalone connector locked tests, fake-browser/fake-WeChat integration, dependency/license/security checks, and checks that neither project links across the connector boundary.
- [ ] 8.4 Add staged/package verification for Windows, macOS, and Linux proving the Markion bundle contains required client/browser assets but excludes `connectors/`, standalone service binaries, state files, bearer tokens, AppID/AppSecret values, and WeChat access tokens.
- [ ] 8.5 Run formatting, linting, `cargo test --workspace`, the connector's independent full suite, MarkNice browser compatibility tests, contract conformance, package inspections, and cross-platform OS-keyring smoke tests; record results in the change.
- [ ] 8.6 Deploy the reference connector to an HTTPS test environment, save a mixed-content article to an authorized Official Account, inspect the draft box for metadata/theme/code/math/image/cover fidelity, verify no publication call occurred, and record sanitized manual release evidence.
