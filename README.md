<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">
</p>

<p align="center">
  <strong>English</strong> · <a href="README.zh-CN.md">简体中文</a>
</p>

# Markion

Markion is a native desktop Markdown editor built with Rust and GPUI. It combines responsive source editing, a source-backed Visual Edit mode, live preview, workspace tools, and multi-format export in one lightweight application. Markdown remains the canonical document format—no Electron, Tauri, or WebView.

## Install

Download the latest build from [GitHub Releases](https://github.com/willmove/markion/releases).

| Platform | Release packages | Target |
|---|---|---|
| Windows | NSIS `.exe` installer | x86_64 |
| Linux | `.deb` and AppImage | x86_64 |
| macOS | `.app` and `.dmg` | Apple Silicon (arm64), macOS 11+ |

Releases are not platform code-signed. Windows SmartScreen may require **More info → Run anyway**, and macOS Gatekeeper may require right-clicking the app and choosing **Open**. **Help → Check for Updates…** offers an actionable update prompt on every platform: tagged Windows x86_64 NSIS installations get a cryptographically verified (cargo-packager Minisign) one-click download-and-install that refuses to start while any document has unsaved changes, while macOS and Linux open the matching release file in the system browser. Intel Macs can run the arm64 build through Rosetta; a universal binary and Apple notarization are not currently provided.

## Editing modes

Markion has four view modes. Split Preview is the default.

- **Edit** — a focused raw Markdown source editor.
- **Visual Edit** — a WYSIWYG-first, source-backed surface. Prose stays rendered with progressive syntax reveal; ordinary fenced-code payloads, block math, inline image fields, and GFM table cells have exact direct editors. Constructs whose WYSIWYG rendering is not yet implemented — decoded HTML entities, YAML front matter, indented code, and malformed or byte-ambiguous syntax — keep an exact source-backed editing affordance as a transitional measure, tracked on the WYSIWYG coverage roadmap. A slash-command palette and a compact right-click block context menu offer exact block transforms (paragraph, headings, lists, quote, fenced code, divider, table), duplicate, move up/down, source-safe drag reorder, and delete; a selection-contextual formatting toolbar and a visual link editor perform one exact source-backed mutation per action. This is not a separate rich-text document model—the underlying Markdown is always the source of truth.
- **Split Preview** — source and rendered preview side by side, with an optional source-mapped Sync scroll setting that keeps both panes on the same document location rather than scrolling by whole-document percentage.
- **Read** — a rendered, non-editing view centered at a readable 860 px maximum width by default; Preview adaptive width can use the full pane.

Switching modes preserves the active document, cursor and selection, undo history, and per-tab scroll state.

## Documents and workspace

- Multi-tab editing with per-tab cursor, selection, scroll, undo/redo, preview, outline, and cached derived Markdown state.
- Opening an already-open Markdown or plain-text file focuses its existing tab instead of creating a duplicate.
- **Open Folder** changes the workspace root and populates the Files sidebar with Markdown files, a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`/`.asciidoc`), and supported image files (`.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`/`.tiff`, `.svg`), nested under their folders; empty folders are listed too. Markdown stays visually distinguished, plain-text files open as UTF-8 text, and image files open as read-only image tabs that fit oversized images within the content area.
- Expanding a folder reveals exactly one level of children, so deeply nested workspaces can be drilled into one level at a time.
- A **Show hidden files/folders** preference (default off) reveals dotfile entries plus the Windows hidden-attribute flag, while always-excluded build, dependency, and VCS noise (`target`, `node_modules`, `.git`, …) stays hidden regardless.
- Right-click file-tree menus provide open, open in new tab, create file/folder, rename, delete, reveal in the system file manager, filter, and refresh actions where applicable.
- Files and folders can be named inline; deleting a non-empty folder requires an additional confirmation.
- Markdown files can be dragged from the operating-system file manager into Markion.
- The Files and Outline panels are toggleable, and sidebar and split-pane dividers are draggable.
- The Outline panel lists the document's heading hierarchy as a collapsible tree: each heading with descendants exposes a disclosure control, outlines start fully expanded, folding is per-document and session-only, and the section containing the cursor is highlighted. Clicking a heading jumps to its source position—or to the rendered heading in Read mode.
- The status bar keeps document identity, save state, and transient operation feedback, and adds a compact persistent context: the active document's character and word count, the caret's one-based line and column when an editing surface is present, and the current Git branch when the document or workspace belongs to a repository.

## Markdown editing and preview

- Parsing is powered by `pulldown-cmark` with CommonMark and GFM-oriented support.
- Formatting commands cover bold, italic, inline code, links, images, headings, lists, task lists, blockquotes, fenced code blocks, and source Markdown tables.
- Heading commands expose H1–H5 by default, with an H1–H6 option in Preferences.
- Find and replace supports case sensitivity, regular expressions, next/previous navigation, replace current, and replace all.
- Source table commands can format tables and add, delete, or move rows and columns. Visual Edit tables additionally provide direct source-backed cell editing, Tab traversal, deterministic width reflow, and the same row/column operations; ordinary preview tables remain read-only.
- A local image-resource workflow ingests clipboard images and dragged image files: Markion copies or encodes them into a document-relative asset directory with collision-resistant names, inserts portable relative Markdown links, replaces an existing image without losing its alt text or presentation metadata, exposes practical size/alignment controls, and shows an explicit missing-resource state.
- YAML front matter is parsed and hidden from preview; `title`, `author`, and `date` feed export metadata.
- Document writes are same-directory atomic replacements that preserve the existing path and dirty state when a write fails. Markion tracks the last known on-disk file identity, detects external changes before save and while a document is open, automatically reloads only clean documents, and gives dirty documents an explicit reload, overwrite, or save-copy conflict choice. A recovery manager inventories every recovery snapshot with its original path and disk relationship and supports Restore, Discard, Restore All, and Discard All without deleting unreadable or unselected data.
- Auto-save defaults to a five-second inactivity delay and writes recovery copies for unsaved documents; restored recovery snapshots stay durable until a successful save, explicit discard, or an atomically written successor supersedes them.

Rendered preview supports:

- Bold, italic, strikethrough, inline code, links, highlights, superscript, subscript, footnotes, task lists, common emoji shortcodes, and automatic links.
- Correct ordered-list start numbers, nested lists, per-depth bullets, hanging indentation, images, and embedded HTML.
- Selectable preview text with a context menu for copying as plain text, Markdown, or HTML, plus link-address copying where applicable.
- `$...$` inline math and `$$...$$` block math with simple validation and a readable Unicode fallback.
- Syntax-highlighted fenced code using syntect and the two-face extended grammar set, with a fallback lexer and optional line numbers.

## Themes, languages, and preferences

- Fourteen built-in themes: Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light.
- Custom themes use `.toml` files in Markion's local themes directory. On first use a `typewriter.toml` sample — including the optional `[fonts]` table (`editor`, `rendered`, `code`) that supplies font families for the Markdown source editor, rendered body text, and code surfaces whenever the user has no explicit preference — is installed there as a starting point. Legacy `.theme` files migrate automatically when first loaded.
- Six interface languages: English, Simplified Chinese, Japanese, French, German, and Spanish.
- The in-app Preferences panel covers theme, language, sidebar visibility, Preview adaptive width, focus/typewriter modes, code line numbers, Sync scroll, show-hidden-files, heading-menu depth, and per-plane font families (source, reading, code) with a follow-theme default.
- Preferences persist in `config.toml`; legacy `preferences.conf` files migrate automatically.

All configuration fields are optional. The main defaults and file-only settings are:

```toml
theme = "Paper"
language = "en"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 or 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" or "outline"
show_hidden_files = false

# Optional font families per plane; absent = follow the theme, then the
# built-in default (system UI font for source/reading, JetBrains Mono for code).
# editor_font_family = "Cascadia Code"
# rendered_font_family = "Georgia"
# code_font_family = "JetBrains Mono"

[auto_save]
enabled = true
delay_secs = 5

[export]
pdf_engine = "xelatex"
```

Configuration, recovery files, themes, and rotating diagnostic logs use platform-appropriate Markion data directories. Set `RUST_LOG=debug` before launch for more detailed logs.

## Export

Markion exports to:

- Markdown
- Styled HTML and plain HTML
- LaTeX
- DOCX
- PDF
- PNG and JPEG text snapshots

PDF and DOCX try the absorbed Typune/pandoc export engine first. If pandoc or the selected PDF engine is unavailable, Markion falls back to a simpler built-in writer and reports the backend in the status bar. Installing pandoc and a suitable PDF engine produces richer output. PNG/JPEG and built-in PDF output are intentionally basic text snapshots.

## Performance

- Preview blocks, Visual Edit blocks, outline, statistics, and line counts are cached per document version and shared via `Arc`.
- Syntax highlighting is memoized across edits, and grammar loading is warmed in the background.
- Undo snapshots skip derived caches, while the editor reuses a cached text handle per version.
- Preview/Visual Edit lists update changed ranges, the file tree renders a bounded row set, and wrapped source lines measure their actual rendered height.

Source-mapped Visual Edit incrementally reuses independently parseable regions after localized edits and falls back to a full derivation whenever Markdown context or byte ranges are uncertain. Split/Read preview derivation remains debounced and cached. Markion still uses a `String` buffer rather than a rope, and some semantic reads intentionally require a full parse.

## Current limitations

- Visual Edit is WYSIWYG-first while retaining canonical Markdown; constructs without a proven byte-exact rendering expose exact source as a transitional affordance (tracked on the WYSIWYG coverage roadmap) rather than accepting a guessed rich-tree mutation, and block reordering is offered only when non-overlapping source boundaries are provable.
- Math uses a readable fallback rather than KaTeX/MathJax-quality typesetting.
- Visual Edit table cells support direct plain-text editing, but do not yet provide rich inline-formatting controls inside cells. Reference/multiline images, malformed tables, and decoded HTML entities in prose remain known WYSIWYG coverage roadmap gaps that keep source-backed editing paths.
- One-click updates install the Windows NSIS distribution after Minisign verification; macOS bundle replacement and Linux `.deb`/AppImage self-replacement remain future work, and updater authentication is not Windows Authenticode or Apple notarization.
- Drag-and-drop file-tree moves and a full custom-theme installation UI are not implemented.
- Image export is a basic text snapshot, and very large documents do not yet use a rope or fully incremental parsing across every derived subsystem.

## Development

Rust stable is required. From the repository root:

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

The quality command checks Rust formatting, the full Cargo workspace test suite, and every OpenSpec artifact in strict mode. See the [Visual Edit support and engineering contract](docs/visual-editing-quality.md) for the current WYSIWYG coverage matrix and roadmap, source-range invariants, parser ownership, and required evidence.

The root package is the `markion` application crate. Typune-derived, GPUI-free library crates live under `crates/*`:

```powershell
cargo test -p markdown
cargo test -p export
cargo test --workspace
```

Plain `cargo test` tests only the root package; use `cargo test --workspace` for every member. On Windows the app is a GUI-subsystem executable and can also be launched after a debug build with:

```powershell
.\target\debug\markion.exe
```

## License

Markion is available under the [MIT License](LICENSE).
