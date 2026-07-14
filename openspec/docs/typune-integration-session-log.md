# Typune → Markion 整合：会话执行记录（2026-07-06）

> 本文档记录一次完整工作会话的过程与**实际执行结果**（所有编译/测试数据均为本机实测），作为 `docs/typune-integration-plan.md` 的实施留痕。
> 环境：Linux（无显示器容器），Rust stable，仓库 `github.com:willmove/markion`。
> 会话产出 6 个提交，完成整合计划的 Phase 0–3。

## 成果一览

| 提交 | 时间 | 内容 | 验证结果 |
|---|---|---|---|
| `ff9e0ac` | 下午 | 整合分析与分阶段计划文档（初稿） | 基于两仓库实测调研 |
| `d06aefc` | 19:11 | 计划文档修订：Workspace 利弊分析 + 按仓库最新状态更新 | — |
| `fdaa585` | 20:53 | **Phase 0**：根包 Cargo workspace | `cargo test` 108 全绿 |
| `fbef669` | 21:19 | **Phase 1**：吸收 markdown+export crate，PDF/DOCX 引擎优先+回退 | `--workspace` 556 全绿 |
| `de43695` | 21:43 | **Phase 2**：syntect 语法高亮换入 `highlight_code` 接口后面 | `--workspace` 560 全绿 |
| `220572a` | 22:07 | **Phase 3**：TOML 配置+迁移、tracing 日志、自动保存可配置 | `--workspace` 562 全绿 |

每个实施阶段均按 AGENTS.md 规范先建 OpenSpec change 再动代码（`openspec` CLI 在本机实际不可用，change 文件夹按仓库既有约定手工创建，格式与 `cross-platform-release-packaging` 等一致）。

---

## 1. 整合分析（→ `ff9e0ac`）

**任务**：深入分析 `/root/projects/typune` 与本项目如何整合为一个项目，写入 Markdown 文档。

**实测调研结果**（写入计划文档 §2 的关键事实）：

- Markion：约 1.45 万行单 crate，`cargo check` 通过，105 个测试，OpenSpec 流程活跃开发。
- Typune：约 4.5 万行 workspace（5 库 crate + bin），仅 8 个提交（Kiro 规格批量生成）。**其 `ui` crate 建立在占位 shim（`gpui_shim.rs`）之上，不是真实 GPUI 代码**；只有最后一个提交加了约 300 行最小窗口。
- Typune 整个 workspace 在本机编译失败（`editor` crate 经 reqwest 硬依赖 openssl-sys，系统缺 libssl-dev）；但 **`markdown` 与 `export` crate 可独立编译，约 350 个测试全部通过**（实跑验证）。
- 两者 GPUI 版本一致（0.2.2）；pulldown-cmark 版本不同（Markion 0.13 / Typune 0.11）。

**结论**：方案 A——以 Markion 为宿主，Typune 当零件库分阶段吸收；其 UI 层与顶层应用直接舍弃。产出 `docs/typune-integration-plan.md`（Phase 0–5 路线图 + 冲突对照表 + 风险清单）。

## 2. Workspace 化利弊分析（问答，无代码改动）

**任务**：深入分析 workspace 化的好处/坏处，是否必须。

**结论**（后写入计划文档 §4）：**不是必须的**。好处四条（库 crate 脱离 GUI 独立测试——本 session 已实证、依赖版本并存且 Typune 测试原样保留、增量编译收窄、编译器强制边界）；成本五条（孤儿规则约束 crate 切分线、原型期过早固化、文档 churn、`profile.dev.package."*"` 不覆盖 workspace 成员的坑、跨 crate 内联损失）。推荐**根包 workspace 最小形态**（现有 `Cargo.toml` 加 `[workspace]`，`src/` 不动），并撤回初稿"平移到 `crates/markion-app`"的建议。

## 3. 计划文档修订（→ `d06aefc`）

**任务**：把上述分析写入文档，并按项目最新状态重新检视全文。

**发现的四处过时描述并修正**：LICENSE（MIT）已补、cargo-packager + GitHub Actions 发布流水线已建立（deb/AppImage/dmg/NSIS，强于 Typune 手写脚本 → Phase 5 改为舍弃其 `packaging/`）、Markion 侧 `.kiro` 遗留规格已删除、文件树已改 Markdown-only。推送时遇远端新提交（`590ff84` CI 修复），`git pull --rebase` 后推送成功。

## 4. Phase 0：根包 Workspace（→ `fdaa585`）

