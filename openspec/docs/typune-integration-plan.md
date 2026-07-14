# Typune → Markion 整合方案

> 状态：建议稿（尚未改动任何代码）
> 初稿：2026-07-05 · 修订：2026-07-06（新增 Workspace 化利弊分析；按仓库最新状态更新——LICENSE 已补、cargo-packager 发布 CI 已建立、`.kiro` 遗留规格已删除）
> 范围：分析 `/root/projects/typune` 与本项目（Markion）的整合方式，给出推荐方案与分阶段路线图。

## 1. 背景

Markion 与 Typune 都是 Rust + GPUI 的桌面 Markdown 编辑器，功能目标高度重叠（源码/预览编辑、大纲、文件树、搜索替换、主题、聚焦/打字机模式、自动保存与恢复、多格式导出）。目标是把两者整合为**一个**项目，避免重复维护。

## 2. 两个项目的现状（实测结论）

以下结论来自对两个仓库的代码阅读和在本机的实际编译/测试验证（2026-07-05，表格中 Markion 侧已按 2026-07-06 仓库状态更新）。

| 维度 | Markion（本项目） | Typune |
|---|---|---|
| 代码规模 | 约 1.45 万行，单 crate | 约 4.5 万行，workspace（5 个库 crate + 顶层 bin） |
| UI | **真实可运行的 GPUI 0.2.2 应用**，菜单栏、分栏视图、偏好面板等全部接线完成 | `ui` crate **基于占位 shim**（`gpui_shim.rs`），不依赖真实 GPUI；仅最后一个提交在顶层 `src/main.rs` 里加了一个约 300 行的最小 GPUI 窗口 |
| 编译状态 | `cargo check` 通过 | 整个 workspace 在本机编译失败（`editor` crate 经 `reqwest` 依赖 openssl-sys，缺系统库）；但 **`markdown` 与 `export` crate 可独立编译** |
| 测试 | 105 个 `#[test]`，可运行 | 约 1935 个 `#[test]`；`markdown` + `export` 两个 crate 约 350 个测试**全部通过** |
| Markdown 引擎 | pulldown-cmark 0.13，自研解析/预览模型，按文档版本缓存派生状态（`Arc` 共享） | pulldown-cmark 0.11，独立 AST（`ast.rs`），**块级增量解析**（`incremental.rs`，`Arc` 零拷贝复用未变区域）+ 渲染缓存 |
| 语法高亮 | 自研 `highlight.rs`（约 470 行，标记级高亮，有记忆化） | **syntect 5 + tree-sitter**（150+ 语言，`highlight.rs` 约 1100 行） |
| 导出 | 自研 `export.rs`（约 570 行）：Markdown/HTML/LaTeX/DOCX/简易 PDF/PNG/JPEG | 独立 `export` crate（约 4000 行）：HTML/PDF/DOCX/LaTeX/图片各有专门引擎 + 统一 `engine.rs` 调度 |
| 文本缓冲 | `String` + 自研编辑操作 | **ropey** rope 结构（大文件友好）+ 键位映射系统（`keymap.rs`，支持配置化快捷键） |
| 配置 | 自研文本格式（`storage/preferences.rs` 手写 parse/render） | **TOML**（serde，`filesystem/config.rs`，含字体、自动保存、导出、键位绑定等分节） |
| 日志 | 无 | tracing + 按日轮转（`filesystem/logger.rs`） |
| i18n | **有**（英文/简体中文，`i18n.rs` 约 1300 行） | 无 |
| 主题 | 14 个内置主题 + 自研 `.theme` 文件格式 | 6 个内置主题 + TOML + CSS 覆盖式自定义主题（`theme_definition.rs`、`css_overrides.rs`） |
| 打包/发布 | **有完整发布流水线**：cargo-packager（`packager.toml`）+ GitHub Actions（`.github/workflows/release.yml`），产出 deb/AppImage/dmg(.app)/NSIS 安装包；logo 与各平台图标资源已就位（`assets/`，Windows 图标经 `build.rs` 嵌入） | `packaging/` 下是手写脚本（dmg 脚本、Windows zip、Linux .desktop），**弱于 Markion 现有流水线** |
| 用户文档 | README 功能清单 | `docs/`（快捷键、主题定制、构建指南、FAQ） |
| 开发流程 | **活跃**：OpenSpec 规格驱动，持续提交（品牌资产、发布 CI、文件树改进均为近日新增） | 停滞：8 个提交，Kiro 规格批量生成后未再演进 |
| 许可证 | **MIT**（`LICENSE` 文件已存在，`Cargo.toml` 已声明） | README 声称 MIT OR Apache-2.0，但 LICENSE 文件缺失 |

