# Fluent Editing 与 Markion 文档编辑体验对照研究

## 范围与证据口径

本报告第 1–14 节研究 Quiet.md 本身，重点是文档编辑、交互及其支撑实现；第 15 节起结合本地 Markion 源码、规范与 release 基准给出对照和改造建议。

- 官方仓库：<https://github.com/s1dashu/quiet-md>。
- 默认分支：`main`，由 [GitHub 仓库元数据](https://api.github.com/repos/s1dashu/quiet-md)与 `git ls-remote --symref … HEAD` 双重确认。
- 固定版本：[`57884b9c8e49d22effe18885c6d717d8acb67278`](https://github.com/s1dashu/quiet-md/commit/57884b9c8e49d22effe18885c6d717d8acb67278)，提交时间 `2026-11-09T03:05:09Z`，提交主题为刷新首页截图。下文所有 GitHub 文件链接均固定到这一 commit。
- 本地只读研究副本：`target/tmp/quiet-md-research`，克隆后 HEAD 与上述 SHA 一致，未修改源文件。

证据标记如下：

- **【源码】**：直接由当前 commit 的实现确认，包括条件、限制和调用关系。
- **【指南/演示】**：来自仓库内官方内置指南、截图或脚本化演示；它是产品方材料，不等于本次已实际操作验证。
- **【官方实测】**：项目方公布的性能数据；本报告没有复现。
- **【推断】**：基于已列源码事实总结的迁移价值或取舍，不应理解为 Quiet.md 的官方承诺。

## 结论摘要

Quiet.md 的主路径是**自绘富文本块编辑器**，不是 Markdown 源码编辑器与预览页的拼接。文档在内存中是 block 树，正文富文本由字符串与样式 run 构成，公式等行内对象以 U+FFFC 占位；DocxXML 是原生文档的主要磁盘格式。Markdown 在三处进入这条链路：输入时的即时转换、粘贴/导入时的解析、本机 `.md` 文件的可视化编辑与写回。模式开关提供的是“编辑 / 阅读”，不是“源码 / 预览”。【源码】[架构分层](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/ARCHITECTURE.md#L1-L30)、[文档模型](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L196-L209)、[模式菜单](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/modes.rs#L76-L102)

其最值得关注的交互组合是：

1. 字符、整块、矩形单元格三种选区并存，并有明确的进入、扩展和退出规则。
2. 块手柄、斜杠菜单、快捷键和 Markdown 输入共用同一个编辑内核，而不是各自维护一套文档修改逻辑。
3. Markdown 转换单独进入撤销历史，第一次撤销可恢复刚输入的字面标记。
4. 表格不是纯文本网格：单元格可以包含段落、列表、任务、代码和公式，并支持合并、拖选、整行/列移动。
5. 界面采用按需出现的浮层：选中文字才有格式工具栏，悬停才有手柄，输入时隐藏手柄和鼠标指针。
6. 性能依赖块级虚拟列表、按 generation 失效的派生状态、增量行替换和共享图片缓存；撤销则采用简单但空间成本明确的整文档快照。【源码】对应实现见下文各节。

## 1. WYSIWYG、源码与文件策略

### 1.1 内存模型与渲染路径

**【源码】** `Document` 包含标题、图标、封面和递归的 `Block` 树；`BlockKind` 包括文本、1–9 级标题、三种列表、代码、引用、高亮块、分隔线、图片、表格、分栏、文件及保留原始 XML 的 `Raw`。`RichText` 是 `String + Vec<Run>`，run 覆盖 UTF-8 字节范围并保存粗体、斜体、下划线、删除线、代码、链接和颜色等 marks。容器与可嵌套文本块由不同方法区分，代码块不属于可嵌套块。[`BlockKind` 与嵌套规则](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L718-L799)、[`Block`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L871-L899)、[`Marks` / `RichText`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L33-L68)

**【源码】** 应用是 Rust + GPUI 单进程；`fdl_core` 不依赖 GPUI，负责模型、格式、编辑和撤销，根 crate 负责事件、浮层与绘制。富文本不是 WebView 中的 `contenteditable`，而是 `RichTextElement` 自行排版和绘制文本、选区、光标及 IME marked range。[架构与 crate 边界](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/ARCHITECTURE.md#L1-L30)、[自绘文本元素](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L1-L13)、[布局与绘制入口](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L692-L720)

**【源码】** 对未知 DocxXML 元素，block 级内容以 `Raw` 保留；行内对象同时保存解析后的 `kind` 与原始 `xml`，文本中以 U+FFFC 表示一个对象。这使可视化编辑与格式往返共用一个结构，而不是把原始 Markdown 当作唯一状态。[行内对象模型](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L43-L68)、[DocxXML 往返约定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/FORMAT.md#L59-L90)

### 1.2 没有以“源码视图”为编辑主路径

**【源码】** 文档级模式菜单只有“编辑”和“阅读”；快捷键面板介绍的是 Markdown 快速输入法，而非切换 Markdown 源码。[编辑/阅读实现](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/modes.rs#L76-L102)、[快捷键面板中的 Markdown 说明](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/modes.rs#L157-L213)。固定源码中未找到独立的源码模式或源码/预览双栏入口；这一否定结论仅限于所查当前树，不外推到其他分支。

### 1.3 本机 Markdown 仍是可视化块编辑

**【源码】** `.md`、`.markdown`、`.mdown` 可在原位置打开；`Store::parse_doc` 把它们解析为同一个 `Document`，保存时走 `markdown::write_file_with`，不是切换到一个文本源码控件。[文件类型与原地打开](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/local_files.rs#L1-L25)、[解析与保存分派](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/store.rs#L438-L475)

**【源码】** Markdown 写回会分离并原样保留 front matter，把当前 block 序列与磁盘原文的 block 序列匹配；未变化的 block 可以保留原行和空行，CRLF 也会恢复。保留结果还要重新解析并与规范化输出比较，失败则退回完整规范化写出，避免“为了保留排版而改变文档含义”。匹配使用 LCS，过大中间区域有计算上限。[写回与读回保护](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/markdown.rs#L303-L344)、[按 block 保留原文](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/markdown.rs#L347-L435)、[LCS 匹配](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/markdown.rs#L450-L475)

**【源码】** Markdown 页面会在 UI 中限制格式能力：斜杠菜单去掉文件、分栏、高亮块、公式、按钮、提醒和倒计时，块菜单/工具栏去掉颜色与对齐，页眉不提供图标和封面。注意，这主要是菜单和视图层限制；Core 的 `$…$` 即时转换本身并不读取这个 `markdown` 标志。[格式过滤](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/menus.rs#L184-L187)、[斜杠结果过滤](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L621-L641)、[格式工具栏限制](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L1392-L1422)

**【指南/演示】** 官方数据指南明确展示“原地编辑 Markdown”“只重写变化 block”“保留 front matter”的用户承诺；当前源码给出了相应实现，而不只是宣传语。[官方数据指南](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/src/08-data.xml#L1-L11)

## 2. 块级操作与插入体验

### 2.1 三个等价入口：手柄、斜杠、键盘

**【源码】** 悬停块左侧时出现手柄；普通块手柄包含类型图标和抓握图标，空行则变成加号。点击普通手柄会进入整块选择并打开菜单；点击加号进入插入菜单。输入期间和阅读模式下不显示手柄。[手柄出现条件与交互](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L344-L412)、[菜单点击后的选区](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L2376-L2392)

**【源码】** 块菜单提供类型转换、缩进/减少缩进、对齐、颜色、剪切、复制、删除和“在下方添加”；具体项目随 block 类型收缩。引用和高亮块作为容器处理：转换不是简单改一个枚举，而是包装相邻兄弟；从容器中间移出内容时会拆成前后两个容器。[块菜单](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L713-L855)、[类型转换与容器拆分](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1624-L1777)

**【源码】** `/` 和中文 `、` 都可唤起斜杠菜单；不要求整行只有 `/`，而是允许行首或空白之后触发。标题、代码、非文本块不会触发；URL、双空格尾部、跨块选区或光标离开会关闭。菜单搜索支持中文、英文和拼音关键词，排序依次是条目前缀、分组前缀、条目包含、分组包含；方向键可遍历图标网格和列表，Enter 执行。[斜杠状态与关闭规则](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L592-L677)、[键盘导航](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L680-L780)、[条目和搜索排序](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/menus.rs#L61-L182)、[搜索实现](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/menus.rs#L218-L246)

**【源码】** 执行斜杠项时，先选中并删除 `/query`，再按目标是否为空决定“转换当前块”或“插入新块”。因此菜单并不是只向光标处插文本，而是直接操作 block 树。[斜杠项执行](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L796-L917)

### 2.2 移动、嵌套与空隙点击

**【源码】** 拖动手柄超过 2px 才进入拖动；可携带当前多块选区，折叠标题会携带隐藏的章节。拖动时有跟随指针的内容卡片和 2px 插入线，水平位置可表达嵌套/退出嵌套；靠近页面上下边缘会自动滚动。Core 拒绝把块放进自身后代，并限制允许成为父级的类型和容器。[拖动手柄与选区](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/drag.rs#L184-L254)、[落点与层级判定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/drag.rs#L67-L171)、[边缘滚动与视觉反馈](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/drag.rs#L348-L446)、[实际树移动](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2037-L2077)

**【源码】** `Ctrl+Shift+↑/↓` 在同级块间交换当前选区与相邻块；不是只移动视觉行。[快捷键移动实现](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L2050-L2083)

**【源码】** 对代码、图片、表格等没有可直接点击文本的块，块间或块旁的特定空隙可在鼠标释放时插入段落；移动超过 10px 则视为拖动而不插入。点击正文末尾空白会聚焦或补出末尾空段。[空隙判定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/gap.rs#L1-L33)、[释放时插入](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/gap.rs#L95-L120)、[末尾空段](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2383-L2399)

## 3. 文本选择、光标与 IME

### 3.1 三种选区，而非只有一段文字范围

**【源码】** 字符选区是 `(block id, UTF-8 byte offset)` 的 anchor/head；`Unit` 还区分文本单元与原子块，并记录表格作用域。另有独立的 `BlockSel` 和 `CellSel`，Core 同时持有三者状态，但进入整块或单元格选择时会清理冲突状态。[位置与字符选区](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1-L59)、[Core 的选区状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L405-L431)、[整块选择](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2230-L2250)

**【源码】** 鼠标单击定位，双击按 Unicode word boundary 选词，三击选当前块全部文本，Shift 扩展，拖动更新 head。横向移动按 grapheme 或 word；纵向移动通过已排版行和保存的 `goal_x` 保持视觉 x 位置，而不是用固定字符列。[鼠标选择](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L2200-L2285)、[grapheme/word 导航](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L272-L317)、[视觉行导航](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1192-L1285)

**【源码】** Esc 可从字符选区进入当前块的整块选择；块模式中上下键移动或 Shift 扩展，左右键执行父级/子级导航和折叠。字符选区到块边界后继续 Shift+↑/↓，会先扩到块边缘，再切换为块选择。[Esc 状态转换](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1711-L1738)、[块选择导航](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2401-L2464)、[字符到块选择](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2507-L2516)

**【源码】** `Cmd/Ctrl+A` 是逐级扩大：先选当前块文本，再向单元格、表格、高亮块等外层停留点扩展，最后到整篇；折叠标题隐藏的章节也会被纳入对应块选区。[逐级全选](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2580-L2627)、[折叠内容的选区处理](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2355-L2381)

### 3.2 选择绘制与命中测试共用排版结果

**【源码】** 每个可见文本块把换行、视觉行、对象边界和窗口坐标写入共享 `LayoutCache`。鼠标命中、光标定位、选区矩形、上下导航和 IME 候选窗定位都读取这套布局，避免把“模型字符索引”和“屏幕位置”拆成互不相同的算法。[布局缓存结构](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L15-L46)、[命中与选区矩形](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L95-L160)、[跨块命中测试](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L1384-L1416)

**【源码】** 绘制顺序先画文本背景，再画选区，再画字形；公式等对象有各自的选中背景。查找高亮最后叠加在文字区域上，光标另行自绘。这一顺序也对应 0.1.9 修复“高亮背景使选区不可见”的发布说明。[选区与字形绘制顺序](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L979-L1035)、[对象选区、查找与光标](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L1065-L1096)、[0.1.9 修复说明](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/releases/0.1.9.md#L11-L19)

### 3.3 IME 是输入协议的一部分

**【源码】** `EntityInputHandler` 处理 UTF-16 与 UTF-8 范围转换、marked range、候选窗 bounds 和鼠标字符索引。组合输入更新只替换当前块范围并记录 `marked`，不会在每个组合阶段调用 Markdown 即时转换；提交路径才进入统一的 `type_text`。跨块选区在开始组合时会先收敛/删除，块选择模式不接受文本输入。[IME 字段与订阅](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L288-L328)、[输入提交](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L2548-L2599)、[组合输入与候选窗](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L2601-L2660)

## 4. Markdown 输入转换

### 4.1 块级规则

**【源码】** 即时转换只允许在正文、标题、列表和任务块内，标题与代码块被排除；要求折叠光标，并检查上下文，例如表格单元格内不把 `>` 变成引用，高亮块内不创建代码块。[启用范围与上下文](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2661-L2685)

| 输入 | 当前源码确认的行为 |
|---|---|
| `#` 至 `#########` + 空格 | 转为对应 1–9 级标题 |
| `-` / `*` / `+` + 空格 | 无序列表 |
| 数字 + `.` 或 `。` + 空格 | 有序列表，可保留起始数字；在标题上则启用标题自动编号 |
| `[]` / `[ ]` / `【】` + 空格 | 未完成任务 |
| `>`、`|`、`》` + 空格 | 在允许上下文中包装为引用 |
| `` ``` ``、`` ```lang `` + 空格，或对应前缀 + Enter | 代码块，并解析语言名 |
| `---` | 第三个 `-` 输入后立即成为分隔线 |
| `***` + 空格或 Enter | 分隔线 |

来源：[`block_trigger`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2675-L2717)、[转换执行](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2720-L2755)、[分隔线规则](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2758-L2783)。**【源码】**

### 4.2 行内样式、链接与公式

**【源码】** 每输入一个字符都会尝试闭合行内规则，支持 `***…***`、`___…___`、`**…**`、`__…__`、`*…*`、`_…_`、`~~…~~`、`~…~`（下划线）和反引号代码。转换删除分隔符、保留内部已有 marks，并设置 `pending_marks`，使随后输入不继续携带刚闭合的样式。下划线与 `_`/`__` 等规则要求两侧为空白或行边界，以减少 `file_name` 一类误转换。[行内转换规则与边界](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2812-L2915)

**【源码】** `$x$` 与 `$$x$$` 都转换为一个公式行内对象，而不是产生两种文档节点；空内容、嵌套 `$`、代码范围和不符合边界的输入不会转换。URL 后跟空格时会自动加链接，`www.` 会补成 `https://`。[公式即时转换](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2836-L2865)、[自动链接](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2786-L2810)

### 4.3 转换与输入分开撤销

**【源码】** 空格或闭合标记先作为普通输入进入历史，随后转换调用另一次 `checkpoint(EditKind::Other)` 并打断合并。因此第一次撤销针对转换，能恢复字面前缀/分隔符，而不是把触发字符与转换一次性抹掉。[块转换的撤销分段](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2730-L2740)、[行内转换的撤销分段](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2897-L2912)。官方指南也明确演示“一次撤销恢复字面字符”这一交互。[官方写作指南](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/src/02-writing.xml#L20-L31)

## 5. 列表、表格、代码、公式与图片

### 5.1 列表与标题层级

**【源码】** 列表不是一组带前缀的文本行，而是 `Bullet` / `Ordered` / `Todo` block 加递归 children。Tab 把块移到前一个合法兄弟的 children 末尾；Shift+Tab 把块提升到父级之后，并重新安置后续兄弟。空列表项按 Enter 时，嵌套项先减少缩进，顶层项变回正文。[列表缩进与提升](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1503-L1599)、[Enter 行为](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1143-L1258)

**【源码】** 在普通块开头 Backspace 会按 grapheme 删除；块起始处对标题/列表/任务等先退化为正文，而不是立即与前块合并。代码块只有为空时才在起始 Backspace 退化为正文。[Backspace 分级行为](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1264-L1326)

**【源码】** 有序列表保存可选起始序号，按兄弟作用域计算编号；点击数字可继续上一列表或设置起点。标题也可启用层级自动编号。标题和有子级的文本/列表项可折叠；标题折叠隐藏到同级或更高一级标题之前的章节，编辑后 `reveal_caret` 会展开遮住光标或选区的节点。[序号操作](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1601-L1620)、[折叠与可见顺序](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1812-L1945)、[折叠按钮](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L519-L571)

### 5.2 表格：独立的单元格选择与结构操作

**【源码】** `CellSel` 区分矩形范围、整行和整列；合并单元格会把选区扩大到不切割合并区域的最小矩形。拖出起始单元格即由文本拖选切换为单元格拖选，回到原单元格则恢复文本选择。[单元格选区模型](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/table_ops.rs#L7-L39)、[合并格扩展](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L614-L666)、[文本/单元格拖选切换](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L93-L136)

**【源码】** 行/列灰色手柄支持点击、Shift 扩展和拖动选择；已选行/列可再次拖动排序。边界插入行列，列边界拖动实时预览，释放时才提交宽度；落点不能切开合并单元格。删除全部所选行或列会删除整个表格。[手柄与拖动](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L138-L196)、[拖放合法性](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L206-L270)、[提交与排序](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L300-L349)、[结构编辑与撤销](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/table_ops.rs#L217-L341)

**【源码】** 单元格内容是 `Vec<Block>`，可包含多种块；合并时按阅读顺序收集非空单元格内容。工具栏提供合并/拆分、背景、横纵对齐和文本样式；标题行可在长表格滚动时固定，但首行有跨行合并格时不走该固定路径。[表格单元格渲染](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L479-L568)、[合并/拆分与样式](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/table_ops.rs#L258-L321)、[固定标题行条件](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L509-L518)

**【源码】** Tab/Shift+Tab 在单元格间移动，最后一个单元格按 Tab 会追加一行；Backspace 在单元格边界受作用域约束，持续删除空单元格内容后可选择并删除表格。[单元格键盘导航](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2289-L2321)、[单元格删除边界](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1328-L1343)

### 5.3 代码块

**【源码】** 代码块内部保留纯文本，不带富文本样式或行内对象。Enter 保留当前行缩进；Tab 插入四个空格；Shift+Tab 删除当前行首最多四个空格；Markdown 行内规则在代码中不启用。粘贴富文本块到代码时会扁平化为纯文本行，嵌套每层补两个空格。[代码输入与 Tab](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1631-L1708)、[代码内粘贴](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2954-L2978)、[转换到代码时清除富文本](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1739-L1775)

**【源码】** 代码块支持可搜索语言列表、标题/文件名、自动换行、折叠、复制全文和按主题决定的行号；折叠状态是视图状态，不进入文档历史。高亮不是接入完整语言服务器或通用语法树，而是自研的小型 lexer；未知语言保持不着色，因此“语言列表支持”不应等同于每种语言都有同等深度的语义高亮。[代码块 UI](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L735-L855)、[换行、复制与行号](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L857-L1018)、[语言列表](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/lang.rs#L1-L80)、[高亮器范围](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/highlight.rs#L1-L11)、[高亮分派](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/highlight.rs#L260-L270)

### 5.4 数学公式

**【源码】** 公式是 `InlineKind::Equation`，不是单独的 block 类型。点击公式打开一个多行等宽 LaTeX 编辑器并全选源码；编辑时逐次更新同一对象，Esc 或越过左右边缘关闭并把光标放在对象后/前。公式编辑在撤销合并上采用专门类别，直到显式打断。[公式编辑状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L291-L328)、[打开/关闭编辑器](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L968-L994)、[行内对象修改](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1025-L1036)

**【源码】** 渲染链路为 `katex-rs` 生成 KaTeX HTML，再由项目自己实现的 KaTeX CSS 子集布局成字形、矩形和 SVG，最后用 GPUI 绘制；不依赖 WebView。无效公式显示错误占位而不删除源文。公式布局有 thread-local 缓存，键包含源码、显示模式、字号、颜色和公式编号，超过 2000 项时清空。[数学渲染架构](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/math.rs#L1-L23)、[布局缓存与错误处理](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/math.rs#L121-L181)、[无效公式占位](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L444-L475)

**【源码】** 是否按展示公式布局由 LaTeX 内容决定：`equation`、`split`、`align`、`gather`、`alignat`、`CD` 环境或 `\tag`；`$$…$$` 只是另一种输入定界符。过宽公式会缩小字号以适应当前行。[展示公式判定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/math.rs#L21-L40)、[行内与展示公式几何](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L477-L543)

### 5.5 图片

**【源码】** 可通过斜杠菜单、文件拖入和剪贴板图片插入；文件放入文档库资源目录。JPG/JPEG、PNG、GIF、WebP、BMP 有扩展名与 20 MiB 大小检查，并读取像素尺寸；多个剪贴板图片按顺序插入。拖入时图片与其他附件分别处理，并共用块落点指示线。[图片插入与校验](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L24-L32)、[粘贴图片](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1801-L1878)、[文件拖放](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L1816-L1876)、[资源写入](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L1951-L2004)

**【源码】** 图片第一次点击成为原子块选区，已选中图片再次点击打开查看器；四角手柄保持比例缩放，工具栏提供左中右对齐、描述、裁剪和重置。缩放期间只创建一个撤销 checkpoint；裁剪保存也作为一步。裁剪只保存归一化区域与显示尺寸，不覆盖原图，因此可重置。[图片选中与手柄](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L494-L673)、[缩放事务](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L850-L908)、[裁剪保存/重置](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L916-L978)、[图片工具栏](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L1092-L1240)

**【源码】** 远程图片默认受设置控制：未允许自动下载时显示占位，用户重试才获取；下载在后台执行并缓存。这不能被概括为“任何网页图片粘贴后都立即本地化”。[远程图片策略](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L332-L358)、[后台下载](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L1878-L1926)

## 6. 粘贴与复制

### 6.1 格式优先级和平台边界

**【源码】** 应用内复制同时写入纯文本和携带 DocxXML 的 JSON metadata；粘贴优先识别该内部格式，其次 HTML，再把文本当 Markdown，最后尝试图片。若 HTML 只是重复纯文本，则继续走 Markdown 转换，避免 HTML 外壳阻止 Markdown 解析。[内部复制格式](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1753-L1766)、[粘贴优先级](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1790-L1884)

**【源码】** HTML 读取在 macOS 通过 `NSPasteboard` 的 `public.html` 实现，非 macOS 的 `clipboard_html()` 当前固定返回 `None`。所以 README 的“网页粘贴保留结构”不能在没有额外证据时推广为所有平台同等能力；Windows/Linux 当前源码仍有内部 DocxXML、文本 Markdown 和图片路径，但没有这条系统 HTML 读取路径。[平台剪贴板实现](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/clipboard.rs#L1-L34)、[HTML 粘贴调用条件](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1830-L1842)

### 6.2 “纯文本粘贴”仍解析 Markdown

**【源码】** `Cmd/Ctrl+Shift+V` 确实绕过内部 DocxXML、HTML 和图片，但文本仍调用 `paste_markdown`；注释也明确写着“Markdown still converts”。因此它不是“只插入字面字符、不做任何转换”的命令。[纯文本粘贴入口](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1796-L1799)、[plain 分支](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1854-L1865)

**【源码】** Markdown 粘贴按目标上下文降级：标题和代码中不解析 Markdown；非空目标只应用行内规则，每个换行形成一个段落；只有折叠光标所在的空块才启用完整块级解析，并关闭与目标自身重复的规则。代码中的粘贴保持纯文本，表格单元格内不允许再嵌套粘贴表格。[目标相关降级](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L3060-L3102)、[表格与代码限制](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2922-L2989)

### 6.3 解析器能力与拼接规则

**【源码】** 粘贴和导入共用自行实现的 HTML/Markdown 转换器。HTML 支持常见标题、列表、任务、引用、代码、表格、图片、链接和样式上下文；Markdown 解析支持标题、嵌套列表、任务、围栏代码、引用、GFM 管道表格及行内样式。整行图片只在文件导入/原地文件模式生成图片块，普通行内粘贴明确不启用图片和 `<…>` autolink；不能把它当作完整 CommonMark 兼容层。[HTML 节点转换](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/paste.rs#L259-L396)、[Markdown block 解析](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/paste.rs#L750-L1015)、[Markdown 行内范围](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/paste.rs#L1112-L1120)

**【源码】** 粘贴多块时，首个普通段落与光标前文本合并，光标后原文接到最后一个文本块；若粘到代码块则全部扁平化；若把一个表格粘进表格，则从当前/所选单元格开始填充矩阵，必要时增加行列。粘贴进来的 block 会生成新 id，避免 id 冲突。[块拼接](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2922-L3048)、[表格矩阵粘贴](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/table_ops.rs#L174-L215)

## 7. 撤销、自动保存与历史边界

**【源码】** 每个历史项是整篇 `Document` 加字符选区的快照，存入 `Arc<Snapshot>`。最多 500 步，估算总量约 256 MiB，但即使长文档也至少保留 20 步。Undo/redo 会清除块选区和单元格选区；文档图标和折叠状态保持在当前值，不随历史快照回退。[快照结构及上限](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L319-L352)、[历史外状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L366-L385)、[undo/redo](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L669-L698)

**【源码】** 同一 block 内同类连续输入或删除，在相邻操作间隔小于 700ms 时合并为一个撤销步骤，窗口随输入滑动；公式编辑使用独立类别。Enter、转换、粘贴和结构操作会打断合并，光标主动移动也会打断。[checkpoint 合并规则](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L603-L647)、[光标移动打断合并](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L568-L581)

**【源码】** 编辑协议是视图调用 `checkpoint`，Core 改模型/选区，再由 `Editor::changed()` 展开光标、发变更事件、刷新查找和菜单并安排滚动。保存等待 500ms 静默期，但连续输入最晚从最早未保存变更起 2 秒写一次；`Cmd/Ctrl+S` 立即 flush。保存失败保持未保存状态并在顶栏暴露错误。[变更管线](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L480-L520)、[自动保存调度](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L886-L909)、[保存事件入口](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L545-L567)

**【源码】** 核心 fuzzer 每步检查模型与选区不变量、generation 是否随文档变化，并周期性做 DocxXML 往返；最后完整 undo 到起点、redo 到终点。默认测试配置是 40 个种子、每个 300 步，可由环境变量调整。这是测试覆盖的证据，不是本报告已运行通过的声明。[fuzzer 目标](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/fuzz_tests.rs#L1-L4)、[随机编辑覆盖](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/fuzz_tests.rs#L106-L219)、[往返与撤销检查](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/fuzz_tests.rs#L530-L577)

## 8. 搜索与定位

### 8.1 文档内查找/替换

**【源码】** `Cmd/Ctrl+F` 打开页内查找，`Cmd/Ctrl+Shift+H` 打开替换；非 macOS 还提供 `Ctrl+H`。单行选区可成为初始查询。查找是大小写不敏感、非重叠子串匹配，跳过行内对象；没有正则或全词匹配选项。[快捷键](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L90-L122)、[查询初始化](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/find.rs#L50-L88)、[匹配算法](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/find.rs#L21-L45)

**【源码】** Enter/Shift+Enter 循环下一处/上一处；当前命中若离屏会居中滚动，并避开右上查找框。关闭后保留当前命中为文本选区。每次编辑会刷新命中；单个替换和“全部替换”都各自建立一个 checkpoint，替换文本继承命中起始 run 的样式，但去掉行内对象标记。[命中导航与滚动](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/find.rs#L90-L148)、[替换与历史](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/find.rs#L150-L204)、[查找框 UI](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/find.rs#L221-L399)

### 8.2 文档库搜索

**【源码】** `Cmd/Ctrl+J` 是另一层级的搜索：标题命中排在正文命中之前，正文使用解析后的纯文本，结果显示上下文摘要；空查询显示最近浏览的 20 篇，方向键选择、Enter 打开。它不是当前文档内的查找。[全局搜索快捷键](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L95-L107)、[结果排序与最近浏览](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/search.rs#L241-L293)、[摘要和高亮](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/search.rs#L40-L108)

## 9. 阅读与“安静/专注”体验

**【源码】** 固定源码中未发现名为专注、Zen、打字机模式的独立功能；不要把品牌中的 quiet 直接当成某种专注模式。实际体验由以下按需显现和减少干扰的机制构成。

- **选中才出现工具栏。** 折叠光标、拖动、斜杠菜单、块选择或单元格选择时都不显示文本格式工具栏；输入或 Esc 后暂时隐藏，直到选区变化。工具栏尽量浮在选区上方，空间不足才放到下方。[工具栏显示条件与定位](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L1275-L1303)。**【源码】**
- **输入时收敛指针和手柄。** 输入隐藏鼠标指针直到下次移动，手柄在输入后也等鼠标移动再出现；光标 500ms 闪烁，操作后重新计时。[相关视图状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L139-L162)、[光标闪烁](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L451-L477)、[手柄抑制](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L344-L355)。**【源码】**
- **光标自动可见。** 编辑后安排 reveal；光标距视口边缘不足 40px 时滚动，块选择、图片和查找命中有相应定位分支。[reveal 实现](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L1247-L1292)。**【源码】**
- **阅读模式保留非编辑操作。** 阅读模式没有手柄、工具栏和光标，不注册输入处理器，但移动、选择、复制、折叠仍可用；普通 `E` 返回编辑。除键上下文限制外，还用进入阅读时的文档快照回滚任何漏网编辑，属于双保险。[阅读模式策略](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/reading.rs#L1-L92)、[只读键上下文](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/keymap.rs#L80-L121)。**【源码】**
- **长文导航可收放。** 目录默认只占一个展开控件，固定后才占据侧栏；窗口太窄则变成浮层。目录随滚动/光标高亮当前章节，支持跳转和目录自身的折叠。[目录布局模式](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/catalogue.rs#L35-L85)、[目录刷新与滚动监听](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/catalogue.rs#L200-L269)、[目录 UI](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/catalogue.rs#L288-L345)。**【源码】**
- **可减少外围 chrome。** 侧栏可切换，页面有标准/宽/全宽，支持全屏；文档主题控制正文字体、间距和代码行号等。[侧栏与全屏快捷键](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L90-L122)、[页宽计算](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/catalogue.rs#L165-L170)、[官方个性化指南](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/src/09-personalize.xml#L1-L20)。**【源码/指南】**

## 10. 快捷键体系

以下以 macOS 符号列出默认值。非 macOS 会把 `Cmd` 映射为 `Ctrl`，`Alt` 映射为 `Ctrl` 的词导航/删除，并按平台调整 Home/End；行内代码从 macOS 的 `Cmd+Ctrl+C` 映射为 PC 的 `Ctrl+Shift+C`，避免与复制冲突。[平台映射](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/shortcuts.rs#L14-L39)。**【源码】**

| 类别 | 默认操作 |
|---|---|
| 光标与选择 | 方向键移动；Shift 扩展；Alt+←/→ 词移动；Cmd+←/→ 行首尾；Cmd+↑/↓ 文档首尾；Cmd+A 逐级全选；Esc 进入/退出块选择 |
| 编辑历史 | Cmd+Z 撤销；Cmd+Shift+Z 或 Cmd+Y 重做 |
| 剪贴板与保存 | Cmd+C/X/V；Cmd+Shift+V 文本粘贴但仍解析 Markdown；Cmd+S 立即保存 |
| 段落 | Enter 新块；Shift+Enter 段内换行；Tab/Shift+Tab 缩进或表格单元格导航 |
| 行内样式 | Cmd+B 粗体；Cmd+I 斜体；Cmd+U 下划线；Cmd+Shift+X 删除线；Cmd+Ctrl+C 行内代码；Cmd+K 链接；Cmd+Alt+H 重用上次颜色 |
| 块转换 | Cmd+Alt+0 正文；Cmd+Alt+1…9 标题；Cmd+Shift+7 有序；Cmd+Shift+8 无序；Cmd+Alt+T 任务；Cmd+Alt+C 代码；Cmd+Shift+> 引用 |
| 插入与排列 | Cmd+Alt+S 分隔线；Cmd+Alt+A 3×3 表格；Cmd+Shift+L/E/R 左/中/右对齐；Ctrl+Shift+↑/↓ 移动块；Cmd+Enter 切换任务 |
| 搜索与模式 | Cmd+J 文档库搜索；Cmd+F 页内查找；Cmd+Shift+H 替换；Ctrl+Alt+V 阅读；Ctrl+Alt+E 编辑 |
| 窗口与帮助 | Cmd+N 新文档；Cmd+O 打开 Markdown；Cmd+] 或 Cmd+\ 侧栏；Cmd+/ 快捷键面板；macOS Ctrl+Cmd+F、其他平台 F11 全屏 |

来源：[编辑器完整绑定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/keymap.rs#L80-L180)、[工作区绑定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L90-L122)、[模式绑定](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/modes.rs#L10-L19)。**【源码】**

**【源码】** 快捷键并非硬编码为不可更改的帮助文本：设置页按应用、文档、编辑、文字样式、段落与块分组；用户绑定会重建 GPUI keymap，菜单和工具栏通过 action 名获取当前提示。[自定义绑定重建](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/shortcuts.rs#L134-L164)、[设置中的命令目录](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/shortcuts.rs#L166-L239)

## 11. 性能与支撑架构

### 11.1 块级虚拟化与增量失效

**【源码】** 页面使用 GPUI `List`，条目为标题区、可见 block 和末尾区域；文本、标题、列表的子级会摊平成单独列表行，避免长嵌套列表成为一个巨大条目，而引用、高亮块、分栏和表格等容器仍在自己的条目内绘制。因此虚拟化粒度是 block/可见行容器，不是单字符，也不是表格单元格。[列表条目](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L46-L89)、[Editor 的虚拟化状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L107-L127)

**【源码】** `sync_items` 用 generation、布局尺寸、主题、图片和链接状态等组成帧键；滚动时若依赖未变可跳过整树遍历。需要更新时，为每行计算 key，与上一帧比较前后缀，只对中间变化区间执行 `ListState::splice`，并保护视口顶部的位置。[增量同步](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L1030-L1107)

**【源码】** block key 对文本内容只散列长度与 run 数量，而非完整文本；可见行本来就会重新测量，代价是离屏行在同长度修改后可能暂存过期高度，滚动到该行时才修正。源码注释估计，在百万字符、约 73k block 页面上，散列全文每次按键额外约 3ms。这是有意识的性能/准确性取舍，不应描述为完整内容寻址缓存。[`hash_block`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L91-L129)

**【源码】** 文档顺序单元和公式编号按 `core.generation` 缓存，目录也按 generation 重算；block 路径有带校验的 memo，并依据命中/缺失情况退避重建。浮层主要使用上一帧布局定位，检测到边界或内容原点过期时请求下一帧修正。[帧级派生状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L1330-L1354)、[路径缓存](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L466-L488)、[浮层修正](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L1387-L1397)

### 11.2 图片与数学资源

**【源码】** 文档图片通过共享缓存按可见窗口保留；离屏解码数据超过 64 MiB 时优先淘汰最久未使用项，其余离屏项 30 秒后回收，每 2 秒清理并释放 GPU 图像。并发解码上限为 2，已失效排队任务会跳过；图片通过 `load_visible_only` 延迟到实际可见时加载。[图片缓存](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L180-L215)、[淘汰与并发解码](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L217-L305)。公式使用前述独立 thread-local 布局缓存。

### 11.3 官方性能数字的正确解释

**【官方实测】** README 报告百万字符混合文档中输入 P95 5.78ms、退格 P95 5.55ms、滚动 P95 4.14ms。性能文档给出的环境是 Apple M5 Pro / 48 GiB、macOS 26.5、release build；样例为 1,001,307 字符、549 章，包含段落、列表、表格和代码。[README 数据](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/README.md#L79-L97)、[测试环境](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/PERFORMANCE.md#L1-L19)

这些数字**不是本报告实测，也不是当前固定 commit 的测试结果**：性能文档标注测试源码为 `0e92b51`。每次操作后同步 `window.draw`，统计应用内帧处理，不含屏幕呈现延迟，不等于键盘到屏幕的端到端延迟。官方脚本确实依次执行 100 次输入、退格和滚动，而不是只渲染一张静态图。[原始结果与口径](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/PERFORMANCE.md#L15-L35)、[性能脚本](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/tools/harness/examples/readme-performance.txt#L1-L15)

**【源码】** 撤销的整文档快照意味着长文档每次独立编辑需要复制较多状态；项目用 500 步/约 256 MiB/至少 20 步限制其增长。虚拟列表降低绘制成本，但不能据此推断所有超大表格、复杂公式或图片文档都与基准样例相同。[历史上限](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L319-L345)

## 12. 官方演示与截图能提供什么证据

**【指南/演示】** 固定 commit 的官网 HTML 只嵌入静态首页截图，没有链接动态演示；仓库中也未找到已发布的 MP4/WebM/GIF 文件。官网当前抓取结果与这一结构一致，但网页本身会变化，所以报告以[固定官网源码](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/site/public/index.html#L90-L98)和[截图引用](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/site/public/index.html#L170-L172)为准。这一结论不排除其他渠道存在视频。

仓库内有三类更接近交互的一手材料：

1. **可编辑的内置指南。** 官方欢迎指南明确说这些文档由 Quiet.md 写成、可以直接编辑，不是只读手册；写作、块、表格、代码、公式、图片各有实例。[欢迎指南](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/src/01-welcome.xml#L1-L21)。**【指南/演示】**
2. **指南截图及生成步骤。** `steps.txt` 逐行创建示例文档、用 Shift+方向键选择文字、截图格式工具栏，再输入 `/` 截图斜杠菜单。对应 [`toolbar.jpg`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/assets/toolbar.jpg) 与 [`slash.jpg`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/assets/slash.jpg) 展示的是选择工具栏、底部选区字数和分组插入菜单。[截图脚本](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/shared-mock/guides/shots/steps.txt#L1-L44)。**【指南/演示】**
3. **可复现的 UI harness。** 支持键盘、鼠标、IME、模拟剪贴板、截图、逐像素回归和录屏命令；菜单示例实际打开模式菜单、斜杠菜单和块菜单。不过录屏能力存在不等于仓库已发布对应视频。[harness 说明](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/tools/harness/README.md#L1-L52)、[菜单演示脚本](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/tools/harness/examples/menus.txt#L1-L49)。**【指南/演示】**

本报告查看了上述截图和脚本，但没有运行安装包、录制新演示或把截图当作对全部交互细节的独立验证。

## 13. 可迁移的交互模式

以下为**【推断】**，是从 Quiet.md 源码抽象出的设计模式，不代表应原样复制其实现。

1. **把“字符—块—单元格”设计成显式状态机。** 价值在于双击、Esc、Shift+方向键、拖出单元格等操作都有一致的语义；前提是集中处理状态互斥、作用域和退出路径，而不是让各控件自行解释选区。参考：[Core 状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L405-L431)、[拖选切换](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/table.rs#L111-L136)。
2. **所有入口汇入同一个 Core。** Markdown、斜杠、块菜单和快捷键只在“表达意图”上不同，后续共享树操作、选区修复和撤销。这样更容易让同一操作在鼠标与键盘下保持一致。参考：[转换入口](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1164-L1176)、[Core 转换](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1624-L1640)。
3. **让自动转换可逐层撤销。** 触发字符与格式转换分为两步，能缓解“编辑器擅自吞掉输入”的失控感。迁移时需要同时设计 coalescing 中断，否则分步可能被再次合并。参考：[转换 checkpoint](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2730-L2740)。
4. **粘贴采用“来源格式 × 目标上下文”的矩阵。** 应用内结构、网页 HTML、Markdown 文本、图片分别处理，同时根据标题、代码、空块、非空块和表格决定降级。这个模型比单一“粘贴为文本/富文本”更能保留结构；但“纯文本仍解析 Markdown”的命名是否合适，应单独评估，不能当作普遍最佳实践。参考：[粘贴分派](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1801-L1879)、[目标降级](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L3073-L3102)。
5. **用渐进显现降低视觉噪声。** 无选区不显示格式栏，输入时隐藏手柄和指针，目录可收成单按钮，阅读时移除编辑 chrome。关键是同时保证键盘路径和可发现性，不能只依赖悬停。参考：[工具栏条件](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L1275-L1287)、[阅读键上下文](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/keymap.rs#L80-L121)。
6. **富文本源码保真可以采用“未变 block 原样写回 + 读回验证”。** 对本机 Markdown 而言，这比每次全文件规范化更能保护用户原有排版；读回验证和规范化兜底是必要安全网，不能只靠文本匹配。参考：[写回保护](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/markdown.rs#L327-L344)。
7. **让行内对象成为文本坐标中的原子。** U+FFFC 为公式等对象提供稳定的文本位置，渲染、选择、删除和粘贴都能围绕同一坐标工作；需要额外处理对象的宽度、命中和选区绘制。参考：[对象模型](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L43-L68)、[对象选区绘制](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/text.rs#L986-L1031)。
8. **性能优化要与不变量测试绑定。** generation、虚拟化、增量 splice、缓存和快照各有独立取舍；随机编辑后检查选区、树结构、往返与 undo/redo，比只测静态打开速度更接近真实编辑。参考：[fuzzer](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/fuzz_tests.rs#L530-L577)、[增量行同步](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L1087-L1107)。

## 14. 后续对照时应观察的维度

以下只是后续比较清单，不包含对任何本地实现的结论：

- **文档真值与格式保真**：源码、block 树还是二者映射；未知语法如何保留；保存是否重写未编辑行；front matter、CRLF、图片路径与冲突如何处理。
- **选区状态机**：字符/块/单元格如何互斥；双击、三击、Esc、Cmd/Ctrl+A、Shift+方向键、原子块和折叠内容的规则；跨容器与跨单元格边界。
- **Unicode 与 IME**：UTF-8/UTF-16 映射、grapheme/word 导航、组合输入是否触发过早转换、候选窗位置和阅读模式行为。
- **输入转换**：每类标记的触发字符与上下文；是否允许中文符号；转换后的 marks 延续；第一次撤销是否恢复字面文本。
- **块操作可达性**：斜杠、手柄、快捷键和拖放是否等价；空块与非空块行为；嵌套限制；多块与折叠章节是否一起移动。
- **列表与表格边界**：Enter/Backspace/Tab 在空项、首行、末格和合并格中的结果；表格内允许多少种块；整行/列与矩形选择的差别。
- **剪贴板矩阵**：内部格式、HTML、Markdown、纯文本、图片的实际优先级；各平台支持的 clipboard flavor；“纯文本”是否真的字面粘贴；代码与表格目标如何降级。
- **撤销与保存**：输入合并窗口、结构操作粒度、图片调整是否只产生一步、历史内存上限、图标/折叠是否入史、连续输入时最迟保存时间。
- **搜索作用域**：库搜索与页内查找是否分离；是否搜索折叠内容、代码和行内对象；替换是否继承样式、如何分组撤销；离屏命中如何定位。
- **专注与阅读**：哪些控件只在需要时出现；无鼠标路径是否完整；阅读模式是否同时从键绑定、输入处理和模型回滚防止编辑。
- **性能口径**：不仅比较 P50/P95，还要记录文档组成、块数量、表格/公式/图片比例、构建版本、机器、窗口与缩放；区分应用内帧处理、键盘到屏幕延迟、内存和磁盘成本。
- **退化与恢复**：无效公式、缺失/远程图片、超大图片、保存失败、外部修改、未知 DocxXML/Markdown 结构是否保留内容并提供可恢复路径。

## 证据局限

1. 本报告是固定 commit 的静态源码、官方文档、截图与脚本研究；没有安装或操作 Quiet.md，也没有运行其测试、fuzzer、harness 或性能脚本。因此“源码确认”表示实现路径和条件可见，不等于所有平台交互都已实测。
2. README 的“网页粘贴保留结构”和性能数字均未被直接照收为跨平台事实；前者存在当前源码中的非 macOS HTML 读取缺口，后者属于项目方在不同 commit、单一高端 Mac 上的应用内帧测量。
3. 官方固定官网只有静态首页图；本报告没有找到官方仓库内已发布的动态演示文件。截图可证明视觉布局与演示意图，不能证明动画手感、输入延迟或全部边界行为。
4. 文档可能与代码不同步。例如 [`docs/FORMAT.md`](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/FORMAT.md#L40-L47) 仍写有“FORMAT 3 的直接 Markdown 文件支持不属于本分支”，但当前 `Store`、本地文件模块和 0.1.8 发布说明已经包含该能力。本报告遇到冲突时以当前源码为准，并显式保留这类不确定性。[0.1.8 发布说明](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/docs/releases/0.1.8.md#L19-L29)
5. 对“未提供独立源码模式/专注模式”的否定判断只适用于本次固定树及所查入口；没有对历史版本、未公开构建或其他发布渠道作断言。


## 15. Markion 对照：结论与现状基线

### 15.1 结论

**不建议为了 Quiet.md 的体验而把 Markion 改写成另一套富文本文档树，更不需要更换技术栈。**两者当前都是 Rust + GPUI 自绘桌面应用；差异主要在文档真值、交互状态和派生状态管线，而不是 GUI 框架。

Markion 应保留以下已有优势：

- `MarkdownDocument.text` 是唯一可编辑真值，所有可视修改经过带文档身份、版本和原始范围校验的 mutation boundary；不需要在保存时把另一套富文本树翻译回 Markdown。证据：`src/lib.rs:588-678`、`src/lib.rs:1264-1377`。
- Visual Edit 已是默认模式，而非尚待增加的源码旁预览；`ViewMode::default_mode()` 当前返回 `VisualEdit`。证据：`src/model.rs:1254-1287`。`docs/keyboard-shortcuts.md:141` 仍称 Split 为默认，属于文档漂移，不应作为现状依据。
- 已有 source/display projection、caret affinity、IME marked range、按视觉行导航、虚拟列表、块稳定 ID、语义撤销和增量区域解析。证据：`src/visual.rs:109-294`、`src/app/state.rs:270-607`、`src/app/state.rs:794-982`、`src/source_mapped.rs:1-334`。
- 斜杠、块转换/复制/删除/拖动、直接表格单元格编辑、代码/数学/图片编辑器、富文本和图片粘贴已经存在。证据：`src/block_edit.rs:11-535`、`src/app/editing.rs:341-639`、`src/lib.rs:3287-3357`、`docs/visual-editing-quality.md:5-32`。

因此，目标应是**减少操作摩擦、补齐文本编辑状态机、提高大文档连续性**，不是再做一次“支持 WYSIWYG”。

### 15.2 Quiet.md 与 Markion 的核心模型差异

Quiet.md 的 `Document -> Block tree -> RichText runs/inline objects` 是编辑真值；本机 Markdown 在打开时解析、保存时写回，并通过未变 block 原样保留和读回验证控制格式化损耗。[Quiet 模型](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/model.rs#L196-L209)、[Markdown 写回](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/markdown.rs#L303-L344)。

Markion 则直接修改 Markdown 源文本，视觉模型只是带精确 source range 的投影。这一区别意味着：

1. Quiet 的列表嵌套、容器拆分、富文本单元格、合并格和图片裁剪可以直接操作树节点；Markion 必须先证明相应 Markdown/HTML 源范围能无损修改。
2. Quiet 的即时输入转换会实际删除内部标记；Markion 的标记必须保留在 canonical source 中，只能改变显示与撤销分组，不能机械复制其内部转换算法。
3. Markion 不需要引入 Quiet 的 DocxXML 或“保存时整篇重建”。对标准 Markdown 用户，直接源修改在可解释性、外部工具兼容和冲突恢复上更有优势。

## 16. 编辑体验逐项差距

### 16.1 文本编辑基线：优先级高，且不必先改文档模型

**Quiet.md：**单击定位、双击按 Unicode word boundary 选词、三击选块文本、Shift 扩展、grapheme/word 横向移动、视觉行纵向导航、逐级全选，并有字符选区到块选区的显式转换。[选择与导航](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L2200-L2285)、[word/grapheme](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L272-L317)、[逐级全选](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2580-L2627)。

**Markion 当前：**

- 主键表有逐 grapheme 左右、上下、Shift 扩展、Home/End，但没有按词移动/删除、Shift+Home/End、PageUp/PageDown 等常见绑定。证据：`src/app/bootstrap.rs:266-315`、`src/app/editing.rs:3572-3820`。
- Visual 有双击选词，但 `click_count >= 2` 都走同一选词路径，没有独立三击块/段落选择。证据：`src/app/preview.rs:1223-1316`。
- Source 的鼠标按下只区分 Shift 与普通点击，没有使用 `click_count` 做双击/三击选择。证据：`src/app/editing.rs:4440-4468`。
- `Ctrl/Cmd+A` 直接选择整篇，不具备当前块 -> 容器 -> 整篇的逐级扩张。证据：`src/app/editing.rs:3766-3780`。
- Source 的右键只在 AI 开启且存在选区时打开另一菜单，没有常规的剪切/复制/粘贴上下文菜单；Visual 则走块菜单。证据：`src/app/root_view.rs:542-554`、`src/app/preview.rs:5084-5095`。

**建议：**先建立一份跨 Source/Visual/代码/表格/浮层输入框的“文本编辑基线”，统一词移动、词删除、行/块选择、上下文菜单和平台快捷键。Redo 还应重新评估 macOS 的 `Cmd+Shift+Z`：当前稳定规范明确禁止它，只允许 `Cmd/Ctrl+Y`（`openspec/specs/markdown-editing/spec.md:1600-1620`），Quiet 则同时提供两种重做绑定。[Quiet 快捷键](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/keymap.rs#L80-L180)。这是产品契约变化，需走 OpenSpec，而不是悄悄添加别名。

### 16.2 选区模型：从单一 Range 升级为显式状态机

**Quiet.md：**字符、块和单元格选区是三个明确状态；Esc、Shift+方向、块手柄和表格拖选负责状态转换，块选择可多选/拖动，单元格选择支持矩形、整行和整列。[Core 选区状态](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L405-L431)、[表格选区](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/table_ops.rs#L7-L39)。

**Markion 当前：**`DocumentTabState` 只有一个 canonical `selected_range: Range<usize>`；VisualBlockId 可用于定位操作目标，但没有块选区/多块选区状态，表格也没有独立矩形选区。证据：`src/app/state.rs:794-982`。Visual 的选区格式功能只接受一个安全 inline run 或一个表格单元格内的选择；跨 run、跨单元格、跨 source island 不提供格式化。证据：`src/app/editing.rs:296-339`。

**建议重构：**增加 GPUI-free 的交互模型，而不是把 Block tree 引入持久化：

```text
SelectionState
  ├─ Text { anchor, head, affinity, table_scope }
  ├─ Blocks { anchor_block, head_block }
  └─ Cells { table_block, anchor_cell, head_cell, axis }

SelectionState + VisualBlock/source map
  -> copy/delete/format/drag 所需的精确 SourceRangeSet
```

关键是集中实现进入、扩展、收缩、Esc 退出、模式/Tab 切换和 stale version 清理；各菜单不得自行解释选区。文本选区仍映射到 canonical byte range，块/单元格选区只是交互层，不成为第二份文档。

### 16.3 Markdown 即时输入：从“输入后碰巧解析”变成可预测规则

**Quiet.md：**支持标题、三种无序列表、任务、引用、代码、分隔线、闭合行内样式、公式与 URL 自动链接，并按代码、表格、标题等上下文限制；还支持部分中文触发符号。[块规则](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2661-L2783)、[行内规则](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2812-L2915)。

**Markion 当前：**源输入经 parser 会在下一次派生时成为相应结构，也有自动配对和 emoji 补全，但没有独立的输入规则事务层。`auto_pair_action` 在行首等上下文也对 `*` 无条件配对；因此输入常见的 `* ` 无序列表前缀需要先处理配对与列表语义的冲突。证据：`src/auto_pair.rs:11-145`。规范只覆盖“前缀输入后派生成结构”，没有覆盖中文触发、输入规则上下文和转换历史边界。证据：`openspec/specs/markdown-editing/spec.md:1496-1548`。

**建议：**在 canonical mutation 前增加 GPUI-free `InputRuleEngine`：输入意图先由普通文本规则处理，再按当前 VisualBlock/字段类型决定是否触发配对、块转换、行内样式、链接或 emoji。所有规则返回同一类 `EditTransaction`，包含精确范围、selection_after、origin 和 undo 边界。

Quiet 把“触发字符输入”和“格式转换”拆成两个历史步骤，第一次撤销可恢复字面标记。[Quiet 转换历史](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2730-L2740)。Markion 不应机械复制其双 checkpoint：其 canonical Markdown 本来就保留标记，转换可能只是 projection 改变。更合适的验收是“Undo 不吞掉意外整段输入、配对/结构命令可独立撤销”；若产品坚持逐层恢复视觉转换，需要显式定义 presentation-only 历史状态，而不能引入隐式第二文档。

### 16.4 块操作：从单块安全命令扩展到容器与多块

**Quiet.md：**块手柄同时承担整块选择和菜单入口，空行为加号；支持多块拖动、水平嵌套/退出嵌套、折叠章节随父块移动、容器拆分和边缘自动滚动。[块手柄](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L344-L412)、[拖动与嵌套](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/drag.rs#L184-L254)。

**Markion 当前：**已有悬停/焦点 grip、右键/键盘菜单、斜杠转换和单块 reorder，但 `block_can_transform_at`/`block_can_reorder_at` 会排除带嵌套子级的列表父项、引用组和其他无法证明源所有权的块；reorder 还构造整篇 replacement。证据：`src/block_edit.rs:191-249`、`src/block_edit.rs:328-391`。

**建议：**先把 `block_edit.rs` 的单块 `source_unit` 扩展为可组合的 `SourceBlockUnit`：记录容器路径、完整子树范围、前后分隔符和允许的目标父级。优先实现整块选择、多块删除/复制和父项携带子树移动，再实现水平拖放嵌套、引用拆分与折叠章节。无法证明时继续拒绝，不为功能覆盖率猜测源范围。

### 16.5 列表与表格边界：补齐高频键盘闭环

**列表：**Quiet 的 Tab/Shift+Tab 是结构降级/升级，空嵌套项 Enter 先减少层级，顶层空项再变正文。[Quiet 列表](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L1503-L1599)。Markion 已有 Enter 续列表、空项退出和 Backspace 降级/减少缩进；但普通 Visual 列表的折叠选区 Tab 最终会插入四个空格，`visual_editor_tab_target` 只处理表格字段。证据：`src/app/editing.rs:3981-4045`、`src/lib.rs:2675-2785`、`src/lib.rs:3431-3474`。应优先补结构 Tab/Shift+Tab、跨块合并和空项状态测试。

**表格：**Quiet 的矩形/行/列选择、末格 Tab 加行、富文本多块单元格和合并拆分形成完整闭环。[Quiet 表格键盘行为](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/crates/core/src/editor/core.rs#L2289-L2321)。Markion 已有直接单元格编辑、Tab 遍历、行列操作与宽度拖动，基础能力并不空；但末格 Tab 当前交给相邻块而非自动加行，且每次单元格输入会重新解析并 `format_markdown_table` 后替换完整表格。证据：`src/lib.rs:3313-3345`、`src/lib.rs:3433-3474`。

建议优先补 CellSel、末格加行、行/列复制与多块粘贴；把普通单元格文本输入改为局部 cell range patch，将全表对齐重排留给显式 Format/结构命令。GFM 不表达合并格或任意多块单元格，这些 Quiet 能力不能在不引入自定义扩展或另一文档格式的前提下等价照搬。

### 16.6 按需浮层与“安静”体验

Quiet 的安静不是独立 Zen 模式，而是：选区出现时才在附近显示格式工具栏，悬停才显示手柄，输入时抑制手柄和鼠标指针，目录可收起，阅读时去掉编辑 chrome。[Quiet 工具栏](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/overlays.rs#L1275-L1303)、[输入时收敛](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L139-L162)。

Markion 当前有三处明显远程交互：

- 斜杠面板固定 `right = 28px, top = 72px`，而非跟随查询或光标。证据：`src/app/root_view.rs:1347-1417`。
- 链接编辑器和 inline token/math 编辑器固定 `right = 24px, top = 62px`，并使用部分固定浅色样式。证据：`src/app/root_view.rs:1812-1903`、`src/app/root_view.rs:2043-2188`。
- 自动选区格式工具栏曾被移除了，理由正是固定位置遮挡和视觉噪声；当前格式操作在右键菜单中。证据：`openspec/changes/move-visual-selection-formatting-to-context-menu/proposal.md:1-12`、对应 `design.md:1-11`。

**建议不是恢复旧固定工具栏，而是建立统一 AnchoredOverlay：**根据选区/光标/atom 的已绘制 bounds 定位，优先上方、空间不足转下方，避免遮挡当前行，视口边缘夹紧，滚动/输入/模式变化有统一生命周期。工具栏仅在稳定文本选区上出现，块菜单仍可按右键/键盘调用；斜杠、链接和公式共享同一几何服务。若追求 Quiet 默认体验，可让“按需选区工具栏”成为可配置行为，同时保留当前低噪声菜单路径。

### 16.7 复制、粘贴与平台能力

Quiet 的应用内复制携带纯文本和 DocxXML metadata，粘贴按内部结构、HTML、Markdown 文本、图片及目标上下文分派；但系统 HTML 读取当前只在 macOS 实现。[Quiet 剪贴板](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L1753-L1884)、[平台边界](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/clipboard.rs#L1-L34)。

Markion 的 HTML/TSV/Word/图片粘贴与资源策略已经很强，不应以 Quiet 的非 macOS 缺口为目标降级。真正的差异是 Visual 的 Copy 固定复制 canonical Markdown source；例如选择渲染后的粗体文本，普通 Copy 得到的是带 `**` 的源片段。证据：`src/app/editing.rs:4397-4405`，当前契约亦见 `openspec/changes/fix-visual-undo-viewport-and-selection-copy/design.md:13-29`。

建议提供明确的格式语义：

- 普通 Copy：`text/plain` 为可见文本，同时携带安全 HTML fragment 和可选的应用内 Markdown metadata。
- Copy as Markdown：保留精确 authored source，服务源码工作流。
- Paste：继续保留 Markion 现有 HTML/图片能力，并按正文、标题、代码、表格单元格、link editor 等目标建立降级矩阵。

GPUI 已有 `ClipboardItem::new_html` 数据模型（`vendor/zed/crates/gpui/src/platform.rs:1512-1605`），但当前 Windows/macOS/Wayland 写出路径并未把该 HTML 作为原生格式发布：Windows 只写首项字符串/图片，macOS 对单字符串写纯文本，Wayland 只 offer 文本和自有 MIME。证据：`vendor/zed/crates/gpui/src/platform/windows/clipboard.rs:159-177`、`vendor/zed/crates/gpui/src/platform/mac/platform.rs:1027-1047`、`vendor/zed/crates/gpui/src/platform/linux/wayland/client.rs:822-839`、`vendor/zed/crates/gpui/src/platform/linux/wayland/clipboard.rs:205-219`。所以富格式 Copy 不只是应用层调用一个构造函数，还需要一个受测试覆盖的 GPUI 平台适配变更。

### 16.8 代码、公式、图片与阅读体验：以局部完善为主

- **代码：**Quiet 的可搜索语言、文件名、折叠等可借鉴；Markion 已有 syntect/two-face 高亮、IME、行号与换行策略，无需替换高亮器。优先补 Enter 保持缩进、代码折叠和语言选择可达性。证据：Markion `openspec/specs/code-and-math/spec.md:7-110`；[Quiet 代码 UI](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/render.rs#L735-L855)。
- **公式：**Quiet 点击 atom 后打开就近的多行 LaTeX 编辑器并支持 Esc/越界退出；Markion 已有异步 RaTeX、inline/display 映射和 stable off-flow editor，优先把固定右上编辑器改成附近多行编辑，并保持错误时源码可改。[Quiet 公式编辑](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/view.rs#L968-L994)；Markion `openspec/specs/code-and-math/spec.md:113-155`。
- **图片：**Markion 的本地资源、粘贴、拖放、下载/上传和恢复策略更适合 Markdown 工作流。可补原子图片选择、就近工具栏和等比缩放手柄；裁剪/查看器属于较低优先级，且裁剪结果如何以通用 Markdown/HTML 保存必须先有格式契约。[Quiet 图片操作](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/image.rs#L494-L673)。
- **阅读/导航：**Markion 已有 Read、焦点模式、打字机模式和可折叠大纲，不需要另造 Zen 名词。可增加一个组合式“无干扰写作”开关，收起侧栏/状态/编辑 chrome 并进入全屏；Quiet 的目录自动收起和 F11 全屏可作为参考。[Quiet 阅读与目录](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/editor/reading.rs#L1-L92)。
- **保存：**Quiet 连续输入时有最长写回间隔；Markion 当前每次变更重新安排 5 秒 inactivity autosave，未看到“最早未保存变更起算”的最大等待限制。证据：`src/app/application.rs:1779-1803`、`src/model.rs:999-1006`；[Quiet 保存调度](https://github.com/s1dashu/quiet-md/blob/57884b9c8e49d22effe18885c6d717d8acb67278/src/app.rs#L886-L909)。可在不打断输入的前提下增加 max-burst 保存/恢复快照，但这属于可靠性增强，不是视觉对齐的主线。

## 17. 需要优先重构的支撑架构

### 17.1 统一 Input Router 与 EditTransaction

当前 `MarkionApp::insert_newline` 依次判断 Git 字段、搜索控件、块菜单、命名输入、链接编辑器、斜杠、emoji、表格字段、代码字段和结构编辑；左右键、粘贴、剪切也各自重复焦点和浮层分派。证据：`src/app/editing.rs:3822-3966`、`src/app/editing.rs:4269-4438`。

建议逐步抽成：

```text
Key/pointer/IME
  -> FocusOwner + InputRouter
  -> EditIntent / NavigationIntent / SelectionIntent
  -> InputRuleEngine 或 SelectionReducer
  -> EditTransaction { exact edits, selection_after, origin, undo_boundary }
  -> CheckedMutation boundary
  -> ChangeSet / after-change subscribers
```

这不改变 canonical text，也不强制一次重写所有 handler；可先让 Enter/Backspace/Tab 和复制/粘贴走新接口，再迁移其余命令。价值是同一操作从键盘、斜杠、手柄和菜单进入时共享选区修复与历史，而不是继续增加条件分支。

### 17.2 统一 Visual Inline Layout 与几何缓存

Markion 普通链接文本走 shaped `VisualEditableText`；含数学、HTML 图片、显式换行、脚注导航、上下标或 stable link affordance 时走另一套 mixed flex-wrap 路径。证据：`src/app/preview.rs:2912-2994`、`src/app/preview.rs:3302-3361`。两条路径都需要独立保证 wrap、selection、caret、IME 和导航，维护成本会随 atom 类型继续增长。

建议把共同部分收成一个 `VisualInlineLayout` 接口：统一的 source/display segments、原子对象边界、视觉行、命中测试、selection quads 和 IME bounds。现有 `VisualProjectionSegment.atomic` 已能表达原子范围，不必复制 Quiet 的 U+FFFC 内部文本模型。几何缓存应以 `VisualBlockId + document version + layout inputs` 为键，而不是同时维护 block index -> snapshot 与 index -> id 两套映射；当前结构见 `src/app/state.rs:873-904`、`src/app/preview.rs:1155-1203`。

### 17.3 让增量派生真正局部化

当前 `SourceMappedCache` 可以复用独立区域的语义解析，但后续仍会：

1. 对整篇重新 `split_regions`，并组装/克隆/平移所有区域的 preview blocks。证据：`src/source_mapped.rs:206-294`、`src/source_mapped.rs:419-497`。
2. 每次缓存失效都对全部 preview blocks 调 `build_visual_blocks`，其中还扫描全文引用定义并逐块生成 inline runs。证据：`src/lib.rs:3202-3281`、`src/visual.rs:595-871`、`src/visual.rs:2058-2105`。
3. `reconcile_visual_block_ids` 对每个旧块在新列表执行 `position`，第二轮还执行 `any` 和完整 `filter`，存在明确的块数平方级路径。证据：`src/source_mapped.rs:599-683`。
4. 每次文档变化清空全部 visual navigation snapshots，并在搜索打开时同步重算所有匹配。证据：`src/app/application.rs:901-930`、`src/app/application.rs:1987-2089`。

建议分两步：

- **低风险热点修复：**先给块协调建立 range/kind 索引或有序双指针，去除重复线性查找；减少仅为比较而做的整块 clone/shift；给各阶段加计时和分配计数。此步应先 profile 验证，不把任何单一候选预先认定为唯一原因。
- **结构性局部化：**让 CachedRegion 同时持有 VisualBlock 切片和稳定 lineage；局部编辑只重建受影响区域及必要边界，后缀只重定位偏移；全文引用/front matter/未闭合 fence 等仍走经过验证的全局 fallback。

文本容器目前确实是 `String`，`replace_range` 会移动后缀字节（`src/lib.rs:1418-1470`），但不应在没有 profile 的情况下先换 rope。先消除全量 VisualBlock 重建和平方级协调，通常比改变所有 UTF-8 offset/API 的 rope 迁移更小、更可逆。

### 17.4 用 ChangeSet 管理副作用，而不是全量 after_document_changed

建议 mutation receipt 衍生一次 `ChangeSet`，至少区分：

- `TextOnly` / `InlineStructure` / `BlockStructure`；
- `OutlineChanged` / `ReferencesChanged` / `AssetsChanged`；
- 精确旧范围、增量和受影响的稳定块 ID；
- selection/IME/geometry 是否需要修复。

Visual rows、预览、outline、搜索、图片操作、Git、状态栏和 autosave 各订阅所需范围。当前活跃的编辑行应同步得到局部结果；非关键的全文搜索、预览和资源工作可以 debounce/background，并通过 document identity + version 丢弃迟到结果。不要简单把整个 Visual Edit 派生异步化后用旧 selection 操作新文档。

## 18. 推荐实施顺序

| 阶段 | 内容 | 体验收益 | 重构风险 |
|---|---|---|---|
| P0：建立对照与性能基线 | 固定相同 fixture；记录文本/IME/列表/表格/复制/撤销路径；给 Visual 派生分阶段计时；先修复可证明的平方级热点 | 避免只按功能清单开发；大文档卡顿可量化 | 低，主要是诊断与局部优化 |
| P1：统一编辑基本功 | 词/行/块导航和选择，Source 双击/三击与上下文菜单，结构 Tab，输入规则与配对上下文，常用快捷键 | 最快的日常手感提升 | 低至中，无需改文档真值 |
| P1：就近且按需的交互 | AnchoredOverlay、选区工具栏、光标旁斜杠、局部链接/公式编辑、输入时收敛 chrome | 对齐 Quiet 的安静和直接操纵 | 中，需统一焦点/滚动/遮挡生命周期 |
| P2：显式 SelectionState + Transaction | 字符/块/单元格状态机，统一 EditTransaction，多块复制/删除/移动，多格式复制 | 对齐 Quiet 最核心的编辑模型 | 高，但可在 GPUI-free 层独立测试 |
| P2：真正局部的 Visual 派生 | Visual region cache、线性身份协调、ChangeSet 订阅、搜索/预览调度 | 解决长文连续输入和 IME 稳定性 | 高，必须保留全量解析 correctness oracle |
| P3：复杂块与高级内容 | 容器拆分、多块/子树拖动、表格矩形选择与末格加行、代码折叠、图片操作 | 接近完整 Quiet 工作流 | 中至高，逐项受 Markdown 可表达性约束 |

建议按多个小型 OpenSpec changes 落地，而不是一个“对齐 Quiet”巨型变更。当前仓库已有大量 Visual Edit 行为契约；自动选区工具栏、Redo 键位、复制格式、末格 Tab 等已有相反或较窄的稳定要求，proposal 必须明确修改它们，不能只改代码。

## 19. 验收标准与本次证据边界

### 19.1 建议验收矩阵

至少用同一份混合 fixture 在两款应用执行并比较：

1. 英文连续输入、中文/日文 IME、emoji 与组合字符；
2. `# `、`* `、`- `、`[] `、```lang + Space/Enter、闭合粗体/斜体/行内代码/链接/公式；
3. 单击、双击、三击、Shift 扩展、按词移动/删除、逐级全选、Esc 块选择；
4. 列表 Enter/Backspace/Tab/Shift+Tab，首项/末项/嵌套/空项；
5. 表格首末格、跨格拖选、行列选择、粘贴矩阵、撤销；
6. 块手柄、斜杠、右键、快捷键、拖动的等价结果；
7. 可见文本/Markdown/HTML 复制，正文/代码/表格目标的粘贴降级；
8. 输入、点击、搜索、撤销、模式切换时的 caret 和 viewport；
9. malformed Markdown、无效公式、缺失图片、CRLF、外部修改和恢复；
10. 100 KB、300 KB、600 KB、1 MB 文档的模型耗时与真实 GPUI 输入到绘制耗时。

每个结构命令仍应满足：精确 UTF-8 range、一个可解释的 undo 单元、正确的 selection_after、stale event 拒绝、非编辑交互不提升 document version、未修改源字节保持不变。性能门槛应分机器记录趋势；真实交互仍需人工或 harness 验证。

### 19.2 Markion release 诊断结果

本次在 Windows 当前工作区执行：

```text
cargo run --release --example bench_large_doc
```

该基准测量局部文本变更后读取 VisualBlock 模型的成本，不包含 GPUI element/layout/paint，也不是键盘到屏幕的端到端延迟。结果如下：

| 文档大小 | 每轮次数 | P50 | P95 | 备注 |
|---|---:|---:|---:|---|
| 约 100 KB | 100 | 10.28 ms | 11.96 ms | 模型层局部输入 + Visual 读取 |
| 约 300 KB | 100 | 160.56 ms | 199.96 ms | 已明显超过常见帧预算 |
| 约 600 KB | 100 | 687.48 ms | 801.82 ms | 呈明显非线性增长 |

命令在 300 秒后台时限到达时只完成到上述 600 KB 行，后续基准没有形成结果；不应把缺失项写成通过或失败。机器未建立跨设备基准，以上仅是本次诊断。基准自身的口径和排除项见 `examples/bench_large_doc.rs:1-15`、`examples/bench_large_doc.rs:131-198`。

这些结果与源码中的全量 VisualBlock 重建和平方级身份协调路径共同说明：**大文档输入链路应先做 profile 和局部化，不能仅凭“区域解析已复用”推断每键成本已有界。**现有确定性测试只断言 499 个区域被复用、至多 1 个区域重新解析（`src/source_mapped.rs:1422-1446`），没有约束后续 VisualBlock 重建、克隆和身份协调的工作量。

### 19.3 本次未验证事项

- 未安装或实际运行 Quiet.md；Quiet 侧是固定 commit 的源码、官方文档、截图与脚本证据。
- 未对 Markion 做完整人工 GUI 对照或键盘到屏幕测量；本地差距来自当前源码与规范。
- 未运行 Quiet 官方性能 harness；其百万字符 P95 数字是不同 commit、单一高端 Mac 的项目方应用内帧测量，不能与本表直接排名。
- 未为本次研究修改任何编辑实现或运行完整测试套件；唯一新增交付物是本报告。临时 Quiet 克隆位于 `target/tmp/quiet-md-research`，不属于产品源码。
