# 人工实机执行指南

从[统一入口](README.md)选择平台和用例。本指南只准备测试；不发布版本、不改日常配置。所有命令从仓库根目录执行。

## 1. 固定候选和记录

复制[报告模板](templates/release-report.md)。记录 `git rev-parse HEAD`、`git status --short`、安装包及最终执行程序哈希、构建类型、资源来源。脏工作区补充保存 `git diff --binary HEAD` 和未跟踪文件的路径／哈希；不要只用 HEAD 冒充构建源码身份。保留构建命令及工作流链接。

```powershell
git rev-parse HEAD
git status --short
Get-FileHash -Algorithm SHA256 -LiteralPath 'C:/QA/packages/Markion-setup.exe'
```

实际程序必须再计算一次哈希。源码、程序、配置或资源变化后，标记失效结果并重跑受影响项。先使用候选包，开发二进制只能做预检；最终标签包生成后仍执行 P06/P07。

## 2. 创建素材和独立环境

```powershell
python docs/testing/fixtures/prepare.py prepare --output target/tmp/release-qa/run-001
```

已有目录会被拒绝；下一次改用 `run-002`。输出含 `workspace/`、`baseline/`、`profile/`、`evidence/`、manifest 和空报告。默认准备本地 Git 双端仓库；无 Git 时显式 `--without-git`，G 组写 BLOCKED。每个改变基线或故障注入用例使用新 run；仅连续 Undo 比较可以沿用当前 run。

Windows 用 PowerShell 7 启动已准备的**独立候选程序副本**，保留其完整资源布局。下面替换程序路径后执行；环境变量只传给子进程，不改变当前用户设置：

```powershell
$qaRun = (Resolve-Path -LiteralPath 'target/tmp/release-qa/run-001').Path
$qaExe = (Resolve-Path -LiteralPath 'C:/QA/candidate/markion.exe').Path
$qaProcess = Start-Process -FilePath $qaExe -PassThru -Environment @{
    APPDATA = (Join-Path $qaRun 'profile/roaming')
    LOCALAPPDATA = (Join-Path $qaRun 'profile/local')
} -RedirectStandardOutput (Join-Path $qaRun 'evidence/stdout.log') -RedirectStandardError (Join-Path $qaRun 'evidence/stderr.log')
$qaProcess.Id
```

测试 GUI 是有意显示的交互窗口。NSIS 安装、更新、卸载会改变该用户的安装位置，P01/P06 使用专用系统测试账户或 VM 快照；环境重定向本身不能隔离安装器、系统凭据或文件关联。

Linux 示例（替换绝对程序路径）；`qa_run` 必须位于可写测试目录：

```sh
qa_run="$(pwd)/target/tmp/release-qa/run-001"
env XDG_CONFIG_HOME="$qa_run/profile/config" XDG_CACHE_HOME="$qa_run/profile/cache" \
  /absolute/path/to/markion >"$qa_run/evidence/stdout.log" 2>"$qa_run/evidence/stderr.log"
```

macOS 用专用测试账户；直接启动 `/Applications/Markion.app/Contents/MacOS/markion` 时可同样传递 XDG 配置／缓存变量，但日志仍来自该测试账户的 `~/Library/Logs/Markion`。首次浏览器下载／Gatekeeper 启动应通过 Finder 实测，不能用终端启动替代。

当前路径实现见 [src/paths.rs](../../src/paths.rs)：

| 内容 | Windows | Linux／macOS |
| --- | --- | --- |
| 配置、session、AI、Git journal、图片恢复 | `APPDATA/Markion/` | `XDG_CONFIG_HOME/markion/` |
| 文档恢复 | `LOCALAPPDATA/Markion/Recovery/` | `XDG_CACHE_HOME/markion/recovery/` |
| 日志 | `LOCALAPPDATA/Markion/Logs/` | Linux：`XDG_CACHE_HOME/markion/logs/`；macOS：`~/Library/Logs/Markion/` |

素材工具预置两类平台配置，关闭启动查更新和自动保存。先通过 UI 改一个无害设置并核对其写入上述 run 内配置、日志和进程路径，再开始编辑用例；若写入日常目录，退出该测试进程并修正隔离，不继续测试。系统安全凭据不受 XDG/APPDATA 隔离，因此 AI 用无密钥 mock；不要删除原有凭据。需要验证旧配置迁移时，只复制去除密钥引用、个人路径和历史的合成配置。

## 3. 执行、比较与恢复

在文件菜单打开 `workspace/`。先 P01 启动，再按各用例的素材、步骤、变体执行。`Primary` 表示 Windows/Linux Ctrl、macOS Cmd；词操作在 macOS 用 Option。菜单当前显示的绑定优先，检查偏好是否有重绑定；不以被 OS 截获的快捷键推断功能不存在。

每次需要验证撤销：记录独立基线 → 只执行指定编辑 → 保存并对照预期 → 一次 Undo → 保存 → 与原基线比较。保存本身不应制造一次编辑撤销。构造预期文件时由测试者写出预期改动，不从应用结果反向复制“预期”。

```powershell
python docs/testing/fixtures/prepare.py compare --expected target/tmp/release-qa/run-001/baseline/01-editing.md --actual target/tmp/release-qa/run-001/workspace/01-editing.md
```

返回 0 仅说明两文件字节相同；返回非零即不相同或读取失败，不能忽略。需要带预期改动时将独立预期写到 `evidence/expected/` 并用同一命令比较。截图用于布局，磁盘内容用于数据正确性，两者都保留。

强制结束、损坏恢复文件、只读权限、篡改签名等仅限专用测试进程／合成文件／复制的安装包。先记录 PID、路径和快照，再执行用例；不按进程名结束所有 Markion，不清空整个用户目录。测试后仅退出自己启动的进程，保留失败现场。

## 4. 收尾

逐个展开 `results.csv` 的变体行，写实际观察和证据。出现缺陷先保存原始日志、截图、输入／输出及哈希，再复测；保留首次失败行，追加修复后的新 attempt，不覆盖失败历史。

按模板汇总平台／包覆盖、ignored、BLOCKED、SKIP 和缺陷；适用必需项未完成默认未就绪。维护者本次发布例外必须有可追溯授权，不接受以前版本的跳过记录。原始 run 位于 target 下可能被清理；发布前把脱敏报告和证据复制到 `docs/verification/release-vX.Y.Z/`，在该版本 preflight 中链接。报告不能包含密钥、真实笔记或个人路径截图。