### 关键判断

1. **Markion 是"能跑的产品"，Typune 是"高质量的零件库 + 不能用的 UI"。** Typune 的 `ui` crate 写在 GPUI shim 之上，整套组件（按钮、对话框、面板、偏好界面）都不是真实 GPUI 代码，作为 UI 无法直接使用；而 Markion 的整个交互层是真实接线、可运行、有活跃规格约束的，且已具备完整的跨平台发布流水线。
2. **Typune 的价值集中在与 UI 无关的库 crate**：`markdown`（增量解析、syntect 高亮、AST、表格操作、数学公式）、`export`（多格式导出引擎）、`filesystem`（TOML 配置、tracing 日志、自动保存/恢复）、`editor`（ropey 缓冲、keymap）。其中 `markdown` 与 `export` 已实测可独立编译、测试全绿。
3. 两者 GPUI 版本一致（0.2.2），这为渐进整合扫清了最大障碍。

## 3. 方案对比与推荐

### 方案 A（推荐）：以 Markion 为宿主，按能力吸收 Typune 的库 crate

- Markion 保持为应用主体（窗口、交互、i18n、发布流水线、OpenSpec 流程全部不动）。
- 在现有 `Cargo.toml` 里加一个 `[workspace]` 段（**根包 workspace**，Markion 包留在根目录，详见 §4），把 Typune 的库 crate 按需放入 `crates/`，以**替换 Markion 中较弱的对应模块**为目标，每次替换走一个 OpenSpec change。
- Typune 的 `ui` crate、顶层 bin 和 `packaging/` 不搬运，只"采摘"个别设计（TOML 主题格式、keymap 配置化思路）。

**优点**：始终有一个可运行、可发布的应用；每一步都能用两边现有测试兜底；符合本仓库 AGENTS.md 的工作方式。
**缺点**：整合周期较长（分多个 change）；短期内存在双 AST/双模型的过渡态。

### 方案 B：以 Typune 为宿主，把 Markion 的 UI 移植过去

Typune 架构骨架（workspace 分层）更好看，但它的 UI 层是空壳、顶层窗口只是雏形、项目已停滞、且整个 workspace 目前编译不过。把 1.4 万行活跃开发的真实 UI（外加 Markion 已建好的发布流水线）移植进一个不能编译的宿主，风险和工作量都最大。**不推荐。**

### 方案 C：新建第三个仓库，两边重写合并

工作量最大，丢掉两边的 git 历史和 OpenSpec 上下文，没有额外收益。**不推荐。**

## 4. Workspace 化的利弊分析：不是必须，推荐"根包"最小形态

整合 Typune 的代码，真正需要的是"能引入独立的 package"，Workspace 只是实现它的一种方式。本节展开利弊，并给出比初稿更轻的落地形态。

### 4.1 好处（结合本项目实际）

