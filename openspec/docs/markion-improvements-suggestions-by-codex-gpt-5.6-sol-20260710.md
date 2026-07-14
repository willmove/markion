## 核心结论

Markion 已经不再是“缺基础功能”的原型：多标签、实时预览、文件树、导出、恢复、主题、i18n、查找替换等骨架已经比较完整。下一阶段不宜继续零散堆功能，最合理的主线是：

**工程基线收敛 → 建立真实性能基准 → 小步实现 Visual Edit → 补齐工作区与文件可靠性 → 提升数学、图片和导出质量。**

其中，最值得警惕的性能点已经不是预览解析，而是**源编辑器仍会对整个文档进行排版，以及每次编辑仍有整篇文本复制/扫描**。

## 当前真实状态

- 工作区干净，`main` 与远端一致。
- `cargo test --workspace`：**474 项通过、3 项忽略、0 失败**。
- 严格 `cargo clippy --workspace --all-targets -- -D warnings` 未通过，共发现约 15 项问题，主要集中在 `crates/markdown`，以可维护性问题为主。
- GitHub 当前没有开放 Issue 或 PR，因此下面的优先级主要依据代码、规格和架构风险，而不是已有用户投票。
- OpenSpec 结构校验为 **27/27 通过**，但内容和版本控制状态存在严重问题：

  - 当前有 16 个活跃 change，其中 8 个已经 100% 完成但未归档。
  - 5 个 change 只剩手工验证或一次测试。
  - 真正尚未开始的是 `add-visual-edit-mode`（0/27）、`add-file-tree-entry-context-actions`（0/18）和 `outline-preview-jump`（0/5）。
  - `openspec/` 被 [.gitignore](/C:/Coding/EditorProjects/markion/.gitignore:17) 整体忽略，而且提交 `730cb9e` 曾将其从仓库删除；这和项目声明“OpenSpec 是 source of truth”直接冲突。
  - [openspec/config.yaml](/C:/Coding/EditorProjects/markion/openspec/config.yaml:4) 仍写着“单 crate、无 workspace”，已经过时。
  - 部分稳定规格也与代码不一致，例如 branding spec 要求两个不存在的 SVG，实际只有 `assets/logo.svg`；主题规格中仍混杂旧 `.theme`/键值格式，而实现已经迁移到 TOML。

这意味着当前第一优先级不是新功能，而是先恢复“规格可信”。

## 推荐推进方向

| 优先级 | 方向               | 建议                                                                                                                           |
| --- | ---------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| P0  | OpenSpec 与开发基线收敛 | 重新纳入版本控制；修正 config；归档 8 个完成 change；完成 5 个手工验证；对齐稳定 specs、README、FAQ                                                          |
|     |                  |                                                                                                                              |
| P0  | CI 质量门禁          | 当前 [release.yml](/C:/Coding/EditorProjects/markion/.github/workflows/release.yml:44) 只构建和打包，不运行测试、fmt 或 clippy。增加独立质量 workflow |
| P1  | 大文档性能 v2         | 测量真实 input-to-paint，而不是只测 Markdown 解析；根据数据决定源编辑器虚拟化、rope/piece table 和增量解析的顺序                                                |
| P1  | Visual Edit 薄切版本 | 先做架构 spike，再只支持段落、标题、粗体、斜体和行内代码；列表、链接随后；表格、数学、HTML 继续用 source island                                                         |
| P1  | 文件可靠性            | 原子保存、外部修改检测、冲突提示、全部恢复文件管理、多标签会话恢复                                                                                            |
| P2  | 工作区效率            | Open Folder、最近工作区、快速打开、全局搜索、命令面板、文件树键盘导航与文件监听                                                                                |
| P2  | 图片资产工作流          | 粘贴/拖入图片后复制到资源目录并插入相对链接；检测丢失资源；支持重命名联动                                                                                        |
| P2  | 输出质量             | 高质量数学渲染、富 PDF/图片导出、自包含 HTML、打印预览                                                                                             |
| P3  | 深度编辑             | 表格单元格编辑、outline 折叠、引用/脚注辅助、backlinks                                                                                         |
| P3  | 分发与生态            | 签名/公证、自动更新、macOS universal、插件体系；等核心体验稳定后再做                                                                                   |

### Visual Edit 是最有产品辨识度的方向，但应先重新做架构决策

现有 [add-visual-edit-mode/design.md](/C:/Coding/EditorProjects/markion/openspec/changes/add-visual-edit-mode/design.md:1) 计划新增第三套 `VisualBlock`/`VisualInlineRun` 模型。风险是项目当前已经存在：

1. 根 crate 的 `PreviewBlock` 解析模型；
2. `crates/markdown` 的 AST 和增量解析器；
3. 准备新增的 Visual Edit source-range 模型。

归档的 AST 评估明确写了两个重新开启条件，其中之一就是“出现真正需要 richer AST 的功能”。Visual Edit 已经满足这个条件。因此在执行 27 个任务前，应做一个短 spike，对比：

