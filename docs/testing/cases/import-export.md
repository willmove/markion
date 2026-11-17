# 导入导出、浏览器与资源策略

遵循[统一入口](../README.md)和[隔离指南](../human-runbook.md)。素材路径相对于 run/workspace；逐个变体新建结果行，破坏性分支使用新 run。

## X01 内置导出逐格式检查

**优先级：** P1

**前置与素材：** 07-exports.md（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 选择内置后端，分别导出 styled/plain HTML、DOCX、PDF、LaTeX、PNG/JPEG（界面提供的入口）。
2. 实际用浏览器、Word、PDF 查看器、图片查看器打开。
3. 核对标题、中文、列表、表格、图片、公式、图表和分页；每格式取消保存一次。

**预期：** 按格式已支持语义检查；文件非空且可打开，无意外缺页/缺字；HTML 静态公式/图表可显示；取消不落半成品，原文不变。

**变体（逐项建行）：** 字体/纸张/方向/页边距；图片策略；长文多页；PNG/JPEG snapshot。

**证据与收尾：** 记录实际查看器版本、输出哈希和页面截图；合法容器不等于排版正确。

## X02 Pandoc 与工具失败

**优先级：** P1

**前置与素材：** 07-exports.md（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 记录 Pandoc 和实际 PDF engine 版本，选择该后端导出 Word/PDF。
2. 改为不存在的工具路径，另测程序非零退出、不可写目的与取消。
3. 恢复内置后端再导出并实际打开。

**预期：** 实际执行工具且产物正确；错误/回退行为有反馈，不伪报成功；不损原文；内置和外部引擎结果各自标注。

**变体（逐项建行）：** 工具存在／缺失；合法／缺字体；内置与 Pandoc 分开。

**证据与收尾：** 命令日志、工具版本、实际产物和查看截图；缺工具 BLOCKED，不能利用提前 return 的测试计通过。

## X03 真实 Office 来源导入资格

**优先级：** P1

**前置与素材：** imports/synthetic.docx + 外部 corpus（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 先用 synthetic.docx 熟悉文件 → Import Word → 报告 → 保存 Markdown。
2. 按 corpus 模板登记至少 20–30 份合法可分发/脱敏 Word、WPS、LibreOffice 样本及原软件版本。
3. 逐份对照 accepted-view 正文顺序、标题、列表、合并表、图片、OMML、脚注和损失诊断。

**预期：** 每个真实来源分别有结果；插入/move-to 保留、删除/move-from 省略；不支持内容显式诊断；未有真实样本就不声称兼容资格。

**变体（逐项建行）：** 三类 producer，Markion/MarkNice 导出再导入另列；小/图多/近限制。

**证据与收尾：** corpus 登记、原始/预期/输出哈希、报告截图、耗时/峰值；synthetic 不能填真实 producer。

## X04 导入取消、目标与资源发布

**优先级：** P0

**前置与素材：** imports/synthetic.docx、invalid.docx（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 分别取消输入 picker、转换任务、损失报告、保存 picker。
2. 选已有 Markdown/asset目录或已打开的目标，另测中文空格目录和不可写目的。
3. 断网导入新路径；连同 assets 移到另一位置，四模式重开。

**预期：** 源 DOCX/原标签不变；不覆盖已有目标；资源完整后才出现 Markdown；取消不遗留可误认成功的半文档；无需外部 converter。

**变体（逐项建行）：** 合成有效／坏 ZIP／真实有损 corpus；每个取消点；保存阶段完成边界单列。

**证据与收尾：** 输出目录前后清单、源哈希、报告、四模式截图及失败残留检查。

## X05 MarkNice 正常流程和查看器

**优先级：** P1

**前置与素材：** 07-exports.md（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 从 Markion 打开本地排版工作区，改变主题、字号和正文。
2. Copy MD 到纯文本编辑器，Copy WeChat 到实际微信编辑器；下载 HTML/DOCX 并实际打开。
3. 导入 Word 替换浏览器正文，Copy MD 回到新笔记；打印并实际保存 PDF 后打开。

**预期：** Copy MD 等于当前 textarea；浏览器操作不写回原 Markdown；微信/Word 格式可用；PDF 不含工具栏/手机框，实际落盘。

**变体（逐项建行）：** 每个支持默认浏览器/平台；Word 与第二查看器分别记录；15 主题自测是补充。

**证据与收尾：** 浏览器/微信/Word版本、接收内容、下载哈希和PDF截图；同步既有 MarkNice 矩阵。

## X06 MarkNice 权限、会话与资源失败

**优先级：** P1

**前置与素材：** 07-exports.md、05-images.md（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 拒绝剪贴板权限、取消下载/打印、重复复制导出，再使旧浏览器会话过期。
2. 测试本地图片省略确认／取消，大于导出阈值的图片，工作区外路径资源。
3. 断开外网加载本地 runtime；对远程图片不可用观察反馈。

**预期：** 错误/限制明确；不泄露工作区外路径；大图有可见回退；纯本地 runtime 不依赖 CDN；远程图片不被承诺离线；原文完整。

**变体（逐项建行）：** Windows/macOS/X11/Wayland 默认浏览器；HTML/Word/PDF 独立。

**证据与收尾：** 授权框、失败/取消结果、资源与原文比较；无法控制真实权限弹窗记 BLOCKED。

## X07 图片资源策略、路径与取消

**优先级：** P0

**前置与素材：** 05-images.md、images/checker.png（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 偏好 Images 分别设置 local keep/copy、clipboard save、remote keep/download。
2. 对未命名文档插图时取消保存，再保存到中文空格路径插入；复制资源目录后重开。
3. 粘贴含图 Markdown/富文本，再 Save As；传输中更改文档目标。

**预期：** 策略只在显式插入/操作触发；开/渲染/保存不自动传输；取消不遗留资源；相对URL可移植；失效传输不写错新目标，恢复入口可找回。

**变体（逐项建行）：** 文件/剪贴板图片/远程下载；重复资源；引用同 URL 的两个 occurrence。

**证据与收尾：** 资源清单/哈希、网络操作次数或服务日志、插入与后续 rewrite 分开 Undo。

## X08 外部上传与失败恢复

**优先级：** P0

**前置与素材：** images/checker.png，专用 uploader（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 按图片指南配置专用 PicGo HTTP/Core 或自定义程序，先 Test upload，再上传一个指定 occurrence。
2. 对自定义脚本可用 Python+uploader.py：参数一行一个，脚本绝对路径后接完整 {file}；只返回 loopback URL。
3. 测非零退出、多 URL、超时/取消；成功后在应用前改源使结果失效，检查恢复，Undo/Redo。

**预期：** 仅接受一个合法HTTP(S) URL；失败/过期不写错源；恢复保留可恢复信息；Undo/Redo 不重复传输；Windows 无多余控制台。

**变体（逐项建行）：** HTTP/Core/command；成功/失败/超时；真实服务缺失 BLOCKED；无需真实云密钥。

**证据与收尾：** 命令/服务版本、去敏日志、恢复记录和源字节；本地 stub 不证明真实云 provider 兼容。
