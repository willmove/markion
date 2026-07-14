# Typune → Markion 整合方案（深入分析）

> **状态**：建议稿，尚未改动任何代码。
> **日期**：2026-07-07
> **范围**：深入分析 `C:\Coding\EditorProjects\typune` 与本项目（Markion，`C:\Coding\EditorProjects\markion`）的整合方式，给出推荐方案、分阶段路线图、冲突解决办法与风险控制策略。
> **方法**：本结论来自对两个仓库源码的逐文件阅读与实测验证。所有版本号、代码规模、文件路径、测试数量均来自实际仓库，非臆测。

---

## 0. TL;DR（先看结论）

- **以 Markion 为宿主**，把 Typune 当作"高质量零件库"逐个吸收。Typune 的 UI 层和顶层应用**直接舍弃**（理由见 §3.1）。
- **整合顺序按"收益/风险比"从高到低**：`export` 引擎 → syntect 语法高亮 → filesystem（TOML 配置 + tracing 日志）→（评估后）增量解析/渲染树/ropey。
- **每一步都保持 Markion 可编译、可运行、测试全绿**，每一步走一个 OpenSpec change。
- **绝不破坏 Markion 的四条性能不变量**（§5.2），任何侵入解析/编辑路径的改动都要挂在现有缓存接口后面。
- Typune 真正值得搬运的资产约 **1.2–1.5 万行可独立编译、测试全绿的纯 Rust 代码**；其余（UI shim、顶层 bin、dormant desktop 渲染器、Kiro 任务遗留物）一律丢弃。

---

## 1. 背景与目标

### 1.1 为什么整合

Markion 与 Typune 都是 Rust + GPUI 的桌面 Markdown 编辑器，功能目标高度重叠：源码/预览编辑、大纲、文件树、搜索替换、主题、聚焦/打字机模式、自动保存与恢复、多格式导出。两个项目各自从零实现了大量相同的底层能力（Markdown 解析、高亮、导出、配置、文件树），重复维护成本高。目标是把两者整合为**一个**项目，取长补短，避免双线维护。

### 1.2 整合成功的标准（验收总纲）

1. 整合完成后，仓库里只剩**一个**可发布的桌面应用（即 Markion），Typune 不再作为独立产品存在。
2. Markion 的所有现有功能（14 主题、i18n、视图模式、大纲、文件树、搜索替换、聚焦/打字机、自动保存/恢复、8 格式导出）不退化，且在吸收 Typune 后**显著增强**（真实语法高亮、更高质量的导出）。
3. Markion 的四条性能不变量（§5.2）在打字路径上仍然成立。
4. Typune 吸收进来的每个模块都带有其原有单元测试，且在 Markion 内全绿。
5. 全程遵循 OpenSpec：先提案后实施，每阶段一个 change。

---

## 2. 两个项目的现状（实测对比）

### 2.1 总览

| 维度 | Markion（本项目） | Typune |
|---|---|---|
| **代码规模** | ~1.45 万行，单 crate | ~4.5 万行，Cargo workspace（5 个库 crate + 顶层 bin） |
| **edition** | **2024** | **2021** |
| **GPUI** | 0.2.2（crates.io，真实接入） | 0.2.2（仅顶层 bin 真实接入；`ui` crate 用占位 shim） |
| **pulldown-cmark** | **0.13.4**（支持 super/sub/highlight/math 事件） | **0.11**（features: `simd`） |
| **UI 状态** | **完整可运行的 GPUI 应用**：菜单栏、分栏视图、偏好面板、文件树、大纲、查找替换全部接线完成 | **UI 是空壳**：`ui` crate 基于占位 `gpui_shim.rs`；顶层 `src/main.rs` 只是约 300 行的最小 GPUI 窗口 |
| ⚠️ **关键事实** | — | Typune 的 `main.rs` **只渲染原始缓冲文本行 + `│` 光标指示符**，完全忽略了自己精心设计的 `RenderedDocument` 渲染树、主题、高亮、数学渲染。即 Typune 从未以"格式化视图"形态运行过。 |
| **编译状态** | `cargo check` 通过 | 整个 workspace 编译失败（`editor` crate 经 `reqwest` 依赖 openssl-sys）；但 **`markdown` 与 `export` crate 可独立编译** |
| **测试** | 105 个 `#[test]`，可运行 | ~1935 个 `#[test]`；`markdown` + `export` 两个 crate 约 350 个测试**全部通过** |
| **活跃度** | **活跃**：OpenSpec 规格驱动，近期持续提交 | **停滞**：8 个提交，Kiro 规格批量生成后未再演进 |
| **许可证** | 无 LICENSE 文件（README 声明 MIT） | README 声称 MIT/Apache-2.0，但 **LICENSE 文件缺失** |

