# Markion v0.4.7

Renders GFM alerts as styled, foldable callouts with optional custom titles, adds definition lists across every editing surface and export, extends Visual Edit to indented and unclosed code blocks, and fixes Visual Edit table cell navigation and empty-cell editing.

## Highlights

### Styled GFM alert callouts

- GFM alerts (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`) now render in Split Preview and Read mode as distinct callout cards: a kind-colored left border, a subtle tinted background, and a title row with the kind's icon and label. Plain blockquotes keep their ordinary look.
- In Split Preview and Read mode, clicking the callout title row folds and unfolds the alert body. Folding is view-only and session-only — it never edits the document.
- Visual Edit colors the whole quote group with the kind's color and shows the kind icon in the callout title row, so alerts look consistent across all three surfaces.
- Icons are bundled Lucide SVGs, so callouts render identically on every platform without relying on emoji fonts.

### Custom titles for alerts

- Alerts written the Obsidian way, with a title after the marker (`> [!NOTE] Release notes`), are now recognized: the custom title replaces the default label in Split Preview, Read mode, and Visual Edit. In Visual Edit, focusing the title row still reveals the exact authored marker line for editing.
- Exports show the title as well: DOCX, PDF, and LaTeX label the alert with it; HTML export gives every alert a title paragraph and marks a titled alert as an alert quote. Untitled alerts also gain their default label in HTML and LaTeX exports.

### Definition lists

- GFM/Pandoc-style definition lists (`Term` followed by `: definition`) now render in Split Preview, Read mode, and Visual Edit: terms in bold, definitions indented, and the `:` marker hidden until the caret reveals it — the same mechanism as list markers.
- Built-in exports support them: HTML emits real `<dl>/<dt>/<dd>` elements; DOCX, PDF, and LaTeX render bold terms with indented definitions.
- Compatibility note: a line starting with `:` directly after a paragraph line now forms a definition list where it previously stayed literal prose. This matches Pandoc, PHP Markdown Extra, and Obsidian behavior. No stored data changes.

### Visual Edit: indented and unclosed code blocks

- Indented (four-space) code blocks now render as highlighted code blocks with a payload editor instead of raw-source islands. The indentation stays in the source but is hidden in the display; Enter and multi-line paste re-add it so new lines stay inside the block.
- Unclosed or malformed fenced code blocks keep their payload editor and language label, with the payload running to the end of the block — an unclosed fence you are still typing no longer jumps between a raw island and a decorated code block as the closing fence appears and disappears.

### Fixes

- **Visual Edit table navigation**: Up/Down inside a GFM table now moves through the current cell's wrapped lines and then to the adjacent row in the same column, leaving the table only at its outer row boundaries. Previously the caret escaped the table at the active cell's line boundary.
- **Empty table cells**: empty cells now have a reliable click target and a painted caret, backed by source positions without inserting placeholder bytes into the Markdown. Cell interiors and padding are clickable, with regression tests covering formatted three-column tables, empty rows, selection, wrapping, input/undo, and boundary handoff.

## Compatibility

- No migration is required for Markdown files, preferences, or workspace data.
- One parsing behavior change: `: definition` after a paragraph now starts a definition list, matching Pandoc and Obsidian.
- Windows and macOS packages remain unsigned; first launch may require SmartScreen or Gatekeeper bypass.

## Downloads

- Windows x64: NSIS installer and portable `.zip`.
- macOS Apple Silicon: DMG.
- Linux x86_64: DEB, RPM, and AppImage.

## Verification

- `cargo test --workspace` passed locally, and the MarkNice workspace bundle verified.
- Windows, macOS, and Linux native CI builds, package extraction, and MarkNice bundle verification passed.
- Windows updater signature and `update.json` published; Aliyun OSS mirror verified.

**Full comparison**: https://github.com/willmove/markion/compare/v0.4.6...v0.4.7

---

# Markion v0.4.7（中文说明）

GFM 警示块现在渲染为带样式、可折叠的标注卡片并支持自定义标题；定义列表在所有编辑视图与导出格式中生效；可视化编辑补齐缩进与未闭合代码块；并修复可视化编辑表格的单元格导航与空单元格编辑。

## 主要更新

### 带样式的 GFM 警示块

- GFM 警示块（`> [!NOTE]`、`[!TIP]`、`[!IMPORTANT]`、`[!WARNING]`、`[!CAUTION]`）在分屏预览与阅读模式下渲染为独立标注卡片：按类型着色的左侧边框、浅色渐变底色，以及带图标与标签的标题行。普通引用块保持原样。
- 在分屏预览与阅读模式中，点击标注标题行可折叠/展开正文。折叠仅为查看态、会话级状态，绝不改动文档。
- 可视化编辑中整个引用组按类型着色，标题行显示类型图标，三个视图的观感保持一致。
- 图标为内置 Lucide SVG，所有平台渲染一致，不依赖 emoji 字体。

### 警示块自定义标题

- 按 Obsidian 习惯在标记后写标题（`> [!NOTE] 发布说明`）现在可被识别：自定义标题在分屏预览、阅读模式与可视化编辑中取代默认标签。在可视化编辑中聚焦标题行仍会显示作者书写的完整标记行以便编辑。
- 导出同样显示标题：DOCX、PDF 与 LaTeX 以其为标注标签；HTML 导出为每个警示块生成标题段落，并将带标题的警示块标记为 alert 引用。无标题警示块在 HTML 与 LaTeX 导出中也会显示默认标签。

### 定义列表

- GFM/Pandoc 风格定义列表（`术语` 后跟 `: 定义`）现可在分屏预览、阅读模式与可视化编辑中渲染：术语加粗、定义缩进，`:` 标记隐藏并在光标到达时显示——与列表标记同一机制。
- 内置导出全面支持：HTML 输出真正的 `<dl>/<dt>/<dd>` 元素；DOCX、PDF 与 LaTeX 渲染加粗术语与缩进定义。
- 兼容性说明：段落后以 `:` 开头的行现在会构成定义列表，此前保持字面文本。这与 Pandoc、PHP Markdown Extra 和 Obsidian 行为一致。不涉及任何存储数据变更。

### 可视化编辑：缩进与未闭合代码块

- 缩进（四空格）代码块现在渲染为带语法高亮与载荷编辑器的代码块，不再是原始源码孤岛。缩进保留在源码中但显示时隐藏；回车与多行粘贴会重新补上缩进，新行保持在块内。
- 未闭合或格式错误的围栏代码块保留载荷编辑器与语言标签，载荷延伸到块尾——正在输入的未闭合围栏不会再随着闭合围栏的出现与消失在原始孤岛与代码块之间来回切换。

### 修复

- **可视化编辑表格导航**：GFM 表格内的上/下方向键现在先在当前单元格的换行内移动，再进入相邻行的同一列，仅在表格最外侧行边界离开表格。此前光标会在当前单元格的行边界处直接逃出表格。
- **空表格单元格**：空单元格现在拥有可靠的点击目标与绘制的光标，由源码位置支撑，不向 Markdown 插入占位字节。单元格内部与内边距均可点击；针对格式化三列表格、空行、选区、换行、输入/撤销与边界交接均有回归测试覆盖。

## 兼容性

- Markdown 文件、偏好设置与工作区数据无需迁移。
- 一处解析行为变化：段落后的 `: 定义` 现在会构成定义列表，与 Pandoc 和 Obsidian 一致。
- Windows 与 macOS 安装包仍未签名，首次启动可能需要手动绕过 SmartScreen 或 Gatekeeper。

## 下载

- Windows x64：NSIS 安装程序与便携 `.zip`。
- macOS Apple Silicon：DMG。
- Linux x86_64：DEB、RPM 与 AppImage。

## 验证

- 本地 `cargo test --workspace` 通过，MarkNice 工作区校验通过。
- Windows、macOS、Linux 原生 CI 构建、安装包解压与 MarkNice 工作区校验通过。
- Windows 更新器签名与 `update.json` 已发布；阿里云 OSS 镜像已验证。

**完整变更对比**: https://github.com/willmove/markion/compare/v0.4.6...v0.4.7
