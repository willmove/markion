# 合入代码审计报告：哪些已接线、哪些还是库存、下一步整合什么

> 日期：2026-07-06 · 审计范围：Phase 0–3 合入的全部代码（`crates/markdown`、`crates/export` 及 Markion 侧接线）
> 起因：用户观察"貌似没用上 syntect"。本报告先给出接线验证与该现象的解释，再逐模块盘点使用状况，最后给出按优先级排序的待整合清单。
> 所有结论均有实测依据（调用点行号、测试名、实跑输出）。

## 1. 结论先行

1. **syntect 确实接在预览渲染路径上**（证据见 §2），但**syntect 默认语法集只有 75 种语法，现代主流语言几乎全在盲区**：TypeScript、TOML、Kotlin、Swift、Dockerfile、PowerShell、Elixir、Vue、GraphQL、Terraform、Dart、Zig、Nix、Protobuf、Julia、Solidity —— 本机实测 `registry.find()` 对这 16 个语言**全部返回 NONE**，按设计回退旧词法器。**如果你用这些语言测试，看到的就是旧效果——这不是没接线，是语法集覆盖缺口**，也是本报告的第一优先级整合项（§4-P1a）。
2. 真正被 Markion 调用的 Typune 代码只有三处：`LanguageRegistry`（高亮）、`Parser`（导出输入）、`PdfExporter`/`DocxExporter`（导出引擎）。**约 1.76 万行合入代码中，运行路径实际使用的不到 4 千行**；其余是"带测试的库存"，其中一部分对应明确的后续阶段，一部分需要做出"整合或修剪"的决定（§3、§5）。
3. 若怀疑"没生效"，最快的排除法：确认重新编译过（`cargo run` 会自动重编；日志首行 `Markion starting` 带版本号），然后用 **rust 块注释跨行测试**（syntect 覆盖 rust，效果差异是决定性的），而不是用 ts/toml 测试。

## 2. 接线验证（现状证据）

| 功能 | 调用链证据 | 无 GUI 自证命令 |
|---|---|---|
| syntect 高亮 | `src/main.rs:554` `highlight_code(code, language)`（预览代码块渲染，128 条 LRU 式缓存）→ `src/highlight.rs` syntect 优先 → `typune_markdown::highlight::LanguageRegistry` | `cargo test highlights_multiline_constructs_across_lines`（中间行全 Comment 是旧词法器物理上做不到的） |
| pandoc PDF/DOCX | `src/lib.rs` `export_to` Pdf/Docx 分支 → `src/export.rs:15-16` → `typune_export::{PdfExporter,DocxExporter}` + `typune_markdown::Parser` | 有 pandoc 的机器导出 DOCX 后 `unzip -v note.docx`：`Defl:N`=引擎，`Stored`=回退 |
| TOML 配置/迁移 | 启动 `load_app_preferences`（`src/storage/preferences.rs`） | `cargo test legacy_preferences_migrate_to_toml_once` |
| 可配置自动保存 | `src/main.rs` `schedule_autosave` 读 `[auto_save]` | 改 `config.toml` 的 `delay_secs` 观察节奏 |
| tracing 日志 | `main()` 首行 `init_logging` | `RUST_LOG=debug cargo run` 后查 `~/.cache/markion/logs/`，应有 `highlight grammar registry ready` |

Markion 源码中 `typune_markdown::`/`typune_export::` 的全部引用共 3 行（`src/highlight.rs:14`、`src/export.rs:15-16`）——接线面很窄是刻意的（应用层只认 Markion 模型），但也意味着大量库存未启用。

## 3. 逐模块使用盘点

### `crates/markdown`（约 1.1 万行 + 测试）