### 2.2 能力逐项对比

| 能力 | Markion | Typune | 谁更强 |
|---|---|---|---|
| 视图模式 | Source / Split / Preview（三种） | Live Preview / Source Mode（单面实时预览） | 各有侧重；Typune 的单面 live preview **仅库逻辑，未接线** |
| Markdown 解析 | pulldown-cmark 0.13，自研 PreviewBlock 模型，按版本缓存派生状态（`Arc` 共享） | pulldown-cmark 0.11，独立 AST（`ast.rs`），**块级增量解析**（`incremental.rs`，`Arc` 零拷贝复用未变区域）+ 渲染缓存 | Typune 的增量解析 + 渲染树理论上更强（针对 10k+ 行 60 FPS），但**未在真实 UI 中验证** |
| 语法高亮 | 自研 `highlight.rs`（~470 行，标记级，53 语言，记忆化缓存上限 128） | **syntect 5 + tree-sitter**（150+ 语言，`highlight.rs` ~1100 行，含异步高亮 + 语法注册表） | **Typune 明显更强** |
| 数学公式 | `math.rs`：校验 + Unicode 可读回退（`∑∫√≤≥`、希腊字母、简单分式）；**非 KaTeX** | `math.rs`：AST 解析 + 校验；**KaTeX 未真正集成**（`_engine: ()`，返回占位 SVG） | **打平**（两边都不是真 KaTeX） |
| 表格 | GFM 表格解析/格式化 + 行列增删移动 + 预览工具栏；预览单元格**纯文本**（不渲染行内样式） | GFM 表格 AST + `table_ops.rs`（增删移动行列、校验结构） | 打平（库逻辑 Typune 略全，但都没接到真实单元格编辑 UI） |
| 大纲 | `outline()`：标题层级 + 源偏移 + 点击跳转 + 当前段高亮（无折叠） | `OutlineGenerator`：`OutlineNode` 层级 | 打平 |
| 文档统计 | 字节/字符/单词/行/标题数，按版本缓存 | 无独立实现 | **Markion** |
| 搜索替换 | 正则 + 大小写敏感，上下条导航，替换全部 | `SearchEngine`/`ReplaceEngine`，`SearchState` | 打平 |
| 撤销/重做 | `EditorSnapshot` 栈（上限 200，跳过派生缓存） | `EditHistory` + `EditGroup`（破坏性/非破坏性分组） | Typune 分组更细，但未接线 |
| 多光标/选择 | 无 | `CursorManager`/`SelectionManager` | Typune（未接线） |
| 文本缓冲 | `String` + 自研编辑操作 | **ropey** rope（大文件友好）+ 键位映射系统（`keymap.rs`，可配置快捷键） | Typune |
| 导出 | 自研 `export.rs`（~570 行）：Markdown/HTML/LaTeX/DOCX/简易 PDF/PNG/JPEG（PDF/图片为 8x8 位图文字快照） | 独立 `export` crate（~4000 行）：HTML/PDF/DOCX/LaTeX/图片各有专门引擎 + 统一 `engine.rs` 调度，219 测试全绿 | **Typune 明显更强** |
| 配置 | 自研文本格式（`storage/preferences.rs` 手写 parse/render） | **TOML**（serde 派生，`filesystem/config.rs`，含字体/自动保存/导出/键位绑定等分节） | **Typune 更规范** |
| 日志 | **无** | tracing + 按日轮转 + 7 天保留（`filesystem/logger.rs`） | **Typune**（Markion 完全缺） |
| 自动保存/恢复 | 5 秒空闲计时 + 恢复副本 + 启动恢复（已接 UI） | 8 秒默认（可配置）+ 恢复（proptest 覆盖） | Typune 更可配置，Markion 已接 UI |
| 文件树 | `storage/file_tree.rs`（~357 行）：工作区扫描、过滤、点击打开、增删改、当前文件标记 | `filesystem/file_tree.rs`（~1321 行）：功能更全 | Typune 功能更全 |
| i18n | **有**（英文/简体中文，`i18n.rs` ~1466 行，编译期校验的 `Msg` 枚举） | **无** | **Markion**（关键差异化能力） |
| 主题 | **14 个内置** + 自研 `.theme` 文件格式 + 偏好面板色板 | 6 个内置 + TOML + CSS 覆盖式自定义（`theme_definition.rs`、`css_overrides.rs`） | Markion 数量更多；Typune 格式更规范 |
| Mermaid 图表 | 无（零出现） | 无（零出现） | 都缺 |
| 多文档/标签页 | **无**（单窗口单文档） | `WindowManager`/`Tab` 模型存在但**未渲染** | 都缺（运行态） |
| 虚拟化渲染 | 无（文件树有界，编辑器非虚拟化） | `BlockLayout` + `RenderCache`（视口→块 O(log n) 映射，10k+ 行 60 FPS 目标） | Typune（未接线） |
| 打包/发布 | cargo-packager（NSIS/.app/.dmg/.deb/.AppImage）+ GitHub Actions 3 平台矩阵 | `packaging/`（dmg/zip/.desktop 脚本）+ Makefile | Markion 更完整（CI 已就绪） |
| 开发流程 | OpenSpec 规格驱动 | Kiro 规格批量生成（已停滞） | Markion |
| 图片处理 | 仅链接式 `![](url)` | `ImageHandler`（拖拽→插入）+ `ImageCache` + `ImageUploader`（reqwest 图床，可配置鉴权） | Typune（但 reqwest 引入 openssl 依赖） |