OpenSpec change：`adopt-root-package-workspace`（含新能力 `crate-architecture` 的 spec delta，固化两条结构不变量：成员 crate 不得依赖 gpui；打字路径成员需显式 dev profile 覆盖）。

**实施与实测**：

- `Cargo.toml` 加 `[workspace] members = ["crates/*"]`，`src/` 未动。
- **实测发现计划未料到的细节**：cargo 对父目录不存在的 members glob 直接报错（`failed to load manifest for workspace member`），需 `crates/.gitkeep` 占位；目录存在时空匹配合法。已写回 proposal 与计划文档。
- AGENTS.md 的 "single crate" 表述替换为根包 workspace 布局说明。
- 验证：`cargo metadata` 确认根包为唯一成员；`cargo check` 4.73s 通过；`cargo test` **102 + 6 = 108 个测试全绿**，行为与 workspace 化前一致。

## 5. Phase 1：吸收 export 引擎（→ `fbef669`）

OpenSpec change：`absorb-typune-export-engine`（`export` 能力 MODIFIED delta）。

**范围修正（实施调研发现原计划误判）**：Typune 的 PDF/DOCX/图片导出器是 **pandoc（PDF 还需 xelatex）/ wkhtmltoimage 子进程封装**，只有 HTML/LaTeX 是原生 Rust。整体替换会把无 pandoc 用户从"导出质量差"退化成"导不出"。落地改为：**PDF/DOCX 引擎优先、任何失败静默回退内置实现**；HTML/LaTeX/图片维持 Markion 原生路径。

**实施与实测**：

- `crates/markdown` + `crates/export` 从 typune@`0b9e313` 原样拷入（61 文件、约 1.76 万行，含全部测试），仅 manifest 适配 workspace 继承表；根包以 `typune-markdown`/`typune-export` 重命名引用（Markion 自身有 `mod export`，同名依赖触发 E0659 歧义——实测确认后规避）。
- 两 crate 在 Markion workspace 内测试**约 448 个全绿**。
- **端到端验证**（本机有 pandoc、无 xelatex）：
  - DOCX 经 `export_to` 产出 **10,408 字节 deflate 压缩包**（真实 pandoc 产物；内置回退产物为 3,961 字节）——引擎路径实证可用。
  - PDF 因缺 xelatex 正确回退（707 字节内置产物）；pandoc"PDF 写 stdout"的管道协议另以 `--pdf-engine=pdfroff` 实测（安装 groff 后产出 **7,383 字节真实 PDF**），排除"引擎 PDF 在任何机器都走不通"的设计风险。
  - 清空 PATH 模拟无 pandoc：两格式均正确回退。
  - 遗留观察：`PdfExporter` 硬编码 `--pdf-engine=xelatex`，已记录待做成可配置。
- 测试适配两处：PDF 魔数断言 `%PDF-1.4` → `%PDF-`（引擎产物版本更高）；DOCX 元数据测试改测内置写入器（pandoc 产物是压缩包，原始字节不可见 XML）。
- 全 workspace `cargo test --workspace` **556 通过、0 失败**。

**环境事故 #1（磁盘写满）**：成员 crate 链接时 ENOSPC。清理本 session 早前在 Typune 仓库产生的 `target/`（`rm -rf`）与 `~/.npm` 缓存（`npm cache clean --force`，回收约 7GB）后恢复。

## 6. Phase 2：syntect 语法高亮（→ `de43695`）

OpenSpec change：`syntect-code-highlighting`（`code-and-math` 能力 MODIFIED delta）。

**两处设计调整（对原计划）**：

1. 无需再抽独立 `crates/highlight`——Phase 1 已带入整个 `markdown` crate，直接复用其 `LanguageRegistry`。
2. **不用 Typune 的 `SyntaxHighlighter` 门面**：它返回写死 syntect 主题（base16-ocean.dark）的具体 RGBA 颜色，与 Markion 14 主题体系冲突。改为直接驱动 scope 解析（`ParseState`/`ScopeStack` 跨行持久），scope 栈由内向外归类回 Markion 的 `HighlightKind`，颜色继续由 Markion 主题决定。细节：punctuation 透明（引号并入字符串 span）、`keyword.operator` 保持 Plain 贴近原观感。

**实施与实测**：