1. **库 crate 可以脱离 GUI 独立编译和测试——这是本项目里已被验证的最大收益。** Typune 的 `markdown`/`export` crate 在本机 13 秒编译完、350 个测试全绿，而任何碰到 gpui 的东西都要拖上 wayland/x11 系统库、编译几百秒。Markion 目前是单 crate，跑 `parse.rs` 的一个小测试也必须完整链接 gpui 应用。拆出核心库后，`cargo test -p markdown` 的迭代是秒级的，CI 也能在没有 GUI 系统库的环境里跑核心逻辑测试。
2. **依赖版本可以并存，且 Typune 代码一行不改。** `pulldown-cmark` 0.11（Typune）和 0.13（Markion）作为两个 package 的依赖可以在同一构建里共存。单 crate 里靠依赖重命名（`pulldown_cmark_old = { package = "pulldown-cmark", version = "0.11" }`）虽也能做到，但要求改写 Typune 所有 `use pulldown_cmark::...`，连带测试文件——而"原样保留 Typune 的约 1935 个测试作为安全网"正是整合策略的支柱。
3. **增量编译范围收窄。** 现在改 `lib.rs` 任何一行都重编整个 14.5k 行的 crate；拆分后改应用层不重编解析/导出库，反之亦然，且多 crate 可并行编译。
4. **边界由编译器强制。** "UI 不许伸手进解析器内部状态"从口头约定变成编译错误，对一个准备吸收 4.5 万行外来代码的项目，这个约束的价值随时间增长。

### 4.2 坏处和成本

1. **跨 crate 就要有公共 API，孤儿规则开始咬人。** 模块间随便调私有函数的日子结束。具体的坑：若把 `model.rs` 的类型下沉到核心 crate，就不能在应用 crate 里为它们实现 gpui 的 trait（外部 trait + 外部类型违反孤儿规则），只能让核心 crate 依赖 gpui（层次白分）或到处包 newtype。**切分线必须画对：所有碰 gpui 的类型留在应用 crate。** crate 间禁止循环依赖（模块间允许），有些现在耦合着的东西可能拆不干净，切错了返工很贵。
2. **Markion 还是内部结构频繁变动的原型。** `main.rs` 5.8k 行、近日仍在快速改动（文件树行为、品牌接入），过早固化边界意味着之后每次跨 crate 线的重构都要动可见性和多份 `Cargo.toml`。
3. **文档与流程 churn。** AGENTS.md 明确写着 "single crate, no workspace"，OpenSpec 各能力 spec、README 构建说明需要跟着改。若按初稿把 `src/` 搬到 `crates/markion-app/`，还会留一个大 rename 提交污染 git blame，并且 `cargo packager`/release CI 默认作用于根包，虚拟 workspace 需要额外改造发布配置。
4. **一个容易被忽略的 profile 细节。** Markion 靠 `[profile.dev.package."*"] opt-level = 2` 保证依赖（尤其 gpui）在 dev 构建里被优化，但 `package."*"` **不覆盖 workspace 成员**——Typune 的 crate 一旦成为成员，dev 下就掉到 opt-level 1。syntect 高亮、增量解析这类计算密集代码需要逐个补 `[profile.dev.package.markdown]` 式的显式覆盖，否则 `cargo run` 的打字手感可能变差。
5. **跨 crate 内联损失。** dev 构建下非泛型函数跨 crate 默认不内联。打字路径上的调用多是大颗粒函数（整段 parse/highlight），实际影响很小；release 已配置 `lto = "thin"`，无碍。但 AGENTS.md 把打字性能列为不变量，实施时留意即可。

### 4.3 不做 Workspace 的替代路径

- **路径 A：模块合并。** 把 Typune 的导出引擎、syntect 高亮直接抄成 `src/` 下的模块。对 Phase 1/2 完全可行（这两块本来就要写适配层对接 Markion 的模型）。代价：Typune 的测试要跟着改写、依赖版本冲突靠重命名 trick、单编译单元继续膨胀。适合"只要 syntect 和导出引擎，其他都不要"的最小整合。
- **路径 B（推荐）：根包 Workspace。** 在现有 `Cargo.toml` 里加：

  ```toml
  [workspace]
  members = ["crates/*"]
  ```

  Markion 包留在根目录，`src/` 一个文件都不动，`cargo run`/`cargo test`/`cargo packager` 与发布 CI 行为全部不变，AGENTS.md 只需改一句话。Typune 的 crate 按需放进 `crates/`，共享同一个 `Cargo.lock` 和 `target/`。这拿到 §4.1 的第 1、2、3 条好处，几乎不付 §4.2 第 2、3 条成本——因为**没有拆 Markion 自己**，只是给外来代码一个户口。

