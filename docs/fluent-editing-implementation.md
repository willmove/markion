# Fluent Editing P0–P3 实施与验证

> 状态核对：2026-11-13。P0–P3 与 Windows IME/帧调度收尾均已有实现和归档记录；最终真实 OS 输入结果以[后续验证报告](verification/windows-native-input-2026-11-11.md)为准。本文保留各阶段原始基准和回调回放结果。[项目进展](project-status.md)另列本次发布检查与尚缺的平台证据。

## 开工时的核对结果

依据 `docs/fluent-editing-research.md` 第 18 节，对照源码、稳定规范、归档变更和既有测试记录核对。调研文档是分析依据，其中的外部项目工作流没有作为仓库指令执行。

按用户要求，先将全部既有 P0/P1 工作提交为 `fbd0253`（`Checkpoint fluent editing P0 and P1 implementation`），再开始此次实现。

| 阶段 | 开工时已有内容 | 开工时尚缺内容 |
| --- | --- | --- |
| P0 性能基线 | 分阶段 profile、range 索引、线性候选检查、重复范围保护和 release 基准 | Windows 实机闭环；Visual 构建仍是全量工作 |
| P1 编辑基本功 | 词/行/页导航、双击/三击、Source 菜单、逐级全选、Redo 别名、基本列表 Tab、URL/配对规则 | 嵌套父项仍拒绝结构 Tab；实机输入与完整高级场景未验证 |
| P1 就近交互 | 锚定浮层、选区格式工具栏、链接/公式编辑器、输入后收敛 grip | 只有文本选区，尚无多块与单元格交互状态 |
| P2 选区与事务 | 已有 UTF-8 checked mutation、语义撤销、稳定块身份 | 缺少显式块/单元格状态、统一结构事务、多块操作及组合格式剪贴板 |
| P2 局部派生 | 已有语义区域缓存；P0 已去除身份协调平方级热点 | 每键仍构建全部 VisualBlock；首开区域划分有重复扫描；搜索和几何失效较粗 |
| P3 高级块 | 已有单块移动、表格单元格编辑及行列菜单/宽度、图片源编辑/尺寸/对齐/拖动缩放 | 容器拆分、多块/子树操作、矩形选区/矩阵粘贴、末格 Tab 加行、代码折叠 |

## 本次实现

### P2 基础与前序收尾

