# Markion v0.6.5

This release adds an in-app gallery for the bundled examples and templates, imports academic Word documents with formatting and formulas intact, and fixes 34 issues reported on GitHub (#107–#140) along with Visual Edit rendering and macOS quick-menu reliability.

## Changes

- **New: browse examples and templates inside Markion.** Help → Examples and Templates opens a scrollable thumbnail gallery in the app; open any sample as an editable tab with working relative links and images while your workspace stays available. The Windows action that reveals the bundled folder in Explorer is fixed, with status-bar feedback when the file manager cannot launch.
- **Academic Word imports keep their structure.** Heading bookmarks become identifiers instead of stray markup, emphasis survives Word runs split mid-phrase, Word equations convert to renderable LaTeX with clear diagnostics for unsupported ones, quote styling, thematic breaks and table alignment are retained, formula-heavy documents render smoothly near the viewport, and the outline can summarize heading formulas.
- **34 fixes across the editor** (issues #107–#140): keyboard Replace All, ordered-list continuation and DOCX list starts/nesting, emoji shortcode completion, word selection and Unicode word navigation, source/visual page navigation, recent-workspace tab restoration, rename-aware history diffs, reference links and HTML copy, UTF-8 BOM handling, PDF alternative text and combining-mark offsets, lossless image URL/title serialization, and more.
- **More dependable Visual Edit.** Mermaid diagrams no longer stick on the loading placeholder in any rendered mode, task checkboxes share consistent geometry, and rendered HTML keeps its authored alignment without card frames while the collapsible source editor remains. On macOS the block-grip quick menu opens reliably despite trackpad jitter, mid-click reflow, micro-scrolls and IME commits, and display-math blocks now expose the block grip.
- **Steadier Arch/Omarchy launches.** The pacman package declares the Lavapipe software Vulkan driver as a runtime dependency, and its launcher retries a failed renderer startup, recovering installations whose system Vulkan stack crashes at launch.

**Compatibility:** No migration is required. Both macOS DMGs are Developer ID signed and Apple notarized; drag Markion to Applications and open normally. Windows NSIS has no Authenticode signature, so SmartScreen may prompt.

**Downloads:** Windows x64 (NSIS/ZIP), macOS arm64/x64 (DMG), Linux x86_64 (DEB/RPM/AppImage/pacman `.pkg.tar.zst`); see the attached assets. On Arch/Omarchy, run `sudo pacman -U markion-0.6.5-1-x86_64.pkg.tar.zst`.

**Verification:** The complete local workspace gates passed on the release tree, native packaging CI passed for all four platforms, and the Windows candidate package passed resource, launch and native-smoke checks. macOS/Linux real-device acceptance, previous-version signed upgrades and real IME/browser/Word interop were not executed (no environments available) and are disclosed in the [verification record](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.5-preflight.md).

**Full comparison:** https://github.com/willmove/markion/compare/v0.6.4...v0.6.5

---

# Markion v0.6.5（中文说明）

本版本新增应用内的示例与模板浏览画廊，完整保留学术 Word 文档的格式与公式导入，并修复了 GitHub 上报告的 34 个问题（#107–#140）以及可视化编辑渲染与 macOS 快捷菜单的可靠性。

## 更新

- **新增：在 Markion 内浏览示例与模板。**「帮助 → 示例与模板」在应用内打开可滚动的缩略图画廊；任一示例可在独立标签页中打开编辑，相对链接与图片可用，当前工作区不受影响。Windows 上在资源管理器中显示 bundled 目录的操作已修复，文件管理器无法启动时状态栏会给出提示。
- **学术 Word 导入保留结构。** 标题书签映射为标识符而不再暴露书签标记；跨运行拆分的强调短语得以保留；Word 公式转换为可渲染的 LaTeX（不支持的公式有明确诊断）；引用样式、分隔线与表格对齐被保留；公式密集文档按视口就近渲染；大纲可摘要标题中的公式。
- **编辑器 34 项修复**（issues #107–#140）：键盘触发全部替换、有序列表续行与 DOCX 列表起始编号/嵌套、表情短代码补全、词选择与 Unicode 词导航、源码/可视化页面导航、最近工作区标签页恢复、感知重命名的历史差异、引用链接与 HTML 复制、UTF-8 BOM 处理、PDF 替代文本与组合字符偏移、图片 URL/标题无损序列化等。
- **可视化编辑更可靠。** Mermaid 图在全部渲染模式下不再停留在加载占位符；任务复选框尺寸一致；渲染 HTML 保持作者对齐且不再带卡片边框，可折叠源码编辑器保留。macOS 上块手柄快捷菜单在触控板抖动、点击中回流、微滚动与输入法提交下都能可靠打开；显示公式块也提供块手柄。
- **Arch/Omarchy 启动更稳。** pacman 包将 Lavapipe 软件 Vulkan 驱动声明为运行时依赖，其启动器在渲染器启动失败时自动重试，可恢复因系统 Vulkan 崩溃而无法启动的安装。

**兼容性：** 无需迁移。两个 macOS DMG 均使用 Developer ID 签名并通过 Apple 公证，拖入「应用程序」后正常打开。Windows NSIS 没有 Authenticode 签名，可能出现 SmartScreen 提示。

**下载：** Windows x64（NSIS/ZIP）、macOS arm64/x64（DMG）、Linux x86_64（DEB/RPM/AppImage/pacman `.pkg.tar.zst`），见附件。Arch/Omarchy 运行 `sudo pacman -U markion-0.6.5-1-x86_64.pkg.tar.zst` 安装。

**验证：** 发布代码树上的本地完整工作区门禁已通过，四个平台的原生打包 CI 已通过，Windows 候选包通过资源、启动与原生冒烟检查。macOS/Linux 实机验收、旧版本签名升级链与真实 IME/浏览器/Word 互操作未执行（无可用环境），详见[验证记录](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.5-preflight.md)。

**完整变更对比：** https://github.com/willmove/markion/compare/v0.6.4...v0.6.5
