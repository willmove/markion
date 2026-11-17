# 发版前实机验收

这是人工与 Codex 共用的验收入口。文档交付、自动化通过和实机验收通过是三个不同结果；本目录不包含任何新版本已经通过验收的声明。发布操作仍以[发布流程](../release-process.md)为准。

## 一次完整验收

1. 阅读[人工执行指南](human-runbook.md)；Agent 同时阅读[Codex 执行指南](codex-runbook.md)。记录候选提交、工作区差异、安装包和实际运行程序的 SHA-256。
2. 按下方矩阵确定环境，复制[报告模板](templates/release-report.md)，为每个平台／安装包／输入法等变体建立结果行。
3. 按[素材说明](fixtures/README.md)创建一个全新的独立运行目录。只修改工作副本；每个破坏性用例使用新目录。
4. 运行自动化门禁，保留原始日志和 ignored 项；再按 D → E → R → W → X → G → A → P 执行适用用例。安装启动 P01 必须先完成。
5. 关键编辑保存后比较完整文件，执行一次 Undo 后再比较基线。每个分支单独记结果，不用“整体看起来正常”替代。
6. 打标签前完成候选包验收；标签构建后执行 P06/P07 的最终包与升级验证，并将结论链接到 `docs/verification/release-vX.Y.Z-preflight.md`。

快速准备（仓库根目录，Python 3.10+，标准库；`run-001` 必须不存在）：

```powershell
python docs/testing/fixtures/prepare.py prepare --output target/tmp/release-qa/run-001
pwsh.exe -File scripts/check-quality.ps1
pwsh.exe -File scripts/verify-examples.ps1
```

`prepare` 也复制空报告和结果表到运行目录。Python/Git 缺失应记录；不要为运行此套件自动安装软件或操作日常配置。

## 用例目录

| 编号 | 详细步骤 | 范围 |
| --- | --- | --- |
| D01–D10 | [数据安全](cases/data-safety.md) | 保存、冲突、恢复、标签历史、文本类型 |
| E01–E13 | [编辑与输入](cases/editing.md) | 四模式、选择、IME、剪贴板、块菜单、查找替换 |
| R01–R08 | [表格与渲染](cases/rendering.md) | 表格、代码、公式、图表、导航、图片 |
| W01–W06 | [工作区](cases/workspace.md) | 文件树、文件操作、监听、打开、标签、图片只读 |
| X01–X08 | [导入导出与资源](cases/import-export.md) | 原生/Pandoc/浏览器导出、DOCX、资源策略及上传 |
| G01–G05 | [备份与同步](cases/git.md) | 配置、双端同步、冲突、中断、版本恢复 |
| A01–A05 | [AI](cases/ai.md) | 本地 mock、输入、附件、写作、文件操作恢复 |
| P01–P07 | [平台与发布](cases/platform-release.md) | 安装、样例、偏好、窗口、性能、升级、最终发布 |

以上 62 个编号是用例组。每组的分支、模式、平台及安装格式分别建行，组内部分通过不代表整组通过。分支名称可自定但应具体，例如 `E05/Visual/table/Pinyin/delayed-commit`。

## 优先级和结果

- **P0**：数据损坏、崩溃、错误目标、恢复、安装／更新安全。出现失败即停止普通发布准备，保留现场与缺陷证据。
- **P1**：功能、显示、互操作与性能观察。完整发版验收执行所有适用的 P0/P1，不因补丁版本自动省略。
- **NOT_RUN**：尚未执行；**PASS**：当前构建下操作与断言均通过；**FAIL**：实际结果不符；**BLOCKED**：缺少环境／工具／输入手势；**SKIP**：明确决定本次不执行；**N/A**：当前产品确实不支持该能力或平台不适用，必须说明依据。
- NOT_RUN/BLOCKED/SKIP 不计入通过。必需项未通过时默认未就绪；只有维护者对本次版本明确记录范围、原因和授权证据的例外，才能发布且必须披露。Agent 不自行豁免，旧版本例外不沿用。P0 失败先修复并复测，不用跳过掩盖。
- 记录分为 `OS_INPUT`（真实系统输入）、`HUMAN`、`CALLBACK_REPLAY`、`GPUI_TEST`、`CLI_CHECK`。后面三种不能填写原生 IME／跨应用剪贴板交互为 PASS。

