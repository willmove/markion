<p align="center"><img src="assets/markion-logo.svg" width="80" height="80" alt="Markion"></p>
<h1 align="center">Markion</h1>
<p align="center"><strong>專注表達，保留 Markdown 的自由。</strong><br>從日常筆記、技術文件到文章發布，使用 Rust + GPUI 原生編輯器。</p>

<p align="center"><a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · <strong>繁體中文</strong> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a></p>

<p align="center">
  <a href="https://github.com/willmove/markion/releases/latest"><img src="https://img.shields.io/github/v/release/willmove/markion?color=168a86" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-536878" alt="Windows, macOS, Linux">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/willmove/markion?color=168a86" alt="MIT"></a>
</p>

<p align="center"><a href="#install"><strong>立即下載</strong></a> · <a href="#new-in-v061">新版亮點</a> · <a href="#performance">實測效能</a> · <a href="docs/user-guide.md#simplified-chinese">使用指南</a></p>

![Windows 實機截圖：檔案樹、視覺化 Markdown、任務列表與表格，以及獨立的右側大綱](docs/images/markion-workspace-windows.png)

<p align="center"><sub>Windows 實機拍攝，v0.6.0 開發建置 · <a href="docs/images/README.md">截圖說明與示範文件</a></sub></p>

## 為什麼選擇 Markion

| ✍️ 寫作少一點打擾 | 📁 檔案始終屬於你 | 📤 從草稿走向發布 |
| --- | --- | --- |
| 視覺化編輯按需隱藏語法，隨時切換原始碼或分欄預覽。 | 普通本地 Markdown 檔案，無需帳號。Rust + GPUI 原生編輯器，無 Electron 或內嵌 WebView。 | 匯入 Word，編寫程式碼、公式與圖表，再匯出 PDF、Word、HTML 或圖片。 |

<a id="new-in-v060"></a><a id="new-in-v061"></a>

## v0.6.1 新亮點

| 亮點 | 帶來的改變 |
| --- | --- |
| **韓語介面** | 八種完整介面語言，在偏好設定中選擇 **한국어**。 |
| **依習慣調整間距** | 在**偏好設定 → 外觀**中設定 75–300% 行距與 0–128px 段距。 |
| **背景準備更新** | 每五小時檢查，預設自動下載；確認後才安裝，並保護尚未儲存的編輯。 |
| **插入時保留原內容** | 精簡插入選單與「在下方新增表格/分隔線」保留既有文字及表格，插入分隔線後可繼續輸入。 |

<p align="center"><img src="docs/images/markion-block-menu-windows.png" width="760" alt="Markion 實機塊選單：標題層級、列表、塊操作與行內格式"></p>
<p align="center"><sub>v0.6.0</sub></p>

[完整發布說明](docs/release-notes/release-notes-v0.6.1.md) · [專案進展與驗證記錄](docs/project-status.md)

<a id="install"></a>

## 下載，即刻開始

