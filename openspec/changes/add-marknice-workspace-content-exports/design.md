## Context

See `proposal.md` for motivation. This change extends the editor-only browser workspace introduced by `add-local-marknice-publishing-workspace`. Its implemented code and strict OpenSpec validation form the baseline for this work; the prerequisite change's remaining cross-platform/browser/WeChat manual evidence is collected in parallel and remains required before either related release is archived. The existing workspace receives an immutable Markion snapshot through an authenticated loopback session, then permits session-local browser editing and presentation changes. Its current bridge already holds the live textarea value, sanitized MarkNice preview HTML, selected theme and typography offsets, and protected-image blobs/object URLs, but it exposes only WeChat rich copy.

The sibling MarkNice revision pinned by the bundle already contains its Markdown formatting toolbar and shortcuts, Copy Markdown, standalone HTML download, and an `html-docx-js` browser export path. The checked-in Markion subset intentionally omitted those sections and the CDN-loaded DOCX runtime. Markion packages must remain self-contained and offline-capable, and downloaded files must remain usable after the loopback session disappears. The existing one-way document and immutable resource boundaries remain authoritative.

The relevant data flow becomes:

```text
Markion MarkdownDocument (unchanged)
        │ explicit workspace launch, once
        ▼
immutable authenticated snapshot
        │
        ▼
browser textarea ──render──▶ sanitized MarkNice preview + presentation state
        ▲                              │
        ├─ formatting toolbar/keys     │
        ├─ Copy Markdown               └─ build sanitized export snapshot
        │                                      │
        │                         ┌────────────┴────────────┐
        │                         ▼                         ▼
        │                  standalone HTML          browser DOCX Blob
        │                         │                         │
        └─────────────────────────┴──────────────▶ user download/clipboard

No arrow returns to MarkdownDocument or its versioned caches.
```

## Goals / Non-Goals

**Goals:**

- Reuse the pinned MarkNice user-facing behavior while adapting it to the authenticated, offline Markion workspace.
- Provide the pinned MarkNice formatting command set in a compact, keyboard-accessible toolbar without writing browser edits back to Markion.
- Make every operation consume the exact current browser state, including edits whose debounced render has not yet fired.
- Produce portable artifacts without tokens, ephemeral URLs, executable authored content, or hidden conversion requests.
- Keep the browser DOCX path clearly separate from Markion's richer native/Pandoc DOCX exporter.

**Non-Goals:**

- A loopback endpoint that receives edited Markdown or generated HTML.
- Fidelity parity between browser `html-docx-js` output and Markion's native/Pandoc DOCX pipeline.
- Fetching authored remote images for embedding, publishing local images remotely, or changing the rich-copy omission policy.
- Persisting browser export settings or browser-edited content across independent publishing sessions.

## Decisions

### 1. Generate all three outputs in the authenticated browser tab

Copy Markdown reads the textarea directly. HTML and DOCX are produced as browser Blobs and downloaded through temporary object URLs created in the same user-initiated action. No new service route accepts edited content, and the Rust workspace service remains read-only.

This keeps session-local edits within their existing trust boundary and avoids adding a second HTML-to-DOCX implementation to Rust. Calling Markion's native export path was rejected because it would require posting browser-edited state back to the process, would not naturally preserve MarkNice presentation choices, and would blur the product distinction between the two DOCX formats. Reusing the hosted MarkNice or a conversion API was rejected because it would violate offline operation and content privacy.

### 2. Build one immutable export snapshot per action

Before either download, the bridge cancels the pending debounce and renders the current textarea value synchronously. It then clones the sanitized publishing DOM and captures the document title, safe base filename, theme identifier, typography offsets, locale, and managed-resource metadata into an operation-local snapshot. HTML and DOCX preparation consume separate clones of this snapshot so one transformation cannot affect the live preview or the other format.

