# Markion v0.4.3

Adds portable Windows and RPM Linux packages, plus Visual Edit and menu polish since v0.4.1.

## Highlights

### Packaging

- **Windows portable `.zip`**: extract and run `markion.exe` with no installer.
- **Linux `.rpm`**: native package for Fedora/RHEL-family systems, alongside the existing DEB and AppImage.
- Both new packages ship the same pinned MarkNice publishing workspace and pass the same package verification as every other format.

### Visual Edit

- Table toolbar now waits for a short hover dwell and hides after a delay, so it no longer flashes while moving the pointer across a table.
- Closer source fidelity for YAML front matter, reference images, ragged tables, and invalid math blocks.

### Menus and localization

- Menu dropdowns align to their menu buttons using measured label widths in every UI language (English, Simplified Chinese, Traditional Chinese, Japanese, French, German, Spanish).
- New READMEs for Traditional Chinese, Japanese, French, German, and Spanish.

### Git sync

- Sync actions regrouped under **File → Backup and Sync** and **File → Advanced Git Tools**.
- Simpler onboarding: address correction, clearer validation, and a streamlined setup dialog.

## Compatibility

- No migration is required for Markdown files, preferences, or workspace data.
- Windows and macOS packages remain unsigned; first launch may require SmartScreen or Gatekeeper bypass.
- The `v0.4.2` tag exists but never published a GitHub Release (Linux CI dependency failure). Use v0.4.3.

## Downloads

- Windows x64: NSIS installer and portable `.zip`.
- macOS Apple Silicon: DMG.
- Linux x86_64: DEB, RPM, and AppImage.

## Verification

- `cargo test --workspace` passed locally.
- Windows, macOS, and Linux native CI builds, package extraction, and MarkNice bundle verification passed.
- Windows updater signature and `update.json` published.

**Full comparison**: https://github.com/willmove/markion/compare/v0.4.1...v0.4.3

---

# Markion v0.4.3（中文说明）

自 v0.4.1 以来：新增 Windows 便携包与 Linux RPM，并改进 Visual Edit 与菜单体验。

## 主要更新

### 发布打包

- **Windows 便携 `.zip`**：解压后直接运行 `markion.exe`，无需安装。
- **Linux `.rpm`**：面向 Fedora/RHEL 系的原生安装包，与现有 DEB、AppImage 并列。
- 两个新包同样内置固定的 MarkNice 发布工作区，并通过与其他格式一致的打包校验。

### Visual Edit

- 表格工具栏改为悬停短暂停留后显示、延迟后隐藏，指针划过表格时不再闪现。
- YAML 头、引用图片、参差表格与无效公式块的源码保真度更高。

### 菜单与本地化

- 各界面语言（英语、简体中文、繁体中文、日语、法语、德语、西班牙语）的菜单下拉均按实测标签宽度对齐到菜单按钮。
- 新增繁体中文、日语、法语、德语、西班牙语 README。

### Git 同步

- 同步相关入口重组到 **文件 → 备份与同步**、**文件 → 高级 Git 工具**。
- 引导更简单：地址纠正、更清晰的校验与更精简的设置对话框。

## 兼容性

- Markdown 文件、偏好设置与工作区数据无需迁移。
- Windows 与 macOS 安装包仍未签名，首次启动可能需要手动绕过 SmartScreen 或 Gatekeeper。
- 存在 `v0.4.2` 标签，但因 Linux CI 依赖问题未创建 GitHub Release。请使用 v0.4.3。

## 下载

- Windows x64：NSIS 安装程序与便携 `.zip`。
- macOS Apple Silicon：DMG。
- Linux x86_64：DEB、RPM 与 AppImage。

## 验证

- 本地 `cargo test --workspace` 通过。
- Windows、macOS、Linux 原生 CI 构建、安装包解压与 MarkNice 工作区校验通过。
- Windows 更新器签名与 `update.json` 已发布。

**完整变更对比**: https://github.com/willmove/markion/compare/v0.4.1...v0.4.3