| 模块 | 行数 | 状态 | 说明 |
|---|---|---|---|
| `highlight.rs` | ~1140 | **部分使用** | `LanguageRegistry` 在用；`SyntaxHighlighter`（写死主题色门面）、`AsyncHighlighter`、`HighlightCache` 未用 |
| `parser.rs` | ~1500 | **部分使用** | 仅作导出适配层输入（`Parser::parse`）；预览不经过它 |
| `ast.rs` | ~500 | **部分使用** | 仅导出路径的 `Document` 类型 |
| `renderer.rs` | ~2350 | **间接使用一角** | `render_to_markdown` 被 export crate 的 pdf/docx 内部调用；RenderNode/RenderTheme 整套 UI 渲染模型未用 |
| `incremental.rs` | ~1320 | 未使用 | Phase 4 评估对象（块级增量解析） |
| `render_cache.rs` | ~640 | 未使用 | 同上 |
| `table_ops.rs` | ~600 | 未使用 | 与 Markion `src/table.rs`（400 行）功能重复 |
| `math.rs` | ~450 | 未使用 | 与 Markion `src/math.rs`（143 行）重复 |
| `emoji.rs` / `extended_inline.rs` | ~500 | 未使用 | 与 Markion `parse.rs` 内实现重复 |

### `crates/export`（约 4 千行 + 测试）

| 模块 | 状态 | 说明 |
|---|---|---|
| `pdf.rs` / `docx.rs` | **在用** | 引擎优先路径 |
| `engine.rs` | **部分使用** | `Exporter` trait + `ExportOptions` 在用；`ExportEngine` 注册表与 `export_with_fallback`（含 HTML 兜底、失败上下文）未用——Markion 用自己的静默回退 |
| `html.rs` / `latex.rs` | 未使用 | 原生 Rust 实现，Phase 1 刻意推迟切换 |
| `image.rs` | 未使用 | 依赖已弃维护的 wkhtmltoimage，计划弃用 |

## 4. 待整合清单（按优先级）

> **状态更新（2026-07-06 晚）**：P1a、P1b、P2a、P2b 已全部实施/完成（OpenSpec change `grammar-coverage-export-feedback`）。各项落地情况与 P2a 评估结论见下方对应小节的"✅ 实施结果"。

### P1a：扩展 syntect 语法集，补上现代语言盲区 ⭐ 直接回应本次观察

> **✅ 实施结果**：采用 `two-face` 0.5（默认 feature 与 syntect onig 后端匹配）。语法集从 75 种增至 **220 种**，§1 盲区清单中的 16 个语言**全部覆盖**（含 zig/solidity/julia）；且 two-face 用预编译二进制包，注册表加载实测仅 **2ms**（原默认集约 100ms）。`LanguageRegistry` 增加 `with_syntax_set()` 构造器（对吸收 crate 的第二次增量修改，附测试）。回退测试探针改用仍未覆盖的 `wasm`；新增 TypeScript 跨行块注释测试证明扩展集走语法路径。