The filename comes from the current Markdown title with the launch display name as fallback, is stripped of platform-invalid/control characters, is length-bounded, and receives exactly one `.html` or `.docx` suffix. Export completion wording says that a download was started, because browser JavaScript cannot prove that the browser ultimately wrote the file.

### 3. Apply a format-independent safety and resource pass

The shared export pass starts from the existing MarkNice-sanitized preview but validates it again for a durable-file threat model. It removes scripts, styles supplied by the article, event attributes, frames/plugins, forms, meta refresh, unsafe URL schemes, workspace-only data attributes, session identifiers, and any loopback/blob/file reference that has not been replaced deliberately.

The bridge retains a bounded in-memory mapping from protected resource identity to its already authenticated Blob. For export, managed local images are converted to data URIs from those blobs without another filesystem capability or a widened allowlist. If bytes are not already available, the bridge may use the existing authenticated fixed resource route while the session remains live; canonical containment is therefore rechecked by the service. Images are processed sequentially and subject to per-image and total export-byte limits. A failed or oversized protected image becomes a styled text fallback derived only from safe alt text or a non-sensitive display name, and the UI reports the count.

Authored HTTP(S) images remain authored remote references. The exporter does not proxy or prefetch them, and messaging documents that those resources are not self-contained. Other schemes are removed. This differs from WeChat rich copy: managed images may be embedded in a local file because that file is the final destination, while the clipboard path still omits images that WeChat cannot publish.

### 4. Make themed HTML a standalone, inert document

The HTML artifact wraps the prepared article in a UTF-8 standards-mode document with an escaped title, responsive viewport, restrictive artifact CSP, and no application script. MarkNice theme rules are already inline on article elements. The exporter embeds the bundled KaTeX stylesheet and the WOFF2 font data needed by the rendered math markup so a document with no authored remote resources remains visually useful offline. It does not embed the workspace chrome or controls.

Embedding local style/font data was chosen over references back to the loopback origin, which fail after Markion exits. Copying the original MarkNice HTML wrapper unchanged was rejected because its math presentation is not fully standalone and because a downloaded file needs stricter active-content protection than the loopback page's CSP alone provides.

### 5. Vendor and constrain the pinned MarkNice browser DOCX path

The bundle vendors the exact supported `html-docx-js` browser distribution and license locally, records its version and digest in the existing manifest, and loads it under the existing self-only script CSP. Maintainer refresh may use npm in a temporary directory, but normal build, test, packaging, and runtime use only checked-in files.

Word preprocessing is ported from the pinned MarkNice implementation: soft-break normalization, left-aligned list and table content, compact table cells, bounded image sizing, and the Word-oriented HTML wrapper are retained. The shared resource pass first converts managed local images to data URIs so the converter packages them rather than preserving ephemeral URLs. The browser-generated DOCX is identified separately in labels, documentation, and tests because `html-docx-js` uses a Word-oriented HTML/altChunk compatibility path and is not equivalent to Markion's native OOXML/Pandoc export. Microsoft Word is the compatibility target; behavior in LibreOffice, Pages, web viewers, and previewers is documented as best-effort rather than silently promised.

Alternatives considered were adding the converter to the Rust loopback service, which violates the browser-side requirement and duplicates native export infrastructure, and reusing Markion's native exporter, which cannot represent the current browser-only MarkNice state without adding reverse synchronization.

### 6. Import MarkNice code by semantic regions and verify parity

The maintainer sync command selects the pinned Copy Markdown and Word-export source by stable named section markers or an explicit curated module, rather than introducing additional numeric line slices. Workspace-specific orchestration, localization, resource safety, and downloads live in a small bridge/export module around that imported core. Golden browser tests compare representative prepared HTML and DOCX package structure with the pinned compatibility expectations.

This preserves upstream provenance while keeping Markion-specific authentication and export hardening reviewable. Copying the entire MarkNice application was rejected because it would reintroduce imports, landing content, remote service code, and CDN dependencies that remain out of scope.

### 7. Keep failures local, bounded, and non-mutating