- 兜底与 Phase 1 同构：syntect 默认语法集**不含** TypeScript/TOML/Kotlin/Dockerfile/Zig 等，这些语言及任何解析失败回退到原有手写词法器。既有 ts 高亮测试**未改一字通过**，恰好实证回退路径。
- 对吸收 crate 的第一次主动修改：`LanguageRegistry::syntax_set()` 公开访问器（scope 解析需要 `&SyntaxSet`，原为私有字段；附测试）。
- 语法集 `OnceLock` 惰性加载 + `main()` 后台线程 `warm_highlighter()` 预热。
- 多行块注释/字符串现在跨行正确着色（旧词法器行局部做不到），新测试锁定。
- 全 workspace **560 通过、0 失败**（新增 4 测试）。
- **附带发现**：根包 workspace 下裸 `cargo test` 只跑根包，全量需 `cargo test --workspace`——AGENTS.md 已补充。

## 7. Phase 3：TOML 配置 / 日志 / 自动保存（→ `220572a`）

OpenSpec change：`toml-config-logging-autosave`（`chrome-platform` MODIFIED+ADDED、`workspace` MODIFIED 两个 delta）。

**范围修正（第三次同模式）**：`filesystem` crate **未整体拷入**，三个硬理由：① 依赖 rfd → Linux 需 GTK3 开发库，发布 CI 的 apt 列表没有，整体拷入会**弄断 Linux 发布构建**（Markion 用 GPUI 原生对话框，不需要 rfd）；② logger/config 全是 Typune 品牌硬编码（`.typune_running` 哨兵、MarkdownEditor 目录）；③ 其 Config schema 与 Markion 偏好不匹配、tokio 版 AutoSaver 与 Markion 已工作的 GPUI 定时器机制重复。吸收的是**设计**，在 Markion `storage/` 层实现。

**实施与实测**：

- **TOML 配置**：`preferences.conf`（手写 k=v）→ `config.toml`（serde+toml，全字段可缺省，Typune 式"顶层字段 + `[auto_save]` 分节"布局）。启动一次性迁移旧文件（留在原地、此后忽略）；旧解析器保留为 `parse_legacy_app_preferences`。测试覆盖：往返、局部文件缺省、迁移后旧文件再变不生效。
- **自动保存**：`[auto_save] enabled/delay_secs`（默认 true/5s，按计划取 Markion 语义弃 Typune 的 8s），仅配置文件可改。
- **日志**：`storage/logging.rs`——按日轮转保留 7 份、`RUST_LOG` 覆盖（默认 info）、Markion 品牌平台目录、初始化失败不影响启动。两处主动偏离 Typune 设计：纯文本而非 JSON、不引入崩溃哨兵。首批事件：启动、偏好迁移、自动保存失败、**导出引擎回退原因**（Phase 1 的静默回退自此可诊断）、语法集预热耗时。
- 全 workspace **562 通过、0 失败**；README 两行同步更新。

**环境事故 #2（磁盘再次写满）**：定位到 `target/debug/deps` 积累 12GB 历史产物；`cargo clean -p markion -p markdown -p export` 回收 **7.4GB**（保留 gpui 等外部依赖缓存）。另发现 `/root/projects/zedian/target` 占 6.7GB（他项目构建缓存，未动，磁盘紧张时可考虑清理）。

---

## 关键实测数据汇总

| 指标 | 数值 |
|---|---|
| 测试规模演进 | 108（Phase 0）→ 556（Phase 1）→ 560（Phase 2）→ 562（Phase 3），全程 0 失败 |
| 拷入代码 | `crates/markdown` + `crates/export`：61 文件 / 约 1.76 万行（typune@`0b9e313`，MIT） |
| DOCX 引擎 vs 回退产物 | 10,408 B（pandoc） vs 3,961 B（内置） |
| PDF 管道协议验证 | pdfroff 引擎经 stdout 产出 7,383 B 真实 PDF |
| 磁盘清理 | npm 缓存约 7GB + 自有构建产物 7.4GB |

## 遗留事项

1. **Phase 4（评估性，未实施）**：增量解析/ropey/keymap——计划明确"无性能问题不动"；统一 pulldown-cmark 0.11→0.13 需单独立项（change + design.md）。
2. **Phase 5（未实施）**：主题 `.theme` → TOML 迁移、Typune `docs/` 改编为 Markion 文档、归档 Typune 仓库。
3. PDF 引擎路径在装有 xelatex 的机器上的端到端验证未做（本机无 xelatex；管道协议与 DOCX 同构路径均已实证）；`--pdf-engine` 可配置化待做。
4. 本机 `openspec` CLI 缺失（AGENTS.md 称已全局安装）——4 个 change 均手工创建，未跑 `openspec validate`；全部 change（含此前的 3 个）尚未 archive 同步进 `openspec/specs/`。
5. GUI 内视觉确认（高亮观感、主题配色）留待有显示器环境 `cargo run` 复核——渲染路径本身未改动。
