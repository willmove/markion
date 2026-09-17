<a id="english"></a>

<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">
</p>

<p align="center">
  <strong>English</strong> · <a href="#simplified-chinese">简体中文</a>
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
- **Visual Edit** — a WYSIWYG-first, source-backed surface. Prose stays rendered with progressive syntax reveal; ordinary fenced-code payloads, block math, registered diagrams (Mermaid), inline image fields, and GFM table cells have exact direct editors. Constructs whose WYSIWYG rendering is not yet implemented — YAML front matter, indented code, and malformed or byte-ambiguous syntax — keep an exact source-backed editing affordance as a transitional measure, tracked in the [Visual Edit WYSIWYG coverage matrix and roadmap](docs/visual-editing-quality.md). A slash-command palette and a compact right-click block context menu offer exact block transforms (paragraph, headings, lists, quote, fenced code, divider, table), duplicate, move up/down, source-safe drag reorder, and delete; a selection-contextual formatting toolbar and a visual link editor perform one exact source-backed mutation per action. Markdown delimiters auto-pair by default, `:shortcode` opens emoji completion, task-list boxes can be toggled directly, and leaving a heading, list, task, or quote row returns it immediately to its rendered form. This is not a separate rich-text document model—the underlying Markdown is always the source of truth.
- **Split Preview** — source and rendered preview side by side, with an optional source-mapped Sync scroll setting that keeps both panes on the same document location rather than scrolling by whole-document percentage.
- **Read** — a rendered, non-editing view centered at a readable 860 px maximum width by default; Preview adaptive width can use the full pane.

Switching modes preserves the active document, cursor and selection, undo history, and per-tab scroll state.

The **View → Source/Split Preview** command (`Ctrl+/` on Windows/Linux, `Cmd+/` on macOS) alternates the two source-oriented layouts. From Visual Edit or Read, its first invocation enters Edit.

## Documents and workspace

