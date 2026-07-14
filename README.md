<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">
</p>

# Markion

Markion is a native Markdown editor built with Rust and GPUI, focusing on responsive source editing, live preview, flexible export options, and a streamlined desktop workflow. No Electron. No Tauri. No WebView.

## Install

Download the latest build from [GitHub Releases](https://github.com/willmove/markion/releases).

Release packages are built by GitHub Actions for:

- Windows x86_64: NSIS installer
- Linux x86_64: `.deb` and AppImage
- macOS Apple Silicon: `.app` and `.dmg`

Builds are currently unsigned, so macOS Gatekeeper and Windows SmartScreen may require a manual trust step on first launch.

The project logo is maintained at `assets/markion-logo.svg`; the generated `markion.png`, `markion.ico`, and `markion.icns` assets provide the Linux, Windows, and macOS platform icons respectively.

## Highlights

- GPUI desktop app with Edit, Split Preview, and Read modes.
- Multi-tab editing with per-tab cursor, scroll, undo, preview, outline, and cached derived Markdown state.
- Reopens an already-open Markdown file by focusing its existing tab instead of creating duplicates.
- File tree sidebar with filtering, Markdown file opening, current-file marking, and file/folder create, rename, refresh, and delete actions.
- Drag and drop Markdown files from the OS file manager into the editor.
- Toggleable file and outline panels, draggable sidebar and split-preview dividers, visible draggable pane scrollbars, and a compact status bar.
- In-window and native menus for File, Edit, View, Format, Export, and Help actions.
- In-app Preferences panel for theme, interface language, sidebar visibility, preview behavior, focus/typewriter modes, code line numbers, sync scroll, and heading-menu depth.
- Six interface languages: English, Simplified Chinese, Japanese, French, German, and Spanish.

## Editing and Preview

- Markdown parsing powered by `pulldown-cmark` with CommonMark/GFM-oriented behavior.
- Formatting actions for bold, italic, inline code, links, images, headings, lists, task lists, blockquotes, fenced code blocks, and source Markdown tables.
- Heading commands expose H1-H5 by default, with an H1-H6 option in Preferences.
- Source table commands can format tables and add, delete, or move rows and columns.
- Preview tables include a compact toolbar for adding, deleting, and moving source table rows or columns.
- Find and replace supports case-sensitive search, regex search, next/previous navigation, replace current, and replace all.
- YAML front matter is parsed; preview hides front matter and HTML export uses title, author, and date metadata.
- Auto-save writes changed files after a configurable inactivity interval and saves recovery copies for unsaved documents.

The preview supports:

- Native inline styles for bold, italic, strikethrough, inline code, links, highlights, superscript, subscript, and footnote references.
- Ordered lists with correct start numbers, nested list structure, task-list checkboxes, per-depth bullets, and hanging indentation.
- Images, embedded HTML, automatic links, footnotes, task lists, common emoji shortcodes, and extended inline syntax.
- Markdown math parsing for `$...$` and `$$...$$`, with readable fallback rendering and simple validation errors.
- Syntax-highlighted code blocks using syntect plus the two-face extended grammar set, with fallback token classification and optional line numbers.
- Optional Sync scroll in Split Preview mode, coupling source and preview panes proportionally without reparsing Markdown.

## Themes and Preferences

- Fourteen built-in themes: Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light.
- Custom `.toml` theme files in Markion's local themes directory extend the theme list; legacy `.theme` files migrate automatically on first load.
- Preferences persist to `config.toml`; legacy `preferences.conf` files migrate automatically.
- Config-only options include auto-save settings and the PDF export engine.

## Export

Markion can export to:

- Markdown
- Styled HTML
- Plain HTML
- LaTeX
- DOCX
- PDF
- PNG/JPEG text snapshots

DOCX and PDF export use the absorbed Typune export engine where available, with built-in fallback paths for simpler outputs.

## Performance Notes

- Derived Markdown state such as preview blocks, outline, stats, and line counts is cached per document version and shared via `Arc`.
- Syntax highlighting is memoized across edits.
- Undo snapshots skip derived caches.
- The editor reuses a cached text handle per version.
- Preview rendering updates changed ranges and the file tree renders a bounded number of rows per frame.
- Wrapped source lines measure their rendered height so long soft-wrapped lines scroll fully and multi-line selections render correctly.

## Not Yet Implemented

- Single-surface WYSIWYG editing that hides Markdown markers around the cursor.
- KaTeX/MathJax-quality math rendering.
- Direct cell-level preview table editing.
- Drag-and-drop file tree moves.
- Full custom theme installation UI.
- Rich image export and virtualized rendering for very large documents.
- Inline styles inside preview table cells; HTML export keeps full table-cell fidelity.

## Development

```powershell
cargo run
cargo test
cargo build
```

The root package is the Markion app crate. Additional Typune-derived library crates live under `crates/*`; use `cargo test -p <member>` for one member or `cargo test --workspace` for every crate.

On Windows the app is built as a GUI subsystem executable, so the editor window is not tied to a console window lifecycle. After `cargo build`, you can also launch:

```powershell
.\target\debug\markion.exe
```

Save prompts for a path the first time a document is saved. Export actions prompt for an output path and suggest a filename based on the current document.
