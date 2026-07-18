<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion 标志" width="128" height="128">
</p>

<p align="center">
  <a href="README.md">English</a> · <strong>简体中文</strong>
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

当前发布版本尚未签名。Windows SmartScreen 可能要求选择“更多信息 → 仍要运行”，macOS Gatekeeper 可能要求右键应用并选择“打开”。Intel Mac 可通过 Rosetta 运行 arm64 版本；目前尚不提供通用二进制、Apple 公证和自动更新。

## 编辑模式

Markion 提供四种视图模式，默认使用分栏预览。

- **编辑** — 专注的原始 Markdown 源码编辑器。
- **可视化编辑** — 面向所见即所得、基于源码的编辑界面。正文保持渲染并渐进显示必要语法；普通围栏代码正文、块级数学公式、行内图片字段与 GFM 表格单元格提供精确的直接编辑器。HTML、YAML 前言、已注册图表、格式错误及其他含义不明确的结构保留完整、保守的源码编辑区。它不是独立的富文本文档模型——底层 Markdown 始终是唯一数据源。
- **分栏预览** — 左右同时显示源码和渲染预览，可选择按比例同步滚动。
- **阅读** — 不可编辑的渲染视图，默认居中并限制为适合阅读的 860 px 最大宽度；启用“预览自适应宽度”后可使用整个面板。

切换模式会保留当前文档、光标和选区、撤销历史以及每个标签页的滚动状态。

## 文档与工作区

- 多标签页编辑，每个标签页分别保存光标、选区、滚动、撤销/重做、预览、大纲和派生 Markdown 缓存状态。
- 再次打开已打开的 Markdown 文件时会聚焦现有标签页，而不会创建重复标签页。
- **打开文件夹**可切换工作区根目录，并在“文件”侧边栏中显示 Markdown 文件和文件夹。
- 文件树右键菜单会根据目标提供打开、在新标签页打开、新建文件/文件夹、重命名、删除、在系统文件管理器中显示、筛选和刷新等操作。
- 文件和文件夹可就地命名；删除非空文件夹需要二次确认。
- 可从操作系统文件管理器将 Markdown 文件拖入 Markion。
- “文件”和“大纲”面板均可切换显示，侧边栏与分栏预览的分隔线可拖动调整。

## Markdown 编辑与预览

- 使用 `pulldown-cmark` 解析 Markdown，面向 CommonMark 和 GFM。
- 格式化命令支持粗体、斜体、行内代码、链接、图片、标题、列表、任务列表、引用、围栏代码块和源码 Markdown 表格。
- 标题命令默认显示 H1–H5，可在“偏好设置”中扩展为 H1–H6。
- 查找与替换支持区分大小写、正则表达式、上一个/下一个匹配、替换当前项和全部替换。
- 源码表格命令可格式化表格并新增、删除或移动行列。可视化编辑中的表格还支持直接编辑单元格、使用 Tab 遍历、确定性宽度重排，以及同样的源码驱动行列操作；普通预览表格保持只读。
- 可解析 YAML 前言并在预览中隐藏；其中的 `title`、`author` 和 `date` 会用于导出元数据。
- 自动保存默认在停止输入五秒后执行，并为未保存文档写入恢复副本。

渲染预览支持：

- 粗体、斜体、删除线、行内代码、链接、高亮、上标、下标、脚注、任务列表、常用 emoji 短代码和自动链接。
- 正确的有序列表起始编号、嵌套列表、分层项目符号、悬挂缩进、图片和嵌入式 HTML。
- 可选择预览文本，并通过右键菜单复制为纯文本、Markdown 或 HTML；在适用位置还可复制链接地址。
- `$...$` 行内公式和 `$$...$$` 块级公式，提供简单校验和可读的 Unicode 降级显示。
- 使用 syntect 和 two-face 扩展语法集高亮围栏代码，对未覆盖语言使用后备词法分析，并可显示行号。

## 主题、语言与偏好设置

- 十四款内置主题：Paper、Ink、Solar、Forest、Rose、Graphite、GitHub Light/Dark、Solarized Light/Dark、One Light/Dark 和 Tokyo Night/Light。
- 自定义主题使用 Markion 本地主题目录中的 `.toml` 文件；旧版 `.theme` 文件会在首次加载时自动迁移。
- 六种界面语言：英语、简体中文、日语、法语、德语和西班牙语。
- 应用内偏好设置面板可配置主题、语言、侧边栏显示、预览自适应宽度、专注/打字机模式、代码行号、同步滚动和标题菜单层级。
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

[auto_save]
enabled = true
delay_secs = 5

[export]
pdf_engine = "xelatex"
```

配置、恢复文件、主题和按日轮换的诊断日志均使用适合各平台的 Markion 数据目录。启动前设置 `RUST_LOG=debug` 可获得更详细的日志。

## 导出

Markion 可导出为：

- Markdown
- 带样式 HTML 和纯 HTML
- LaTeX
- DOCX
- PDF
- PNG 和 JPEG 文本快照

PDF 和 DOCX 会优先尝试已整合的 Typune/pandoc 导出引擎。如果 pandoc 或选定的 PDF 引擎不可用，Markion 会回退到较简单的内置写入器，并在状态栏中说明所用后端。安装 pandoc 和合适的 PDF 引擎可获得更丰富的输出。PNG/JPEG 和内置 PDF 输出有意保持为基础文本快照。

## 性能

- 预览块、可视化编辑块、大纲、统计信息和行数均按文档版本缓存，并通过 `Arc` 共享。
- 语法高亮会跨编辑复用，语法库在后台预热。
- 撤销快照不包含派生缓存；编辑器按版本复用缓存的文本句柄。
- 预览/可视化编辑列表仅更新变化范围，文件树限制每帧渲染的行数，换行后的源码行会测量实际渲染高度。

局部编辑后，带源码映射的可视化编辑模型会增量复用可独立解析的区域；当 Markdown 上下文或字节范围无法确定时，则回退为完整派生。分栏/阅读预览仍使用防抖与缓存。Markion 仍使用 `String` 而非 rope 文本缓冲区，部分语义读取也会有意执行完整解析。

## 当前限制

- 可视化编辑以尽可能接近所见即所得为目标，同时保留标准 Markdown；不支持、格式错误或字节映射含义不明确的结构会有意显示精确源码，而不会猜测富文本树变更。
- 数学公式使用可读的降级显示，而非 KaTeX/MathJax 级排版。
- 可视化表格单元格支持直接纯文本编辑，但尚未提供单元格内的富行内格式控件。引用式/多行图片、已注册图表、HTML 和 YAML 前言保留源码驱动编辑路径。
- 尚未实现文件树拖放移动和完整的自定义主题安装界面。
- 图片导出是基础文本快照，超大文档尚未在所有派生子系统中使用 rope 或完全增量解析。

## 开发

需要 Rust stable。在仓库根目录运行：

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

质量命令会检查 Rust 格式、完整 Cargo workspace 测试套件，以及严格模式下的所有 OpenSpec 工件。当前支持/回退矩阵、源码范围不变量、解析器职责与所需证据详见[可视化编辑支持与工程契约](docs/visual-editing-quality.md)。

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
