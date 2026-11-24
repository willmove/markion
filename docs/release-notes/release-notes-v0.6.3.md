# Markion v0.6.3

This patch adds an Arch/Omarchy-compatible native package, safer code-block formatting, and more consistent table rendering and theming.

## Changes

- Format fenced JSON without extra tools, or use configured Biome/Prettier, google-java-format, rustfmt, or ruff for JavaScript/TypeScript, Java, Rust, and Python. Optional format-on-paste and `Ctrl+Alt+L` / `Cmd+Option+L` preserve Markdown fences and undo history.
- Render math inside GFM table cells across Visual Edit, Split Preview, and Read modes, with wide formulas scaled to their cells and source boundaries preserved during interaction.
- Table selections, drag targets, controls, and collapsible source editors now follow the active built-in or custom theme.
- Install on Arch Linux and Omarchy with the new `.pkg.tar.zst` package. Non-AppImage Arch-family update prompts offer that native package; AppImage launches retain signed in-place updates.

**Compatibility:** No migration is required. External code formatters are configured separately and are not bundled. Both macOS DMGs are Developer ID signed and Apple notarized; drag Markion to Applications and open normally. Windows NSIS has no Authenticode signature, so SmartScreen may prompt. The pacman package is manually installed and is not part of the signed self-update channel.

**Downloads:** Windows x64 (NSIS/ZIP), macOS arm64/x64 (DMG), and Linux x86_64 (DEB/RPM/AppImage/pacman `.pkg.tar.zst`); see the attached assets. On Arch/Omarchy, run `sudo pacman -U markion-0.6.3-1-x86_64.pkg.tar.zst`.

**Verification:** Workspace gates and native packaging CI passed; the pacman package passed Arch installation and resource checks. Omarchy desktop interaction, real IME/browser/Word/WeChat interoperability, and previous-version signed upgrades remain unverified; the maintainer authorized a one-time v0.6.3 exception for these unrun checks. See the [verification record](https://github.com/willmove/markion/blob/v0.6.3/docs/verification/release-v0.6.3-preflight.md).

**Full comparison:** https://github.com/willmove/markion/compare/v0.6.2...v0.6.3

---

# Markion v0.6.3（中文说明）

本次补丁新增 Arch/Omarchy 兼容的原生软件包，并改进代码块格式化、表格渲染和主题一致性。

## 更新

- 无需额外工具即可格式化 JSON 代码块；也可配置 Biome/Prettier、google-java-format、rustfmt 或 ruff 处理 JavaScript/TypeScript、Java、Rust 和 Python。可选粘贴时格式化与 `Ctrl+Alt+L` / `Cmd+Option+L` 会保留 Markdown 围栏和撤销历史。
- 在可视化编辑、分栏预览和阅读模式的 GFM 表格单元格中渲染公式；过宽公式会缩放到单元格内，交互时仍保留准确的源码边界。
- 表格选区、拖放目标、控件和可折叠源码编辑器现在会跟随当前内置或自定义主题。
- Arch Linux 和 Omarchy 可使用新的 `.pkg.tar.zst` 软件包。非 AppImage 的 Arch 系安装会在更新提示中选择该原生包；AppImage 仍使用签名原地更新。

**兼容性：** 无需迁移。外部代码格式化工具需单独配置，不会随应用捆绑。两个 macOS DMG 均使用 Developer ID 签名并通过 Apple 公证，拖入「应用程序」后正常打开。Windows NSIS 没有 Authenticode 签名，可能出现 SmartScreen 提示。pacman 包需要手动安装，不属于签名自更新通道。

**下载：** Windows x64（NSIS/ZIP）、macOS arm64/x64（DMG）、Linux x86_64（DEB/RPM/AppImage/pacman `.pkg.tar.zst`），见附件。Arch/Omarchy 运行 `sudo pacman -U markion-0.6.3-1-x86_64.pkg.tar.zst` 安装。

**验证：** 工作区门禁和原生产打包 CI 已通过；pacman 包已通过 Arch 安装与资源检查。Omarchy 桌面交互、真实输入法、浏览器／Word／微信互操作，以及上一版本签名升级仍未验证；维护者已明确授权 v0.6.3 对这些未执行项目使用一次性例外。详见[验证记录](https://github.com/willmove/markion/blob/v0.6.3/docs/verification/release-v0.6.3-preflight.md)。

**完整变更对比：** https://github.com/willmove/markion/compare/v0.6.2...v0.6.3