### 4.4 结论

- **必须吗？不必须。** 只做 Phase 1/2 的话，模块合并也能走通。
- **值得吗？值得，但用最小形态。** 根包 workspace 成本接近零，却保住"Typune 代码和测试原样进来"这个最重要的安全网，以及核心逻辑脱离 GUI 测试的能力。
- **把 Markion 应用层自身拆成多 crate（虚拟 workspace + `crates/markion-app`）是另一个独立决策**，只有当应用层大到需要分层时才值得做，与本次整合无关，初稿中相应建议已撤回。

## 5. 推荐路线图（方案 A 分阶段）

每个阶段对应一个（或多个）OpenSpec change，实施顺序按"收益/风险比"从高到低排列。阶段之间相互独立，可以随时停在任何一个阶段而不留下坏状态。

### Phase 0 — 准备（低风险，先行）✅ 已实施（2026-07-06）

> 实施记录：OpenSpec change `adopt-root-package-workspace`。根包 `[workspace]` 已加入 `Cargo.toml`（含成员 dev-profile 覆盖的注释预留，另实测发现 `members` glob 要求 `crates/` 目录存在，已用 `.gitkeep` 占位）；AGENTS.md 的 "single crate" 表述已替换为根包 workspace 布局与两条结构不变量；命名（保留 Markion）、引入方式（拷贝 + 注明来源 commit）、许可（按 Typune 的 MIT 选项并入）三项决策记录于该 change 的 proposal。`cargo check`/`cargo test` 回归通过。

1. **确定产品名**：建议保留 **Markion** 作为项目/产品名（活跃仓库、品牌资产与发布流水线已就位）；Typune README 宣传内容合并进 Markion README 的对应条目。
2. **引入 Typune 代码的方式**：直接**拷贝所需 crate 进 `crates/` 并在提交信息中注明来源 commit（`0b9e313`）**。Typune 历史只有 8 个提交，`git subtree` 的历史价值有限，拷贝可以顺手裁剪。
3. **根包 workspace**：按 §4.3 路径 B，在现有 `Cargo.toml` 加 `[workspace]` 段，`src/` 不动；同步更新 AGENTS.md 中 "no separate workspace" 的表述和 OpenSpec 相关 spec。为后续搬入的计算密集 crate 预留 `[profile.dev.package.<name>]` 覆盖（见 §4.2 第 4 条）。
4. ~~补 LICENSE~~ **已完成**：Markion 已有 MIT `LICENSE` 且 `Cargo.toml` 已声明。剩余动作：吸收 Typune 代码时在文件头或 NOTICE 中注明来源（Typune 自称 MIT OR Apache-2.0 双许可但无 LICENSE 文件，按其 MIT 选项并入 MIT 项目无障碍；两仓库同属一个作者，风险实际为零）。

### Phase 1 — 吸收 `export` crate（收益高、耦合低）✅ 已实施（2026-07-06，范围有修正）

