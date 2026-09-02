# Local MarkNice publishing workspace

## User workflow

Open a Markdown document and choose **Export → Publish for WeChat (MarkNice)**.
Markion takes one immutable snapshot of the active in-memory text—including
unsaved edits—and opens a private loopback URL in the default browser. The URL
contains a one-time capability in its fragment; the page removes it immediately
and exchanges it for a token kept in that browser tab's `sessionStorage`.

The editor, all 15 pinned MarkNice themes, marked, MathJax, and CSS are
packaged with Markion and load without external networking. The workspace
chrome is an editor skin derived from the pinned MarkNice editor section: dual
rounded cards, traffic-light headers, indigo accent, Inter / PingFang SC /
Microsoft YaHei UI fonts, an SF Mono editor stack, SVG formatting and preview
toggles, grouped font/spacing steppers, and a 375px phone frame with a notch.
Session-local and privacy disclosures stay visible; the MarkNice marketing
navbar, hero, features, guide, and footer are not packaged. Formulas are
typeset by MathJax into self-contained inline SVG (the same approach as the
MarkNice Obsidian plugin), so they survive the WeChat editor's paste-time
filtering without the duplication that hidden MathML caused. Edits made in the
browser are session-local: there is no endpoint that saves them to Markion.
Return to Markion to make durable edits and launch again for a new snapshot.

Managed images under the saved document's `<document>.assets` directory are
available only through authenticated opaque resource IDs and browser blob URLs.
They can be previewed but cannot survive a paste into WeChat. When local images
are present, WeChat copying offers cancel or an explicit copy-without-local-images
choice and reports the omitted count. Missing, untitled, out-of-scope, and newly
typed local paths remain unresolved. Remote HTTP(S) images retain MarkNice's
normal behavior and may disclose the browser's IP address to their authored
hosts; requests carry no referrer from the workspace.

Above the Markdown editor, the P0 formatting toolbar provides H1/H2/H3, bold,
italic, underline, ordered/unordered lists, inline code, link, quote, fenced code
block, image syntax, and table commands. Ctrl/Cmd+B, Ctrl/Cmd+I, Ctrl/Cmd+U,
and Ctrl/Cmd+K invoke the matching commands while the editor is focused. These
changes update only the current browser session and preview; they are not saved
back to Markion. The image command inserts syntax only and does not upload a
file.

The export toolbar also provides browser-local recovery/export actions:

- **Copy MD** copies the exact current textarea value as plain text. It
  includes tab-local edits, whitespace, and line endings; it never substitutes
  the snapshot that opened the workspace.
- **Save as HTML** creates a standalone MarkNice-styled UTF-8 HTML file from
  the current theme and typography settings. Formulas are embedded as
  self-contained inline SVG (no CSS or web fonts required), as are safely
  readable managed raster images. An image larger
  than 8 MiB or when the export would exceed 24 MiB is replaced with a visible
  non-sensitive text fallback. Authored remote HTTP(S) images remain linked to
  their original hosts, so such an export is not fully offline.
- **Save as Word** creates a browser-generated MarkNice DOCX from the same
  current presentation. It uses a Word-oriented HTML compatibility path and is
  distinct from Markion's native/Pandoc DOCX export. Microsoft Word desktop is
  the compatibility target; LibreOffice, Pages, web viewers, and OS previews
  are best-effort only.
- **Save as PDF** prints a clone of the current themed, sanitized preview
  (theme plus font-size and paragraph-spacing offsets) through the browser
  print dialog. Choose “Save as PDF” there. JavaScript cannot prove a file was
  written. This path is distinct from Markion **Export → PDF** (native/Pandoc).
  Workspace chrome, the disclosure banner, toolbars, and the phone bezel are
  not part of the print document.

**Import Word** on the left panel reads a `.docx` file entirely in the browser
(JSZip 3.10.1 plus the pinned MarkNice parser). It replaces only the current
tab's Markdown and preview. It never POSTs the file or Markdown to Markion, so
the open document's bytes, version, dirty flag, selection, undo history, and
derived caches stay unchanged. After a successful import, the status states
that the result is session-local and that bringing it into Markion requires
Copy MD or another explicit save of the session Markdown. Files that are not
readable `.docx`, exceed 20 MiB, or fail to parse leave the current Markdown
in place. Word-embedded images become session data URIs, not managed loopback
resources.

The workspace does not import Markdown files, PDFs (including OCR), or local
images, and it has no sample or clear-document action.

