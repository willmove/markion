# Visual Edit P1：编辑基线与就近交互报告

> 历史阶段报告。P1 后已完成结构选区与事务、多块/子树操作、区域局部派生、表格矩阵及代码折叠；见 [P0–P3 最终结果](fluent-editing-implementation.md)、[Windows 真实输入验证](verification/windows-native-input-2026-11-11.md)和[项目进展](project-status.md)。下文的阶段测试数及操作范围保留为历史证据。

## 范围

P1 由两个 OpenSpec change 组成：

- `align-text-editing-baseline`：统一 Source 与 Visual 的文本编辑基本功。
- `anchor-visual-editing-overlays`：让命令和编辑器靠近内容，并减少输入时的视觉干扰。

P0 的性能诊断与身份协调优化已完成，报告见 `docs/visual-p0-performance.md`。P1 不改变 canonical Markdown、版本化 mutation、语义撤销或 per-version 派生缓存契约。

## P1A：编辑基线

### 键盘与选区

新增固定编辑动作：

- Unicode 按词左右移动、按词扩展选择、按词删除；macOS 使用 Option，Windows/Linux 使用 Ctrl。
- Shift+Home/End、PageUp/PageDown，以及对应的 Shift 扩展选择。
- Select All 先选择当前段落或 VisualBlock 的精确 editable payload，再次执行才选择整篇。
- Redo 同时支持 Ctrl/Cmd+Y 和 Ctrl/Cmd+Shift+Z，菜单与快捷参考同步更新。
- 为避免 Windows/Linux 的 Ctrl+Delete 同时触发文件树删除，文件树快捷键改为 Ctrl/Cmd+Shift+Delete；文档按词前向删除保留标准 Ctrl+Delete。

`src/text_util.rs` 使用 `split_word_bound_indices`，空白段不作为词目标；测试覆盖 ASCII、CJK、标点、emoji、组合字符和 CRLF。PageUp/PageDown 复用 Visual 的视觉行导航，Source 使用逻辑行导航。

### 鼠标与上下文菜单

- Source 双击选词、三击选择当前段落。
- Visual 三击选择当前精确文本块。
- Source 普通右键菜单提供 Cut、Copy、Paste、Select All，并保留 AI 选区动作；菜单打开与关闭不修改文档。

### 列表与输入规则

- 折叠光标位于无嵌套子树的列表项时，Tab/Shift+Tab 在合法范围内使用现有 `indent_lines`/`outdent_lines` 完成一次源码修改；非法转换不写入文本。
- 自动配对产生的 `**` 在合法块起始处输入 Space 会修复为 `* `，避免 `* ` 无序列表前缀被配对规则破坏。
- 普通 prose 中的 HTTP(S) 和 `www.` token 后输入空白会生成一个 Markdown 链接；代码、数学、HTML、front matter、其他受限字段及 IME 组合输入不转换。
- 斜杠识别支持 `/` 与中文 `、`，可出现在行首或空白后，URL 与普通非命令前缀不会触发。

### 测试覆盖

- 词移动/删除、行首选择、翻页、逐级全选。
- 合法/非法列表缩进及一步 Undo。
- Source 双击/三击、右键复制、Visual 三击。
- 自动项目符号、URL 转换及代码字段中的字面输入。
- 两种 Redo 键位及全语言快捷参考。

## P1B：就近浮层与低干扰显示

### 共享锚点策略

`src/app/overlay.rs` 按以下顺序解析位置：painted caret → source-owning block → Visual input bounds。下方菜单使用源块下沿，选区工具栏优先使用源块上沿；没有已绘制几何时使用有界回退位置。

已接入：

- 斜杠菜单；
- 文档链接编辑器；
- inline math/raw-token 编辑器；
- 自动选区格式工具栏。

链接和 token 编辑器不再使用固定浅色背景，改为当前主题 palette。

### 选区格式工具栏

工具栏仅在精确、非空、可格式化的 Visual 选区上出现，提供 Bold、Italic、Inline Code、Link。原有右键格式菜单保留。

按钮携带打开工具栏时的 `VisualSelectionFormatTarget`。点击时重新计算当前目标并逐项比较 document version、selected range 和 block id；旧选区不会被误格式化。文档修改后工具栏隐藏，选区再次变化时才重新评估。

### 输入时减少 chrome

canonical edit 后暂时隐藏块 grip，指针再次移过 Visual surface 时恢复。隐藏/恢复不修改文档版本、选区或派生缓存。

## 自动化验证命令

```bash
cargo test --lib text_util
cargo test --lib auto_pair
cargo test --lib input_rules
cargo test --lib slash_query

cargo test --bin markion baseline_
cargo test --bin markion visual_list_tab_uses_legal_source_backed_indentation
cargo test --bin markion source_mouse_selection_and_editor_context_copy_work
cargo test --bin markion visual_triple_click_selects_the_exact_paragraph
cargo test --bin markion contextual_bullet_and_url_input_use_one_exact_mutation
cargo test --bin markion url_conversion_stays_literal_in_code_payload
cargo test --bin markion redo_aliases
cargo test --bin markion menu_shortcut_metadata_supports_both_redo_bindings

cargo test --bin markion visual_selection_toolbar_
cargo test --bin markion slash_and_inline_editors_anchor_near_the_source
cargo test --bin markion typing_suppresses_block_chrome_until_pointer_move
cargo test --bin markion overlay::tests

cargo test --workspace
openspec validate align-text-editing-baseline
openspec validate anchor-visual-editing-overlays
```