> 实施记录：OpenSpec change `absorb-typune-export-engine`。**实施调研发现原计划的一个误判**：Typune 的 PDF/DOCX/图片导出器是 **pandoc（PDF 还需 xelatex）/ wkhtmltoimage 子进程封装**，并非原生 Rust——只有 HTML/LaTeX 是原生实现。外部工具不能假设最终用户装了，整体替换会把无 pandoc 用户从"导出质量差"退化成"导不出"。因此实际落地为：
>
> - `crates/markdown` + `crates/export` 原样拷入（来源 Typune@`0b9e313`，`export` 的输入类型是 `markdown::Document` 所以两个一起来；含全部测试）。根包以重命名依赖 `typune-markdown`/`typune-export` 引用（markion 自身有 `mod export`，同名直接依赖会产生 E0659 歧义）。
> - **PDF/DOCX：引擎优先、静默回退**——先走 pandoc 引擎，任何失败（pandoc 缺失、xelatex 缺失、转换出错）回退到 Markion 内置实现，导出永不失败。已端到端验证：有 pandoc 时 DOCX 为真实 pandoc 产物；pandoc 输出 PDF 到 stdout 的管道协议实测可行（pdfroff 引擎验证）；无 pandoc 时（清空 PATH 模拟）两格式均正确回退。
> - **HTML/LaTeX/图片：维持 Markion 原生路径**——Markion 的实现全保真且深度集成（数学标注、front matter、高亮 CSS），Typune 原生 HTML/LaTeX 输出不同但无明确增益，切换推迟再评估；其图片导出依赖已弃维护的 wkhtmltoimage，不接。（2026-07-06 晚已完成对照评估：确认维持 Markion 原生；Typune LaTeX 的行内样式/表格对齐/lstlisting 三项优势列为待移植项——详见 `docs/typune-integration-audit.md` §P2a/P2c。）
> - 测试适配两处：PDF 魔数断言放宽为 `%PDF-` 前缀（引擎产物版本号更高）；DOCX 元数据检查测试改为直接调用内置写入器（引擎产物是 deflate 压缩包，原始字节不可见 XML）。全 workspace 556 测试通过、0 失败（Markion 108 + 成员约 448）。注意根包 workspace 下 `cargo test` 只跑根包，全量需 `cargo test --workspace`（AGENTS.md 已更新）。
> - 遗留观察：Typune 的 `PdfExporter` 硬编码 `--pdf-engine=xelatex`，装了 pandoc 但用别的 PDF 引擎的用户仍会走回退，未来可做成可配置。

原计划内容（供对照，第 3 步的 HTML/LaTeX 切换已按上述修正推迟）：

1. 拷入 `crates/export`，让根包依赖它。
2. 写一层薄适配：Markion 的文档模型 → export crate 的输入类型（接口在 `engine.rs`）。
3. ~~逐格式切换：先 HTML/LaTeX（最容易对照验证），再 DOCX/PDF/图片。每切换一个格式，删除 `export.rs` 中对应的旧实现。~~（HTML/LaTeX 推迟；内置 PDF/DOCX 保留为回退，不删除）
4. 验收：Markion 现有导出相关单测改指向新引擎后全绿；人工对比几份样例文档的导出产物。

### Phase 2 — 吸收 syntect 语法高亮（用户可感知的最大提升）✅ 已实施（2026-07-06）

> 实施记录：OpenSpec change `syntect-code-highlighting`。要点与对原计划的两处调整：
>
> - **无需再抽独立 `crates/highlight`**——Phase 1 已把整个 `markdown` crate 搬入，直接复用其 `LanguageRegistry`（语法集 + 别名表）即可；Markion 端只新增对 `syntect` 的直接依赖。
> - **没有采用 Typune 的 `SyntaxHighlighter` 门面**：它返回写死 syntect 主题（base16-ocean.dark）的具体颜色，与 Markion 的 14 主题体系冲突。改为直接驱动 syntect 的 scope 解析（`ParseState`/`ScopeStack` 跨行持久，多行字符串/块注释正确），把 scope 栈由内向外归类回 Markion 的 `HighlightKind`（punctuation 透明使引号并入字符串 span；`keyword.operator` 保持 Plain 贴近原有观感），颜色继续由 Markion 主题决定。`highlight_code` 接口与 `main.rs` 的 `(语言, 代码)` 记忆化缓存不变。
> - **兜底与 Phase 1 同构**：syntect 默认语法集不含 TypeScript/TOML/Kotlin/Dockerfile/Zig 等，这些语言（以及任何 syntect 解析失败）自动回退到原有手写词法器，无语言从"有着色"退化为纯文本。
> - 语法集惰性加载（`OnceLock`）+ 启动时后台线程 `warm_highlighter()` 预热（约 100ms 不再落在首个代码块上）。
> - 对吸收 crate 的第一次主动修改：`LanguageRegistry` 增加 `syntax_set()` 公开访问器（scope 解析需要，附测试）。
> - 全 workspace 560 测试通过、0 失败；既有高亮契约测试未改动即通过（ts 断言恰好验证了回退路径），新增多行跨行、空行契约、未覆盖语言回退等 4 个测试。

