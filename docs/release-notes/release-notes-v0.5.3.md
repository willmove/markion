# Markion v0.5.3

> Preparation draft. Publication and platform-signing claims remain pending until the [release preflight](../verification/release-v0.5.3-preflight.md) and canonical release checks are complete.

Visual editing gains structured block and table operations, local derivation reuse, and Windows input fixes; Linux Wayland gets usable client window controls and refined chrome.

## Highlights

### Structured editing without losing Markdown fidelity

- Select, copy, delete or move multiple visual blocks; list-parent operations carry the full subtree, including nested code. Keyboard movement and grip dragging share checked source operations, and each successful command is one semantic undo step.
- Select table rectangles, rows or columns, copy TSV plus HTML, clear selected cells, or paste a matrix. Tab from the final cell appends a body row. Existing-cell edits preserve untouched source bytes, alignment and stored widths where exact mapping is available.
- Indent/outdent complete nested-list subtrees, split a quote around a paragraph converted to ordinary text, and fold ordinary code blocks per tab. Navigation into folded code expands its editor.
- Word/line/page navigation, double/triple click, progressive Select All, paired-input rules and nearby formatting/link/math overlays improve the everyday editing loop. Visual block copy provides readable text and HTML; Source retains exact source copy.

### Large-document work and Windows input

- Local edits reuse unchanged semantic/visual regions and shared rich-text payloads. Large-document search runs after debounce in the background and discards stale results. Global or ambiguous Markdown semantics retain the full correctness fallback.
- Recorded Windows 1 MiB mixed-document results: model-edit P50/P95 **10.33/11.24 ms**; a separate 1,080-event real OS-input run recorded callback-to-caret-paint P50/P95/max **26.63/33.14/47.95 ms** with the complete saved file checked. These are historical fixture-specific measurements, excluding complete OS delivery and GPU presentation, not universal latency guarantees. See [model results](../fluent-editing-implementation.md) and [native evidence](../verification/windows-native-input-2026-11-11.md).
- Windows publishes valid CF_HTML alongside text; IME candidate pauses remain one composition-scoped undo action. Input-triggered frame requests avoid the measured low-priority paint backlog. Clearing/undoing table selections preserves the viewport, and code Copy controls keep their position.

### Linux window controls and reliability

- Wayland explicitly requests client decorations, exposing minimize/maximize/restore/close controls and a draggable title region. Rounded corners, transparent shadow margins and resize bands respect maximized/fullscreen/tiled states. Existing GNOME Wayland appearance evidence is recorded; live resize-cursor/drag acceptance remains pending.
- Fixed Files/Outline tab and content background overlap.
- Fixed reviewed AI folder plans becoming stale on Unix solely because a child-directory operation changed its parent's link count. Directory volume/object identity remains checked; regular-file hard-link, symlink, containment, review and replacement checks remain enforced. Existing version-1 journals keep their format.

### Current documentation

- Updated all README editions, corrected package formats, source-only JSON/YAML/TOML editing, file-tree moves, existing WYSIWYG support, export defaults and Linux AppImage updating, and added a dated [project-status index](../project-status.md). These corrections describe existing capabilities rather than presenting them all as new in this release.

## Compatibility

- No migration is required for Markdown files, preferences or workspace data. Structured selections and code folds are session state; canonical Markdown remains the persisted document, and existing AI journals retain schema version 1.
- GFM merged cells, arbitrary block-valued cells, image pixel cropping and ambiguous source rewrites are outside the completed editing scope. macOS/Linux rich-clipboard interoperability and the full browser/manual release matrix remain explicitly unverified where recorded evidence is absent.
- Windows packages remain Authenticode-unsigned and may require **More info → Run anyway**. Minisign authenticates updater payloads. macOS notarization status must be confirmed from this tag's build logs before publication; an ad-hoc build requires `xattr -cr /Applications/Markion.app` after a browser download, or download with `curl -LO`.
- Signed automatic installation supports Windows NSIS and a running Linux AppImage. macOS and Linux DEB/RPM use the browser-download flow.

## Downloads

- Windows x64: NSIS installer and portable ZIP.
- macOS Apple Silicon and Intel: separate DMGs.
- Linux x86_64: DEB, RPM and AppImage.
- Publication must include the Windows/AppImage signatures and version-matched `update.json`, with the OSS mirror verified separately.

## Verification

- Local workspace suite: **2136 passed, 5 existing ignored, 0 failed**. Build, Clippy, formatting, bundle and strict spec checks pass. Publication/manual checks remain pending in the [preflight report](../verification/release-v0.5.3-preflight.md); branch-build success is not tag-publication evidence.
- Existing Windows editing tests, callback replay, real IME/OS input and full-file comparisons are linked above with their original scope and dates.

**Full comparison**: https://github.com/willmove/markion/compare/v0.5.2...v0.5.3

---

# Markion v0.5.3（中文说明）

