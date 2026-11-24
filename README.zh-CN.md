<p align="center"><img src="assets/markion-logo.svg" width="80" height="80" alt="Markion"></p>
<h1 align="center">Markion</h1>
<p align="center"><strong>专注表达，保留 Markdown 的自由。</strong><br>从日常笔记、技术文档到文章发布，使用 Rust + GPUI 原生编辑器。</p>

<p align="center"><a href="README.md">English</a> · <strong>简体中文</strong> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a></p>

<p align="center">
  <a href="https://github.com/willmove/markion/releases/latest"><img src="https://img.shields.io/github/v/release/willmove/markion?color=168a86" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-536878" alt="Windows, macOS, Linux">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/willmove/markion?color=168a86" alt="MIT"></a>
</p>

<p align="center"><a href="#install"><strong>立即下载</strong></a> · <a href="#new-in-v061">新版亮点</a> · <a href="#performance">实测性能</a> · <a href="docs/user-guide.md#simplified-chinese">使用指南</a></p>

![Windows 实机截图：文件树、可视化 Markdown、任务列表与表格，以及独立的右侧大纲](docs/images/markion-workspace-windows.png)

<p align="center"><sub>Windows 实机拍摄，v0.6.0 开发构建 · <a href="docs/images/README.md">截图说明与演示文档</a></sub></p>

## 为什么选择 Markion

| ✍️ 写作少一点打扰 | 📁 文件始终属于你 | 📤 从草稿走向发布 |
| --- | --- | --- |
| 可视化编辑按需隐藏语法，随时切换源码或分栏预览。 | 普通本地 Markdown 文件，无需账户。Rust + GPUI 原生编辑器，无 Electron 或内嵌 WebView。 | 导入 Word，编写代码、公式与图表，再导出 PDF、Word、HTML 或图片。 |

<a id="new-in-v060"></a><a id="new-in-v061"></a>

## v0.6.1 新亮点

| 亮点 | 带来的改变 |
| --- | --- |
| **韩语界面** | 八种完整界面语言，在偏好设置中选择 **한국어**。 |
| **按习惯调整间距** | 在**偏好设置 → 外观**中设置 75–300% 行距与 0–128px 段距。 |
| **后台准备更新** | 每五小时检查，默认自动下载；确认后才安装，并保护未保存的编辑。 |
| **插入时保留原内容** | 紧凑插入菜单与“在下方添加表格/分隔线”保留已有文字及表格，插入分隔线后可继续输入。 |

<p align="center"><img src="docs/images/markion-block-menu-windows.png" width="760" alt="Markion 实机块菜单：标题层级、列表、块操作与行内格式"></p>
<p align="center"><sub>v0.6.0</sub></p>

[完整发布说明](docs/release-notes/release-notes-v0.6.1.md) · [项目进展与验证记录](docs/project-status.md)

<a id="install"></a><a id="安装"></a>

## 下载，即刻开始