### 2.3 Typune 内部的"死代码"地图

深入阅读发现 Typune 有大量**已存在但永远不会被执行**的代码路径，整合时必须识别并丢弃：

| 路径 | 状态 | 处置 |
|---|---|---|
| `src/desktop/mod.rs` + `renderer.rs` | 基于 `winit` + `softbuffer` + `cosmic-text` 的 CPU 像素渲染；`mod.rs` 引用了不存在的 `event_loop` 模块，**无法编译**；从未被 `main.rs` 调用 | **丢弃** |
| `src/app.rs`、`src/window_manager.rs`、`src/main_window.rs` | 数据模型（CLI 参数、窗口/标签管理），标注 `#[allow(unused)]`，注释自称"GPUI integration pending" | **丢弃** |
| `src/editor/`、`src/ui/`（顶层 src 下） | 与 `crates/editor`、`crates/ui` 平行的旧桩代码，未被 `main.rs` 使用 | **丢弃** |
| `crates/ui/`（基于 `gpui_shim.rs`） | 占位 GPUI 类型（`ViewContext`/`AnyElement`/`Hsla` 假实现），整套组件库**不绑定真实 GPUI** | **丢弃**（仅参考 TOML 主题格式思路） |
| `crates/markdown/src/math.rs` 的 KaTeX 部分 | `_engine: ()`，返回占位 SVG | **丢弃**（保留 LaTeX 校验逻辑） |
| `crates/editor` 的 `image_uploader`（reqwest） | 引入 openssl-sys 硬依赖，导致 workspace 编译失败 | **改为可选 feature，默认关闭** |
| `.kiro/`、`TASK_*.md`、根目录 `test_*.rs` | Kiro 任务生成遗留物 | **丢弃** |

### 2.4 Typune 真正值得吸收的资产（精确定位）

经过筛选，Typune 中**可独立编译、测试全绿、与 UI 无关、可直接服务于 Markion** 的代码：