> 发布准备草稿。[发布前核对](../verification/release-v0.5.3-preflight.md)及规范流程完成前，不宣称已发布或已完成各平台签名验证。

可视化编辑新增结构化块与表格操作、局部派生复用及 Windows 输入修复；Linux Wayland 补齐客户端窗口控制并改善窗口外观。

## 主要更新

### 结构化编辑与 Markdown 保真

- 可选择、复制、删除或移动多个可视化块；列表父项操作携带完整子树及嵌套代码。键盘移动与手柄拖动共用经校验的源码操作，每个成功命令只产生一次语义撤销。
- 表格支持矩形、整行和整列选区、TSV/HTML 复制、清空与矩阵粘贴，最后一格按 Tab 新增正文行。能证明精确映射时，已有单元格编辑保留未修改源码、对齐及列宽。
- 可缩进/反缩进完整嵌套列表子树，将引用中间段落转为正文并拆开两侧容器，按标签页折叠普通代码块；导航进入折叠代码时自动展开。
- 词/行/页导航、双击三击、逐级全选、配对输入和就近格式/链接/公式浮层完善日常输入。可视化块复制提供可读文本与 HTML，源码模式保留原文复制。

### 大文档与 Windows 输入

- 局部编辑复用未变化的语义和可视化区域，以及共享富文本载荷。大文档搜索防抖后在后台执行，并丢弃过期结果；全局或不确定 Markdown 语义仍走完整正确性回退。
- 既有 Windows 1 MiB 混合文档模型输入 P50/P95 为 **10.33/11.24 ms**；另一轮 1080 次真实 OS 输入的回调至光标 paint P50/P95/最大值为 **26.63/33.14/47.95 ms**，并逐字核对完整保存文件。这些是对应机器与 fixture 的历史数据，不含完整 OS 输入传输和 GPU 呈现，不能作为通用延迟保证。详见[模型结果](../fluent-editing-implementation.md)与[原生验证](../verification/windows-native-input-2026-11-11.md)。
- Windows 同时发布有效 CF_HTML 与文本；等待 IME 候选后提交仍归入一次组合输入撤销。输入后帧请求消除实测的低优先级绘制等待。表格清空/撤销保持视口，代码 Copy 控件不再跳位。

### Linux 窗口控制与可靠性

- Wayland 显式请求客户端装饰，显示最小化、最大化/还原、关闭按钮及可拖动标题区。圆角、透明阴影边距和缩放边缘适配最大化、全屏及平铺。已有 GNOME Wayland 外观记录，缩放光标与拖动仍待实机验收。
- 修复文件/大纲侧栏标签与内容背景重叠。
- 修复 Unix 上 AI 已审核目录操作仅因子目录改变父目录链接数就被判过期的问题。仍检查目录卷与对象身份，保留文件硬链接、符号链接、范围、审核及替换防护；既有版本 1 日志格式不变。

### 当前文档

- 更新全部 README 语言版本，纠正安装格式、JSON/YAML/TOML 纯源码编辑、文件树移动、既有 WYSIWYG 支持、导出默认值和 Linux AppImage 更新说明，新增带日期的[项目进展](../project-status.md)。这些修正描述现有能力，不将它们全部当作本版本新增功能。

## 兼容性

- Markdown 文件、偏好设置及工作区数据无需迁移。结构选区与代码折叠属于会话状态，Markdown 仍是持久化文档，AI 日志仍使用版本 1。
- GFM 合并格、任意多块单元格、图片像素裁剪和无法证明源码边界的改写不在完成范围内。macOS/Linux 富文本剪贴板互操作和完整浏览器/人工发布矩阵，在缺少记录处仍明确标记待验证。
- Windows 包未做 Authenticode 签名，可能需要“更多信息 → 仍要运行”；Minisign 用于认证更新载荷。发布前须以本标签构建日志确认 macOS 公证状态；ad-hoc 构建经浏览器下载后需要 `xattr -cr /Applications/Markion.app`，或改用 `curl -LO` 下载。
- 签名自动安装支持 Windows NSIS 与正在运行的 Linux AppImage；macOS 和 Linux DEB/RPM 使用浏览器下载路径。

## 下载

- Windows x64：NSIS 安装程序与便携 ZIP。
- macOS Apple Silicon 与 Intel：独立 DMG。
- Linux x86_64：DEB、RPM 与 AppImage。
- 发布还须包含 Windows/AppImage 签名及版本匹配的 `update.json`，并单独核验 OSS 镜像。

## 验证

- 本地完整工作区：**2136 通过、5 项既有忽略、0 失败**；构建、Clippy、格式、bundle 与严格规范校验通过。发布与人工检查仍见[发布前核对](../verification/release-v0.5.3-preflight.md)，分支构建成功不能代替标签发布证据。
- 既有 Windows 编辑测试、回调回放、真实 IME/OS 输入与完整文件核对见上述链接，保留原始日期与范围。

**完整变更对比**: https://github.com/willmove/markion/compare/v0.5.2...v0.5.3