- Multi-tab editing with per-tab cursor, selection, scroll, undo/redo, preview, outline, and cached derived Markdown state.
- Opening an already-open Markdown or plain-text file focuses its existing tab instead of creating a duplicate.
- **Open Folder** changes the workspace root and populates the Files sidebar with Markdown files, a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`/`.asciidoc`), and supported image files (`.png`, `.jpg`/`.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`/`.tiff`, `.svg`), nested under their folders; empty folders are listed too. Markdown stays visually distinguished, plain-text files open as UTF-8 text, and image files open as read-only image tabs that fit oversized images within the content area.
- Registered PDF files remain visible in the workspace without increasing the core installer by a bundled renderer. Opening one offers the separately downloaded, signed **Markion PDF Viewer** in **Preferences → Plugins**; installation is always explicit and supplies continuous multi-page scrolling, zoom, and page navigation without restarting the app. See [Optional first-party plugins](docs/plugins.md).
- Expanding a folder reveals exactly one level of children, so deeply nested workspaces can be drilled into one level at a time.
- A **Show hidden files/folders** preference (default off) reveals dotfile entries plus the Windows hidden-attribute flag, while always-excluded build, dependency, and VCS noise (`target`, `node_modules`, `.git`, …) stays hidden regardless.
- Right-click file-tree menus provide open, open in new tab, create file/folder, rename, delete, reveal in the system file manager, filter, and refresh actions where applicable.
- Files and folders can be named inline; deleting a non-empty folder requires an additional confirmation.
- Markdown files can be dragged from the operating-system file manager into Markion.
- The Files and Outline panels are toggleable, and sidebar and split-pane dividers are draggable.
- The Outline panel lists the document's heading hierarchy as a collapsible tree: each heading with descendants exposes a disclosure control, outlines start fully expanded, folding is per-document and session-only, and the section containing the cursor is highlighted. Clicking a heading jumps to its source position—or to the rendered heading in Read mode.
- The native window title shows the active file name after the Markion brand, with a `*` suffix when the document has unsaved changes. The status bar keeps save state and transient operation feedback, plus a compact persistent context: the active document's character and word count, the caret's one-based line and column when an editing surface is present, and the current Git branch when the document or workspace belongs to a repository.
- **Backup and Sync** keeps a notes folder protected at a sync location with a one-click **Sync Now** flow and plain-language status. Git powers the safe version history underneath; technical controls stay in Advanced Git details. See the [Backup and Sync guide](docs/git-sync.md) for setup, authentication, conflicts, and recovery.

## Markdown editing and preview

- Parsing is powered by `pulldown-cmark` with CommonMark and GFM-oriented support.
- Formatting commands cover paragraph (`Ctrl+0` on Windows/Linux, `Cmd+0` on macOS), headings, bold, italic, inline code, links, images, lists, task lists, blockquotes, fenced code blocks, and source Markdown tables. Paragraph converts intersected ATX headings back to ordinary text without changing non-heading lines.
- Markdown auto-pairing is enabled by default in source and supported Visual Edit fields: delimiters can be paired, selections wrapped, existing closers skipped, and empty pairs removed with Backspace. It stays disabled during IME composition and inside code, block-math, and diagram payload editors.
- Typing an open `:shortcode` at a word boundary opens emoji completion in source and Visual Edit; confirming inserts canonical Markdown that remains one undoable source edit.
- Pasting rich text copied from Word, Google Docs, or web pages converts the clipboard's HTML flavor into formatted Markdown — headings, emphasis, links, nested lists, tables, code — as a single undoable step; plain-text clipboards still paste verbatim.
- Heading commands expose H1–H5 by default, with an H1–H6 option in Preferences.
- Find and replace supports case sensitivity, regular expressions, next/previous navigation, replace current, and replace all.
- Source table commands can format tables and add, delete, or move rows and columns. Visual Edit tables additionally provide direct source-backed cell editing, Tab traversal, deterministic width reflow, and the same row/column operations; ordinary preview tables remain read-only.
- An image-resource workflow ingests clipboard images, dragged files, explicit file/URL insertion, and existing document images. Policies can keep, copy/download, or upload through PicGo HTTP, PicGo Core, or a literal custom command; see the [image handling and upload guide](docs/image-handling.md). Markion uses collision-resistant document-relative resources, preserves exact image metadata, and keeps transfer recovery outside Markdown.
- YAML front matter is parsed and hidden from preview; `title`, `author`, and `date` feed export metadata.
- Document writes are same-directory atomic replacements that preserve the existing path and dirty state when a write fails. Markion tracks the last known on-disk file identity, detects external changes before save and while a document is open, automatically reloads only clean documents, and gives dirty documents an explicit reload, overwrite, or save-copy conflict choice. A recovery manager inventories every recovery snapshot with its original path and disk relationship and supports Restore, Discard, Restore All, and Discard All without deleting unreadable or unselected data.
- Auto-save defaults to a five-second inactivity delay and writes recovery copies for unsaved documents; restored recovery snapshots stay durable until a successful save, explicit discard, or an atomically written successor supersedes them.

Rendered preview supports:

- Bold, italic, strikethrough, inline code, links, highlights, superscript, subscript, footnotes (hover a reference to read its definition), task lists, common emoji shortcodes, and automatic links.
- In-document `[TOC]` / `[toc]` tokens rendered as a live, clickable outline, and `[text](#heading)` / `{#id}` heading jumps that stay inside the document.
- Correct ordered-list start numbers, nested lists, per-depth bullets, hanging indentation, images, and embedded HTML.
- Supported inline HTML has consistent semantics across mixed Markdown, standalone HTML blocks, table cells, and Visual Edit, including safe color spans, links and linked images, `kbd`/`samp`, authored breaks, and positioned superscript/subscript; malformed or unsupported markup falls back without executing scripts.
- Selectable preview text with a context menu for copying as plain text, Markdown, or HTML, plus link-address copying where applicable.
- `$...$` inline math and `$$...$$` block math, typeset offline into cached SVG by an embedded RaTeX (KaTeX-compatible) engine with bundled fonts — no network access or external LaTeX install required.
- ` ```mermaid ` fenced diagrams (flowcharts, sequence diagrams, and more) rendered to sanitized, cached SVG.
- Syntax-highlighted fenced code using syntect and the two-face extended grammar set, with a fallback lexer and optional line numbers.

## Themes, languages, and preferences

- Fourteen built-in themes: Paper, Ink, Solar, Forest, Rose, Graphite, GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light.
- Custom themes use `.toml` files in Markion's local themes directory. On first use a `typewriter.toml` sample — including the optional `[fonts]` table (`editor`, `rendered`, `code`) that supplies font families for the Markdown source editor, rendered body text, and code surfaces whenever the user has no explicit preference — is installed there as a starting point. Legacy `.theme` files migrate automatically when first loaded.
- Seven interface languages: English, Simplified Chinese, Traditional Chinese, Japanese, French, German, and Spanish.
- The in-app Preferences panel covers language, sidebar visibility, Preview adaptive width, focus/typewriter modes, Markdown auto-pairing, code line numbers, Sync scroll, show-hidden-files, and heading-menu depth on **General**; theme, per-plane font families (source, reading, code), font sizes, and paragraph spacing on **Appearance**; and signed first-party plugin installation, updates, rollback, storage, and removal on **Plugins**.
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
markdown_auto_pair = true

# Optional font families per plane; absent = follow the theme, then the
# built-in default (system UI font for source/reading, JetBrains Mono for code).
# editor_font_family = "Cascadia Code"
# rendered_font_family = "Georgia"
# code_font_family = "JetBrains Mono"

[auto_save]
enabled = true
delay_secs = 5

[git]
background_check = false

[export]
pdf_engine = "xelatex"
```

Configuration, recovery files, themes, and rotating diagnostic logs use platform-appropriate Markion data directories. Set `RUST_LOG=debug` before launch for more detailed logs.

## Export

Markion exports to:

- A local MarkNice workspace for preparing rich content for WeChat
- Markdown
- Styled HTML and plain HTML
- LaTeX
- DOCX
- PDF
- PNG and JPEG text snapshots

PDF and DOCX try the absorbed Typune/pandoc export engine first. If pandoc or the selected PDF engine is unavailable, Markion falls back to a simpler built-in writer and reports the backend in the status bar. Installing pandoc and a suitable PDF engine produces richer output. PNG/JPEG and built-in PDF output are intentionally basic text snapshots.

## Word import

Choose **File → Import Word (.docx)** to convert one DOCX locally into a new
saved Markdown document. Markion shows a conversion report before the save
picker and requires an explicit choice when content may be lost. Supported
embedded images are saved beside the Markdown in `<stem>.assets/`; keep that
directory with the `.md` file. The importer is bounded, cancellable, offline,
and never overwrites an existing Markdown or asset path. It uses the accepted
view of tracked changes, preserves Word heading levels more faithfully, and
normalizes bold-to-word boundaries for interoperable Markdown. Legacy `.doc`, `.docm`, encrypted files, PDF/OCR and
page-layout fidelity are outside this workflow. See
[`docs/word-import.md`](docs/word-import.md) for the supported semantic subset,
limits, diagnostics and cleanup behavior.

Choose **Export → Publish for WeChat (MarkNice)** to open the active in-memory
document in a private loopback workspace in your default browser. The bundled
editor skin tracks the pinned MarkNice editor section (dual cards, traffic-light
headers, SVG toolbar, 375px phone frame) as closely as a local, non-marketing
shell allows. Themes, renderer, math typesetting, and application scripts work
offline. Formulas are typeset as self-contained inline SVG (MathJax), so they
survive the WeChat editor's paste-time filtering without duplication. Browser
edits stay in that tab and never write back to Markion. Its **Import Word** converts
a `.docx` file only inside that browser session; recover it into Markion with
**Copy MD** or by saving the session Markdown elsewhere. Managed local images
can be previewed, but copying requires explicitly omitting them because a local
blob cannot be published by WeChat; remote images may still contact their
authored hosts. The workspace can also copy its exact current Markdown, save
standalone themed HTML, save a browser-generated MarkNice Word file, or **Save
as PDF** by printing the current themed sanitized preview (choose “Save as PDF”
in the print dialog). Browser Word and PDF are distinct from Markion's
native/Pandoc exporters. The workspace does not import Markdown files, PDFs, or
local images, and it has no sample-document action. If clipboard permission or
the two-hour session expires, allow the permission or relaunch from Markion.

## Performance

- Preview blocks, Visual Edit blocks, outline, statistics, and line counts are cached per document version and shared via `Arc`.
- Syntax highlighting is memoized across edits, and grammar loading is warmed in the background.
- Undo snapshots skip derived caches, while the editor reuses a cached text handle per version.
- Preview/Visual Edit lists update changed ranges, the file tree renders a bounded row set, and wrapped source lines measure their actual rendered height.

Source-mapped Visual Edit incrementally reuses independently parseable regions after localized edits and falls back to a full derivation whenever Markdown context or byte ranges are uncertain. Split/Read preview derivation remains debounced and cached. Markion still uses a `String` buffer rather than a rope, and some semantic reads intentionally require a full parse.

## Current limitations

- Visual Edit is WYSIWYG-first while retaining canonical Markdown; constructs without a proven byte-exact rendering expose exact source as a transitional affordance (tracked on the [WYSIWYG coverage roadmap](docs/visual-editing-quality.md)) rather than accepting a guessed rich-tree mutation, and block reordering is offered only when non-overlapping source boundaries are provable. Current primary gaps include decoded HTML entities, front matter, and indented code blocks; secondary malformed or unsupported constructs remain listed in the matrix.
- On-screen rendering (Split/Read preview and Visual Edit) typesets math with the embedded RaTeX engine; LaTeX export keeps native `$...$`/`$$...$$` source for the reader's own toolchain, and the built-in DOCX export fallback (used only when pandoc is unavailable) still degrades formulas to a readable plain-text approximation rather than embedding typeset glyphs.
- Visual Edit table cells support direct editing plus Bold/Italic/Inline Code/Link on a selection that stays inside one cell. GFM column widths can be dragged and persisted as an HTML comment (`<!-- markion-cols:… -->`) immediately before the table. Inline images accept integer widths from 10–100% and can be drag-resized. Reference/multiline images and malformed tables remain known WYSIWYG coverage roadmap gaps that keep source-backed editing paths.
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

PDFium is not discovered by the core application. The optional PDF worker and
its pinned runtime are composed and verified as a signed target package by the
native plugin workflows; see [the plugin architecture](docs/plugin-platform.md).

The quality command checks Rust formatting and lints, the full Cargo workspace
test suite, the pinned MarkNice bundle, and every OpenSpec artifact in strict
mode. See the [local MarkNice workspace guide](docs/marknice-workspace.md) and
the [Visual Edit support and engineering contract](docs/visual-editing-quality.md).

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

---

<a id="simplified-chinese"></a>

<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion 标志" width="128" height="128">
</p>

<p align="center">
  <a href="README.md#english">English</a> · <strong>简体中文</strong> · <a href="README.md#simplified-chinese">合并版</a>
</p>

# Markion

Markion 是一款使用 Rust 和 GPUI 构建的原生桌面 Markdown 编辑器。它在一个轻量应用中提供流畅的源码编辑、基于源码的可视化编辑、实时预览、工作区工具与多格式导出。Markdown 始终是文档的标准数据格式——不使用 Electron、Tauri 或 WebView。

## 安装

请从 [GitHub Releases](https://github.com/willmove/markion/releases) 下载最新版本。

| 平台 | 发布包 | 目标架构 |
|---|---|---|
| Windows | NSIS `.exe` 安装程序 | x86_64 |
| Linux | `.deb` 和 AppImage | x86_64 |
| macOS | `.app` 和 `.dmg` | Apple Silicon（arm64），macOS 11+ |

当前发布版本尚未进行平台代码签名。Windows SmartScreen 可能要求选择“更多信息 → 仍要运行”，macOS Gatekeeper 可能要求右键应用并选择“打开”。“帮助 → 检查更新…”在所有平台都提供可操作的更新提示：带标签的 Windows x86_64 NSIS 安装可进行经过 cargo-packager Minisign 加密验证的一键下载并安装，且当任一文档存在未保存更改时拒绝启动；macOS 与 Linux 会在系统浏览器中打开对应的发布文件。Intel Mac 可通过 Rosetta 运行 arm64 版本；目前尚不提供通用二进制和 Apple 公证。

## 编辑模式

Markion 提供四种视图模式，默认使用分栏预览。

- **编辑** — 专注的原始 Markdown 源码编辑器。
- **可视化编辑** — 所见即所得优先、基于源码的编辑界面。正文保持渲染并渐进显示必要语法；普通围栏代码正文、块级数学公式、已注册的图表（Mermaid）、行内图片字段与 GFM 表格单元格提供精确的直接编辑器。尚未实现所见即所得渲染的结构——YAML 前言、缩进代码及格式错误或字节映射不明确的语法——暂时保留精确的源码编辑通道，并登记在[可视化编辑 WYSIWYG 覆盖矩阵与路线图](docs/visual-editing-quality.md)中等待逐步关闭。斜杠命令面板与紧凑的右键块级菜单提供精确的块级转换（段落、标题、列表、引用、围栏代码、分隔线、表格）、复制、上移/下移、源码安全的拖拽重排与删除；选区上下文格式工具栏与可视化链接编辑器对每次操作执行一次精确的源码变更。Markdown 分隔符默认自动配对，输入 `:shortcode` 会打开 emoji 补全，任务列表复选框可直接点击切换；光标离开标题、列表、任务或引用行后，该行会立即恢复为渲染形态。它不是独立的富文本文档模型——底层 Markdown 始终是唯一数据源。
- **分栏预览** — 左右同时显示源码和渲染预览，可选择启用同步滚动：基于源码映射按文档位置对齐两个面板，而非按整篇滚动百分比同步。
- **阅读** — 不可编辑的渲染视图，默认居中并限制为适合阅读的 860 px 最大宽度；启用“预览自适应宽度”后可使用整个面板。

切换模式会保留当前文档、光标和选区、撤销历史以及每个标签页的滚动状态。

使用**视图 → 源码/分栏预览**（Windows/Linux 为 `Ctrl+/`，macOS 为 `Cmd+/`）可在两种源码导向布局之间切换；从可视化编辑或阅读模式首次执行时会进入编辑模式。

## 文档与工作区

- 多标签页编辑，每个标签页分别保存光标、选区、滚动、撤销/重做、预览、大纲和派生 Markdown 缓存状态。
- 再次打开已打开的 Markdown 或纯文本文件时会聚焦现有标签页，而不会创建重复标签页。
- **打开文件夹**可切换工作区根目录，并在“文件”侧边栏中显示 Markdown 文件、精选的纯文本文件（`.txt`、`.text`、`.log`、`.csv`、`.tsv`、`.org`、`.rst`、`.adoc`/`.asciidoc`）以及受支持的图片文件（`.png`、`.jpg`/`.jpeg`、`.gif`、`.webp`、`.bmp`、`.tif`/`.tiff`、`.svg`），按目录层级嵌套显示；空文件夹也会列出。Markdown 文件保持醒目区分，纯文本文件以 UTF-8 文本打开，图片文件以只读图片标签页打开，并将超尺寸图片适配到内容区域内显示。
- 展开文件夹时只显示下一级的子项，可逐级深入查看深层嵌套的工作区。
- **显示隐藏文件/文件夹**偏好设置（默认关闭）可显示点号开头（dotfile）以及 Windows 隐藏属性的条目；而始终排除的构建、依赖和 VCS 噪声目录（`target`、`node_modules`、`.git` 等）在任何情况下都保持隐藏。
- 文件树右键菜单会根据目标提供打开、在新标签页打开、新建文件/文件夹、重命名、删除、在系统文件管理器中显示、筛选和刷新等操作。
- 文件和文件夹可就地命名；删除非空文件夹需要二次确认。
- 可从操作系统文件管理器将 Markdown 文件拖入 Markion。
- “文件”和“大纲”面板均可切换显示，侧边栏与分栏预览的分隔线可拖动调整。
- “大纲”面板以可折叠树形结构列出文档标题层级：含有下级标题的标题会显示展开/折叠控件，新打开的文档大纲默认完全展开，折叠状态按文档独立、仅作用于当前会话，并会高亮显示光标所在章节。点击标题会跳转到对应源码位置——在阅读模式下则跳转到对应的渲染标题。
- 原生窗口标题在 Markion 品牌文字后显示当前打开的文件名，文档有未保存更改时会带 `*` 后缀。状态栏保留保存状态和瞬时操作反馈，并提供紧凑的持久上下文：当前文档的字符数与字数、在存在编辑界面时显示光标所在的行号与列号（从 1 开始），以及在文档或工作区属于 Git 仓库时显示当前分支名。
- **备份与同步**通过一次点击的“立即同步”和清晰易懂的状态提示，将笔记文件夹安全地保存在同步位置。底层由 Git 提供可靠的版本历史，技术操作则收纳在“高级 Git 详情”中。设置、认证、冲突与恢复说明见[备份与同步指南](docs/git-sync.md)。

## Markdown 编辑与预览

- 使用 `pulldown-cmark` 解析 Markdown，面向 CommonMark 和 GFM。
- 格式化命令支持段落（Windows/Linux 为 `Ctrl+0`，macOS 为 `Cmd+0`）、标题、粗体、斜体、行内代码、链接、图片、列表、任务列表、引用、围栏代码块和源码 Markdown 表格。“段落”可将选区内的 ATX 标题恢复为普通文本，并保持非标题行不变。
- 源码编辑器及受支持的可视化编辑字段默认启用 Markdown 自动配对：可补齐分隔符、包裹选区、跳过已有闭合符，并用 Backspace 删除空配对；IME 组合期间以及代码、块级公式和图表正文编辑器内不会触发。
- 在词语边界输入未闭合的 `:shortcode` 会在源码和可视化编辑中打开 emoji 补全；确认后插入标准 Markdown，并保持为一次可撤销的源码编辑。
- 粘贴来自 Word、Google Docs 或网页的富文本时，会将剪贴板中的 HTML 转换为带格式的 Markdown（标题、强调、链接、嵌套列表、表格、代码），整个粘贴可一次撤销；纯文本剪贴板仍按原样逐字粘贴。
- 标题命令默认显示 H1–H5，可在“偏好设置”中扩展为 H1–H6。
- 查找与替换支持区分大小写、正则表达式、上一个/下一个匹配、替换当前项和全部替换。
- 源码表格命令可格式化表格并新增、删除或移动行列。可视化编辑中的表格还支持直接编辑单元格、使用 Tab 遍历、确定性宽度重排，以及同样的源码驱动行列操作；普通预览表格保持只读。
- 图片资源工作流支持剪贴板、拖入文件、显式文件／URL 插入以及已有文档图片。可按来源选择保留、复制／下载，或通过 PicGo HTTP、PicGo Core、字面参数自定义命令上传；详见[图片处理与上传指南](docs/image-handling.md)。Markion 使用抗冲突的文档相对资源，精确保留图片元数据，并将传输恢复数据置于 Markdown 之外。
- 可解析 YAML 前言并在预览中隐藏；其中的 `title`、`author` 和 `date` 会用于导出元数据。
- 文档写入采用同目录的原子替换，写入失败时保留文档路径和脏状态。Markion 会跟踪最近已知的磁盘文件标识，在保存前及文档打开期间检测外部更改，仅自动重新加载干净文档，并为脏文档提供重新加载、覆盖或另存为副本的冲突选择。恢复管理器会列出每个恢复快照及其原始路径和磁盘关系，支持还原、丢弃、全部还原与全部丢弃，且不会删除无法读取或未选中的恢复数据。
- 自动保存默认在停止输入五秒后执行，并为未保存文档写入恢复副本；已还原的恢复快照会保持持久，直到成功保存、显式丢弃或被原子写入的后继恢复所取代。

渲染预览支持：

- 粗体、斜体、删除线、行内代码、链接、高亮、上标、下标、脚注、任务列表、常用 emoji 短代码和自动链接。
- 正确的有序列表起始编号、嵌套列表、分层项目符号、悬挂缩进、图片和嵌入式 HTML。
- 受支持的行内 HTML 在混合 Markdown、独立 HTML 块、表格单元格和可视化编辑中保持一致语义，包括安全的颜色 span、链接与链接图片、`kbd`/`samp`、原样换行以及正确定位的上下标；畸形或不支持的标记会安全回退，不会执行脚本。
- 可选择预览文本，并通过右键菜单复制为纯文本、Markdown 或 HTML；在适用位置还可复制链接地址。
- `$...$` 行内公式和 `$$...$$` 块级公式，由内嵌的 RaTeX（兼容 KaTeX）引擎离线排版为带内嵌字体的 SVG 并缓存——无需联网，也无需安装外部 LaTeX。
- ` ```mermaid ` 围栏图表（流程图、时序图等）渲染为经过消毒处理的 SVG 并缓存。
- 使用 syntect 和 two-face 扩展语法集高亮围栏代码，对未覆盖语言使用后备词法分析，并可显示行号。

## 主题、语言与偏好设置

- 十四款内置主题：Paper、Ink、Solar、Forest、Rose、Graphite、GitHub Light/Dark、Solarized Light/Dark、One Light/Dark 和 Tokyo Night/Light。
- 自定义主题使用 Markion 本地主题目录中的 `.toml` 文件；首次使用时会安装 `typewriter.toml` 示范文件(含可选的 `[fonts]` 表——`editor`、`rendered`、`code`,在用户没有显式偏好时为源码编辑器、正文和代码平面提供字体)。旧版 `.theme` 文件会在首次加载时自动迁移。
- 七种界面语言：英语、简体中文、繁體中文、日语、法语、德语和西班牙语。
- 应用内偏好设置面板在**通用**中可配置语言、侧边栏显示、预览自适应宽度、专注/打字机模式、Markdown 自动配对、代码行号、同步滚动、显示隐藏文件和标题菜单层级；在**外观**中可配置主题、按平面的字体（源码/阅读/代码，默认跟随主题）、字号和段落间距。
- 偏好设置保存在 `config.toml`；旧版 `preferences.conf` 会自动迁移。

所有配置字段均可省略。主要默认值和仅能通过文件配置的选项如下：

```toml
theme = "Paper"
language = "en"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 或 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" 或 "outline"
show_hidden_files = false
markdown_auto_pair = true

# 可选的按平面字体;缺省时先跟随主题,再回退到内置默认
# (源码/正文为系统 UI 字体,代码为 JetBrains Mono)。
# editor_font_family = "Cascadia Code"
# rendered_font_family = "Georgia"
# code_font_family = "JetBrains Mono"

[auto_save]
enabled = true
delay_secs = 5

[git]
background_check = false

[export]
pdf_engine = "xelatex"
```

配置、恢复文件、主题和按日轮换的诊断日志均使用适合各平台的 Markion 数据目录。启动前设置 `RUST_LOG=debug` 可获得更详细的日志。

## 导出

Markion 可导出为：

- 用于准备微信富文本内容的本地 MarkNice 工作区
- Markdown
- 带样式 HTML 和纯 HTML
- LaTeX
- DOCX
- PDF
- PNG 和 JPEG 文本快照

PDF 和 DOCX 会优先尝试已整合的 Typune/pandoc 导出引擎。如果 pandoc 或选定的 PDF 引擎不可用，Markion 会回退到较简单的内置写入器，并在状态栏中说明所用后端。安装 pandoc 和合适的 PDF 引擎可获得更丰富的输出。PNG/JPEG 和内置 PDF 输出有意保持为基础文本快照。

## Word 导入

选择 **文件 → 导入 Word（.docx）**，可以完全在本地把一份 DOCX 转换为新的已保存 Markdown 文档。Markion 会先显示转换报告，再打开保存对话框；如果可能丢失内容，必须明确选择继续。支持的内嵌图片保存在 Markdown 旁的 `<文件名>.assets/` 中，移动或复制时请让它与 `.md` 文件同行。导入过程有资源上限、可取消、无需联网，而且不会覆盖已有 Markdown 或资源目录。修订按“接受后的视图”导入；Word 标题层级会得到更准确的保留，粗体与后续文字的边界会规范化为兼容性更好的 Markdown。旧版 `.doc`、`.docm`、加密文件、PDF/OCR 和页面版式复刻不在此流程范围内。支持的语义子集、限制、诊断和清理行为详见 [`docs/word-import.md`](docs/word-import.md)。

选择 **导出 → 发布到微信公众号（MarkNice）** 可在默认浏览器中将当前内存文档打开到一个私有的本地回环工作区。内置编辑器外观尽量贴近固定版本的 MarkNice 编辑区（双卡片、红绿灯标题栏、SVG 工具栏、375px 手机框），而不会带入营销站点。主题、渲染器、数学公式排版和应用脚本全部离线可用；公式以自包含的内联 SVG（MathJax）渲染，粘贴到公众号草稿时不会被清洗破坏，也不会出现重复。浏览器中的编辑仅保留在该标签页内，永远不会写回 Markion。浏览器工作区里的 **导入 Word** 只在当前浏览器会话中把 `.docx` 转成 Markdown；若要带回 Markion，请使用 **复制 MD** 或自行另存会话中的 Markdown。托管的本地图可以预览，但复制时会显式排除它们，因为本地 blob 无法发布到微信；远程图片仍可能访问其原始主机。工作区还可以复制当前 Markdown、将主题化预览存为 HTML、存为浏览器生成的 Word，或通过打印对话框 **存为 PDF**（打印的是当前主题化、已消毒的 MarkNice 预览）。浏览器 Word/PDF 与 Markion 原生/Pandoc 导出不同。工作区不提供导入 Markdown、导入 PDF、导入本地图片或示例文档。如果剪贴板权限或两小时会话过期，请重新授权或从 Markion 重新启动。

## 性能

- 预览块、可视化编辑块、大纲、统计信息和行数均按文档版本缓存，并通过 `Arc` 共享。
- 语法高亮会跨编辑复用，语法库在后台预热。
- 撤销快照不包含派生缓存；编辑器按版本复用缓存的文本句柄。
- 预览/可视化编辑列表仅更新变化范围，文件树限制每帧渲染的行数，换行后的源码行会测量实际渲染高度。

局部编辑后，带源码映射的可视化编辑模型会增量复用可独立解析的区域；当 Markdown 上下文或字节范围无法确定时，则回退为完整派生。分栏/阅读预览仍使用防抖与缓存。Markion 仍使用 `String` 而非 rope 文本缓冲区，部分语义读取也会有意执行完整解析。

## 当前限制

- 可视化编辑以所见即所得为默认呈现契约，同时保留标准 Markdown；暂无字节精确渲染证明的结构会以源码作为过渡编辑通道（登记在 [WYSIWYG 覆盖路线图](docs/visual-editing-quality.md)中），而不会猜测富文本树变更；仅当可证明存在不重叠的源码边界时才提供块级重排。目前的主要缺口包括已解码 HTML 实体、前言与缩进代码块；其他畸形或尚未支持的结构列在矩阵的次级缺口中。
- 屏幕渲染（分栏/阅读预览与可视化编辑）使用内嵌的 RaTeX 引擎排版数学公式；LaTeX 导出保留原生 `$...$`/`$$...$$` 源码交给读者自己的工具链处理，内置 DOCX 导出后备通道（仅在 pandoc 不可用时使用）仍会将公式降级为可读的纯文本近似显示，而非嵌入排版好的字形。
- 可视化表格单元格支持直接纯文本编辑，但尚未提供单元格内的富行内格式控件。引用式/多行图片和畸形表格仍是 WYSIWYG 覆盖路线图上的已知缺口，暂时保留源码驱动编辑路径。
- 一键更新在完成 Minisign 验证后会安装 Windows NSIS 版本；macOS 包替换与 Linux `.deb`/AppImage 自替换仍是后续工作，且更新身份验证并非 Windows Authenticode 或 Apple 公证。
- 尚未实现文件树拖放移动和完整的自定义主题安装界面。
- 图片导出是基础文本快照，超大文档尚未在所有派生子系统中使用 rope 或完全增量解析。

## 开发

需要 Rust stable。在仓库根目录运行：

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

质量命令会检查 Rust 格式与代码检查（lint）、完整 Cargo workspace 测试套件、固定的 MarkNice bundle，以及严格模式下的所有 OpenSpec 工件。另请参阅[本地 MarkNice 工作区指南](docs/marknice-workspace.md)与[可视化编辑支持与工程契约](docs/visual-editing-quality.md)。

根包是 `markion` 应用 crate。源自 Typune 且不依赖 GPUI 的库 crate 位于 `crates/*`：

```powershell
cargo test -p markdown
cargo test -p export
cargo test --workspace
```

普通 `cargo test` 只测试根包；使用 `cargo test --workspace` 可测试全部成员。在 Windows 上，应用构建为 GUI 子系统可执行文件；完成调试构建后还可直接运行：

```powershell
.\target\debug\markion.exe
```

## 许可证

Markion 使用 [MIT 许可证](LICENSE)。