### Phase 3 — 吸收 `filesystem` 的配置/日志/自动保存 ✅ 已实施（2026-07-06，范围有修正）

> 实施记录：OpenSpec change `toml-config-logging-autosave`。**范围修正（与 Phase 1/2 同一模式）：`filesystem` crate 未整体拷入**，原因有三——它依赖 rfd（Linux 上要 GTK3 开发库，发布 CI 的 apt 列表没有，整体拷入会弄断 Linux 发布构建，而 Markion 用 GPUI 原生对话框根本不需要 rfd）；logger/config 全是 Typune 品牌硬编码（`.typune_running` 哨兵、MarkdownEditor 目录名）；其 `Config` schema 与 Markion 偏好不匹配、tokio 版 AutoSaver 与 Markion 已工作的 GPUI 定时器机制重复。实际吸收的是它的**设计**，在 Markion 自己的 `storage/` 层实现：
>
> - **TOML 配置**：`config.toml`（serde + toml，所有字段可缺省），布局采用 Typune 的"顶层字段 + `[auto_save]` 分节"式样；启动时发现旧 `preferences.conf` 且无 `config.toml` 则一次性迁移（旧文件留在原地、此后忽略），旧格式解析器保留为迁移读取器 `parse_legacy_app_preferences`。
> - **自动保存可配置**：`[auto_save] enabled/delay_secs`（默认 true/5 秒——按计划以 Markion 语义为准，不取 Typune 的 8 秒），仅配置文件可改（不进偏好面板）。
> - **tracing 日志**：`storage/logging.rs`，按日轮转保留 7 份、`RUST_LOG` 覆盖（默认 info）、Markion 品牌平台目录；两处主动偏离 Typune 设计——文件用纯文本而非 JSON（桌面应用日志要让用户能读），不引入崩溃哨兵（Markion 已有恢复子系统）。首批日志事件：启动、偏好迁移、自动保存失败、导出引擎回退原因、语法集预热耗时。
> - 全 workspace 562 测试通过、0 失败；对应 spec delta 已修改 `chrome-platform`（偏好持久化 + 新增日志需求）与 `workspace`（自动保存不再是"固定 5 秒不可配置"）。

1. **配置**：用 Typune 的 TOML 配置（serde 派生）替换 Markion 手写的偏好文本格式。写一次性迁移：启动时若发现旧格式偏好文件则读取并转存为 TOML。Markion 的偏好项（主题、语言、聚焦/打字机模式、行号）并入 Typune 的配置结构，同时保留 Typune 的字体/自动保存延迟/键位绑定等分节，作为后续功能的落点。
2. **日志**：引入 tracing + 轮转日志（Markion 目前完全没有日志）。
3. **自动保存/恢复**：两边都有实现且 Markion 的已接入 UI；对比后择优（Typune 的 `auto_saver.rs` 更完善、可配置延迟），以 Markion 的行为语义为准（5 秒 vs 8 秒默认值在配置里解决）。
4. 注意：`filesystem` crate 不依赖 reqwest，可整体拷入。它的 `file_tree.rs`（1300 行，通用文件树）与 Markion 的 `storage/file_tree.rs`（357 行，近日已改为 Markdown-only + 空态启动）**行为语义已有分歧**：短期各用各的；长期若合并，以 Markion 的产品行为（Markdown-only）为准、择优吸收 Typune 的实现细节。

