## Why

Markion's local MarkNice workspace can prepare WeChat-ready rich content, but users must still move that content into the Official Account draft box manually. A self-hosted publishing connector lets Markion offer a deliberate one-click draft-save workflow without embedding WeChat application secrets or coupling the desktop application to WeChat token and media APIs.

## What Changes

- Add a **Save to WeChat draft box** action to the local publishing workspace, using the browser session's finalized article HTML and confirmed article metadata.
- Define a versioned, vendor-neutral connector contract for capability discovery, idempotent draft-job submission, asset transfer, status polling, and sanitized results.
- Add Markion-side connector setup, connection testing, secure credential storage, confirmation, progress, ambiguous-outcome handling, and localized success/error states.
- Add an independently deployable reference WeChat connector that owns Official Account credentials, access-token caching, media upload and URL rewriting, draft API calls, job state, and operational safeguards. It is not linked into or packaged with the Markion application.
- Preserve the current document and publishing-session boundaries: draft submission is an external side effect and never writes browser edits back into the open Markdown document or invalidates/recomputes its derived caches.
- Require the connector to upload both local and remote article images to WeChat-controlled media URLs, resolve a required cover image, and reject unresolved or unsafe content before creating the draft.
- Make `add-local-marknice-publishing-workspace` an implementation prerequisite because this change extends that workspace and its authenticated loopback bridge.

**Non-goals:** publishing or scheduling a draft, managing multiple Official Accounts in one connector instance, storing AppID/AppSecret in Markion, calling WeChat APIs directly from Markion or browser JavaScript, and synchronizing browser edits back to the Markdown document.

## Capabilities

### New Capabilities

- `wechat-draft-publishing`: Covers connector configuration and trust boundaries, browser-to-Markion submission, the versioned self-hosted connector contract, WeChat-compatible asset preparation, draft-job lifecycle, user confirmation and feedback, and reference connector behavior.

### Modified Capabilities

None.

## Impact

- **Markion UI and loopback workspace:** publishing controls, metadata/cover confirmation, protected submission routes, progress/results, and seven-locale strings.
- **GPUI-free Rust code:** connector protocol/client, article package construction, validation, credential abstraction, and mockable job orchestration; GPUI-specific state remains in the root crate.
- **Self-hosted system:** a separately built and deployed reference connector with its own runtime configuration, persistence, container assets, tests, and operator documentation.
- **Security and privacy:** Markion stores only the connector origin plus a scoped connector credential in the operating-system credential store; the connector alone stores WeChat credentials and communicates with WeChat APIs.
- **Packaging and release:** the desktop distributions include the connector client and browser assets, but exclude the connector service and all WeChat secrets.
- **Architecture invariants:** document-version derived state, syntax-highlighting memoization, cached text handles, undo snapshots, and bounded file-tree rendering are unaffected; submission consumes an immutable publishing-session snapshot instead of introducing typing-path recomputation.