Each action has an in-progress guard and disables only its own control. Temporary object URLs are revoked after the download has been initiated, including failure cleanup. Clipboard and download failures update localized status without clearing the editor, rerendering a different document, or changing theme controls. Session expiry during a resource-dependent export uses the existing relaunch guidance; source copy remains available from the textarea even after protected resource access expires when the page is otherwise still usable.

No export operation calls into `MarkdownDocument`, increments document version, or touches shared preview/outline/statistics caches. Rust-side tests retain the base change's unchanged-state assertions, while browser tests assert that live editor and presentation state survive success and failure.

### 8. Port the pinned MarkNice formatting command layer as browser-session behavior

The sync flow extracts the pinned selection/caret formatting logic into a generated `marknice-format-runtime.js`. The P0 command set is H1/H2/H3, bold, italic, underline, ordered/unordered lists, inline code, link, quote, fenced code block, image syntax, and table. Commands act on the current or last remembered selection, toggle supported wrappers or prefixes, insert localized placeholders for empty selections, restore textarea focus and selection, and synchronously refresh the current preview.

Ctrl/Cmd+B, Ctrl/Cmd+I, Ctrl/Cmd+U, and Ctrl/Cmd+K are handled while the textarea is focused; Alt-modified and unsupported shortcuts remain untouched. The horizontally scrollable toolbar uses localized titles and accessible names and remains usable at the narrow layout breakpoint. MarkNice's local-image upload control is deliberately excluded: the image command inserts syntax only, preserving the existing no-upload boundary.

Formatting mutates only the authenticated browser session's textarea and preview. It does not call a Markion mutation endpoint, change the GPUI document version, or imply a save back to disk. Reimplementing each command independently was rejected because it would risk drifting from MarkNice selection/toggle behavior; importing image upload was rejected because it would expand security and persistence scope.

## Risks / Trade-offs

- [Browser DOCX uses an older Word-oriented altChunk path and has uneven non-Word support] → Pin the exact runtime, target documented Microsoft Word versions, test representative output manually, label the action as browser-generated, and retain Markion's native DOCX export for portable/advanced use.
- [Standalone KaTeX fonts can make HTML files larger] → Embed WOFF2 only, include only the maintained KaTeX stylesheet/font set required by the compatibility corpus, and record size deltas in release evidence.
- [Large or numerous images can create high transient browser memory use] → Reuse existing resource limits, add an aggregate export budget, process sequentially, release intermediate buffers/object URLs promptly, and fall back visibly rather than exhausting memory.
- [Asynchronous preparation can interact differently with browser download policies] → Start from an explicit user gesture, avoid popups, use a temporary anchor download, and cover supported default browsers with manual and automated evidence.
- [Remote images make an otherwise portable artifact network-dependent] → Never silently prefetch them, preserve only authored HTTP(S) references, and disclose their presence in export feedback/documentation.
- [Exported authored HTML could become active outside the loopback CSP] → Run the durable-artifact safety pass, add an artifact CSP, and test scripts, handlers, forms, unsafe URLs, frames, SVG/HTML edge cases, and token/reference leakage.
- [Generated code extraction can drift when MarkNice changes] → Use semantic markers, keep provenance and digests, and fail the sync/parity check when source regions or normalized outputs change unexpectedly.

## Migration Plan

1. Confirm the implemented `add-local-marknice-publishing-workspace` baseline and its strict validation before implementation; carry its outstanding manual evidence forward and reconcile both changes against the archived capability before release/archive.
2. Add the generated Markdown formatting runtime, localized toolbar/shortcuts, local converter asset, export module, manifest/license changes, and compatibility fixtures in the checked-in workspace bundle.
3. Regenerate canonical LF-normalized bundle digests and run source-tree, browser, package, and cross-platform verification before release.
4. No stored data or configuration migration is required. Rollback removes the formatting and export controls/runtimes, restores the previous manifest/assets, and leaves existing sessions and Markion documents unchanged.