### Phase 4 — 评估性引入：增量解析与 ropey（收益大但侵入性强，放最后） ✅ 评估已完成（2026-07-07）

> 结果：pulldown-cmark 已全 workspace 统一到 0.13（OpenSpec change `unify-pulldown-cmark`，迁移面实测仅一处破坏模式），并顺带解锁吸收解析器的数学解析（`enable_math` 默认开——P2a 发现的 Typune LaTeX 数学乱码根源就是这个缺失的选项位）。**AST 统一/增量解析/render_cache 采纳：推迟**——实测无性能问题（388 KiB 文档全量重解析 45ms、增量 14ms，收益仅 ~3 倍而非数量级），本 Phase 自设的 gate（"无性能问题不动"）未触发；数据、决策与重启触发条件见该 change 的 design.md。keymap/ropey/图床仍按下方原则按需触发。

1. **增量解析**（`markdown/incremental.rs`）：Typune 的块级增量解析 + `Arc` 区域复用，对大文档打字性能是质变。但它绑定 Typune 自己的 AST 与 pulldown-cmark 0.11，而 Markion 用 0.13 和自己的预览块模型。两条路：
   - 短期：不动。Markion 现有"按版本缓存派生状态"已够用。
   - 长期：把 Typune 的 `markdown` crate 升级到 pulldown-cmark 0.13 后，作为 Markion 的解析层，Markion 的 `parse.rs`/`model.rs` 改为消费它的 AST。这是整个整合中最大的一步，值得单独立项（独立 OpenSpec change + design.md）。
   - 过渡期两个 crate 并存不同版本的 pulldown-cmark（Cargo 允许），不阻塞其他阶段。
2. **ropey 缓冲**：仅当 Markion 出现大文件编辑性能问题时再引入，否则不动（`String` 缓冲 + 现有缓存在当前规模下已验证够快）。
3. **keymap 配置化**（`editor/keymap.rs`）：作为独立功能引入（用户自定义快捷键），与 ropey 解耦。
4. ⚠️ `editor` crate 依赖 reqwest（图床上传功能）导致 openssl 链接问题。搬运时把 `image_uploader` 挪到 `[features]` 可选特性后面，默认关闭，消除对 openssl 的硬依赖。

### Phase 5 — 采摘杂项 + 退役 Typune

1. **主题**：保留 Markion 的 14 个主题和 `.theme` 机制；参考 Typune 的 TOML 主题定义把 `.theme` 升级为 TOML 格式（一次性迁移脚本），CSS 覆盖机制仅在预览走 HTML 渲染时才有意义，暂不引入。
2. **用户文档**：Typune 的 `docs/`（快捷键、主题定制、FAQ）**作为写作参考**改编为 Markion 文档——内容描述的是 Typune 的功能集，不能直接照搬。
3. **打包**：~~拷入 Typune 的 packaging/~~ **不再需要**——Markion 已有 cargo-packager + GitHub Actions 的完整发布流水线（deb/AppImage/dmg/NSIS），强于 Typune 的手写脚本，后者直接舍弃。
4. **不搬运（直接舍弃）的部分**：
   - `crates/ui` 全部组件（shim 之上的空壳，Markion 有真实实现）；
   - Typune 顶层 `src/`（最小窗口雏形、`error_notification.rs` 等，功能被 Markion 覆盖）；
   - `packaging/` 全部脚本（被 Markion 发布流水线取代）；
   - 根目录的 `test_*.rs`、`TASK_*.md` 及 `.kiro/`（Kiro 任务遗留物；Markion 侧的 `.kiro` 已于近日删除，Typune 侧的同样只作历史参考，不并入）。
5. 全部完成后归档 Typune 仓库（README 顶部注明 "merged into markion"）。

## 6. 主要技术冲突与解决办法汇总

