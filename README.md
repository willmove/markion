<a id="english"></a>

<p align="center"><img src="assets/markion-logo.svg" width="80" height="80" alt="Markion"></p>
<h1 align="center">Markion</h1>
<p align="center"><strong>Write naturally. Keep your Markdown.</strong><br>A native editor for notes, technical docs and your next published piece. Built with Rust + GPUI.</p>

<p align="center"><strong>English</strong> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a></p>

<p align="center">
  <a href="https://github.com/willmove/markion/releases/latest"><img src="https://img.shields.io/github/v/release/willmove/markion?color=168a86" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-536878" alt="Windows, macOS, Linux">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/willmove/markion?color=168a86" alt="MIT"></a>
</p>

<p align="center"><a href="#install"><strong>Download</strong></a> · <a href="#new-in-v060">What's new</a> · <a href="#performance">Performance</a> · <a href="docs/user-guide.md">User guide</a></p>

![Markion on Windows: rendered Markdown, tasks and a table between the file tree and independent right outline](docs/images/markion-workspace-windows.png)

<p align="center"><sub>Actual Windows capture, v0.6.0 development build · <a href="docs/images/README.md">Screenshots and sample workspace</a></sub></p>

## Why Markion

| ✍️ Write with less friction | 📁 Keep control | 📤 Take your work further |
| --- | --- | --- |
| Visual Edit hides syntax as you write; switch to source or split preview whenever you need. | Ordinary local Markdown files, no account required. Native Rust + GPUI editor, without Electron or an embedded WebView. | Import Word, work with code, math and diagrams, then export to PDF, Word, HTML or images. |

<a id="new-in-v060"></a>

## New in v0.6.0

| Highlight | What it means for your writing |
| --- | --- |
| **Files + Outline, together** | Keep files on the left and a resizable outline on the right. |
| **Compact `/` and emoji palettes** | Insert structure quickly; optional automatic slash popup and up to 36 emoji candidates. |
| **Direct block and table controls** | Compact menus, Image Properties, row/column actions and Shift+Enter inside cells. |
| **5 offline guides · 12 templates** | Start a meeting note, project plan or article from **Help → Examples and Templates**. |

<p align="center"><img src="docs/images/markion-block-menu-windows.png" width="760" alt="Actual Markion block menu: heading levels, lists, block actions and inline formatting"></p>

[Release notes](docs/release-notes/release-notes-v0.6.0.md) · [Project status and verification](docs/project-status.md)

<a id="install"></a><a id="安装"></a>

## Download and start writing