完整结果见“验证结果”一节。

## 人工实机验证

### 启动与测试文档

在仓库根目录运行：

```powershell
cargo run --release
```

使用 File → Open 打开 `docs/fixtures/visual-p1-smoke.md`。默认进入 Visual Edit；Source 与 Visual 可用 Ctrl+/ 和 Ctrl+E 切换。

### 1. Source 编辑基线

按 Ctrl+/ 进入 Source：

1. 双击 `hello`，应只选中该单词；三击同一段，应选中整个 `hello world second paragraph`。
2. 按 Ctrl/Cmd+A 一次选择当前段，再按一次选择整篇。
3. 用平台词移动键在单词间移动；按词删除后用 Ctrl/Cmd+Z 恢复。
4. 验证 Shift+Home/End、PageUp/PageDown 及其 Shift 扩展选择。
5. 右键选区，确认 Cut/Copy/Paste/Select All 可用，Copy 不修改文本或 dirty 状态。
6. 按 Ctrl/Cmd+Y 和 Ctrl/Cmd+Shift+Z，分别确认都能执行一次 Redo。

### 2. Visual 选区与工具栏

按 Ctrl+E 进入 Visual Edit：

1. 三击一个段落，应选择该段精确文本。
2. 选中 `hello`，确认 Bold/Italic/Inline Code/Link 工具栏出现在选区附近，而不是固定窗口角落。
3. 点击 Bold，应得到 `**hello**`；Undo 一次恢复。工具栏在编辑后隐藏。
4. 改变选区后，旧工具栏动作不得格式化新选区。
5. 右键同一段，确认原有块菜单和格式动作仍可用。

### 3. 斜杠、链接和公式

1. 在空行输入 `/h2` 和 `、h2`，两种标记都应出现标题命令，菜单位置应靠近当前行。
2. 输入普通 URL（如 `https://example.com`），不应误触发斜杠菜单。
3. 光标进入 `[label](url)`，按 Ctrl/Cmd+K，链接编辑器应出现在链接附近；取消不应修改文档。
4. 在偏好设置中启用 Keep inline syntax hidden，点击 `$E = mc^2$`，公式编辑器应靠近公式；取消同样不修改文档。
5. 切换 One Dark 等深色主题，并将窗口缩窄，检查文字、边框、菜单是否可读且不被视口裁切。

### 4. 列表与即时输入

1. 在空行输入 `* `，应得到无序列表前缀，而不是 `**` 配对或强调文本。
2. 在 `- second` 上按 Tab，应合法缩进；Undo 一次恢复。顶层无缩进项按 Shift+Tab 不应改写源码。
3. 对含子级的列表父项，当前实现会保守拒绝结构转换；不要把它误认为缺陷。完整子树移动属于后续工作。
4. 输入 `https://example.com` 后按 Space，应生成 Markdown 链接；在代码块中重复输入，内容必须保持字面文本。
5. 使用中文 IME 输入“中文输入”，确认候选窗跟随光标，组合输入不会被 URL 或配对规则打断；Undo/Redo 各一次检查组合输入历史。

### 5. 低干扰与性能

1. 连续输入时块 grip 应隐藏，移动鼠标后恢复。
2. 分别打开约 100 KB、300 KB、600 KB 的真实文档，连续输入并滚动，观察光标、IME、视口是否跳动。
3. 模型层基准可用以下命令复核：

   ```bash
   BENCH_SIZES_KB=100,300,600 BENCH_STROKES=20 BENCH_VISUAL_ONLY=1 \
     cargo run --release --example bench_large_doc
   ```

   该命令不包含 GPUI 绘制，不能替代人工键盘到屏幕测量。当前优化后的诊断值和测试环境见 P0 报告，不应把单次机器结果当成跨设备门槛。

## 验证结果

- P1A/P1B 新增纯函数与 GPUI 针对性回归均已通过，覆盖本报告列出的键鼠、输入、锚点、旧选区和 chrome 状态。
- `openspec validate align-text-editing-baseline`、`anchor-visual-editing-overlays` 与 P0 change 均通过。
- 本阶段 Rust 文件的 `rustfmt --check` 和 `git diff --check` 通过。
- `cargo test --workspace` 全部通过：根库 729 passed / 1 ignored，应用 779 passed / 2 ignored；其他 workspace crate、集成测试与 doctest 均通过。
- 首次全量回归发现的图片标签页 panic、快捷注册表别名映射和文件树 Ctrl+Delete 冲突均已修复并复测通过；最终全量复跑无失败。

## 边界与后续工作

- P1 当时未包含的 SelectionState、EditTransaction 与多块/子树操作已在 P2/P3 完成。
- P1 当时未包含的 VisualBlock 区域级派生和 ChangeSet 副作用调度已在 P2 完成；P0 的身份协调优化仍作为前序基础。
- 引用容器拆分与整棵列表子树移动已在 P3 完成；GFM 合并格、任意多块单元格与图片像素裁剪仍不受支持。
- 本阶段只隐藏块 grip，没有隐藏系统鼠标指针，也没有重写 overlay 文本字段为多行富文本编辑器。
- 人工 GPUI 输入到屏幕结果与模型基准是不同证据，不能用其中一个替代另一个。
