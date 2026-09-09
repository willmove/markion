# Design: rich text (HTML) paste

## Context

`MarkionApp::paste` (`src/app/editing.rs:2583`) reads a GPUI `ClipboardItem` and either imports an image entry as a managed asset or inserts `item.text()` verbatim. The vendored GPUI clipboard model (`vendor/zed/crates/gpui/src/platform.rs:1512-1597`) has only `String` and `Image` entries, and no platform backend reads an HTML flavor: macOS reads `public.utf8-plain-text` + image types, Windows reads `CF_UNICODETEXT`/`CF_HDROP`, Wayland accepts two plain-text mimes, and X11 has the `text/html` atom commented out. The only HTML→Markdown converter in the repo is the DOM-dependent JS function inside the MarkNice browser bundle (`assets/marknice-workspace/static/marknice-word-import-runtime.js:652-776`), not callable from the app.

Word, Google Docs, LibreOffice, and browsers all place a `text/html` flavor on the clipboard next to plain text, so reading that flavor and converting it to Markdown is the standard rich-text paste design (Typora, Obsidian, Zed all do this).

## Decisions

### 1. `ClipboardItem` carries an optional `html` field, not a new entry variant

`ClipboardItem { entries, html: Option<String> }` with `html()`, a `pub(crate)` setter, and a public `new_html(text, html)` constructor (tests, and symmetry with `new_string`). Alternatives considered: a `ClipboardEntry::Html` variant — rejected because every exhaustive match on entries (macOS RTF composition, Windows writer, the app's image/text split) would need churn, and an HTML flavor is never an independent payload, only an alternative representation of the text payload. Writes never emit HTML, so copy behavior is unchanged.

### 2. Extend the vendored GPUI backends instead of adding a clipboard crate

The repo already carries gpui as a `[patch.crates-io]` path dependency and pins its whole dependency graph; adding a second clipboard library (e.g. arboard) would introduce a new dependency family and a parallel OS clipboard connection. Backend changes are small and local:

- **Wayland**: `DataOffer::read_text` already reads one mime through a pipe; add `read_html` (`text/html`) and attach it in `Clipboard::read`/`read_primary` after a successful text read. Self-owned offers and image offers are untouched.
- **X11**: enable the commented `text/html` atom; in `get_any`, after the text result, run one more `Inner::read(&[HTML_MIME])` and attach on success. `Inner::read` already returns `ContentNotAvailable` quickly when the target is unoffered (TARGETS probe), including when we own the selection, so failure is quiet and cheap. This adds one extra selection round-trip per paste — user-action latency only.
- **macOS**: in `read_string_from_clipboard`, additionally read `public.html` from the pasteboard and attach it.
- **Windows**: register `"HTML Format"` (CF_HTML) lazily like the other custom formats, read it when the best-match entry is text, and parse the CF_HTML header (`StartFragment`/`EndFragment` byte offsets) to extract the fragment; fall back to the whole payload when the header is missing/invalid. CF_HTML is UTF-8 per Microsoft docs.

Only the Linux backends can be compiled and exercised in this environment; macOS/Windows code mirrors the file's existing patterns and relies on CI builds.

### 3. New GUI-free member crate owns conversion

`crates/html-import` (package `markion-html-import`) exposes `pub fn html_to_markdown(html: &str) -> String` with zero external dependencies, per the crate-architecture spec (member crates are GUI-free; a paste-time converter still gets a dev-profile opt-level override like `markion-docx-import`). A hand-rolled tolerant tokenizer + tree builder is preferred over an HTML-parser dependency: clipboard HTML is machine-generated and structurally simple, and the repo already hand-rolls HTML parsing (`src/parse.rs`) with no HTML crate in the lockfile to reuse.

Semantics follow the two in-repo references: the MarkNice JS `htmlToMarkdown` (tag vocabulary, adjacent-marker merging, blank-line collapsing) and `markion-docx-import`'s render layer (`**`/`*`/`~~`/`<u>`/`<sub>`/`<sup>` markers, escaping policy, GFM tables with empty header for headerless input, merged cells as one raw HTML table). Beyond those references, the renderer handles `font-weight`/`font-style`/`text-decoration` style attributes because Google Docs expresses all inline formatting through styled spans, and Word uses them alongside tags.

Notable policies:

- Non-content markup (`head`, `script`, `style`, comments including Word conditional comments, `meta`, `link`, `title`, `xml`) is skipped; unknown tags are transparent (children kept).
- Links keep `http(s)`, `mailto`, `ftp`, schemeless, and `#fragment` targets; other schemes render as their text. URLs escape spaces/parens like docx-import.
- Images are emitted by reference; `data:` sources (potentially megabytes) reduce to their alt text.
- Lists nest with marker-width continuation indent; `ol` honors `start`; `<input type=checkbox>` list items become GFM task items.
- Whitespace collapses per HTML rules outside `pre`; `pre` content is emitted raw inside a fence sized to the content; inline code sizes its backtick run likewise.
- Text is backslash-escaped with the docx-import character set, plus line-start block markers and `|` inside table cells, so pasted prose cannot form unintended Markdown structures.
- Output has no leading/trailing blank lines; blocks are separated by exactly one blank line.

### 4. Paste precedence and fallback

Inside `paste()`: image entry (unchanged) → text-input focus (raw text, unchanged) → **HTML present: insert converted Markdown** → plain text verbatim. If conversion returns an empty string (e.g. the HTML carried only scripts/styles), fall back to the plain text. An item with HTML but no text entry inserts the converted Markdown directly. Insertion reuses `replace_text_in_range` wrapped in the existing atomic undo capture, so one undo removes the whole paste and Visual/Source modes treat the inserted Markdown like any other.

## Risks

- **macOS/Windows readers are compile-unverified locally.** Mitigation: minimal, pattern-mirroring code; CI builds all targets; runtime evidence recorded as unavailable rather than claimed.
- **X11 extra round-trip** when the owner is slow: bounded by the existing 4s read timeout and only on paste.
- **Over-eager conversion**: any HTML flavor now becomes Markdown, including simple single-paragraph copies — output equals the plain text in that case, so behavior only differs when formatting exists.