| Platform | v0.6.0 downloads | Download size |
| --- | --- | --- |
| **Windows x64** | [Installer](https://github.com/willmove/markion/releases/download/v0.6.0/markion_0.6.0_x64-setup.exe) · [Portable ZIP](https://github.com/willmove/markion/releases/download/v0.6.0/markion_0.6.0_x64-portable.zip) | 22.0 MiB installer |
| **macOS 11+** | [Apple Silicon](https://github.com/willmove/markion/releases/download/v0.6.0/Markion_0.6.0_aarch64.dmg) · [Intel](https://github.com/willmove/markion/releases/download/v0.6.0/Markion_0.6.0_x64.dmg) | 27.1 MiB arm64 DMG |
| **Linux x86_64** | [AppImage](https://github.com/willmove/markion/releases/download/v0.6.0/Markion_0.6.0_x86_64.AppImage) · [DEB](https://github.com/willmove/markion/releases/download/v0.6.0/markion_0.6.0_amd64.deb) · [RPM](https://github.com/willmove/markion/releases/download/v0.6.0/markion_0.6.0_x86_64.rpm) | 52.4 MiB AppImage |

[All releases and verification](https://github.com/willmove/markion/releases) · Download sizes are not memory use.

- **Windows:** run the installer or extract the ZIP. No Authenticode signature yet; SmartScreen may require **More info → Run anyway**.
- **macOS:** drag Markion into Applications. **v0.6.0 is ad-hoc signed and not notarized.** For a trusted browser-downloaded copy, clear quarantine after installation:

  ```bash
  xattr -cr /Applications/Markion.app
  ```

  Repeat after browser-downloaded updates. [First-launch details](docs/user-guide.md#install).
- **Linux:** make the AppImage executable (`chmod +x Markion*.AppImage`) and run it, or install DEB/RPM. Without FUSE, use `--appimage-extract-and-run`.

Open a Markdown file or folder. To try the examples, **copy the entire examples folder, including images, before editing**. [Getting started](assets/examples/README.md).

<a id="editing-modes"></a><a id="编辑模式"></a>

## One document, four ways to work

| Mode | Best for |
| --- | --- |
| **Visual Edit** · default | Write in rendered Markdown; directly edit text, tables, code and math. |
| **Source** | Full control of Markdown syntax. |
| **Split Preview** | Source and preview together, with optional synchronized scrolling. |
| **Read** | A clean, non-editing reading view. |

Switch modes while keeping your place, selection and undo history. [Shortcuts](docs/keyboard-shortcuts.md).

<a id="documents-and-workspace"></a><a id="文档与工作区"></a>

<a id="markdown-editing-and-preview"></a><a id="markdown-编辑与预览"></a>

<a id="themes-languages-and-preferences"></a><a id="主题语言与偏好设置"></a>

<a id="export"></a><a id="导出"></a>

<a id="word-import"></a><a id="word-导入"></a>

| From first draft to delivery | Included |
| --- | --- |
| **Technical writing** | GFM tables, highlighted code, LaTeX math, Mermaid, footnotes, callouts and linked outline. |
| **Bring existing work** | Local DOCX import and rich-text paste from Word, Google Docs or the web. [Word import](docs/word-import.md) |
| **Export and publish** | Markdown, styled/plain HTML, LaTeX, DOCX, PDF, PNG/JPEG and a local MarkNice workspace for WeChat. [Publishing](docs/marknice-workspace.md) |
| **Images and assets** | Paste, drop, copy/download or upload through PicGo/custom commands. [Image guide](docs/image-handling.md) |
| **Make it yours** | Themes, per-surface fonts and typography in **Preferences → Appearance**; focus/typewriter modes and custom shortcuts. |
| **Optional connected tools** | Git-backed Backup and Sync; AI Agent with your own key or local models. AI is **off by default**. [Sync](docs/git-sync.md) · [AI](docs/ai-agent-guide.md) |

<a id="performance"></a><a id="性能"></a>

## Performance you can inspect

**Historical Windows release-build measurements · 2026-11-11.** Intel Core Ultra 5 225U, ~31 GiB RAM; 100 localized edits per size. This compares **model processing** across document sizes, excluding UI layout/paint and GPU presentation.

| Mixed Markdown | Initial model build | Edit P50 | Edit P95 |
| --- | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | **0.64 ms** | 1.21 ms |
| 300 KiB | 28.67 ms | **2.55 ms** | 3.69 ms |
| 600 KiB | 55.03 ms | **5.81 ms** | 6.86 ms |
| 1 MiB | 96.84 ms | **10.33 ms** | 11.24 ms |

In a separate **1080-system-key-event** run on a 1 MiB document, application callback → caret paint was **26.63 ms P50 / 33.14 ms P95**. It excludes hardware/OS delivery and GPU presentation. These are earlier recorded results, not a fresh v0.6.0 benchmark or a competitor ranking. [Method, raw data and reproduction](docs/performance.md).

<a id="current-limitations"></a><a id="当前限制"></a>

## Know before you start

- Visual Edit keeps Markdown canonical; unsupported or byte-ambiguous syntax uses conservative source editing. GFM merged cells and image pixel cropping are unsupported. [Coverage and remaining gaps](docs/visual-editing-quality.md).
- Built-in PDF/DOCX need no external converter; optional Pandoc falls back to built-in export. Fidelity varies by format. The MarkNice browser workspace follows its pinned editor skin; Word import is session-local, never writes back, and requires **Copy MD** or an explicit Markdown save to bring work back. Its Word export and themed print-to-PDF differ from native export; Markdown/PDF/image import and sample actions are unavailable there.
- Windows installers and Linux AppImages support Minisign-verified in-app installation; macOS and DEB/RPM updates open a download. Update verification is separate from OS code signing.
- **Current source: 8 README and 8 UI languages**, including Korean alongside English, Simplified/Traditional Chinese, Japanese, French, German and Spanish. Select **한국어** in Preferences. Published v0.6.0 binaries still have 7 UI languages; build the current source to use Korean.

<a id="development"></a><a id="开发"></a>

## Build and contribute

With Rust stable and your platform's native build tools ([CI build recipes](.github/workflows/release.yml)), run from the repository root:

```powershell
cargo run
cargo test --workspace
pwsh ./scripts/check-quality.ps1
```

<a id="license"></a><a id="许可证"></a>

[User guide / 使用指南](docs/user-guide.md) · [FAQ](docs/faq.md) · [Report a bug](https://github.com/willmove/markion/issues) · [Release process](docs/release-process.md) · [MIT license](LICENSE)

<a id="simplified-chinese"></a>

**[阅读完整简体中文介绍 →](README.zh-CN.md)**