## 平台、包和组合矩阵

| 环境 | 包格式必须逐个安装／展开验证 | 平台重点 |
| --- | --- | --- |
| Windows x86_64 | NSIS、portable ZIP | Microsoft Pinyin、CF_HTML、系统对话框、签名升级 |
| macOS Apple Silicon | arm64 DMG | 浏览器下载后启动、实际签名公证、Cmd/Option、系统 IME、public.html |
| macOS Intel | x64 DMG，在实际 Intel 环境 | 同上；交叉编译通过不能代替实机 |
| Linux X11 x86_64 | DEB、RPM 在兼容发行版；AppImage | IBus/Fcitx、剪贴板、GPU／软件 Vulkan、窗口边角 |
| Linux Wayland x86_64 | 适用的包格式 | 记录 compositor；窗口控制、首帧光标、IME、剪贴板 |

每个平台至少完整执行一轮适用用例；同平台其他包格式复用功能结果前必须证明相同应用／资源身份，仍逐包执行 P01/P02/P06。浏览器、Office 查看器和 IME 矩阵不能因此省略。

每种界面语言、每个内置主题切换一次；选择代表明／暗主题与 100%/150%/200% **真实**缩放、最小支持窗口、长文件名组合复查浮层、设置、表格和 AI。macOS 使用实际显示缩放档位记录。布局模拟不填作真实设备缩放。X11 至少记录 IBus/Fcitx 是否可用，中文／韩文历史回归分开记录；不可用写 BLOCKED。

## 基线、已知限制与维护

本套件初始审阅于 2026-11-17。稳定规范见 [OpenSpec](../../openspec/specs/)；执行前同时审阅候选提交对应的 active changes。当前右键菜单以紧凑直接控制、Row/Column 子菜单、H1–H5、图片属性编辑器为准；H6 仍可由其他入口使用。不要照历史报告要求旧选区工具栏或图片原始源码悬浮展开。

GFM 合并单元格不属于 pipe-table 编辑能力；使用 HTML 表格验证合并显示。多行／畸形图片允许已知的保守源码路径，但必须保留原文。直接微信草稿发布等未实现提案不加入已支持能力的通过声明。未明确覆盖的限制记录到报告；产品行为改变时在同一 OpenSpec change 中更新用例，保留旧编号，新增分支或新编号。

## 现有自动化与专项证据

- [质量门禁](../../scripts/check-quality.ps1)：包括整个 workspace；普通 `cargo test` 只跑根包。
- [Windows 窗口回调回放](../../scripts/Invoke-FluentEditingSmoke.ps1)：真实窗口和 paint，应用回调输入，不证明 OS 候选窗。
- [AppImage 启动脚本](../../scripts/smoke-test-appimage.sh)：Xvfb/icewm/firejail 下非空白窗口，不证明完整编辑交互；只在专用测试用户环境运行。
- [编辑覆盖契约](../visual-editing-quality.md)、[Windows OS 输入历史](../verification/windows-native-input-2026-11-11.md)、[菜单变更记录](../../openspec/changes/archive/2026-11-17-unify-compact-block-context-menu/tasks.md)。不能用某次旧修订的结果证明后续修订。
- [DOCX 真实来源矩阵](../docx-import-compatibility.md)、[MarkNice 必需人工矩阵](../marknice-workspace-release-evidence.md)、[AI 验收](../ai-agent-acceptance.md)。本套件链接并补充它们，不把待验证行改成通过。
- [记录模板](templates/release-report.md)、[逐项结果 CSV](templates/results.csv)、[真实 DOCX 样本登记](templates/docx-corpus.csv)。
