# Markion v0.4.5

Reworks Visual Edit table editing into Notion-style row and column handles that never move the document, fixes list-format shortcuts that never fired, and closes an autosave race that could resurrect discarded changes.

## Highlights

### Visual Edit tables: row and column handles

- **No more layout jumps**: the old table toolbar lived inside the document, so hovering or clicking into a table shifted everything below it — the click could even land on a different line. All table controls are now overlays and never change the table's or the document's layout.
- **Row handles**: hovering a row reveals a grip in its leading padding. The row menu offers Insert above / Insert below, Move up / Move down, Duplicate, Clear contents, and Delete. Actions that would break the table header (deleting, moving, or inserting above the header row) are disabled.
- **Column handles**: hovering a column reveals a grip in the header cell's top padding. The column menu offers Insert left / Insert right, Move left / Move right, Align left / center / right, Duplicate, Clear contents, and Delete.
- **Drag to reorder**: drag a row handle onto another row, or a column handle onto another column, to move it. Each move is a single undo step, and saved column widths travel with a moved column.
- **Edge "+" strips**: while the pointer is over a table, a thin "+" strip appears just below it to append a row and just to its right to append a column.
- Whole-table delete, duplicate, and move remain available through the block grip and the right-click block menu. All new labels and messages are translated in every supported UI language.

### Fixes

- **List format shortcuts never fired**: the ordered-list, unordered-list, and inline-code shortcuts bound to Shifted-symbol keys (for example `Ctrl+Shift+7`) did nothing, because the keyboard stack reports the shifted symbol rather than Shift plus the base key. They now work on every platform, and menus and preferences still display them in the pressed form (`Ctrl+Shift+]`).
- **List formats toggle instead of piling up**: applying the same list format again removes it; switching between list kinds replaces the marker in place instead of nesting one list inside the other; the task-list toggle swaps the `[ ]` checkbox against a plain bullet. Ordered numbering stays contiguous across blank lines.
- **Discarded changes could come back**: an explicit "Don't Save", Close Others, or app quit could race a background autosave, which then silently wrote the file or left a recovery snapshot for changes you had thrown away. Autosave now coordinates with discard paths, so discarded content stays discarded.

## Compatibility

- No migration is required for Markdown files, preferences, or workspace data.
- Windows and macOS packages remain unsigned; first launch may require SmartScreen or Gatekeeper bypass.

## Downloads

- Windows x64: NSIS installer and portable `.zip`.
- macOS Apple Silicon: DMG.
- Linux x86_64: DEB, RPM, and AppImage.

## Verification

- `cargo test --workspace` passed locally, and the MarkNice workspace bundle verified.
- Windows, macOS, and Linux native CI builds, package extraction, and MarkNice bundle verification passed.
- Windows updater signature and `update.json` published.

**Full comparison**: https://github.com/willmove/markion/compare/v0.4.4...v0.4.5

---

# Markion v0.4.5（中文说明）

可视化编辑的表格操作重构为 Notion 风格的行/列把手，不再引起文档跳动；修复从未生效的列表格式快捷键；并修复自动保存可能与“放弃更改”竞争、导致已丢弃内容复活的问题。

## 主要更新

### 可视化编辑表格：行/列把手

- **不再跳动**：旧版表格工具栏位于文档流内，悬停或点击进入表格时会推挤下方内容，点击甚至可能落到别的行。现在所有表格控件均为悬浮层，绝不改变表格与文档布局。
- **行把手**：悬停某一行时，会在该行左侧留白处显示把手。行菜单提供：在上方/下方插入、上移/下移、复制、清空内容、删除行。会破坏表头的操作（删除、移动或在表头上方插入）以禁用状态呈现。
- **列把手**：悬停某一列时，会在表头单元格顶部留白处显示把手。列菜单提供：在左侧/右侧插入、左移/右移、左/居中/右对齐、复制、清空内容、删除列。
- **拖拽排序**：将行把手拖到另一行、或将列把手拖到另一列即可移动。每次移动都是单独一步撤销，已保存的列宽会随列一起移动。
- **边缘"+"条**：指针位于表格上时，表格下方与右侧各显示一条细"+"条，点击即可追加行或列。
- 整表删除、复制、移动仍可通过块把手与右键块菜单完成。所有新文案均已翻译到全部受支持的界面语言。

### 修复

- **列表格式快捷键从未生效**：绑定在带 Shift 符号键上的有序列表、无序列表与行内代码快捷键（例如 `Ctrl+Shift+7`）此前完全无效，原因是键盘层上报的是 Shift 后的符号而非 Shift 加基础键。现在它们在所有平台上都能触发，菜单与偏好设置仍按按键形式显示（如 `Ctrl+Shift+]`）。
- **列表格式改为切换而非叠加**：再次应用同一种列表格式会将其移除；在不同列表类型之间切换会原地替换标记，而不是把一种列表嵌套进另一种；任务列表切换会在 `[ ]` 复选框与普通圆点之间转换。有序列表编号跨空行保持连续。
- **已丢弃的更改可能复活**：显式选择“不保存”、关闭其他标签页或退出应用时，可能与后台自动保存竞争，导致文件被悄悄写入、或为已丢弃的更改留下恢复快照。自动保存现在与放弃路径协调，丢弃的内容保持丢弃。

## 兼容性

- Markdown 文件、偏好设置与工作区数据无需迁移。
- Windows 与 macOS 安装包仍未签名，首次启动可能需要手动绕过 SmartScreen 或 Gatekeeper。

## 下载

- Windows x64：NSIS 安装程序与便携 `.zip`。
- macOS Apple Silicon：DMG。
- Linux x86_64：DEB、RPM 与 AppImage。

## 验证

- 本地 `cargo test --workspace` 通过，MarkNice 工作区校验通过。
- Windows、macOS、Linux 原生 CI 构建、安装包解压与 MarkNice 工作区校验通过。
- Windows 更新器签名与 `update.json` 已发布。

**完整变更对比**: https://github.com/willmove/markion/compare/v0.4.4...v0.4.5
