# Project status / 项目进展

Updated / 更新：2026-11-13。This snapshot describes repository implementation and recorded evidence; publication is tracked separately below. 本页区分代码实现、已有证据与发布状态，不以任务勾选数代替验收结果。

## Release state / 发布状态

- Latest verified public stable tag / 当前公开稳定版本：[`v0.5.2`](https://github.com/willmove/markion/releases/tag/v0.5.2).
- Next patch under preparation / 正在准备的补丁版本：`v0.5.3`, comparing / 对比 [`v0.5.2...main`](https://github.com/willmove/markion/compare/v0.5.2...main).
- Current preflight: **2136 passed / 5 ignored / 0 failed**, bundle verification and build/lint/spec checks pass; publication still awaits credentials and manual evidence. 本次全量回归已通过，发布仍待认证环境与人工记录；详见 [v0.5.3 preflight](verification/release-v0.5.3-preflight.md)。
- Follow the [release process](release-process.md). A prepared version or successful build alone does not establish publication: native packages, updater signatures and manifests, the AppImage smoke test, OSS mirror, release notes, and required manual evidence must be verified. 准备版本号或构建成功不等于发布完成；安装包、签名、元数据、冒烟测试、镜像与人工验证均须核实。

## Implemented progress / 已实现进展

| Area / 领域 | Current implementation / 当前实现 | Evidence / 证据 |
| --- | --- | --- |
| Visual Edit performance / 性能 | P0 profiling and linear identity reconciliation; P2 region-local VisualBlock derivation, shared payloads, conservative ChangeSet invalidation, background large-document search / 分阶段诊断、线性身份协调、区域局部派生、共享载荷与后台搜索 | [P0 historical baseline](visual-p0-performance.md), [P0–P3 final report](fluent-editing-implementation.md), [archived derivation tasks](../openspec/changes/archive/2026-11-11-localize-visual-derivation/tasks.md) |
| Editing baseline / 编辑基本功 | Word/line/page navigation, double/triple click, progressive Select All, paired-input rules, nested list indentation, nearby overlays / 词行页导航、双击三击、逐级全选、配对输入、嵌套列表缩进及就近浮层 | [P1 report](visual-p1-editing.md), [shortcuts](keyboard-shortcuts.md) |
| Structured selection / 结构选区 | Text/block/cell state; multi-block copy/delete/move and subtree drag; one validated transaction and semantic undo / 文本、块、单元格选区，多块复制删除移动、子树拖动及统一事务撤销 | [Structured-selection archive](../openspec/changes/archive/2026-11-11-add-structured-visual-selection/tasks.md), [coverage contract](visual-editing-quality.md) |
| Tables and code / 表格与代码 | Rectangular/row/column selection, TSV/HTML copy, matrix paste, final-cell Tab adds a row, code folding / 矩形与整行整列选区、TSV/HTML 复制、矩阵粘贴、末格 Tab 加行、代码折叠 | [Advanced-block archive](../openspec/changes/archive/2026-11-11-complete-advanced-visual-blocks/tasks.md), [smoke fixture](fixtures/visual-p3-smoke.md) |
| Windows native input / 原生输入 | CF_HTML clipboard publishing, composition-scoped IME undo, stable table viewport, coalesced input-triggered frames / HTML 剪贴板、组合输入撤销、表格视口稳定及输入后帧调度 | [Windows real-input report](verification/windows-native-input-2026-11-11.md), [IME/frame archive](../openspec/changes/archive/2026-11-11-fix-windows-ime-undo-grouping/tasks.md) |
| Linux window chrome / 窗口 | Wayland client controls, rounded corners, shadows and resize margins; sidebar overlap fix / Wayland 客户端窗口按钮、圆角、阴影和缩放边缘，侧栏背景修复 | [Window-control tasks](../openspec/changes/fix-linux-wayland-window-controls/tasks.md), [corner tasks](../openspec/changes/round-csd-window-corners/tasks.md), [resize tasks](../openspec/changes/add-csd-resize-borders/tasks.md) |
| Existing workflows / 既有流程 | Markdown/plain-text/JSON/YAML/TOML documents, native Word import, local MarkNice publishing, AI opt-in, backup/sync and signed Windows/AppImage updates / 文档与数据文件、Word 导入、本地公众号排版、可选 AI、备份同步及签名更新 | [README](../README.md), [Word import](word-import.md), [MarkNice](marknice-workspace.md), [AI guide](ai-agent-guide.md), [sync guide](git-sync.md) |

## Measured results and boundaries / 实测及边界

- The Windows 11 mixed-document model benchmark recorded 1 MiB local-edit P50/P95 of **10.33/11.24 ms**, rebuilding a median 10 blocks and reusing 21,409. This excludes GPUI drawing. Windows 模型基准仅表示该混合文档的模型阶段，不包含绘制。See [final model results](fluent-editing-implementation.md#最终模型性能).
- The final Windows OS-input run verified the entire saved file after **1,080** independent key events: callback-to-caret-paint P50/P95/max **26.63/33.14/47.95 ms**. This excludes hardware/OS delivery and GPU presentation, and is not a cross-platform latency guarantee. Windows 最终实测核对了完整保存文件；计时不包含完整硬件输入传输或 GPU 呈现。See [raw evidence and methodology](verification/windows-native-input-2026-11-11.md#帧调度修复后的实机结果).
- The earlier Windows workspace run recorded **2,122 passed, 5 ignored**; subsequent root desktop regressions recorded **788 passed, 2 ignored**, with three frame-contract integration tests. These are historical results at the commits described in the reports, not a fresh v0.5.3 release gate. 这些是报告对应提交的既有结果，不能冒充本次发布前重新运行的检查。
- Markdown stays canonical; independently parseable regions are reused, with full fallbacks for global or uncertain semantics. Suffix source offsets still require linear relocation, and the text buffer remains a `String`. 保留完整正确性回退，不承诺任意文档每键恒定成本或零卡顿。

## Pending work and verification / 待完成与待验证

- Linux CSD resize cursor/drag interaction still awaits the live-session user exercise in task 3.4. Existing GNOME Wayland screenshots verify appearance; they do not prove drag behavior. Linux 缩放光标与拖动仍待实机操作验收，截图不能代替交互验证。
- macOS/Linux native rich-clipboard interoperability and the full editing/input matrix have no evidence from the Windows P0–P3 campaign. Windows 证据不能外推为其他平台验证。
- The local MarkNice workspace still has pending supported-browser/platform, cancellation/permission/session/image, HTML/DOCX and secondary-viewer evidence; see its [guide](marknice-workspace.md) and [export checklist](../openspec/changes/add-marknice-workspace-content-exports/tasks.md). Word import also retains a pending [compatibility corpus and native-platform matrix](docx-import-compatibility.md).
- WYSIWYG gaps remain for multiline/malformed images and residual ambiguous source ranges. GFM merged cells, arbitrary block content inside cells, and image pixel cropping are outside the completed editing scope. YAML front matter, indented code and proven HTML entities already have implemented editing/rendering paths. 仍有多行或畸形图片等缺口，不能把已支持的前言、缩进代码和实体解码继续列为未实现。
- macOS bundle replacement, Linux DEB/RPM self-replacement, and a full theme-installation UI remain unimplemented. Direct WeChat draft publishing is an active proposal, not a shipped capability. macOS 替换安装、DEB/RPM 自替换、完整主题安装界面及直接微信草稿发布仍不能作为已完成能力宣传。

OpenSpec stable [specs](../openspec/specs/) define the system contract. Active [changes](../openspec/changes/) can contain implemented work awaiting verification or archival as well as unimplemented proposals; read their tasks and evidence individually. 稳定规范与活动变更承担不同职责，本次不批量勾选或归档其他变更。
