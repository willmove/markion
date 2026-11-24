# Project status / 项目进展

Updated / 更新：2026-11-24。This snapshot describes repository implementation and recorded evidence; publication is tracked separately below. 本页区分代码实现、已有证据与发布状态，不以任务勾选数代替验收结果。

## Release state / 发布状态

- v0.6.3 published / 已发布：新增 Arch/Omarchy 兼容 pacman 包、代码块格式化、表格公式渲染与主题一致性改进。最终工作区测试通过；标签工作流十个作业全部成功，八包、两签名、双清单与 OSS 哈希均完成核验，两个 macOS 架构通过签名、公证和安装启动检查。未执行的原生人工项按 V063-EX-01 一次性例外披露，见 [v0.6.3 preflight](verification/release-v0.6.3-preflight.md)。
- v0.6.2 tag preserved, no GitHub Release / 保留标签但未发布 Release：Apple 公证网络故障后未重建或移动公开标签；v0.6.3 包含其后修复并作为当前发布版本。

- v0.6.1 published / 已发布：韩语界面、可调行距与段距、后台更新准备，以及保留内容的表格/分隔线插入；八种 README 已同步。Windows workspace **2,238 passed / 5 ignored / 0 failed**，四个原生构建、AppImage 冒烟、签名、发布和镜像均通过，七包及签名/清单/哈希完成核验，最终 Windows ZIP 保存/撤销子集通过。完整人工矩阵按本次明确授权保留为未验证，见 [v0.6.1 preflight](verification/release-v0.6.1-preflight.md)。

- v0.6.0 published / 已发布：紧凑上下文菜单、表格与图片属性、斜杠/emoji 补全、独立右侧大纲及离线样例库；Windows workspace **2,216 passed / 5 ignored / 0 failed**。四个平台打包、AppImage 冒烟、签名、发布、镜像及哈希验证均完成；未验证人工项按维护者授权明确跳过，见 [v0.6.0 preflight](verification/release-v0.6.0-preflight.md)。