- **问题**：默认 75 种语法缺 TypeScript/TOML/Kotlin/Swift/Dockerfile 等最常见的现代语言（§1 实测清单），这些语言全部回退旧词法器，用户感知"syntect 没生效"。
- **方案**：首选评估 [`two-face`](https://crates.io/crates/two-face) crate（打包了 bat 项目维护的扩展语法集，含上述语言，与 syntect 5 兼容）替换/合并 `LanguageRegistry` 内的 `SyntaxSet`；备选是捆绑所需 `.sublime-syntax` 文件用 `SyntaxSetBuilder` 加载（syntect 需开 `yaml-load` feature 或预编译 pack）。注意维持惰性加载 + 预热（语法集变大，加载时间需重新实测）。
- **工作量**：小（1 个 change）；风险：低（回退机制原样保底）。

### P1b：导出引擎的用户可见反馈

> **✅ 实施结果**：`export_to` 返回 `ExportBackend`（PandocEngine/BuiltIn）；PDF/DOCX 导出的状态栏消息区分"（pandoc 引擎）"与"（内置简易导出——安装 pandoc 可获得更高质量）"，中英双语（`Msg::StatusExportedEngine/StatusExportedBuiltin`）。`PdfExporter` 增加 `with_pdf_engine()`（对吸收 crate 的第三次增量修改，附测试），`config.toml` 新增 `[export] pdf_engine = "xelatex"`（仅配置文件可改），经 `export_to_with` 贯通。

- **问题**：引擎/回退目前对用户完全静默（仅日志可见）。用户导出 PDF 得到 707 字节文本 dump 时不知道"装 pandoc + xelatex 就能变好"。
- **方案**：状态栏消息区分"engine/built-in"（走 `i18n.rs`）；可借鉴未启用的 `export_with_fallback` 的错误上下文与建议文案设计。顺带把 `PdfExporter` 硬编码的 `--pdf-engine=xelatex` 做成可配置（`config.toml` 预留 `[export]` 分节），本机无 xelatex 但有其他引擎的用户即可受益。
- **工作量**：小-中；风险：低。

### P2a：HTML/LaTeX 导出器切换评估（Phase 1 遗留）

- 对同一批样例文档分别产出 Markion 原生与 `html.rs`/`latex.rs` 的输出做对照（数学、脚注、front matter、表格对齐），择优或确认维持现状。结论应写回计划文档；若确认不用，进入 P5 修剪清单。

> **✅ 评估结论（2026-07-06，夹具含 front matter/表格对齐/任务列表/脚注/高亮/上下标/emoji/行内与块级数学/代码块）**：
>
> - **HTML：维持 Markion 原生**。Markion 把数学输出为带 `data-latex`/`data-valid` 的结构化节点 + Unicode 兜底；Typune 把 `$...$` 原样留成正文文本。其余维度（表格对齐、任务列表、脚注、front matter 元数据）两者相当。→ `html.rs` 进入 P5 修剪候选。
> - **LaTeX：暂维持 Markion 原生，但记录 Typune 的三项明确优势待移植**——① 行内样式保真（`\textbf`/`\sout`/`\hl`/`\textsuperscript`/`\href`/`\footnote`，Markion 全部丢为纯文本）；② 表格对齐正确（`{lcr}`，Markion 输出错误的 `{lll}`）；③ 代码用 `lstlisting`（Markion 用 verbatim）。不切换的原因：Typune 解析器无数学选项（`ParserOptions` 无此开关），公式被转义成 `\$a...` 乱码，**数学正确性压倒样式丰富度**。数学接通属 P3（AST 统一）范畴，届时重评。
> - **对照过程中发现两个 Markion 自身缺陷（新增待办 P2c）**：① HTML 路径行内公式被上标扩展破坏（`$a^2+b^2=c^2$` 输出为 `a<sup>2+b</sup>2=c^2`，连 `data-latex` 也被污染；LaTeX 路径不受影响，说明是 HTML 渲染链的扩展语法次序问题）；② LaTeX 路径行内样式全丢、表格对齐硬编码 `l`、任务列表被拆成两个 itemize。

### P2c（新增）：对照评估暴露的 Markion 原生导出缺陷

- HTML：行内数学与 `^上标^` 扩展互扰（见 P2a 结论 ①）——修复方向：数学片段先于扩展内联语法提取。
- LaTeX：移植 Typune LaTeX 渲染器的行内样式/表格对齐/lstlisting 三项优势（见 P2a 结论 ②③）。
- 两项均为独立小 change，不依赖 P3。

> **✅ 实施结果（2026-07-07，OpenSpec change `native-export-fidelity`）**：
>
> - HTML：扩展内联渲染将数学容器（`math-inline`/`math-display`）视为原始文本（与 `code`/`pre` 同等），`$a^2+b^2=c^2$` 的 `data-latex` 原样保留，数学外的 `x^2^` 上标不受影响（含回归测试）。
> - LaTeX：新增 `render_latex_rich_text` 按 span 样式输出 `\textbf`/`\textit`/`\sout`/`\hl`/`\textsuperscript`/`\textsubscript`/`\texttt`/`\href`；表格对齐改从 pulldown 的 `Tag::Table(alignments)` 携带进 `PreviewBlock::Table`（实施中发现原 `source_range` 机制在该路径下不可靠，返回 `0..0`），列规格如 `{lcr}`；代码块改 `lstlisting`（仅对 listings 认识的语言标注 `language=`）；连续同类列表项合并单一环境，任务复选框输出 `$\boxtimes$`/`$\square$`；导言区补 `ulem(normalem)/soul/listings/amssymb`。Typune LaTeX 的三项优势全部移植完成。

### P2b：`supported_highlight_languages()` 与真实覆盖联动

- 目前返回固定 53 项清单，与"registry 实际 75+ 种、扩展后近 200 种"脱节；帮助菜单展示的语言列表应并集化（涉及签名或惰性静态化）。

> **✅ 实施结果**：改为"旧词法器清单 ∪ 注册表语法名（小写）"的排序去重并集，惰性构建一次（签名保持 `&'static [&'static str]`）。审计时另发现该函数其实没有 UI 调用点（是对外声明的能力 API + README 依据），故无帮助菜单改动；README 的 "50+" 声明已更新为扩展集事实。

### P3：AST 统一 + 增量解析 + render_cache（= 原 Phase 4 大项）

- 前置：把 `crates/markdown` 从 pulldown-cmark 0.11 升到 0.13，与根包统一；然后评估 Markion `parse.rs`/`model.rs` 改为消费其 AST，顺带解锁 `incremental.rs`、`render_cache.rs`、并消除 `table_ops`/`math`/`emoji`/`extended_inline` 四处功能重复。**这是剩余库存里价值最大的一块，也是唯一需要 design.md 的一块。** 在此之前，这些模块保持库存状态是合理的。

> **✅ 完成（2026-07-07，OpenSpec change `unify-pulldown-cmark`，含 design.md）**：
>
> - **版本统一已实施**：全 workspace 单一 pulldown-cmark 0.13；迁移面实测仅一处破坏（`TagEnd::BlockQuote` 增加 kind 参数），453 个吸收 crate 测试在 0.13 下全绿。
> - **意外解锁——数学解析**：迁移中发现吸收解析器早已处理 `InlineMath`/`DisplayMath` 事件、AST 与 LaTeX 导出器均支持数学节点，缺的只是 `ENABLE_MATH` 一个选项位（这正是 P2a 中 Typune LaTeX 数学乱码的根源）。已加 `enable_math`（默认开），散文内嵌数学（`prose $a^2+b^2$ end`）现在正确产出 InlineMath；对被 pulldown 拒绝的候选（如 `$ x $`）做 `$` 引导文本运行的试探合并以保留旧启发式，且不破坏 `\^` 转义语义（均有回归测试）。
> - **AST 采纳评估：推迟（gate 未触发）**。实测（37 KiB/388 KiB 合成文档）：Markion 全量重解析 4.5ms/45ms，增量解析 1.1ms/14.1ms——现实笔记尺寸下无性能问题，增量收益仅约 3 倍而非数量级，不足以支付重写整个 PreviewBlock 消费链（渲染/编辑/导出 + ~150 测试）的成本。重启触发条件与数据详见 change 的 design.md。P5 修剪（`html.rs`/`latex.rs`/`image.rs`/`SyntaxHighlighter` 门面）由此解除阻塞。

### P4：可选零件

- `AsyncHighlighter`/批处理高亮：仅当出现超大代码块卡顿再启用（现有 128 条缓存 + syntect 已够快）。
- Typune `editor` crate（未拷入）：keymap 配置化、ropey、图床上传（需 feature 化摆脱 openssl）——各自独立小项，按需求触发。

### P5：修剪与收尾（= 原 Phase 5 扩充）

- P2a/P3 决策后删除确认不用的模块（`SyntaxHighlighter` 门面、`image.rs`、未采纳的导出器/渲染模型），避免库存永久化；主题 TOML 迁移；Typune `docs/` 改编；归档 Typune 仓库。

## 5. 库存策略说明

未使用模块暂不删除的理由：① 均带测试、CI 全绿，维护成本目前仅为编译时间；② P2a/P3 的评估结论直接决定其去留，现在删了可能马上要捡回来。但设定**去留判定点**：P2a 与 P3 各自完成评估后，立即执行对应修剪——避免"库存"变成永久死代码。

## 6. 快速验证手册（汇总）

```bash
# 1) 确认跑的是新二进制：日志首行带版本
RUST_LOG=debug cargo run   # 另开文档，查 ~/.cache/markion/logs/markion.*.log

# 2) syntect 生效判别（用 rust，别用 ts/toml——那些在盲区，按设计走旧词法器）
#    预览里贴跨行块注释，中间行呈注释色 = syntect
cargo test highlights_multiline_constructs_across_lines   # 无 GUI 等价证明

# 3) 语言是否在 syntect 覆盖内（NONE = 走旧词法器）
#    参见 §1 盲区清单；rust/python/js/sql/bash/html/css/yaml/json/java/c/cpp/go/ruby 等在覆盖内

# 4) 导出引擎判别
unzip -v note.docx | head    # Defl:N=pandoc 引擎, Stored=内置回退

# 5) 依赖树证明
cargo tree -i syntect | head -3
```
