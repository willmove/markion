# Markion v0.4.4

Adds file-tree duplication and live workspace watching, and fixes Git sync conflicts that could leave a repository permanently read-only.

## Highlights

### File tree

- **Duplicate**: files and folders now have a **Duplicate** item in their context menu. The copy is placed beside the original under a free, localized name.
- **Live workspace watching**: the file tree refreshes automatically when files are added, removed, or renamed outside Markion. If a filesystem watcher cannot be started, Markion falls back to a timed refresh. Manual **Refresh** is still available, and automatic refreshes do not overwrite the status bar.

### Fixes

- **Stale Git conflict locks**: when an interrupted sync conflict was later resolved elsewhere (a later sync succeeded, or the merge was finished or aborted outside Markion), every note in the repository could refuse typing, undo, save, and file-tree deletes with "Operation running" until restart. Markion now detects the stale session, releases the lock, and reconciles the recorded sync state.
- **Automatic cleanup of finished sync records**: sync records whose work is already in the repository history are retired at startup and after each sync, so **Backup and Sync** no longer stays stuck on "Choose which note changes to keep". Records that still hold an unsaved draft are kept and offered as **Open draft** or **Discard record**.
- **Conflicts only lock the conflicted notes**: while a real conflict awaits your decision, only the conflicted files are protected; other notes in the same repository stay editable, savable, and manageable in the file tree.
- **Clearer busy messages**: the status now distinguishes "Git is working right now" from "an unfinished sync conflict needs attention", with a direct entry to resolve it.

## Compatibility

- No migration is required for Markdown files, preferences, or workspace data. Existing Git sync journals are cleaned up automatically on first launch; no manual action is needed.
- Windows and macOS packages remain unsigned; first launch may require SmartScreen or Gatekeeper bypass.

## Downloads

- Windows x64: NSIS installer and portable `.zip`.
- macOS Apple Silicon: DMG.
- Linux x86_64: DEB, RPM, and AppImage.

## Verification

- `cargo test --workspace` passed locally, and the MarkNice workspace bundle verified.
- Windows, macOS, and Linux native CI builds, package extraction, and MarkNice bundle verification passed.
- Windows updater signature and `update.json` published.

**Full comparison**: https://github.com/willmove/markion/compare/v0.4.3...v0.4.4

---

# Markion v0.4.4（中文说明）

新增文件树“创建副本”与工作区实时监听，并修复 Git 同步冲突可能导致整个仓库一直无法编辑的问题。

## 主要更新

### 文件树

- **创建副本**：文件与文件夹的右键菜单新增 **创建副本**，副本放在原条目旁边，并自动使用不冲突的本地化名称。
- **工作区实时监听**：在 Markion 之外新增、删除或重命名文件时，文件树会自动刷新。若无法启动文件系统监听，则退回为定时刷新。手动 **刷新** 仍然保留，自动刷新不会覆盖状态栏信息。

### 修复

- **过期的 Git 冲突锁**：同步冲突中断后，如果之后在别处已被解决（后续同步成功，或在 Markion 之外完成/中止了合并），仓库内所有笔记都可能拒绝输入、撤销、保存和文件树删除，并提示“操作进行中”，直到重启应用。现在 Markion 会识别过期会话、释放锁，并校正记录的同步状态。
- **自动清理已完成的同步记录**：内容已进入仓库历史的同步记录会在启动时和每次同步后自动清除，**备份与同步** 不再一直停留在“选择要保留的笔记更改”。仍带有未保存草稿的记录会保留，并提供 **打开草稿** 或 **丢弃记录** 两个选项。
- **冲突只锁定相关笔记**：真实冲突等待处理期间，只保护发生冲突的文件；同一仓库中的其他笔记仍可编辑、保存并在文件树中管理。
- **更清晰的忙碌提示**：状态栏会区分“Git 正在执行操作”和“有未完成的同步冲突需要处理”，并提供直接进入处理的入口。

## 兼容性

- Markdown 文件、偏好设置与工作区数据无需迁移。已有的 Git 同步记录会在首次启动时自动清理，无需手动操作。
- Windows 与 macOS 安装包仍未签名，首次启动可能需要手动绕过 SmartScreen 或 Gatekeeper。

## 下载

- Windows x64：NSIS 安装程序与便携 `.zip`。
- macOS Apple Silicon：DMG。
- Linux x86_64：DEB、RPM 与 AppImage。

## 验证

- 本地 `cargo test --workspace` 通过，MarkNice 工作区校验通过。
- Windows、macOS、Linux 原生 CI 构建、安装包解压与 MarkNice 工作区校验通过。
- Windows 更新器签名与 `update.json` 已发布。

**完整变更对比**: https://github.com/willmove/markion/compare/v0.4.3...v0.4.4
