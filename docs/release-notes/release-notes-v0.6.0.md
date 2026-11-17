# Markion v0.6.0

Markion v0.6.0 makes everyday Markdown editing easier to discover and control. Welcoming APEC — Shenzhen welcomes you!（喜迎APEC，深圳欢迎您）

## Changes

- Keep Files and Outline visible together with an independent, resizable right outline pane, or retain the left outline layout.
- Use compact slash commands, an optional automatic popup, and a matching emoji palette with up to 36 visible candidates.
- Edit through compact block menus, hover flyouts, precise selection actions and Image Properties; use table Row/Column controls and Shift+Enter cell breaks. Fixes include menu dismissal, New Tab crashes and empty callout editing.
- Start from five offline guides and twelve reusable templates through Help → Examples and Templates. Copy the complete folder before editing.

**Compatibility:** No manual document migration is required; new preferences are optional. Windows has no Authenticode signature; updater payloads use Minisign. macOS signing/notarization will be confirmed from the tag build; non-notarized downloads require `xattr -cr /Applications/Markion.app` after installation, or a `curl -LO` download. [Installation guidance](https://github.com/willmove/markion#install).

**Downloads:** Windows x64 (NSIS/ZIP), macOS arm64/x64 (DMG), Linux x86_64 (DEB/RPM/AppImage); attached after the tag workflow passes.

**Verification:** Windows workspace: 2,216 passed, 5 existing ignored; formatting, Clippy, bundle, example and strict spec checks pass. Native package/signing/mirror results and unavailable manual scope are recorded in the [release evidence](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.0-preflight.md). [OpenSpec review](https://github.com/willmove/markion/blob/main/docs/verification/openspec-archive-review-2026-11-17.md).

**Full comparison:** https://github.com/willmove/markion/compare/v0.5.3...v0.6.0

---

# Markion v0.6.0（中文说明）

Markion v0.6.0 让日常 Markdown 编辑更易上手、操作更顺畅。喜迎APEC，深圳欢迎您！

## 更新

- 大纲支持独立可调宽的右侧面板，与左侧文件树同时显示，也可保留左侧大纲布局。
- 斜杠命令采用紧凑面板，可关闭自动弹出；emoji 补全统一外观，最多显示 36 个候选。
- 紧凑块菜单、悬停子菜单、精确选区操作与图片属性改善编辑体验；表格支持行列菜单及 Shift+Enter 单元格换行，并修复菜单关闭、新建标签页崩溃和空提示块编辑问题。
- 通过“帮助 → 样例与模板”获取 5 篇离线指南和 12 份模板；编辑前请复制整个目录。

**兼容性：** 无需手动迁移文档，新增偏好均为可选字段。Windows 尚无 Authenticode 签名，更新包采用 Minisign 验证。macOS 签名／公证状态将以标签构建结果为准；未经公证的版本安装后需执行 `xattr -cr /Applications/Markion.app`，或使用 `curl -LO` 下载。[安装说明](https://github.com/willmove/markion#install)。

**下载：** Windows x64（NSIS/ZIP）、macOS arm64/x64（DMG）、Linux x86_64（DEB/RPM/AppImage）；标签流水线通过后附上。

**验证：** Windows workspace 共 2,216 项通过、5 项既有忽略；格式、Clippy、bundle、样例与严格规格检查通过。原生打包、签名、镜像及无法执行的人工项见[发布记录](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.0-preflight.md)；另见 [OpenSpec 审查](https://github.com/willmove/markion/blob/main/docs/verification/openspec-archive-review-2026-11-17.md)。

**完整变更对比：** https://github.com/willmove/markion/compare/v0.5.3...v0.6.0
