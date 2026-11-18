# Markion v0.6.2

This patch makes Markion easier to install on macOS and fixes reliability issues in Markdown editing and recovery.

## Changes

- Open Markion normally from Developer ID-signed, Apple-notarized Apple Silicon and Intel DMGs.
- Manual saves keep priority over older autosaves, recovered drafts stay separate across launches, and Source Select All respects CRLF paragraph boundaries.
- Moving or indenting list items keeps their nested content together. Table editing preserves escaped pipes and nearby prose, and table controls follow the active theme.
- HTML and Word import preserve literal text. Word imports also retain more formatting and drawings in their original order.
- Exports preserve explicit link targets. PDF export uses a compatible fallback font when needed. The Visual Edit block grip shows its format, and the app icon has been refreshed.

Documents and settings carry over automatically. On macOS, drag Markion into Applications and double-click it. Windows SmartScreen may still prompt because the installer has no Authenticode signature.

Download the attached Windows x64 installer or portable ZIP, macOS Apple Silicon or Intel DMG, or Linux x86_64 AppImage, DEB or RPM.

The Windows workspace suite passed. Both published macOS DMGs passed signature, notarization, and installed-app launch checks on their matching CPU architectures. The maintainer’s v0.6.2 exception covers unrun manual checks for installed-package interaction and DPI, real IME, browser clipboard and Word/WeChat interoperability (including DOCX producer qualification), and signed upgrades from the previous Windows/AppImage release, including rollback and extended runs. [Verification record](https://github.com/willmove/markion/blob/v0.6.2/docs/verification/release-v0.6.2-preflight.md).

See the [full comparison](https://github.com/willmove/markion/compare/v0.6.1...v0.6.2).

---

## 简体中文

本次补丁简化了 macOS 安装，并修复 Markdown 编辑、保存和恢复中的可靠性问题。

### 更新内容

- Apple Silicon 和 Intel 安装包均使用 Developer ID 签名并通过 Apple 公证，下载后可正常打开。
- 较早的自动保存不会覆盖手动保存。不同草稿在重新启动后仍独立保留，源码模式的全选也会遵循 CRLF 段落边界。
- 移动或缩进列表项时保留其子项。编辑表格时保留转义竖线和相邻正文，表格控件跟随当前主题。
- 导入 HTML 和 Word 时保留原文。Word 导入还会保留更多格式，并按原顺序呈现绘图。
- 导出会保留显式链接地址。PDF 导出会在需要时使用兼容字体。可视化编辑的块首握柄会显示格式，应用图标也已更新。

文档与设置无需手动迁移。macOS 上将 Markion 拖入「应用程序」后双击即可启动。Windows 安装程序尚无 Authenticode 签名，仍可能显示 SmartScreen 提示。

从附件下载 Windows x64 安装程序或便携 ZIP、macOS Apple Silicon 或 Intel DMG，或 Linux x86_64 的 AppImage、DEB 或 RPM。

Windows 工作区测试已通过。两个 macOS DMG 均在对应架构上通过签名、公证与安装启动检查。维护者为 v0.6.2 授权一次性例外，未执行的人工项目包括完整安装包交互与 DPI、真实输入法、浏览器剪贴板及 Word／微信互操作（含 DOCX 生成端兼容性验证），以及从上一版 Windows／AppImage 执行签名升级、失败回退和长时间运行。[验证记录](https://github.com/willmove/markion/blob/v0.6.2/docs/verification/release-v0.6.2-preflight.md)。

查看[完整对比](https://github.com/willmove/markion/compare/v0.6.1...v0.6.2)。