HTML and DOCX artifacts remove workspace controls, scripts, event handlers,
forms, frames, unsafe URL schemes, session data, blob URLs, loopback URLs, and
filesystem references. The browser only reports that a download has started;
the browser or operating system ultimately decides where it is saved. Save as
PDF only reports that the print dialog opened.

Windows manual evidence recorded on 2026-08-23 confirms that **Copy Markdown**
and the browser-generated **Word export** work. On 2026-09-02 the editor-skin
self-test, in-browser Word-import fixture, and themed print-clone checks were
added; macOS and Linux default-browser visual comparison with the sibling
MarkNice editor remains deferred with the existing workspace evidence matrix.
The remaining release matrix still includes standalone HTML, denial/cancellation/repeat/session/image edge
cases, other supported browser/platform combinations, a recorded Microsoft Word
version, and a secondary DOCX viewer.

The browser needs clipboard permission for the preferred `text/html` plus
`text/plain` WeChat path and for preferred Markdown plain-text copying. A
selection-based fallback is used where the relevant clipboard API is
unavailable. Sessions expire after two hours without an authenticated request,
and at most eight snapshots are retained. Relaunch from Markion after expiry.

## Maintainer refresh

The runtime is pinned to MarkNice commit
`c009c1ec7e7c92f89afa5a32edcb126b5296bda7`, marked 15.0.12, MathJax 3.2.2,
html-docx-js 0.3.1, and JSZip 3.10.1.
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
commit, imports the theme/sanitizer core, Word preparation helpers, and Word
import parser by unique named source markers (not line numbers), fetches exact
npm package versions, and rewrites SHA-256 digests. Markion-specific
localization, clipboard, durable-artifact safety, image embedding, Word import
orchestration, and print-to-PDF stay in `static/export-runtime.js` and
`static/bridge.js`. Review the generated runtimes, licenses, compatibility
corpus, and digest diff before committing. Do not add landing/guide pages,
Markdown/PDF/image import, proxy/OCR/OSS, analytics, hosted-update, or a Node
conversion service to the packaged surface.

Source inventory for the pinned revision:

- Copy Markdown behavior follows the `copyMdBtn` handler and reads the live
  `markdownEl.value`; Markion keeps that behavior in its curated export module
  so clipboard fallback, localization, and session isolation remain reviewable.
- The original `saveHtmlBtn` handler supplies the feature intent, while
  Markion's curated wrapper applies the durable-artifact safety pass; formulas
  need no embedded assets because MathJax emits self-contained SVG.
- MarkNice's KaTeX-based `renderMath` is replaced at sync time with the curated
  `static/math-runtime.js` (MathJax tex-svg with `fontCache: 'none'`, matching
  the MarkNice Obsidian plugin), which also provides the linear-text formula
  fallback used by the browser DOCX export.
- The generated `static/marknice-word-runtime.js` is the unique region between
  `// ===== Word export helpers =====` and `// ===== Save as Word =====`.
- The generated `static/marknice-word-import-runtime.js` concatenates
  `src/docx-parser.js`, `mathNodeTex`, and the unique HTML-to-Markdown region
  between `// ===== HTML to Markdown converter =====` and
  `// ===== Word (.docx) import =====`. Sync also asserts the editor-section
  CSS region between `/* ===== Editor Section ===== */` and
  `/* ===== Footer ===== */`.
- `static/vendor/jszip.min.js` is JSZip 3.10.1 (MIT), required by Word import.
  Do not reuse the private zipper inside `html-docx.js`.
- Save as PDF is curated in `static/export-runtime.js`: it prints the sanitized
  themed preview clone and must not port MarkNice web's generic article print
  stylesheet.
- The generated `static/marknice-format-runtime.js` is the unique formatting
  region from `const markdownFormatToolbar =` through the helper immediately
  before `function localImageRecordsForCurrentPreview()`. Markion excludes the
  source application's local-image upload action and supplies localized
  placeholders around the imported selection/caret behavior.
- `static/vendor/html-docx.js` starts from html-docx-js 0.3.1 (MIT). Its
  synthetic MHT `file:///C:/fake/...` locations are deterministically rewritten
  to non-resolving `urn:markion:...` locations so no filesystem reference is
  placed in a downloaded DOCX while package-local image matching is retained.

Digest generation and verification both normalize text-file line endings to LF,
and `.gitattributes` pins `assets/marknice-workspace/**` to LF checkouts, so
the manifest stays byte-stable regardless of `core.autocrlf` or platform.

Release jobs verify the source bundle and extract every NSIS, macOS app/DMG,
DEB, and AppImage output to verify the packaged manifest, local dependency
closure, digests, and absence of remote runtime dependencies.
