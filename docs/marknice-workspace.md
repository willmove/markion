# Local MarkNice publishing workspace

## User workflow

Open a Markdown document and choose **Export → Publish for WeChat (MarkNice)**.
Markion takes one immutable snapshot of the active in-memory text—including
unsaved edits—and opens a private loopback URL in the default browser. The URL
contains a one-time capability in its fragment; the page removes it immediately
and exchanges it for a token kept in that browser tab's `sessionStorage`.

The editor, all 15 pinned MarkNice themes, marked, KaTeX, CSS, and fonts are
packaged with Markion and load without external networking. Edits made in the
browser are session-local: there is no endpoint that saves them to Markion.
Return to Markion to make durable edits and launch again for a new snapshot.

Managed images under the saved document's `<document>.assets` directory are
available only through authenticated opaque resource IDs and browser blob URLs.
They can be previewed but cannot survive a paste into WeChat. When local images
are present, copying offers cancel or an explicit copy-without-local-images
choice and reports the omitted count. Missing, untitled, out-of-scope, and newly
typed local paths remain unresolved. Remote HTTP(S) images retain MarkNice's
normal behavior and may disclose the browser's IP address to their authored
hosts; requests carry no referrer from the workspace.

The browser needs clipboard permission for the preferred `text/html` plus
`text/plain` path. A selection-based fallback is used where the richer API is
unavailable. Sessions expire after two hours without an authenticated request,
and at most eight snapshots are retained. Relaunch from Markion after expiry.

## Maintainer refresh

The runtime is pinned to MarkNice commit
`c009c1ec7e7c92f89afa5a32edcb126b5296bda7`, marked 15.0.12, and KaTeX 0.16.11.
Normal builds use only checked-in files and require neither the sibling MarkNice
repository, Node.js, nor downloads.

To intentionally refresh third-party bytes and regenerate the provenance
manifest, first place the MarkNice checkout at the pinned commit, then run:

```powershell
pwsh ./scripts/sync-marknice-workspace.ps1 `
  -Source C:\Coding\EditorProjects\marknice `
  -RefreshThirdParty
cargo run -p wechat-workspace --bin verify-bundle -- assets/marknice-workspace
```

The refresh command is maintainer-only. It refuses an unexpected MarkNice
commit, imports only the editor theme/sanitizer/copy core, fetches exact npm
package versions, and rewrites SHA-256 digests. Review the generated runtime,
licenses, compatibility corpus, and digest diff before committing. Do not add
landing/guide pages, DOCX/PDF import, proxy/OCR/OSS, analytics, hosted-update,
or duplicate file-export code to the packaged surface.

Digest generation and verification both normalize text-file line endings to LF,
and `.gitattributes` pins `assets/marknice-workspace/**` to LF checkouts, so
the manifest stays byte-stable regardless of `core.autocrlf` or platform.

Release jobs verify the source bundle and extract every NSIS, macOS app/DMG,
DEB, and AppImage output to verify the packaged manifest, local dependency
closure, digests, and absence of remote runtime dependencies.
