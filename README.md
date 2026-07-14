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

Releases are currently unsigned. Windows SmartScreen may require **More info → Run anyway**, and macOS Gatekeeper may require right-clicking the app and choosing **Open**. Intel Macs can run the arm64 build through Rosetta; a universal binary, notarization, and automatic updates are not currently provided.

## Editing modes

Markion has four view modes. Split Preview is the default.

- **Edit** — a focused raw Markdown source editor.
- **Visual Edit** — a single, source-backed live-preview surface for headings, paragraphs, emphasis, links, images, blockquotes, lists, task lists, rules, and tables. Focused syntax can be exposed for precise editing; code, math, HTML, front matter, and ambiguous constructs use conservative source islands. This is not a separate rich-text document model—the underlying Markdown is always the source of truth.
- **Split Preview** — source and rendered preview side by side, with an optional proportional Sync scroll setting.
- **Read** — a rendered, non-editing view centered at a readable 860 px maximum width by default; Preview adaptive width can use the full pane.

Switching modes preserves the active document, cursor and selection, undo history, and per-tab scroll state.

## Documents and workspace

- Multi-tab editing with per-tab cursor, selection, scroll, undo/redo, preview, outline, and cached derived Markdown state.
- Opening an already-open Markdown file focuses its existing tab instead of creating a duplicate.
- **Open Folder** changes the workspace root and populates the Files sidebar with Markdown files and folders.
- Right-click file-tree menus provide open, open in new tab, create file/folder, rename, delete, reveal in the system file manager, filter, and refresh actions where applicable.
- Files and folders can be named inline; deleting a non-empty folder requires an additional confirmation.
- Markdown files can be dragged from the operating-system file manager into Markion.
- The Files and Outline panels are toggleable, and sidebar and split-pane dividers are draggable.

## Markdown editing and preview

- Parsing is powered by `pulldown-cmark` with CommonMark and GFM-oriented support.
- Formatting commands cover bold, italic, inline code, links, images, headings, lists, task lists, blockquotes, fenced code blocks, and source Markdown tables.
- Heading commands expose H1–H5 by default, with an H1–H6 option in Preferences.
- Find and replace supports case sensitivity, regular expressions, next/previous navigation, replace current, and replace all.
- Source table commands can format tables and add, delete, or move rows and columns. Visual Edit tables expose the same source-backed row/column operations; ordinary preview tables remain read-only, and direct visual cell editing is not yet supported.
- YAML front matter is parsed and hidden from preview; `title`, `author`, and `date` feed export metadata.
- Auto-save defaults to a five-second inactivity delay and writes recovery copies for unsaved documents.

Rendered preview supports:

- Bold, italic, strikethrough, inline code, links, highlights, superscript, subscript, footnotes, task lists, common emoji shortcodes, and automatic links.
- Correct ordered-list start numbers, nested lists, per-depth bullets, hanging indentation, images, and embedded HTML.
- Selectable preview text with a context menu for copying as plain text, Markdown, or HTML, plus link-address copying where applicable.
- `$...$` inline math and `$$...$$` block math with simple validation and a readable Unicode fallback.
- Syntax-highlighted fenced code using syntect and the two-face extended grammar set, with a fallback lexer and optional line numbers.

## Themes, languages, and preferences

- Fourteen built-in themes: Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light.
- Custom themes use `.toml` files in Markion's local themes directory. Legacy `.theme` files migrate automatically when first loaded.
- Six interface languages: English, Simplified Chinese, Japanese, French, German, and Spanish.
- The in-app Preferences panel covers theme, language, sidebar visibility, Preview adaptive width, focus/typewriter modes, code line numbers, Sync scroll, and heading-menu depth.
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

Markion still performs full Markdown reparses when document text changes; it does not yet use a rope buffer or a fully incremental parser.

## Current limitations

- Visual Edit is an Obsidian-style source-backed surface, not full Typora-style WYSIWYG; complex or ambiguous constructs may expose Markdown source.
- Math uses a readable fallback rather than KaTeX/MathJax-quality typesetting.
- Direct cell-level Visual Edit table editing and inline styling inside preview table cells are not implemented; HTML export preserves table-cell fidelity.
- Drag-and-drop file-tree moves and a full custom-theme installation UI are not implemented.
- Image export is a basic text snapshot, and very large documents do not yet use fully virtualized/incremental parsing.

## Development

Rust stable is required. From the repository root:

```powershell
cargo run
cargo test
cargo build
```

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