| 冲突点 | 现状 | 解决办法 |
|---|---|---|
| pulldown-cmark 版本 | Markion 0.13 / Typune 0.11 | 过渡期并存（不同 crate 可依赖不同版本）；Phase 4 统一到 0.13 |
| 双 AST/文档模型 | Markion `model.rs` / Typune `ast.rs` | 应用层始终只认 Markion 模型；吸收 Typune crate 时写薄适配层；Phase 4 再决定是否换底 |
| edition | Markion 2024 / Typune 2021 | workspace 成员各自声明，不强行统一 |
| 配置格式 | Markion 自研文本 / Typune TOML | 采用 TOML，启动时自动迁移旧文件 |
| 主题格式 | `.theme` 自研 / TOML+CSS | 采用 TOML 定义，迁移现有 14 主题；CSS 覆盖暂缓 |
| openssl 硬依赖 | Typune `editor` → reqwest | 图床上传改为可选 feature，默认关闭 |
| i18n | 仅 Markion 有 | 保留 Markion 方案；吸收的模块产生的用户可见文案一律走 `i18n.rs` |
| 许可证 | Markion 已有 MIT LICENSE / Typune 声明双许可但无文件 | 按 Typune 的 MIT 选项并入；提交信息与 NOTICE 注明来源（同一作者，实际无风险） |
| dev profile 优化范围 | `package."*"` 不覆盖 workspace 成员 | 为搬入的计算密集 crate 补 `[profile.dev.package.<name>]` 显式覆盖 |
| 文件树行为 | Markion 已改 Markdown-only + 空态启动 / Typune 通用文件树 | 以 Markion 产品行为为准，实现细节择优 |

## 7. 风险与注意事项

1. **Typune 代码未经过真实运行验证**（UI 层是 shim，应用从未完整跑起来），其库 crate 的质量证据主要来自单测（约 1935 个，其中已验证 `markdown`+`export` 全绿）。吸收每个模块时必须连同其测试一起搬入，并补端到端验证。
2. **不要破坏 Markion 的打字路径性能不变量**（派生状态按版本缓存、高亮记忆化——AGENTS.md 有明确要求）。syntect 和增量解析都要放在现有缓存接口后面，而不是绕开它；workspace 成员注意 dev profile 覆盖问题（§4.2 第 4 条）。
3. **crate 切分线要避开孤儿规则**：所有需要实现 gpui trait 的类型留在根包，下沉到库 crate 的只能是纯数据/纯逻辑（§4.2 第 1 条）。
4. **每个阶段一个 OpenSpec change**：本仓库规定"先提案后实施"。本文件只是整体路线建议；动手前按 `/openspec:propose` 为每个 Phase 建 change（Phase 4 需要额外的 design.md）。Phase 0 的 workspace 化会触碰 AGENTS.md 与既有 spec 的表述，需在同一 change 里一并更新。
5. **发布流水线是回归高危区**：整合过程中任何对根包 `Cargo.toml`/构建脚本的改动，都要确认 `cargo packager` 与 `.github/workflows/release.yml` 仍然工作（根包 workspace 形态下默认不受影响，这也是选择它的理由之一）。

## 8. 结论（TL;DR）

**以 Markion 为宿主**，用**根包 workspace**（在现有 `Cargo.toml` 加 `[workspace]`，`src/` 不动）给 Typune 的库 crate 落户——workspace 化本身不是必须的，但这个最小形态成本接近零，且保住了"Typune 代码与测试原样进来"的安全网。然后按 `export` → syntect 高亮 → filesystem（TOML 配置/日志）→（评估后）增量解析/ropey 的顺序逐个吸收；Typune 的 UI 层、顶层应用和打包脚本直接舍弃（Markion 已有更强的 cargo-packager 发布流水线），用户文档改编复用。全程每一步保持 Markion 可编译、可运行、可发布、测试全绿，并走 OpenSpec change 流程。