1. **`crates/export`**（~4000 行，219 测试）—— HTML/PDF/DOCX/LaTeX/图片多格式导出引擎，统一 `engine.rs` 调度。
2. **`crates/markdown/src/highlight.rs`**（~1100 行）—— syntect 5 + tree-sitter，150+ 语言，含异步高亮与语法注册表。
3. **`crates/markdown` 的解析/渲染树**（`parser.rs`/`ast.rs`/`incremental.rs`/`renderer.rs`/`render_cache.rs`）—— 框架无关的 `RenderedDocument`/`RenderNode`/`RenderInline` 树 + 块级增量解析 + 虚拟滚动布局。**这是 Typune 最具架构价值的资产，也是整合中最具侵入性的一步**（§6 Phase 4）。
4. **`crates/filesystem`**（除 `file_tree.rs` 外）—— TOML 配置（serde）、tracing 日志轮转、自动保存/恢复。
5. **`crates/editor/src/keymap.rs`** —— 可配置快捷键系统（与 ropey 解耦）。
6. **`docs/`**（快捷键、主题定制、构建指南、FAQ）与 **`packaging/`**（dmg/zip/.desktop）—— 几乎纯文本搬运。

---

## 3. 关键技术判断

### 3.1 为什么以 Markion 为宿主（而非反过来或新建）

三条不可辩驳的理由：

1. **Typune 从未作为"格式化 Markdown 编辑器"运行过。** 它的 `main.rs` 只画出原始文本行，忽略了自己的渲染树、主题、高亮、数学。这意味着 Typune 的 UI 与库之间的集成**从未被验证**。把 1.45 万行真实可用的 Markion UI 移植进一个从未跑通 UI 的宿主，等于在沙滩上盖楼。
2. **Markion 有 i18n，Typune 没有。** i18n 是 Markion 的硬性差异化能力（英文+简体中文，编译期校验）。整合后所有用户可见文案必须走 `i18n.rs`，而 Typune 的 UI 文案是英文硬编码。以 Markion 为宿主天然保留这个能力。
3. **Markion 有 OpenSpec 与活跃 CI/发布管道。** 整合过程需要规格约束和持续可发布，Markion 已具备，Typune 已停滞 8 提交。

**结论**：Typune 的价值是**库 crate**，不是 UI、不是顶层应用、不是项目骨架。整合 = 把 Typune 当零件库拆解吸收。

### 3.2 为什么"拷贝所需 crate"优于 `git subtree`

Typune 只有 8 个提交，历史价值有限。`git subtree add` 会把整段历史（含 Kiro 遗留物）拉进来。**推荐拷贝所需 crate + 在提交信息中注明来源 commit**，理由：

- 可顺手裁剪死代码（§2.3）；
- 避免 openssl 链接问题随 `editor` crate 进入 Markion（把 `image_uploader` 改为可选 feature）；
- 保留许可证归属即可（§5.1）。

### 3.3 双 AST / 双文档模型如何过渡

Markion 用 `model.rs::PreviewBlock` + `RichText`/`InlineSpan`；Typune 用 `ast.rs::Document` + `renderer.rs::RenderedDocument`/`RenderNode`/`RenderInline`。两者模型不同。

**过渡原则**：应用层（`MarkionApp`、菜单、UI）**始终只认 Markion 模型**。吸收 Typune crate 时写**薄适配层**把 Typune 类型翻译成 Markion 类型。直到 Phase 4（评估后）才决定是否把 Typune 的渲染树换为 Markion 的底座。

短期并存两条 pulldown-cmark 版本（0.13 与 0.11）—— Cargo 允许不同 crate 依赖同一 crate 的不同主版本，不阻塞其他阶段。

---

## 4. 整合方案对比

### 方案 A（推荐）：以 Markion 为宿主，workspace 化后按能力吸收 Typune 库 crate

- Markion 保持为应用主体（窗口、交互、i18n、OpenSpec 流程不动）。
- 把 Markion 从单 crate 改造为 Cargo workspace，现有代码平移为 `crates/markion-app`（bin crate）。
- 将 Typune 的库 crate 逐个搬入 `crates/`，以**替换 Markion 中较弱的对应模块**为目标，每次替换走一个 OpenSpec change。
- Typune 的 `ui` crate 与顶层 bin **不搬运**，只"采摘"个别设计（TOML 主题格式、keymap 配置化思路）。