- 当前 parser + source scan；
- `typune_markdown::IncrementalParser`；
- 独立 VisualBlock 模型。

重点比较 source-range 准确性、嵌套语法处理、100/500 KiB 编辑延迟、集成改动量。不要未经复评就直接形成三套长期 Markdown 模型。

## 性能分析

### 1. 最大风险：源编辑器只虚拟化了 paint，没有虚拟化 layout

[EditorElement](/C:/Coding/EditorProjects/markion/src/main.rs:5106) 在文档版本变化时会对全文调用 `shape_text` 计算高度，而 [prepaint](/C:/Coding/EditorProjects/markion/src/main.rs:5180) 每次渲染又会对全文排版。最后的 paint 虽然只绘制可见行，但此时全文布局成本已经发生。

因此当前状态更准确地说是：

> 预览已虚拟化；源编辑器只做了可见区域绘制优化，尚未实现真正的行级虚拟化。

建议使用：

- 按逻辑行/可见窗口排版；
- 行高索引和前缀和定位；
- 二分查找 offset/y；
- 超长单行单独降级处理。

这是大文档体验最值得优先测量和优化的地方。

### 2. 文本缓冲和撤销仍有 O(n) 编辑成本

文档底层是 `String`；中间插入本身需要移动后续内容。常规编辑前的 [snapshot](/C:/Coding/EditorProjects/markion/src/main.rs:928) 又会复制整个文档。历史记录随后会通过 [compact_history_entry](/C:/Coding/EditorProjects/markion/src/main.rs:656) 扫描新旧全文，把旧快照压缩成 diff。

这已经将撤销历史内存从“最多 200 份全文”优化为“最多一份全文”，但单次输入仍可能产生：

- 一次全文复制；
- 一次前后缀扫描；
- 一次 `String` 中间移动。

长期方案是 rope/piece table，并让编辑操作直接产生 inverse edit，连续输入合并为一个 undo transaction。

### 3. 预览路径目前反而做得比较好

现有实现已经具备：

- 预览解析 debounce；
- 后台线程解析；
- 版本门控；
- `Arc<Vec<PreviewBlock>>` 复用；
- `ListState` 虚拟化；
- changed-range splice；
- 语法高亮缓存；
- 文件树后台扫描和 300 行渲染上限。

因此不建议再把第一轮性能预算投入“继续缓存预览”。

剩余可改善点是后台解析启动前仍会 [复制整个文档](/C:/Coding/EditorProjects/markion/src/main.rs:1704)，解析完成后 block 的前后缀比较也仍是 O(n)。这些应排在源编辑器布局和文本缓冲之后。

### 4. 增量解析值得重测，但不是单独的第一优先级

历史测量中：

- 37 KiB：完整重解析约 4.7 ms，增量约 1.1 ms；
- 388 KiB：完整重解析约 45.5 ms，增量约 14.1 ms。

但当前预览已经转为后台 debounce，这些数字不再直接代表 UI 卡顿。现有 [bench_large_doc.rs](/C:/Coding/EditorProjects/markion/examples/bench_large_doc.rs:1) 又明确不测 GPUI 布局和绘制，因此下一版基准必须覆盖真实窗口事件。

可以设定明确门槛，例如：

- 100 KiB：连续输入 p95 input-to-paint ≤ 16 ms；
- 500 KiB：p95 ≤ 33 ms；
- 1 MiB：不冻结、滚动稳定、内存增长有界。

## 更值得补齐的产品能力

最有实际收益的不是协作或插件，而是以下三个闭环：

1. **可靠写作闭环**：原子保存、外部文件冲突、最近文件、会话恢复、全部 recovery 管理。目前保存主要使用直接 `fs::write`，崩溃或磁盘异常时不如“临时文件 + flush + rename”可靠。

2. **工作区闭环**：Open Folder、快速打开、全局搜索、文件监听、命令面板。当前文件树根目录依赖已打开文件的父目录，查找也主要局限于当前文档。

3. **内容资产闭环**：图片粘贴/拖入、相对路径管理、资源缺失提示、重命名联动。对 Markdown 写作者而言，这通常比新增更多语法扩展更能提升日常效率。

数学高质量渲染、富图片/PDF 导出也值得做，但应根据目标用户决定：如果偏技术写作就提前；如果偏普通笔记，则先完成 Visual Edit 和图片资产工作流。

## 建议的实际执行顺序

1. 收敛、归档并重新跟踪 OpenSpec；同步文档。
2. 给 CI 增加 `fmt --check`、clippy 和 `cargo test --workspace`。
3. 创建 `large-document-performance-v2` 变更，只做基准、观测和性能预算。
4. 重新评估 Visual Edit 的 AST/source-range 方案，并把 27 个任务拆成可独立交付的 3～4 个里程碑。
5. 完成 Visual Edit 最小版本。
6. 推进文件可靠性、会话恢复和工作区搜索。
7. 再做数学、图片和富导出。

本轮只进行了只读分析，没有修改文件。另：Unterm GUI 当前未运行；若下一轮需要我在可见终端中继续操作，请先启动 Unterm。