| 平臺 | v0.6.1 下載 |
| --- | --- |
| **Windows x64** | [安裝程式](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x64-setup.exe) · [便攜 ZIP](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x64-portable.zip) |
| **macOS 11+** | [Apple Silicon](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_aarch64.dmg) · [Intel](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_x64.dmg) |
| **Linux x86_64** | [AppImage](https://github.com/willmove/markion/releases/download/v0.6.1/Markion_0.6.1_x86_64.AppImage) · [DEB](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_amd64.deb) · [RPM](https://github.com/willmove/markion/releases/download/v0.6.1/markion_0.6.1_x86_64.rpm) |

[全部版本與資源校驗資訊](https://github.com/willmove/markion/releases)

- **Windows：**執行安裝程式或解壓 ZIP。暫未提供 Authenticode 簽名，SmartScreen 可能需要選擇**更多資訊 → 仍要執行**。
- **macOS：**將 Markion 拖入“應用程式”。**v0.6.1 為 ad-hoc 簽名，尚未公證。**確認瀏覽器下載的副本可信後，安裝完成執行：

  ```bash
  xattr -cr /Applications/Markion.app
  ```

  每次透過瀏覽器下載更新後需重複執行。[首次啟動詳情](docs/user-guide.md#安装)。
- **Linux：**為 AppImage 新增執行許可權（`chmod +x Markion*.AppImage`）後執行，或安裝 DEB/RPM。無 FUSE 時可用 `--appimage-extract-and-run`。

開啟 Markdown 檔案或資料夾即可寫作。試用內建樣例時，請**先複製整個 examples 資料夾及其中的圖片，再開始編輯**。[上手與模板](assets/examples/README.md)。

## 一份文件，四種工作方式

| 模式 | 適合的場景 |
| --- | --- |
| **視覺化編輯** · 預設 | 在渲染後的 Markdown 中直接編輯正文、表格、程式碼與公式。 |
| **原始碼** | 精確掌控全部 Markdown 語法。 |
| **分欄預覽** | 原始碼與預覽並排，可開啟同步滾動。 |
| **閱讀** | 乾淨、不可編輯的閱讀檢視。 |

切換模式時保留位置、選區與撤銷歷史。[快捷鍵](docs/keyboard-shortcuts.md)。

| 從構思到交付 | 已有能力 |
| --- | --- |
| **技術寫作** | GFM 表格、程式碼高亮、LaTeX 公式、Mermaid、腳註、提示塊與可跳轉大綱。 |
| **遷入既有內容** | 本地 DOCX 匯入；貼上 Word、Google Docs 或網頁中的富文字。[Word 匯入](docs/word-import.md) |
| **匯出與發布** | Markdown、帶樣式/純 HTML、LaTeX、DOCX、PDF、PNG/JPEG，以及本地 MarkNice 公眾號排版工作區。[發布指南](docs/marknice-workspace.md) |
| **圖片與資源** | 貼上、拖入、複製/下載，或透過 PicGo、自定義命令上傳。[圖片指南](docs/image-handling.md) |
| **調整到順手** | **偏好設定 → 外觀**中調整主題、各編輯平面的字型與排版；支援專注/打字機模式及自定義快捷鍵。 |
| **按需連線工具** | Git 驅動的備份與同步；使用自備金鑰或本地模型的 AI Agent。AI **預設關閉**。[同步](docs/git-sync.md) · [AI](docs/ai-agent-guide.md) |

<a id="performance"></a>

## 有據可查的效能

**Windows release 構建歷史實測 · 2026-11-11。**Intel Core Ultra 5 225U，約 31 GiB 記憶體；每種大小 100 次區域性編輯。下表對比不同文件大小的**模型處理耗時**，不含介面排版、paint 或 GPU 呈現。

| 混合 Markdown | 首次模型構建 | 編輯 P50 | 編輯 P95 |
| --- | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | **0.64 ms** | 1.21 ms |
| 300 KiB | 28.67 ms | **2.55 ms** | 3.69 ms |
| 600 KiB | 55.03 ms | **5.81 ms** | 6.86 ms |
| 1 MiB | 96.84 ms | **10.33 ms** | 11.24 ms |

另一輪 1 MiB 文件的 **1080 次系統按鍵**實測中，應用輸入回撥 → 游標 paint 為 **P50 26.63 ms / P95 33.14 ms**，不含硬體/系統輸入傳輸與 GPU 呈現。這些是既有記錄，不是 v0.6.1 的重新跑分或競品排名。[測試方法、原始資料與復現命令](docs/performance.md)。

## 開始前瞭解這些

- Markdown 始終是文件源資料；不支援或無法精確定位的語法保留原始碼編輯路徑。暫不支援 GFM 合併單元格與圖片畫素裁剪。[覆蓋範圍與已知缺口](docs/visual-editing-quality.md)。
- 內建 PDF/DOCX 無需外部轉換器；可選 Pandoc，失敗時回退內建匯出，不同格式保真度不同。MarkNice 瀏覽器工作區沿用固定版本的編輯器外觀；Word 匯入僅在當前會話生效、不寫回 Markion，需**複製 MD**或顯式儲存 Markdown 帶回。其 Word 匯出和主題化列印 PDF 有別於原生匯出；該工作區不提供 Markdown/PDF/圖片匯入與樣例操作。
- Windows 安裝版與 Linux AppImage 支援 Minisign 驗證的一鍵更新；macOS 與 DEB/RPM 開啟下載連結。更新驗證與作業系統程式碼簽名不同。
- **8 種 README 與 8 種介面語言**：英語、簡體/繁體中文、日語、韓語、法語、德語和西班牙語。在偏好設定中選擇 **한국어** 即可使用韓語介面。

## 構建與貢獻

準備 Rust stable 與平臺原生構建工具（[CI 構建配置](.github/workflows/release.yml)），在倉庫根目錄執行：

```powershell
cargo run
cargo test --workspace
pwsh ./scripts/check-quality.ps1
```

[完整使用指南](docs/user-guide.md#simplified-chinese) · [常見問題](docs/faq.md) · [反饋問題](https://github.com/willmove/markion/issues) · [發布流程](docs/release-process.md) · [MIT 許可證](LICENSE)