**优点**：始终有一个可运行、可发布的应用；每一步都能用两边现有测试兜底；符合本仓库 AGENTS.md 的工作方式；保留 i18n 与 OpenSpec 上下文。
**缺点**：整合周期较长（分多个 change）；短期内存在双 AST/双模型的过渡态。

### 方案 B：以 Typune 为宿主，把 Markion UI 移植过去

**不推荐。** Typune 的 UI 层是 shim 空壳、顶层窗口只画原始文本、整个 workspace 目前编译不过、项目已停滞。把活跃开发的真实 UI 移植进不能编译的宿主，风险与工作量最大。

### 方案 C：新建第三个仓库，两边重写合并

**不推荐。** 工作量最大，丢掉两边 git 历史和 OpenSpec 上下文，没有额外收益。

---

## 5. 推荐路线图（方案 A 分阶段）

每个阶段对应一个（或多个）OpenSpec change，按"收益/风险比"从高到低排列。阶段之间相互独立，**可随时停在任一阶段而不留下坏状态**。

### Phase 0 — 准备（低风险，先行）

1. **确定产品名**：保留 **Markion**（活跃仓库、远端已存在）。Typune README 的宣传内容合并进 Markion README 对应条目。
2. **补 LICENSE**：两个仓库都没有 LICENSE 文件。整合前先为 Markion 添加，建议 **MIT OR Apache-2.0**（与 Typune README 声称一致），消除吸收代码时的许可疑问。
3. **workspace 化 Markion**：根 `Cargo.toml` 改为 `[workspace]`，现有代码平移为 `crates/markion-app`（bin crate）。纯机械改动，`cargo test` 全绿即完成。
   - edition 差异：Markion 是 2024，Typune crate 是 2021。workspace 成员允许各自指定 edition，**先不强行统一**。
4. **引入 Typune 代码的方式**：拷贝所需 crate 到 `crates/`，提交信息注明来源 commit。

### Phase 1 — 吸收 `export` crate（收益高、耦合低，优先）

Typune 的导出引擎（~4000 行，219 测试全绿）明显强于 Markion 的 `export.rs`（~570 行，PDF/图片为 8x8 位图文字快照）。

1. 拷入 `crates/export`，让 `markion-app` 依赖它。
2. 写薄适配：Markion 文档模型 → export crate 输入类型（它接收 Markdown AST/字符串，接口在 `engine.rs`）。
3. 逐格式切换：先 HTML/LaTeX（最易对照验证），再 DOCX/PDF/图片。每切换一个格式，删除 `export.rs` 中对应的旧实现。
4. **验收**：Markion 现有导出相关单测改指向新引擎后全绿；人工对比几份样例文档（含表格/代码/数学/front matter）的导出产物。

### Phase 2 — 吸收 syntect 语法高亮（用户可感知的最大提升）

Markion 自研高亮只覆盖常见 token；Typune 的 `highlight.rs` 基于 syntect（150+ 语言）。

1. 把高亮模块抽成独立 `crates/highlight`（**避免过早引入第二套 AST**，不要整个搬 `crates/markdown`）。
2. 保留 Markion 现有的"按文档版本记忆化"缓存策略，把 syntect 作为底层实现**替换进 `src/highlight.rs` 的接口后面**。
3. **启动成本**：syntect 语法集加载较慢，需惰性初始化 + 放到后台线程，避免破坏打字路径性能（不变量 §5.2）。
4. **验收**：现有高亮测试改写后全绿；大文档打字无可感知卡顿（建议用 10k 行文档做手测）。

### Phase 3 — 吸收 `filesystem` 的配置/日志/自动保存

