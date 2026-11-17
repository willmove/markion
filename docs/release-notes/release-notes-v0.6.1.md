# Markion v0.6.1

This patch makes writing more comfortable with Korean UI, adjustable spacing, background update preparation and safer block insertion.

## Changes

- Use Markion in Korean, with all eight UI and README languages kept in step.
- Adjust line spacing to 75–300% and paragraph spacing to 0–128px in Preferences → Appearance, without changing your Markdown.
- Discover updates every five hours and automatically prepare verified Windows/AppImage downloads by default. Choose Restart and Update or Later; unsaved edits prevent installation. Other package formats retain browser downloads.
- Insert tables and dividers below a block without replacing existing content. Compact insertion controls keep headings clear, and dividers leave room to continue typing.

**Compatibility:** No manual document or settings migration is required. Windows has no Authenticode signature; updater Minisign verification is separate. macOS DMGs are ad-hoc signed and not notarized: for a trusted browser-downloaded copy, run `xattr -cr /Applications/Markion.app` after installation and browser-downloaded updates. [Installation help](https://github.com/willmove/markion/blob/v0.6.1/docs/user-guide.md#install).

**Downloads:** Windows x64 (NSIS/ZIP), macOS arm64/x64 (DMG), Linux x86_64 (DEB/RPM/AppImage); see attached assets.

**Verification:** Candidate checks and native publication verification are tracked in the [preflight record](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.1-preflight.md). Full installed-package/DPI, macOS/Linux interaction, browser/WeChat/Word, real IME and previous-version signed-upgrade acceptance remain unverified under an explicit v0.6.1 maintainer exception; these are not passing results.

**Full comparison:** [v0.6.0...v0.6.1](https://github.com/willmove/markion/compare/v0.6.0...v0.6.1).

---

## 简体中文

本次补丁带来韩语界面、可调间距、后台更新准备和保留原内容的块插入，让日常写作更顺手。

### 更新内容

- 提供完整韩语界面，八种界面与 README 语言同步维护。
- 在“偏好设置 → 外观”中调整 75–300% 行距与 0–128px 段距，不改变 Markdown 内容。
- 每五小时发现更新，默认自动准备经过验证的 Windows/AppImage 下载；可选择“重启并更新”或“稍后”，未保存的编辑会阻止安装。其他包格式仍通过浏览器下载。
- 在块下方插入表格或分隔线时保留已有内容；紧凑插入控件清晰显示标题层级，分隔线后可继续输入。

**兼容性：**文档与设置无需手动迁移。Windows 暂无 Authenticode 签名，更新的 Minisign 验证与其独立。macOS DMG 为 ad-hoc 签名、未公证：确认浏览器下载可信后，安装及每次浏览器下载更新后执行 `xattr -cr /Applications/Markion.app`。[安装帮助](https://github.com/willmove/markion/blob/v0.6.1/docs/user-guide.md#安装)。

**下载：**Windows x64（NSIS/ZIP）、macOS arm64/x64（DMG）、Linux x86_64（DEB/RPM/AppImage），见附件。

**验证：**候选检查与原生发布验证见[发布前记录](https://github.com/willmove/markion/blob/main/docs/verification/release-v0.6.1-preflight.md)。完整安装包/DPI、macOS/Linux 交互、浏览器/微信/Word、真实输入法及上一版签名升级验收，按维护者针对 v0.6.1 的明确例外授权保留为未验证，不能视作通过。

**完整对比：**[v0.6.0...v0.6.1](https://github.com/willmove/markion/compare/v0.6.0...v0.6.1)。
