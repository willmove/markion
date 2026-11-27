# Markion v0.6.4

This patch makes Visual Edit tables and rendered formatting more dependable, keeps more of your content intact through import and export, and makes code folding optional.

## Changes

- Tables stay under control in Visual Edit: clicking and dragging inside tall tables no longer scrolls the page, tables inside block quotes remain editable tables, and a table's handle menu now gathers whole-table commands, **Equalize Column Widths** and the same Insert Below palette as paragraphs.
- Rendered output matches what you wrote: headings and images inside block quotes, table header emphasis and column alignment, nested bold/italic, closing punctuation and inline-formula baselines are correct in Visual Edit, Split Preview and Read. On Linux, bold and italic now survive font fallback.
- Code folding is optional: Visual Edit code blocks open fully expanded, and the Fold code / Expand code control is off by default.
- Word import keeps list numbering and hierarchy, manual line breaks and explicit formatting; LaTeX export keeps nested lists, start numbers and literal dollar signs; Word and HTML export keep styled links, row headers and link parameters intact.
- Editing is safer: Read mode no longer changes the document, renaming a file or folder is refused while an open tab has unsaved edits, autosave still runs after you close a neighbouring tab, and typed Find works in Read mode.

**Compatibility:** No migration is required. To restore the Fold code control, turn on **Allow code folding in Visual Edit** under **Preferences → Appearance**; nothing was ever collapsed automatically. **Equalize Column Widths** writes a `<!-- markion-cols:… -->` comment above the table, the same mechanism column dragging already uses. Both macOS DMGs are Developer ID signed and Apple notarized; drag Markion to Applications and open normally. Windows NSIS has no Authenticode signature, so SmartScreen may prompt.

**Downloads:** Windows x64 (NSIS/ZIP), macOS arm64/x64 (DMG), and Linux x86_64 (DEB/RPM/AppImage/pacman `.pkg.tar.zst`); see the attached assets. On Arch/Omarchy, run `sudo pacman -U markion-0.6.4-1-x86_64.pkg.tar.zst`.

**Verification:** The complete local workspace gates passed on the release tree, native packaging CI passed for all four platforms, and the Windows candidate package passed resource, launch and native-smoke checks. Final tagged-package results are recorded in the [verification record](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.4-preflight.md).

**Full comparison:** https://github.com/willmove/markion/compare/v0.6.3...v0.6.4

---

# Markion v0.6.4（中文说明）

本次补丁让可视化编辑中的表格与渲染格式更可靠，让导入导出保留更多原有内容，并将代码折叠改为可选。

## 更新

- 可视化编辑中的表格更可控：在高表格中点击、拖选不再让页面滚动，引用块内的表格仍可按表格编辑；表格手柄菜单集中了整表命令、**均分列宽**，以及与段落一致的“在下方插入”选项。
- 渲染结果与所写一致：引用块内的标题与图片、表头强调与列对齐、嵌套粗斜体、收尾标点和行内公式基线，在可视化编辑、分栏预览和阅读模式下均正确显示。Linux 上字体回退后仍保留粗体与斜体。
- 代码折叠改为可选：可视化编辑的代码块默认完全展开，“折叠代码 / 展开代码”控件默认关闭。
- Word 导入保留列表编号与层级、手动换行和显式格式；LaTeX 导出保留嵌套列表、起始编号和字面美元符号；Word 与 HTML 导出完整保留带格式的链接、行标题和链接参数。
- 编辑更安全：阅读模式不再修改文档；有未保存编辑的标签页打开时，拒绝重命名文件或文件夹；关闭相邻标签页后自动保存仍会执行；阅读模式下可直接输入查找内容。

**兼容性：** 无需迁移。如需恢复“折叠代码”控件，请在**首选项 → 外观**中开启**允许折叠代码（可视化编辑）**；此前也从不自动折叠。**均分列宽**会在表格上方写入 `<!-- markion-cols:… -->` 注释，与拖动列宽使用同一机制。两个 macOS DMG 均使用 Developer ID 签名并通过 Apple 公证，拖入「应用程序」后正常打开。Windows NSIS 没有 Authenticode 签名，可能出现 SmartScreen 提示。

**下载：** Windows x64（NSIS/ZIP）、macOS arm64/x64（DMG）、Linux x86_64（DEB/RPM/AppImage/pacman `.pkg.tar.zst`），见附件。Arch/Omarchy 运行 `sudo pacman -U markion-0.6.4-1-x86_64.pkg.tar.zst` 安装。

**验证：** 发布代码树上的本地完整工作区门禁已通过，四个平台的原生打包 CI 已通过，Windows 候选包通过资源、启动与原生冒烟检查。最终标签包的结果见[验证记录](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.4-preflight.md)。

**完整变更对比：** https://github.com/willmove/markion/compare/v0.6.3...v0.6.4