1. **配置**：用 Typune 的 TOML 配置（serde 派生）替换 Markion 手写偏好格式。写一次性迁移：启动时若发现旧格式偏好文件则读取并转存为 TOML。Markion 偏好项（主题、语言、聚焦/打字机、行号）并入 Typune 配置结构，同时保留 Typune 的字体/自动保存延迟/键位绑定分节作为后续功能落点。
2. **日志**：引入 tracing + 轮转（Markion 目前完全无日志，这是纯增量）。
3. **自动保存/恢复**：两边都有且 Markion 已接 UI。对比择优——Typune 的 `auto_saver.rs` 更完善（可配置延迟），以 Markion 的行为语义为准（5 秒 vs 8 秒默认值在配置里解决）。
4. `filesystem/file_tree.rs`（~1321 行）与 Markion `storage/file_tree.rs`（~357 行）功能重叠：**短期各用各的，长期以功能更全的一方为准**（独立 change 评估）。
5. **验收**：旧偏好文件能正确迁移；日志按预期轮转；自动保存/恢复行为与迁移前一致。

### Phase 4 — 评估性引入：渲染树 / 增量解析 / ropey（收益大但侵入性强，放最后）

⚠️ 这是整个整合中**最大、最需要单独 design.md 的一步**。是否执行取决于 Phase 0–3 完成后 Markion 是否出现性能瓶颈。

1. **渲染树**（`markdown/renderer.rs` 的 `RenderedDocument`/`RenderNode`/`RenderInline`）：Typune 最具架构价值的资产，框架无关、已含 `RenderTheme` 全解析样式。两条路：
   - **短期不动**：Markion 现有 `PreviewBlock` + 按版本缓存已够用。
   - **长期**：把 Typune `markdown` crate 升级到 pulldown-cmark 0.13，作为 Markion 的解析/渲染层；Markion `parse.rs`/`model.rs` 改为消费它的 AST/渲染树。需独立 change + design.md。
2. **增量解析**（`markdown/incremental.rs`）：块级增量 + `Arc` 区域复用，对大文档打字性能是质变。但绑定 Typune AST 与 pulldown-cmark 0.11。**过渡期两版本并存不阻塞其他阶段**。
3. **ropey 缓冲**：仅当 Markion 出现大文件编辑性能问题时再引入，否则不动（`String` + 现有缓存在当前规模下已验证够快）。
4. **keymap 配置化**（`editor/keymap.rs`）：作为独立功能引入（用户自定义快捷键），与 ropey 解耦。
5. ⚠️ **openssl 硬依赖**：`editor` crate 经 reqwest（图床上传）依赖 openssl-sys。搬运时把 `image_uploader` 挪到 `[features]` 可选特性，**默认关闭**，消除硬依赖。

### Phase 5 — 采摘杂项 + 退役 Typune

1. **主题**：保留 Markion 14 主题与 `.theme` 机制；参考 Typune TOML 主题定义把 `.theme` 升级为 TOML（一次性迁移脚本）。CSS 覆盖机制仅在预览走 HTML 渲染时才有意义，**暂不引入**。
2. **文档与打包**：Typune `docs/`（快捷键、主题定制、构建指南、FAQ）和 `packaging/`（dmg/zip/.desktop）拷入并改名为 Markion。纯文本搬运，收益立得。
3. **不搬运（直接舍弃）**：`crates/ui` 全部（shim 空壳）、Typune 顶层 `src/`（最小窗口雏形、`error_notification.rs`、dormant `desktop/`）、`src/app.rs`/`window_manager.rs`/`main_window.rs` 数据模型、`.kiro`、`TASK_*.md`、根目录 `test_*.rs`。
4. 全部完成后**归档 Typune 仓库**（README 顶部注明 "merged into markion" + 指向 Markion 仓库与本文档）。

---

## 6. 技术冲突与解决办法汇总