- GPUI-free 的 `SelectionState`、块/单元格选择和 `EditTransaction`，复用 canonical Markdown 和 checked mutation。
- 所有结构操作保留 UTF-8 范围、版本与目标验证；一次成功命令只有一次历史记录。历史只保存选区书签，Undo/Redo 按当前块身份恢复块/单元格选区。
- 视觉块复制同时发布可读 Unicode 文本和 HTML；单元格复制发布 TSV 和 HTML 表格；Read 的显式 Markdown 复制及 Source 的原文复制保留。
- 实机发现并补齐 Windows GPUI 后端的 HTML 写出：发布 Unicode 文本及 CF_HTML，使用 UTF-8 字节偏移、有效的消息窗口所有者、有界的剪贴板锁重试，并在原生分配转移失败时释放内存。格式依据 [Microsoft CF_HTML 文档](https://learn.microsoft.com/en-us/windows/win32/dataxchg/html-clipboard-format)；头部与中文/emoji 的往返另有独立测试。
- macOS 通过 `public.html`、X11/Wayland 通过 `text/html` 发布同一富文本内容，继续提供文本/TSV；普通文字复制替换此前的富文本载荷。macOS/Linux 尚无本轮原生构建和实机互操作证据。
- 初始区域划分按文档顺序推进；区域 visual 切片共享已发布的平面快照，局部编辑只重建变化区域，并重定位未变区域。
- 富文本、视觉文本与视觉块载荷采用共享及写时复制；平面 Preview 快照在真正读取时组装；撤销快照不持有派生缓存。
- 稳定 front matter 可与正文局部编辑共存；不确定全局语义仍使用完整正确性回退。debug/test 保留完整语义和 Visual 构建对照。
- `ChangeSet` 描述准确的旧/新范围和保守副作用；保留不会因后续行重排而移动的前缀导航几何，其余绝对 Y 几何重新测量。
- 大文档搜索防抖后在后台计算，按文档身份/版本/查询代次发布；匹配行列一次推进，长行结果片段有界。

### P3 高级块

- 完整列表子树与引用组作为源单元处理；父项移动携带子项及嵌套代码，拒绝自身后代和重叠落点。
- 多块复制、删除、键盘移动及 grip 拖动共用源单元；移动只替换起终点之间的范围。
- 引用中间段落转正文会拆开两侧引用；嵌套列表父项 Tab/Shift+Tab 修改完整子树，保留内部相对缩进和 CRLF。
- 表格提供矩形、整行和整列选区、TSV/HTML 复制、清空与矩阵粘贴；末格 Tab 添加一行并进入首格。
- 普通单元格输入只修改精确字段；缺失单元格需要物化时才走结构写回。矩形清空/已有格粘贴也优先保留未修改字节；列宽注释、对齐和 CRLF 有回归覆盖。
- 普通代码块提供本地化折叠/展开控制，折叠是每标签页的临时显示状态；显式导航或输入前展开载荷。
- 保留并回归图片源编辑、宽度、对齐、拖动缩放和语义撤销。

GFM 不表达合并格、任意多块单元格或图片像素裁剪，这些不作为此次标准 Markdown 的新增承诺。复杂源码所有权不能证明时拒绝结构修改。

## 操作方式

使用 `docs/fixtures/visual-p3-smoke.md`：

1. Esc 进入块选择，Shift+Up/Down 扩展；再次 Esc 回到文字。点击 grip 选择，Shift+点击另一个 grip 扩展。
2. Ctrl/Cmd+Shift+Up/Down 移动选中块；拖动选中范围内任一 grip 移动整个范围。靠近编辑区上下边缘自动滚动。
3. 单元格中拖到另一格形成矩形；Shift+点击扩展，Alt+点击直接进入单元格选择。行列 grip 选择相应区域并打开结构菜单。
4. Ctrl/Cmd+C 复制可读格式；Delete 清空选中格；Ctrl/Cmd+V 粘贴 TSV 矩阵。最后一格 Tab 添加行。
5. 点击代码 Fold/Expand 控制。搜索、光标导航或输入需要编辑折叠代码时展开它。

## 验证口径

测试机器：Windows，Intel Core Ultra 5 225U，12 核 / 14 线程，约 31 GiB 物理内存。时间均为本机诊断，不是跨设备固定耗时保证。

开工基线（release，10 次局部输入）：

| 混合文档 | 模型 P50 | 模型 P95 |
| --- | ---: | ---: |
| 100 KB | 10.61 ms | 12.31 ms |
| 300 KB | 34.30 ms | 49.07 ms |
| 600 KB | 83.38 ms | 91.58 ms |
| 1 MB | 374.87 ms | 397.14 ms |

基准不包含 GPUI 绘制。模型 retained 数值是共享分摊的估算，进程 working set/private bytes 才是另外记录的实机内存观察。

复现命令（PowerShell 7）：

```powershell
$env:BENCH_SIZES_KB = '100,300,600,1024'
$env:BENCH_STROKES = '100'
$env:BENCH_VISUAL_ONLY = '1'
$env:BENCH_FIXTURE_DIR = 'target/tmp/fluent-editing-fixtures'
cargo run --release --example bench_large_doc
cargo test --workspace -- --quiet
```

实机可通过 `MARKION_PROFILE_VISUAL_INPUT=1` 记录“文件打开到 Visual 文本 paint”及“原生输入回调到 caret paint”。该指标包含真实 GPUI 处理和 paint，排除 OS 输入传输、GPU 提交/屏幕呈现及自动化工具截图开销；连续输入在同一帧合并时记录这一批最早输入到最终 caret paint 的时间。

原生窗口回放命令：

```powershell
pwsh -NoProfile -File scripts/Invoke-FluentEditingSmoke.ps1
```

启动器使用隔离的 APPDATA/LOCALAPPDATA、临时 fixture 和独立进程，关闭自动保存与更新检查，并恢复测试前可读取的文本/HTML/图片剪贴板内容。JSON 记录构建配置、打开及输入 paint 样本、进程 RSS/commit 和各交互结果；日志及报告保存在 `target/tmp/fluent-editing-fixtures/native-*`。`-SkipBuild` 仅用于已完成对应构建的情况。

本机验证使用以下局部构建参数缩短 GPUI 单体编译时间，未修改仓库的生产 profile：

```powershell
# 测试仍启用 debug assertions、完整派生 oracle 和全部原有断言。
cargo test --workspace --config profile.dev.package.markion.opt-level=0 -- --quiet
# 原生回放为 opt-level=3、Thin LTO，应用包的代码生成单元数为 16。
cargo build --release --bin markion --config profile.release.package.markion.codegen-units=16
pwsh -NoProfile -File scripts/Invoke-FluentEditingSmoke.ps1 -SkipBuild
```

模型基准使用仓库默认 release 配置；原生窗口数据按上述 release 构建配置记录。两种测量分别报告。

CF_HTML codec 可独立复现，无需启动 GPUI：

```powershell
rustc --edition=2024 --test vendor/zed/crates/gpui/src/platform/windows/clipboard_html.rs -o target/tmp/fluent-clipboard-tests.exe
& './target/tmp/fluent-clipboard-tests.exe'
```

### 成本和证据边界

- 混合性能 fixture 包含标题、行内格式、链接、列表、引用、代码与表格，编辑发生在文档中部。它不代表所有 Markdown 形态。
- 未变区域不再重复解析和发现编辑器，但平面快照中的后缀绝对范围仍须线性重定位；当前实现没有承诺任意大小文档每键恒定成本。
- 引用定义、部分 HTML 或无法证明局部语义的源码仍会完整解析。超大单块、重型图片和全局依赖文档需要按实际内容另外测量。
- 首次回放时，Windows 的两个计算机控制入口均返回 `os error 5`。后续已修正 `CODEX_CLI_PATH` 指向目录的问题，并在重启后恢复接口；真实 OS 鼠标、键盘和中文候选窗的补充验证见 [Windows 实机输入验证](verification/windows-native-input-2026-11-11.md)。本节首次回放数据仍只代表应用输入回调。
- 本轮运行和测量的平台为 Windows 11；macOS、Linux/X11、Linux/Wayland 的原生构建及跨应用互操作未验证。
- RSS/commit 是进程内存计数；显存未单独测量。

## 最终结果

### 自动化回归

- 完整工作区：**2122 通过、5 个既有忽略项、0 失败**。其中根库 739 通过，GPUI 桌面测试 785 通过。
- Windows 剪贴板后端补丁后，重新执行全部桌面测试：**785 通过、2 个既有忽略项、0 失败**。
- CF_HTML 独立 codec 测试：**3 通过**，覆盖中文、emoji、组合字符、空片段、嵌入标记和不合法字节偏移。
- 根工作区和修改的 vendored Rust 文件格式检查、`git diff --check`、三项 OpenSpec 变更验证均通过；归档后 `openspec validate --all --strict` 为 **103 项通过、0 失败**。

完整工作区命令没有跳过失败测试。中途一次原子保存测试遇到 Windows 文件替换访问拒绝，单独复查和最终全量回归均通过。旧 grip 拖动测试的文字光标断言已调整为新块选区契约，同时继续断言 drop 前源码、版本、历史和缓存身份不变。

### 最终模型性能

无并行编译负载，每个大小 **100 次**局部输入；时间单位 ms，KB 按 1024 字节计算。

| 文档 | 首次模型构建 | 输入 P50 | 输入 P95 | 派生模型估算 MiB | 每键重建 / 复用块数 P50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 100 KB | 10.77 | 0.64 | 1.21 | 7.79 | 4 / 2121 |
| 300 KB | 28.67 | 2.55 | 3.69 | 23.14 | 4 / 6320 |
| 600 KB | 55.03 | 5.81 | 6.86 | 46.06 | 11 / 12586 |
| 1 MB | 96.84 | 10.33 | 11.24 | 78.33 | 10 / 21409 |

1 MB 的模型输入 P50 观测值由开工时 374.87 ms 降至 10.33 ms。基线为 10 次、最终为 100 次，系统负载也不同，适合说明本机回归趋势，不作为固定倍率承诺。并行编译期间 1 MB 样本波动明显，因此最终表采用构建完成后的独立复测。

### Windows 原生窗口

Windows 11 Pro 10.0.26200，release 回放 **全部通过**。下表打开时间指应用窗口启动完成后，从读取文件到首个 Visual 文本 paint；每个大小回放 20 次输入，输入时间指应用回调到可见光标 paint。

| 文档 | 打开至正文 paint ms | 输入 P50 ms | 输入 P95 ms | RSS MiB | Commit MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 100 KB | 53.44 | 12.23 | 18.01 | 145.45 | 109.73 |
| 300 KB | 57.47 | 14.25 | 20.44 | 170.75 | 135.50 |
| 600 KB | 83.25 | 19.08 | 23.61 | 204.05 | 170.02 |
| 1 MB | 134.62 | 28.04 | 35.84 | 249.36 | 217.71 |

整个回放的 OS 驻留峰值为 **254.39 MiB**。采样和原始 paint 数据保存在 [`verification/fluent-editing-windows-2026-11-11.json`](verification/fluent-editing-windows-2026-11-11.json)。实际窗口验证了：

1. 多块选择、原生 Unicode/HTML 精确往返、删除及一次撤销恢复源码和块选区。
2. 表格矩形选择、中文/emoji TSV 矩阵粘贴、末格 Tab 加行和撤销。
3. 代码折叠后实际行高减小，导航进入代码后展开。
4. 中文组合输入回调、真实字体排版产生的候选定位几何、提交和一次撤销。

首轮原生回放实际检出了 Windows HTML 写出缺口；修复后重跑上述完整流程通过。最终运行日志位于 `target/tmp/fluent-editing-fixtures/native-20261111-180912`。这份结果支持本机混合 1 MB 文档的快速打开和编辑；OS 输入、GPU 呈现、其他系统以及全局回退文档仍按上文证据边界解释，不能据此承诺任意文档、任意负载下完全零卡顿。

三个实施变更已同步到稳定规范并归档：`2026-11-11-localize-visual-derivation`、`2026-11-11-add-structured-visual-selection`、`2026-11-11-complete-advanced-visual-blocks`。

### Windows 真实输入补充验证

后续已恢复 Windows 自动化接口，补齐系统鼠标/键盘和实际中文候选窗验证，并修复停顿候选选择后 Undo 恢复拼音、表格清空时视口跳动、Copy 按钮位移及 Windows 输入后等待绘制的问题。1 MiB 文档最终 1080 次独立 OS 按键的完整保存文件一致，回调到光标 paint P95 为 33.14 ms、最大 47.95 ms；全部原始峰值、内存和测量边界见 [Windows 实机输入验证](verification/windows-native-input-2026-11-11.md)。本次收尾变更也已同步规范并归档。
