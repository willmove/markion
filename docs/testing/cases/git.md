# 备份、同步与版本恢复

先读[统一入口](../README.md)和[人工隔离指南](../human-runbook.md)。G组目录位于run/git；其他素材位于run/workspace。每个变体独立建行，外部工具/平台缺失如实记录。

## G01 同步配置与本地 remote

**优先级：** P1

**前置与素材：** git/local、git/remote.git、git/peer（run 根下）（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 新 run，打开 git/local，在备份与同步界面审阅已有仓库并采用。
2. 分别使用新建普通笔记目录、空目录和混合内容仓库进入配置，切换可见路线再取消。
3. 用生成的 remote.git 绝对路径作测试同步位置；若当前 UI 不接受本地地址，采用 local 已有 origin 路线并把该配置分支记 BLOCKED。

**预期：** 默认路线和提示符合实际目录；取消不改仓库；不把日常仓库或真实账号带入测试；已有安全 origin 可用。

**变体（逐项建行）：** 专用笔记仓库／普通／空／混合；取消／确认；无 Git 可执行程序。

**证据与收尾：** UI截图、本地 git status/remote -v/config；去除个人路径。

## G02 双端同步与暂存区所有权

**优先级：** P0

**前置与素材：** git/local、git/peer（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 在 local 的 note.md 加 LOCAL，UI 立即同步；到 peer 执行 git pull 检查。
2. 在 peer 编辑 other.md，git add other.md、git commit -m QA-peer、git push；回应用立即同步。
3. 在 local 手工暂存 unrelated.md，再编辑另一文件，只审阅选择目标版本/同步。

**预期：** 双方预期修改完整；无关已暂存内容按受支持同步路径被安全保护/拒绝，不悄悄混入提交；同步状态与实际 ahead/behind 一致。

**变体（逐项建行）：** 仅本地／仅远端／双方不同文件；已暂存／未暂存；自动保存进行中。

**证据与收尾：** 每端 status/log/diff、提交文件清单、磁盘比较；命令均用 git -C run/git/peer 或 local。

## G03 冲突、远程删除和重命名

**优先级：** P0

**前置与素材：** 新 run 的 git/local、git/peer（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 两端修改 note.md 的同一行，各自形成提交，peer push 后 local 立即同步。
2. 分别选本地、远端、组合编辑、保留草稿/取消冲突流程。
3. 新 run 在 peer 删除或重命名应用已打开文件并 push，local pull；dirty 缓冲区另测。

**预期：** 冲突明确且双方内容可恢复；合并/取消不丢工作；干净重命名可准确映射，远程删除不被自动保存复活；dirty 不静默覆盖。

**变体（逐项建行）：** 文本冲突／修改删除／重命名；干净／dirty 打开标签；每个解决选择。

**证据与收尾：** 冲突前 refs、选择后内容、日志和恢复草稿；禁止 force push、reset --hard 日常仓库。

## G04 中断与陈旧 checkpoint

**优先级：** P0

**前置与素材：** G03 的未解决冲突和隔离 profile（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 进入确实存在 journal 的同步/解决会话，记录状态与文件，再结束自己的测试 PID。
2. 重启相同 profile，审阅恢复入口，继续或保留草稿/丢弃已审阅记录。
3. 在复制的 run 中外部改目标，再恢复；若无法捕获“本地已提交但上传结果未知”，该分支记 BLOCKED。

**预期：** 不会重复生成内容快照/提交；未解决工作保留；外部新内容不被覆盖；已完成 checkpoint 不永久锁住编辑；未知投递状态需要核对不盲重放。

**变体（逐项建行）：** 解决中／本地操作中／投递未知；继续／取消／外部变化。

**证据与收尾：** journal副本、refs与内容前后比较、重启截图；只在确实触发状态时填通过。

## G05 局部版本、历史比较与恢复

**优先级：** P0

**前置与素材：** git/local（见[素材说明](../fixtures/README.md)）。

**步骤：**

1. 编辑两文件，Create local version 只选择一个路径，核对新提交。
2. 打开文件/仓库 history，多页浏览并 Compare with current。
3. Restore in editor，再一次 Undo；另用 Save a Copy。

**预期：** 提交只含审阅路径并保留暂存区所有权；比较不改文件；恢复为当前文档未保存编辑且可撤销，不直接改其他文件或执行 checkout。

**变体（逐项建行）：** 干净／dirty 当前文档；历史文件已删/重命名；分页/切仓库过期结果。

**证据与收尾：** 提交范围、历史diff、restore前后源、Undo比较和副本哈希。
