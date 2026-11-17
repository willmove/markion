# Markion vX.Y.Z 实机验收记录

> 模板：尚未执行；复制后填写。创建记录不代表测试通过。日期/时间使用 Asia/Shanghai，并保留原始采样时区。

## 构建与运行身份

| 字段 | 值 |
| --- | --- |
| 阶段 | candidate / final-tag |
| 操作者／执行工具与版本 | 待填 |
| 日期与时区 | 待填 |
| 版本／提交／分支／tag | 待填 |
| dirty 状态及 diff、未跟踪文件 manifest | 待填；不能只写 HEAD |
| 构建命令／profile／工作流 URL | 待填 |
| 包路径／包 SHA-256 | 待填 |
| 实际可执行文件路径／PID／SHA-256 | 待填 |
| 包资源来源与哈希记录 | 待填；确认未借仓库资源回退 |
| 上一稳定版本／候选到最终构建映射 | 待填 |
| run 路径／fixture manifest SHA-256 | 待填 |
| 隔离配置／恢复／日志实际位置 | 待填；首次写入位置检查 |

## 环境与必需矩阵

每个平台/包/显示后端/输入法/浏览器/查看器独立一行；从统一入口的完整支持矩阵展开，不把未填环境删除以缩小分母。

| 环境 ID | OS/架构/发行版/compositor | 包格式/哈希 | GPU/驱动/显示缩放/窗口 | IME及版本 | 浏览器/Office/Pandoc版本 | 适用范围 | 结果/证据 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ENV-001 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | NOT_RUN |

## 自动化与执行方式

| 检查 | 命令/构建身份 | 实际执行范围 | 结果 | 原始日志 |
| --- | --- | --- | --- | --- |
| repository gate | 待填 | fmt/Clippy/workspace/bundle/OpenSpec | NOT_RUN | 待填 |
| examples | 待填 | 源集合/解包后集合 | NOT_RUN | 待填 |
| ignored prerequisites | 待填 | 工具版本/是否实际生成产物 | NOT_RUN | 待填 |
| callback replay | 待填 | 不代表OS_INPUT | NOT_RUN | 待填 |

## 逐项结果

附 `results.csv`。行键为 case_id + variant + environment_id + artifact_sha256 + attempt。至少一个用例组占位行不代表覆盖了其全部变体；执行前展开每个要求的分支。result 只使用 NOT_RUN/PASS/FAIL/BLOCKED/SKIP/N/A；method 使用 HUMAN/OS_INPUT/CALLBACK_REPLAY/GPUI_TEST/CLI_CHECK。

每条失败／重要交互附：前置状态、实际输入步骤、预期、实际、完整 expected/actual 比较、截图/日志、缺陷编号。修复后新增 attempt，保留首次失败。

## 单个缺陷记录（按需复制）

- 编号／严重度／对应 case+variant+环境：
- 构建哈希与最小素材：
- 最小复现步骤：
- 预期／实际／首次失败证据：
- 数据影响／临时回避：
- 修复 OpenSpec change／提交／新程序哈希：
- 复测结果／关联回归／残余限制：

## 统计与例外

| 范围 | 应执行变体 | PASS | FAIL | BLOCKED | SKIP | NOT_RUN | N/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 全部（展开后） | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |

应执行项与 N/A 分开；缺少的环境仍在矩阵和统计内。所有适用必需项未PASS时默认不就绪；P0失败先修复并复测。

| 本次版本例外编号 | case/variant/环境/构建 | 未验证内容与风险 | 原因 | 维护者明确授权原文/链接/日期 | 发布说明披露位置 | 后续跟踪 |
| --- | --- | --- | --- | --- | --- | --- |
| 无（未填写不代表已豁免） | | | | | | |

旧版本跳过授权不沿用；例外不改变原结果为PASS。自动化通过不补齐真实浏览器/Word/IME待验。

## 最终包与发布决策

- candidate 适用用例完成情况：NOT_RUN。
- 最终 tag 包哈希/资源映射、P01/P02 核心冒烟与 P06 真实升级：NOT_RUN。
- 七包、两签名、update.json、GitHub/OSS、全部构建与发布作业、双语说明：NOT_RUN。
- 必需 MarkNice/Word 平台矩阵链接及本次结果：待填。
- 当前结论：**尚未验收／不可据此宣称已就绪**。
- 未修复缺陷、未覆盖范围与本次明确授权例外：待填。
- 发布 Release／工作流／preflight 链接：待填。

## 证据归档与环境恢复

记录退出的本次PID、未触碰的日常配置检查、保留现场位置。将脱敏报告/CSV/必要日志/截图/输入输出哈希归档到版本 verification 目录；不要提交大型生成文件、真实笔记、密钥或完整付费服务请求。