| 冲突点 | 现状 | 解决办法 | 影响阶段 |
|---|---|---|---|
| pulldown-cmark 版本 | Markion 0.13 / Typune 0.11 | 过渡期并存（不同 crate 可依赖不同主版本）；Phase 4 统一到 0.13 | Phase 4 |
| 双 AST/文档模型 | Markion `model.rs` / Typune `ast.rs`+`renderer.rs` | 应用层始终只认 Markion 模型；吸收时写薄适配层；Phase 4 再决定是否换底 | 全程 |
| edition | Markion 2024 / Typune 2021 | workspace 成员各自声明，不强行统一 | Phase 0 |
| 配置格式 | Markion 自研文本 / Typune TOML | 采用 TOML，启动时自动迁移旧文件 | Phase 3 |
| 主题格式 | `.theme` 自研 / TOML+CSS | 采用 TOML 定义，迁移现有 14 主题；CSS 覆盖暂缓 | Phase 5 |
| openssl 硬依赖 | Typune `editor` → reqwest | 图床上传改为可选 feature，默认关闭 | Phase 4 |
| i18n | 仅 Markion 有 | 保留 Markion 方案；吸收模块产生的用户可见文案一律走 `i18n.rs` | 全程 |
| 许可证 | 双方均无 LICENSE 文件 | Phase 0 补齐（MIT OR Apache-2.0） | Phase 0 |
| 缓存策略差异 | Markion 按版本缓存 / Typune 渲染缓存 | 任何侵入解析/编辑路径的 Typune 代码都挂在 Markion 现有缓存接口**后面**，不绕开 | Phase 2/4 |
| 视图模式语义 | Markion 三模式 / Typune 单面 live preview | 保留 Markion 三模式；Typune 单面 live preview 作为**可选第四模式**评估（独立 change） | Phase 4 后 |

---

## 7. 风险与不变量保护

### 7.1 风险清单

1. **Typune 代码未经真实运行验证**（UI 层是 shim，应用从未以格式化视图跑起来）。其库 crate 质量证据主要来自单测。**吸收每个模块必须连同测试一起搬入，并补端到端验证**。
2. **openssl 链接问题可能潜伏在多个 crate**。Phase 0 workspace 化后立即跑一次全量 `cargo build`，确保没有隐藏的系统库依赖污染 CI。
3. **双 AST 过渡期容易引入不一致 bug**（例如导出走 Typune AST、预览走 Markion PreviewBlock，两者对同一文档的解析结果细微不同）。对策：每个格式/能力的切换都做**人工对照验证**（同一份样例文档，新旧实现产物 diff）。
4. **整合周期长，中途上下文丢失**。对策：每阶段独立 OpenSpec change，change 的 tasks.md 自带验收清单；本文档作为总纲长期保留。

### 7.2 Markion 四条性能不变量（必须全程保护）

以下四条来自 `AGENTS.md` 与 `openspec/config.yaml`，是 Markion 打字路径性能的根基。任何侵入解析/编辑/渲染路径的整合改动都必须遵守：

1. **派生状态按文档版本缓存，经 `Arc` 共享** —— `MarkdownDocument` 持有 `text_version: u64` + 四个缓存（`cached_preview_blocks` 返回 `Arc<Vec<PreviewBlock>>`，`cached_outline`/`cached_stats`/`cached_line_count`）。访问器在 `version == text_version` 时复用，否则重算。**Phase 4 引入 Typune 渲染树时，必须复用这套版本缓存，不能每键重算。**
2. **语法高亮跨编辑记忆化** —— `MarkionApp.highlight_cache` 按 `(language, code)` 键，上限 128。**Phase 2 引入 syntect 时，syntect 是底层实现，缓存策略不变。**
3. **编辑器按版本复用文本 handle** —— `display_text_cache: (u64, SharedString)`，克隆是 Arc 提升。**Phase 4 若引入 ropey，需保证 SharedString 仍能低成本产出（ropey → String → SharedString 的开销要评估）。**
4. **撤销快照跳过派生缓存** —— 手动 `Clone` 只复制 `text`/`path`/`dirty`/`text_version`，四个缓存置 `None`，保证 200 深撤销栈低开销。**任何新增缓存字段都要在 Clone impl 中置空。**

### 7.3 OpenSpec 合规

本仓库规定"先提案后实施"。本文档只是**整体路线建议**；动手前按 `/openspec:propose` 为每个 Phase 建 change（Phase 4 需要额外 `design.md`）。每阶段完成后按 `/openspec:archive` 归档并同步 delta specs。**不要直接编辑 `openspec/specs/`**。

---

## 8. 工作量估算（粗略）

> 仅为量级估计，实际以各 change 的 tasks.md 为准。