| 平台 | v0.6.3 下载 |
| --- | --- |
| **Windows x64** | [安装程序](https://github.com/willmove/markion/releases/download/v0.6.3/markion_0.6.3_x64-setup.exe) · [便携 ZIP](https://github.com/willmove/markion/releases/download/v0.6.3/markion_0.6.3_x64-portable.zip) |
| **macOS 11+** | [Apple Silicon](https://github.com/willmove/markion/releases/download/v0.6.3/Markion_0.6.3_aarch64.dmg) · [Intel](https://github.com/willmove/markion/releases/download/v0.6.3/Markion_0.6.3_x64.dmg) |
| **Linux x86_64** | [AppImage](https://github.com/willmove/markion/releases/download/v0.6.3/markion_0.6.3_x86_64.AppImage) · [DEB](https://github.com/willmove/markion/releases/download/v0.6.3/markion_0.6.3_amd64.deb) · [RPM](https://github.com/willmove/markion/releases/download/v0.6.3/markion_0.6.3_x86_64.rpm) · [pacman（Arch/Omarchy）](https://github.com/willmove/markion/releases/download/v0.6.3/markion-0.6.3-1-x86_64.pkg.tar.zst) |

[全部版本与资源校验信息](https://github.com/willmove/markion/releases)

- **Windows：**运行安装程序或解压 ZIP。暂未提供 Authenticode 签名，SmartScreen 可能需要选择**更多信息 → 仍要运行**。
- **macOS。**下载与芯片对应的 Apple Silicon 或 Intel DMG，打开后把 Markion 拖入「应用程序」，再双击启动。两个安装包均已通过 Developer ID 签名和 Apple 公证。[首次启动说明](docs/user-guide.md#安装)。
- **Linux：**为 AppImage 添加执行权限（`chmod +x markion*.AppImage`）后运行，或安装 DEB/RPM/pacman。Arch Linux 与 Omarchy 运行 `sudo pacman -U markion-0.6.3-1-x86_64.pkg.tar.zst`。无 FUSE 时可用 `--appimage-extract-and-run`。

打开 Markdown 文件或文件夹即可写作。试用内置样例时，请**先复制整个 examples 文件夹及其中的图片，再开始编辑**。[上手与模板](assets/examples/README.md)。

## 一份文档，四种工作方式

| 模式 | 适合的场景 |
| --- | --- |
| **可视化编辑** · 默认 | 在渲染后的 Markdown 中直接编辑正文、表格、代码与公式。 |
| **源码** | 精确掌控全部 Markdown 语法。 |
| **分栏预览** | 源码与预览并排，可开启同步滚动。 |
| **阅读** | 干净、不可编辑的阅读视图。 |

切换模式时保留位置、选区与撤销历史。[快捷键](docs/keyboard-shortcuts.md)。

| 从构思到交付 | 已有能力 |
| --- | --- |
| **技术写作** | GFM 表格、代码高亮、LaTeX 公式、Mermaid、脚注、提示块与可跳转大纲。 |
| **迁入既有内容** | 本地 DOCX 导入；粘贴 Word、Google Docs 或网页中的富文本。[Word 导入](docs/word-import.md) |
| **导出与发布** | Markdown、带样式/纯 HTML、LaTeX、DOCX、PDF、PNG/JPEG，以及本地 MarkNice 公众号排版工作区。[发布指南](docs/marknice-workspace.md) |
| **图片与资源** | 粘贴、拖入、复制/下载，或通过 PicGo、自定义命令上传。[图片指南](docs/image-handling.md) |
| **调整到顺手** | **偏好设置 → 外观**中调整主题、各编辑平面的字体与排版；支持专注/打字机模式及自定义快捷键。 |
| **按需连接工具** | Git 驱动的备份与同步；使用自备密钥或本地模型的 AI Agent。AI **默认关闭**。[同步](docs/git-sync.md) · [AI](docs/ai-agent-guide.md) |

<a id="performance"></a><a id="性能"></a>

## 有据可查的性能

**Windows release 构建历史实测 · 2026-11-11。**Intel Core Ultra 5 225U，约 31 GiB 内存；每种大小 100 次局部编辑。下表对比不同文档大小的**模型处理耗时**，不含界面排版、paint 或 GPU 呈现。

| 混合 Markdown | 首次模型构建 | 编辑 P50 | 编辑 P95 |
| --- | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | **0.64 ms** | 1.21 ms |
| 300 KiB | 28.67 ms | **2.55 ms** | 3.69 ms |
| 600 KiB | 55.03 ms | **5.81 ms** | 6.86 ms |
| 1 MiB | 96.84 ms | **10.33 ms** | 11.24 ms |

另一轮 1 MiB 文档的 **1080 次系统按键**实测中，应用输入回调 → 光标 paint 为 **P50 26.63 ms / P95 33.14 ms**，不含硬件/系统输入传输与 GPU 呈现。这些是既有记录，不是 v0.6.1 的重新跑分或竞品排名。[测试方法、原始数据与复现命令](docs/performance.md)。

## 开始前了解这些

- Markdown 始终是文档源数据；不支持或无法精确定位的语法保留源码编辑路径。暂不支持 GFM 合并单元格与图片像素裁剪。[覆盖范围与已知缺口](docs/visual-editing-quality.md)。
- 内置 PDF/DOCX 无需外部转换器；可选 Pandoc，失败时回退内置导出，不同格式保真度不同。MarkNice 浏览器工作区沿用固定版本的编辑器外观；Word 导入仅在当前会话生效、不写回 Markion，需**复制 MD**或显式保存 Markdown 带回。其 Word 导出和主题化打印 PDF 有别于原生导出；该工作区不提供 Markdown/PDF/图片导入与样例操作。
- Windows 安装版与 Linux AppImage 支持 Minisign 验证的一键更新；macOS 与 DEB/RPM/pacman 打开下载链接。更新验证与操作系统代码签名不同。
- **8 种 README 与 8 种界面语言**：英语、简体/繁體中文、日语、韩语、法语、德语和西班牙语。在偏好设置中选择 **한국어** 即可使用韩语界面。

## 构建与贡献

准备 Rust stable 与平台原生构建工具（[CI 构建配置](.github/workflows/release.yml)），在仓库根目录运行：

```powershell
cargo run
cargo test --workspace
pwsh ./scripts/check-quality.ps1
```

[完整使用指南](docs/user-guide.md#simplified-chinese) · [常见问题](docs/faq.md) · [反馈问题](https://github.com/willmove/markion/issues) · [发布流程](docs/release-process.md) · [MIT 许可证](LICENSE)