- Latest verified public stable tag / 当前公开稳定版本：[`v0.6.3`](https://github.com/willmove/markion/releases/tag/v0.6.3).
- Published comparison / 本次完整对比：[`v0.6.2...v0.6.3`](https://github.com/willmove/markion/compare/v0.6.2...v0.6.3).
- Previous v0.5.3 preflight / 上一版本记录: **2136 passed / 5 ignored / 0 failed**, bundle verification and build/lint/spec checks pass; the tag packaging/signing/publication/mirror workflow and Windows repository quality gate passed. 本次全量回归、标签构建/签名/发布/镜像及 Windows 仓库质量检查已通过；缺少环境的人工项按维护者授权记录为跳过；详见 [v0.5.3 preflight](verification/release-v0.5.3-preflight.md)。
- Follow the [release process](release-process.md). A prepared version or successful build alone does not establish publication: native packages, updater signatures and manifests, the AppImage smoke test, OSS mirror, release notes, and required manual evidence must be verified. 准备版本号或构建成功不等于发布完成；安装包、签名、元数据、冒烟测试、镜像与人工验证均须核实。

## Implemented progress / 已实现进展

| Area / 领域 | Current implementation / 当前实现 | Evidence / 证据 |
| --- | --- | --- |
| Visual Edit performance / 性能 | P0 profiling and linear identity reconciliation; P2 region-local VisualBlock derivation, shared payloads, conservative ChangeSet invalidation, background large-document search / 分阶段诊断、线性身份协调、区域局部派生、共享载荷与后台搜索 | [P0 historical baseline](visual-p0-performance.md), [P0–P3 final report](fluent-editing-implementation.md), [archived derivation tasks](../openspec/changes/archive/2026-11-11-localize-visual-derivation/tasks.md) |
| Editing baseline / 编辑基本功 | Word/line/page navigation, double/triple click, progressive Select All, paired-input rules, nested list indentation, nearby overlays / 词行页导航、双击三击、逐级全选、配对输入、嵌套列表缩进及就近浮层 | [P1 report](visual-p1-editing.md), [shortcuts](keyboard-shortcuts.md) |
| Structured selection / 结构选区 | Text/block/cell state; multi-block copy/delete/move and subtree drag; one validated transaction and semantic undo / 文本、块、单元格选区，多块复制删除移动、子树拖动及统一事务撤销 | [Structured-selection archive](../openspec/changes/archive/2026-11-11-add-structured-visual-selection/tasks.md), [coverage contract](visual-editing-quality.md) |
| Block grip settings / 块首快捷菜单 | Exclusive selection ends leave the next block's syntax hidden; completed clicks open compact block settings, optional in General preferences; Shift extension and dragging remain available / 修正后续块标记误展开，完成单击后打开紧凑菜单，可在常规偏好中关闭，保留扩选和拖动 | [Change tasks](../openspec/changes/archive/2026-11-17-fix-block-handle-selection-and-menu/tasks.md), [Windows and regression verification](verification/block-grip-menu-windows-2026-11-16.md) |
| Tables and code / 表格与代码 | Rectangular/row/column selection, TSV/HTML copy, matrix paste, final-cell Tab adds a row, code folding / 矩形与整行整列选区、TSV/HTML 复制、矩阵粘贴、末格 Tab 加行、代码折叠 | [Advanced-block archive](../openspec/changes/archive/2026-11-11-complete-advanced-visual-blocks/tasks.md), [smoke fixture](fixtures/visual-p3-smoke.md) |
| Windows native input / 原生输入 | CF_HTML clipboard publishing, composition-scoped IME undo, stable table viewport, coalesced input-triggered frames / HTML 剪贴板、组合输入撤销、表格视口稳定及输入后帧调度 | [Windows real-input report](verification/windows-native-input-2026-11-11.md), [IME/frame archive](../openspec/changes/archive/2026-11-11-fix-windows-ime-undo-grouping/tasks.md) |
| Linux window chrome / 窗口 | Wayland client controls, rounded corners, shadows and resize margins; sidebar overlap fix / Wayland 客户端窗口按钮、圆角、阴影和缩放边缘，侧栏背景修复 | [Window-control tasks](../openspec/changes/archive/2026-11-17-fix-linux-wayland-window-controls/tasks.md), [corner tasks](../openspec/changes/archive/2026-11-17-round-csd-window-corners/tasks.md), [resize tasks](../openspec/changes/add-csd-resize-borders/tasks.md) |
| Existing workflows / 既有流程 | Markdown/plain-text/JSON/YAML/TOML documents, native Word import, local MarkNice publishing, AI opt-in, backup/sync and signed Windows/AppImage updates / 文档与数据文件、Word 导入、本地公众号排版、可选 AI、备份同步及签名更新 | [README](../README.md), [Word import](word-import.md), [MarkNice](marknice-workspace.md), [AI guide](ai-agent-guide.md), [sync guide](git-sync.md) |
| Context menus and cells / 上下文菜单与单元格 | Direct block controls, hover insertion/row/column flyouts, exact selection actions, bounded Image Properties, Shift+Enter cell breaks and menu/tab lifecycle fixes / 直接块操作、悬停插入及行列菜单、精确选区、图片属性、单元格换行和菜单生命周期修复 | [Archived tasks and Windows evidence](../openspec/changes/archive/2026-11-17-unify-compact-block-context-menu/tasks.md) |
| Outline layout / 大纲布局 | Independent resizable right outline alongside left Files, or left placement; separate visibility and persisted layout / 可独立缩放和显隐的右侧大纲，与左侧文件树并排，也支持左侧布局 | [Outline archive](../openspec/changes/archive/2026-11-16-separate-right-outline-from-file-tree/verification.md) |
| Completion and examples / 补全与样例 | Compact H1–H5 slash palette with optional popup, 36-candidate emoji window, five offline guides and twelve templates / 紧凑斜杠补全及开关、36 个 emoji 候选、5 篇离线指南与 12 份模板 | [Examples](../assets/examples/README.md), [archival review](verification/openspec-archive-review-2026-11-17.md) |

## Measured results and boundaries / 实测及边界

- Reorder-separator follow-up: **1,575 passed, 3 existing ignored, 0 failed** in the final root suite (752 library, 818 desktop and 5 integration tests). Moves preserve independent paragraphs at terminal LF/CRLF boundaries and retain tight list subtrees. Eight GPUI drag combinations and an OS drag/save/single-Undo byte comparison close the earlier recorded separator issue. 拖动分隔问题已修复关闭；新增三项库回归和两项桌面回归，实际 Windows 拖动保存及一次撤销的完整源码核对通过。See [follow-up evidence](verification/block-grip-menu-windows-2026-11-16.md#拖动分隔后续修复).

- Initial block-grip verification on 2026-11-16 recorded **1,570 passed, 3 existing ignored, 0 failed** before the separator follow-up above (749 library, 816 desktop and 5 integration tests). Six focused grip tests and Windows OS-input checks cover the new menu and selection boundary. Shift-click and constrained-viewport coverage use GPUI input/layout tests; native held-modifier clicking is unsupported by the available automation APIs. This is an implementation check, not a release gate. 首轮块菜单验证与后续分隔修复的计数分别记录；不得将 GPUI 测试混称为原生 Shift-click 验证，亦不表示已发布。See the [verification record](verification/block-grip-menu-windows-2026-11-16.md).

- The Windows 11 mixed-document model benchmark recorded 1 MiB local-edit P50/P95 of **10.33/11.24 ms**, rebuilding a median 10 blocks and reusing 21,409. This excludes GPUI drawing. Windows 模型基准仅表示该混合文档的模型阶段，不包含绘制。See [final model results](fluent-editing-implementation.md#最终模型性能).
- The final Windows OS-input run verified the entire saved file after **1,080** independent key events: callback-to-caret-paint P50/P95/max **26.63/33.14/47.95 ms**. This excludes hardware/OS delivery and GPU presentation, and is not a cross-platform latency guarantee. Windows 最终实测核对了完整保存文件；计时不包含完整硬件输入传输或 GPU 呈现。See [raw evidence and methodology](verification/windows-native-input-2026-11-11.md#帧调度修复后的实机结果).
- The earlier Windows workspace run recorded **2,122 passed, 5 ignored**; subsequent root desktop regressions recorded **788 passed, 2 ignored**, with three frame-contract integration tests. These are historical results at the commits described in the reports, not a fresh v0.5.3 release gate. 这些是报告对应提交的既有结果，不能冒充本次发布前重新运行的检查。
- Markdown stays canonical; independently parseable regions are reused, with full fallbacks for global or uncertain semantics. Suffix source offsets still require linear relocation, and the text buffer remains a `String`. 保留完整正确性回退，不承诺任意文档每键恒定成本或零卡顿。

## Pending work and verification / 待完成与待验证

- Linux CSD resize cursor/drag interaction still awaits the live-session user exercise in task 3.4. Existing GNOME Wayland screenshots verify appearance; they do not prove drag behavior. Linux 缩放光标与拖动仍待实机操作验收，截图不能代替交互验证。
- macOS/Linux native rich-clipboard interoperability and the full editing/input matrix have no evidence from the Windows P0–P3 campaign. Windows 证据不能外推为其他平台验证。
- The local MarkNice workspace still has pending supported-browser/platform, cancellation/permission/session/image, HTML/DOCX and secondary-viewer evidence; see its [guide](marknice-workspace.md) and [export checklist](../openspec/changes/add-marknice-workspace-content-exports/tasks.md). Word import also retains a pending [compatibility corpus and native-platform matrix](docx-import-compatibility.md).
- WYSIWYG gaps remain for multiline/malformed images and residual ambiguous source ranges. GFM merged cells, arbitrary block content inside cells, and image pixel cropping are outside the completed editing scope. YAML front matter, indented code and proven HTML entities already have implemented editing/rendering paths. 仍有多行或畸形图片等缺口，不能把已支持的前言、缩进代码和实体解码继续列为未实现。
- macOS bundle replacement, Linux DEB/RPM/pacman self-replacement, and a full theme-installation UI remain unimplemented. Direct WeChat draft publishing is an active proposal, not a shipped capability. macOS 替换安装、DEB/RPM/pacman 自替换、完整主题安装界面及直接微信草稿发布仍不能作为已完成能力宣传。

OpenSpec stable [specs](../openspec/specs/) define the system contract. Active [changes](../openspec/changes/) can contain implemented work awaiting verification or archival as well as unimplemented proposals; read their tasks and evidence individually. 稳定规范与活动变更承担不同职责，本次归档与保留原因见 [OpenSpec 审查](verification/openspec-archive-review-2026-11-17.md)，未完成的人工项不会仅因测试通过而勾选。