| 阶段 | 主要工作 | 估计工作量 | 风险 |
|---|---|---|---|
| Phase 0 | workspace 化 + LICENSE + 拷贝机制 | 小（半天–1 天） | 低 |
| Phase 1 | 吸收 export crate + 适配 + 逐格式切换 | 中（2–4 天） | 低 |
| Phase 2 | syntect 高亮替换 + 后台加载 + 缓存接口对接 | 中（2–3 天） | 中（启动性能） |
| Phase 3 | TOML 配置 + 迁移 + tracing 日志 + 自动保存择优 | 中（2–3 天） | 低–中（迁移正确性） |
| Phase 4 | 渲染树/增量解析/ropey（评估后决定） | 大（1–2 周+） | 高（最大侵入） |
| Phase 5 | 主题 TOML 化 + 文档/打包搬运 + 退役 Typune | 小–中（1–2 天） | 低 |

**最小可用整合（Phase 0–3）**：约 1.5–2 周，即可获得"真实语法高亮 + 高质量导出 + TOML 配置 + 日志"的显著增强版 Markion，且全程可发布。Phase 4–5 可按需推进。

---

## 9. 决策清单（开工前需确认）

1. **产品名**：保留 Markion？（推荐：是）
2. **许可证**：MIT OR Apache-2.0？（推荐：是，与 Typune README 一致）
3. **整合触发条件**：是否 Phase 0–3 必做、Phase 4–5 视性能瓶颈而定？（推荐：是）
4. **Typune 来源 commit**：拷贝时记录哪个 commit 作为来源基准？（需在 Typune 仓库 `git log` 确认最新 commit）

---

## 10. 附录：关键文件路径速查

### Markion（宿主）

| 文件 | 作用 |
|---|---|
| `Cargo.toml` | 单 crate 清单（gpui 0.2.2、pulldown-cmark 0.13.4、edition 2024） |
| `src/main.rs`（~5848 行） | 应用引导、窗口、菜单、**全部 UI 渲染与输入**；`MarkionApp` 结构体；`EditorElement` 自定义 GPUI Element |
| `src/lib.rs`（~3264 行） | 模块根 + `MarkdownDocument`（解析/编辑/导出驱动/版本化派生状态缓存） |
| `src/model.rs` | 领域类型：`PreviewBlock`/`RichText`/`InlineSpan`/`ExportFormat`/`ThemeDefinition`/`AppPreferences`/`ViewMode` |
| `src/i18n.rs`（~1466 行） | `Language`/`Msg`/`t()`/`tf()`，编译期校验翻译 |
| `src/highlight.rs` | 自研标记级高亮（Phase 2 替换目标） |
| `src/export.rs` | 简易导出（Phase 1 替换目标） |
| `src/parse.rs`/`render.rs` | Markdown→PreviewBlock / Markdown→HTML/LaTeX |
| `src/storage/` | 持久化（preferences/recovery/theme_file/file_tree） |
| `openspec/specs/` | 8 个能力规格（系统现状，只通过归档 change 更新） |

### Typune（零件库，按价值排序）

| 文件 | 作用 | 处置 |
|---|---|---|
| `crates/export/`（~4000 行，219 测试） | 多格式导出引擎 | **Phase 1 吸收** |
| `crates/markdown/src/highlight.rs`（~1100 行） | syntect 高亮 | **Phase 2 吸收** |
| `crates/markdown/src/{parser,ast,incremental,renderer,render_cache}.rs` | 渲染树 + 增量解析 | **Phase 4 评估吸收** |
| `crates/filesystem/src/{config,logger,auto_saver,recovery}.rs` | TOML 配置/日志/自动保存 | **Phase 3 吸收** |
| `crates/editor/src/keymap.rs` | 可配置快捷键 | **Phase 4 评估吸收** |
| `crates/ui/`（gpui_shim 空壳） | — | **丢弃** |
| `src/main.rs`（~300 行最小窗口） | 顶层 bin | **丢弃** |
| `src/{app,window_manager,main_window,desktop}.rs` | dormant/死代码 | **丢弃** |
| `docs/`、`packaging/` | 文档与打包脚本 | **Phase 5 改名搬运** |

---

*本文档为整合总纲，不替代各阶段的 OpenSpec change 提案。实施时以对应 change 的 `proposal.md`/`tasks.md`/`design.md` 为准。*
