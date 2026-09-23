<a id="traditional-chinese"></a>

<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion 標誌" width="128" height="128">
</p>

<p align="center">
  <a href="README.md#english">English</a> · <a href="README.zh-CN.md">简体中文</a> · <strong>繁體中文</strong> · <a href="README.ja.md">日本語</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a>
</p>

# Markion

Markion 是一款使用 Rust 和 GPUI 打造的原生桌面 Markdown 編輯器。它在單一輕量應用中結合了流暢的原始碼編輯、基於原始碼的視覺化編輯模式、即時預覽、工作區工具與多格式匯出。Markdown 始終是文件的標準資料格式——不使用 Electron、Tauri 或 WebView。

## 安裝

請從 [GitHub Releases](https://github.com/willmove/markion/releases) 下載最新版本。

| 平台 | 發佈套件 | 目標架構 |
|---|---|---|
| Windows | NSIS `.exe` 安裝程式 | x86_64 |
| Linux | `.deb` 和 AppImage | x86_64 |
| macOS | `.app` 和 `.dmg` | Apple Silicon（arm64），macOS 11+ |

目前發佈版本尚未進行平台程式碼簽章。Windows SmartScreen 可能要求選擇「更多資訊 → 仍要執行」，macOS Gatekeeper 可能要求對應用程式按右鍵並選擇「打開」。**說明 → 檢查更新…** 在所有平台都提供可操作的更新提示：帶標籤的 Windows x86_64 NSIS 安裝可進行經過 cargo-packager Minisign 加密驗證的一鍵下載並安裝，且當任一文件存在未儲存變更時拒絕啟動；macOS 與 Linux 會在系統瀏覽器中開啟對應的發佈檔案。Intel Mac 可透過 Rosetta 執行 arm64 版本；目前尚不提供通用二進位和 Apple 公證。

## 編輯模式

Markion 提供四種檢視模式，預設使用分割預覽。

- **編輯** — 專注的原始 Markdown 原始碼編輯器。
- **視覺化編輯** — 所見即所得優先、基於原始碼的編輯介面。正文保持渲染並逐步顯示必要語法；一般圍欄程式碼內容、區塊數學公式、已註冊的圖表（Mermaid）、行內圖片欄位與 GFM 表格儲存格提供精確的直接編輯器。尚未實作所見即所得渲染的結構——YAML 前言、縮排程式碼及格式錯誤或位元組對應不明確的語法——暫時保留精確的原始碼編輯通道，並登記在[視覺化編輯 WYSIWYG 涵蓋矩陣與路線圖](docs/visual-editing-quality.md)中等待逐步關閉。斜線命令面板與精簡的右鍵區塊選單提供精確的區塊轉換（段落、標題、清單、引用、圍欄程式碼、分隔線、表格）、複製、上移/下移、原始碼安全的拖曳重排與刪除；選取範圍上下文格式工具列與視覺化連結編輯器對每次操作執行一次精確的原始碼變更。Markdown 分隔符號預設自動配對，輸入 `:shortcode` 會開啟 emoji 自動完成，任務清單核取方塊可直接點擊切換；游標離開標題、清單、任務或引用列後，該列會立即恢復為渲染形態。它不是獨立的富文字文件模型——底層 Markdown 始終是唯一資料來源。
- **分割預覽** — 左右同時顯示原始碼和渲染預覽，可選擇啟用同步捲動：基於原始碼對應按文件位置對齊兩個窗格，而非按整篇捲動百分比同步。
- **閱讀** — 不可編輯的渲染檢視，預設置中並限制為適合閱讀的 860 px 最大寬度；啟用「預覽自適應寬度」後可使用整個窗格。

切換模式會保留目前文件、游標和選取範圍、復原歷史以及每個分頁的捲動狀態。

使用**檢視 → 原始碼/分割預覽**（Windows/Linux 為 `Ctrl+/`，macOS 為 `Cmd+/`）可在兩種原始碼導向版面之間切換；從視覺化編輯或閱讀模式首次執行時會進入編輯模式。

## 文件與工作區

- 多分頁編輯，每個分頁分別保存游標、選取範圍、捲動、復原/重做、預覽、大綱和衍生 Markdown 快取狀態。
- 再次開啟已開啟的 Markdown 或純文字檔案時會聚焦現有分頁，而不會建立重複分頁。
- **開啟資料夾**可切換工作區根目錄，並在「檔案」側邊欄中顯示 Markdown 檔案、精選的純文字檔案（`.txt`、`.text`、`.log`、`.csv`、`.tsv`、`.org`、`.rst`、`.adoc`/`.asciidoc`）以及受支援的圖片檔案（`.png`、`.jpg`/`.jpeg`、`.gif`、`.webp`、`.bmp`、`.tif`/`.tiff`、`.svg`），按目錄層級巢狀顯示；空資料夾也會列出。Markdown 檔案保持醒目區分，純文字檔案以 UTF-8 文字開啟，圖片檔案以唯讀圖片分頁開啟，並將超尺寸圖片調整到內容區域內顯示。
- 展開資料夾時只顯示下一層的子項目，可逐層深入檢視深層巢狀的工作區。
- **顯示隱藏的資料夾/檔案**偏好設定（預設關閉）可顯示點號開頭（dotfile）以及 Windows 隱藏屬性的項目；而始終排除的建置、相依性和 VCS 雜訊目錄（`target`、`node_modules`、`.git` 等）在任何情況下都保持隱藏。
- 檔案樹右鍵選單會依目標提供開啟、在新分頁開啟、新增檔案/資料夾、重新命名、刪除、在系統檔案管理器中顯示、篩選和重新整理等操作。
- 檔案和資料夾可就地命名；刪除非空資料夾需要二次確認。
- 可從作業系統檔案管理器將 Markdown 檔案拖入 Markion。
- 「檔案」和「大綱」面板均可切換顯示，側邊欄與分割預覽的分隔線可拖曳調整。
- 「大綱」面板以可折疊樹狀結構列出文件標題層級：含有下級標題的標題會顯示展開/折疊控制項，新開啟的文件大綱預設完全展開，折疊狀態按文件獨立、僅作用於目前工作階段，並會醒目顯示游標所在章節。點擊標題會跳轉到對應原始碼位置——在閱讀模式下則跳轉到對應的渲染標題。
- 原生視窗標題在 Markion 品牌文字後顯示目前開啟的檔名，文件有未儲存變更時會帶 `*` 後綴。狀態列保留儲存狀態和暫態操作回饋，並提供精簡的持續性上下文：目前文件的字元數與字數、在存在編輯介面時顯示游標所在的行號與欄號（從 1 開始），以及在文件或工作區屬於 Git 倉庫時顯示目前分支名稱。
- **備份與同步**透過一次點擊的「立即同步」和清晰易懂的狀態提示，將筆記資料夾安全地保存在同步位置。底層由 Git 提供可靠的版本歷史，技術操作則收納在「進階 Git 詳情」中。設定、驗證、衝突與復原說明見[備份與同步指南](docs/git-sync.md)。

## Markdown 編輯與預覽

- 使用 `pulldown-cmark` 解析 Markdown，面向 CommonMark 和 GFM。
- 格式化命令支援段落（Windows/Linux 為 `Ctrl+0`，macOS 為 `Cmd+0`）、標題、粗體、斜體、行內程式碼、連結、圖片、清單、任務清單、引用、圍欄程式碼區塊和原始碼 Markdown 表格。「段落」可將選取範圍內的 ATX 標題恢復為一般文字，並保持非標題列不變。
- 原始碼編輯器及受支援的視覺化編輯欄位預設啟用 Markdown 自動配對：可補齊分隔符號、包裹選取範圍、跳過已有結束符號，並用 Backspace 刪除空配對；IME 組字期間以及程式碼、區塊公式和圖表內容編輯器內不會觸發。
- 在詞語邊界輸入未閉合的 `:shortcode` 會在原始碼和視覺化編輯中開啟 emoji 自動完成；確認後插入標準 Markdown，並保持為一次可復原的原始碼編輯。
- 貼上來自 Word、Google 文件或網頁的富文字時，會將剪貼簿中的 HTML 轉換為帶格式的 Markdown（標題、強調、連結、巢狀清單、表格、程式碼），整個貼上可一次復原；純文字剪貼簿仍按原樣逐字貼上。
- 標題命令預設顯示 H1–H5，可在「偏好設定」中擴展為 H1–H6。
- 尋找與取代支援區分大小寫、正規表示式、上一個/下一個符合、取代目前項目和全部取代。
- 原始碼表格命令可格式化表格並新增、刪除或移動行列。視覺化編輯中的表格還支援直接編輯儲存格、使用 Tab 巡覽、確定性寬度重排，以及同樣的原始碼驅動行列操作；一般預覽表格保持唯讀。
- 圖片資源工作流程支援剪貼簿圖片、拖入檔案、明確檔案／URL 插入以及已有文件圖片。可依來源選擇保留、複製／下載，或透過 PicGo HTTP、PicGo Core、字面參數自訂命令上傳；詳見[圖片處理與上傳指南](docs/image-handling.md)。Markion 使用抗衝突的文件相對資源，精確保留圖片中繼資料，並將傳輸復原資料置於 Markdown 之外。
- 可解析 YAML 前言並在預覽中隱藏；其中的 `title`、`author` 和 `date` 會用於匯出中繼資料。
- 文件寫入採用同目錄的原子替換，寫入失敗時保留文件路徑和髒狀態。Markion 會追蹤最近已知的磁碟檔案身分，在儲存前及文件開啟期間偵測外部變更，僅自動重新載入乾淨文件，並為髒文件提供重新載入、覆寫或另存副本的衝突選擇。復原管理器會列出每個復原快照及其原始路徑和磁碟關係，支援還原、捨棄、全部還原與全部捨棄，且不會刪除無法讀取或未選取的復原資料。
- 自動儲存預設在停止輸入五秒後執行，並為未儲存文件寫入復原副本；已還原的復原快照會保持持久，直到成功儲存、明確捨棄或被原子寫入的後繼復原所取代。

渲染預覽支援：

- 粗體、斜體、刪除線、行內程式碼、連結、醒目標記、上標、下標、註腳（將游標懸停在引用上即可閱讀定義）、任務清單、常用 emoji 短代碼和自動連結。
- 文內 `[TOC]` / `[toc]` 標記會渲染為可點擊的即時目錄，`[text](#heading)` / `{#id}` 標題錨點會在文件內跳轉。
- 正確的有序清單起始編號、巢狀清單、分層項目符號、懸掛縮排、圖片和嵌入式 HTML。
- 受支援的行內 HTML 在混合 Markdown、獨立 HTML 區塊、表格儲存格和視覺化編輯中保持一致語意，包括安全的顏色 span、連結與連結圖片、`kbd`/`samp`、原樣換行以及正確定位的上下標；畸形或不支援的標記會安全回退，不會執行腳本。
- 可選取預覽文字，並透過右鍵選單複製為純文字、Markdown 或 HTML；在適用位置還可複製連結位址。
- `$...$` 行內公式和 `$$...$$` 區塊公式，由內嵌的 RaTeX（相容 KaTeX）引擎離線排版為帶內嵌字體的 SVG 並快取——無需連網，也無需安裝外部 LaTeX。
- ` ```mermaid ` 圍欄圖表（流程圖、循序圖等）渲染為經過消毒處理的 SVG 並快取。
- 使用 syntect 和 two-face 擴充語法集醒目標示圍欄程式碼，對未涵蓋語言使用後備語法分析，並可顯示行號。

## 主題、語言與偏好設定

- 十四款內建主題：Paper、Ink、Solar、Forest、Rose、Graphite、GitHub Light/Dark、Solarized Light/Dark、One Light/Dark 和 Tokyo Night/Light。
- 自訂主題使用 Markion 本機主題目錄中的 `.toml` 檔案；首次使用時會安裝 `typewriter.toml` 示範檔案（含可選的 `[fonts]` 表——`editor`、`rendered`、`code`，在使用者沒有明確偏好時為原始碼編輯器、正文和程式碼平面提供字體）。舊版 `.theme` 檔案會在首次載入時自動遷移。
- 七種介面語言：英語、簡體中文、繁體中文、日語、法語、德語和西班牙語。
- 應用內偏好設定面板在**一般**中可設定語言、側邊欄顯示、預覽自適應寬度、專注/打字機模式、Markdown 自動配對、程式碼行號、同步捲動、顯示隱藏檔案和標題選單層級；在**外觀**中可設定主題、按平面的字體（原始碼/閱讀/程式碼，預設跟隨主題）、字號和段落間距。
- 偏好設定保存在 `config.toml`；舊版 `preferences.conf` 會自動遷移。

所有設定欄位均可省略。主要預設值和僅能透過檔案設定的選項如下：

```toml
theme = "Paper"
language = "en"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 或 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" 或 "outline"
show_hidden_files = false
markdown_auto_pair = true

# 可選的按平面字體；省略時先跟隨主題，再回退到內建預設
#（原始碼/正文為系統 UI 字體，程式碼為 JetBrains Mono）。
# editor_font_family = "Cascadia Code"
# rendered_font_family = "Georgia"
# code_font_family = "JetBrains Mono"

[auto_save]
enabled = true
delay_secs = 5

[git]
background_check = false

[export]
pdf_engine = "xelatex"
```

設定、復原檔案、主題和按日輪換的診斷日誌均使用適合各平台的 Markion 資料目錄。啟動前設定 `RUST_LOG=debug` 可獲得更詳細的日誌。

## 匯出

Markion 可匯出為：

- 用於準備微信富文字內容的本機 MarkNice 工作區
- Markdown
- 帶樣式 HTML 和純 HTML
- LaTeX
- DOCX
- PDF
- PNG 和 JPEG 文字快照

PDF 和 DOCX 會優先嘗試已整合的 Typune/pandoc 匯出引擎。如果 pandoc 或選定的 PDF 引擎不可用，Markion 會回退到較簡單的內建寫入器，並在狀態列中說明所用後端。安裝 pandoc 和合適的 PDF 引擎可獲得更豐富的輸出。PNG/JPEG 和內建 PDF 輸出有意保持為基礎文字快照。

## Word 匯入

選擇 **檔案 → 匯入 Word（.docx）**，可以完全在本機把一份 DOCX 轉換為新的已儲存 Markdown 文件。Markion 會先顯示轉換報告，再開啟儲存對話框；如果可能遺失內容，必須明確選擇繼續。支援的內嵌圖片儲存在 Markdown 旁的 `<檔名>.assets/` 中，移動或複製時請讓它與 `.md` 檔案同行。匯入過程有資源上限、可取消、無需連網，而且不會覆寫已有 Markdown 或資源目錄。修訂按「接受後的檢視」匯入；Word 標題層級會得到更準確的保留，粗體與後續文字的邊界會正規化為相容性更好的 Markdown。舊版 `.doc`、`.docm`、加密檔案、PDF/OCR 和頁面版式複刻不在此流程範圍內。支援的語意子集、限制、診斷和清理行為詳見 [`docs/word-import.md`](docs/word-import.md)。

選擇 **匯出 → 發佈到微信公眾號（MarkNice）** 可在預設瀏覽器中將目前記憶體文件開啟到一個私有的本機回環工作區。內建編輯器外觀盡量貼近固定版本的 MarkNice 編輯區（雙卡片、紅綠燈標題列、SVG 工具列、375px 手機框），而不會帶入行銷網站。主題、渲染器、數學公式排版和應用腳本全部離線可用；公式以自包含的行內 SVG（MathJax）渲染，貼上到公眾號草稿時不會被清洗破壞，也不會出現重複。瀏覽器中的編輯僅保留在該分頁內，永遠不會寫回 Markion。瀏覽器工作區裡的 **匯入 Word** 只在目前瀏覽器工作階段中把 `.docx` 轉成 Markdown；若要帶回 Markion，請使用 **複製 MD** 或自行另存工作階段中的 Markdown。受管理的本機圖片可以預覽，但複製時會明確排除它們，因為本機 blob 無法發佈到微信；遠端圖片仍可能存取其原始主機。工作區還可以複製目前 Markdown、將主題化預覽存為 HTML、存為瀏覽器產生的 Word，或透過列印對話框 **存為 PDF**（列印的是目前主題化、已消毒的 MarkNice 預覽）。瀏覽器 Word/PDF 與 Markion 原生/Pandoc 匯出不同。工作區不提供匯入 Markdown、匯入 PDF、匯入本機圖片或範例文件。如果剪貼簿權限或兩小時工作階段過期，請重新授權或從 Markion 重新啟動。

## 效能

- 預覽區塊、視覺化編輯區塊、大綱、統計資訊和行數均按文件版本快取，並透過 `Arc` 共享。
- 語法醒目標示會跨編輯複用，語法庫在背景預熱。
- 復原快照不包含衍生快取；編輯器按版本複用快取的文字控制代碼。
- 預覽/視覺化編輯清單僅更新變化範圍，檔案樹限制每幀渲染的列數，換行後的原始碼列會測量實際渲染高度。

局部編輯後，帶原始碼對應的視覺化編輯模型會增量複用可獨立解析的區域；當 Markdown 上下文或位元組範圍無法確定時，則回退為完整衍生。分割/閱讀預覽仍使用防抖與快取。Markion 仍使用 `String` 而非 rope 文字緩衝區，部分語意讀取也會有意執行完整解析。

## 目前限制

- 視覺化編輯以所見即所得為預設呈現契約，同時保留標準 Markdown；暫無位元組精確渲染證明的結構會以原始碼作為過渡編輯通道（登記在 [WYSIWYG 涵蓋路線圖](docs/visual-editing-quality.md)中），而不會猜測富文字樹變更；僅當可證明存在不重疊的原始碼邊界時才提供區塊重排。目前的主要缺口包括已解碼 HTML 實體、前言與縮排程式碼區塊；其他畸形或尚未支援的結構列在矩陣的次級缺口中。
- 螢幕渲染（分割/閱讀預覽與視覺化編輯）使用內嵌的 RaTeX 引擎排版數學公式；LaTeX 匯出保留原生 `$...$`/`$$...$$` 原始碼交給讀者自己的工具鏈處理，內建 DOCX 匯出後備通道（僅在 pandoc 不可用時使用）仍會將公式降級為可讀的純文字近似顯示，而非嵌入排版好的字形。
- 視覺化表格儲存格支援直接編輯，且選取範圍完全落在一個儲存格內時可使用粗體/斜體/行內程式碼/連結。GFM 表格欄寬可拖曳，並以緊鄰表格前的 HTML 註解（`<!-- markion-cols:… -->`）寫入原始碼。行內圖片寬度為 10–100% 的整數，且支援拖曳縮放。參考式/多行圖片和畸形表格仍是 WYSIWYG 涵蓋路線圖上的已知缺口，暫時保留原始碼驅動編輯路徑。
- 一鍵更新在完成 Minisign 驗證後會安裝 Windows NSIS 版本；macOS 套件替換與 Linux `.deb`/AppImage 自我替換仍是後續工作，且更新身分驗證並非 Windows Authenticode 或 Apple 公證。
- 尚未實作檔案樹拖放移動和完整的自訂主題安裝介面。
- 圖片匯出是基礎文字快照，超大文件尚未在所有衍生子系統中使用 rope 或完全增量解析。

## 開發

需要 Rust stable。在倉庫根目錄執行：

```powershell
cargo run
cargo build
pwsh ./scripts/check-quality.ps1
```

品質命令會檢查 Rust 格式與程式碼檢查（lint）、完整 Cargo workspace 測試套件、固定的 MarkNice bundle，以及嚴格模式下的所有 OpenSpec 工件。另請參閱[本機 MarkNice 工作區指南](docs/marknice-workspace.md)與[視覺化編輯支援與工程契約](docs/visual-editing-quality.md)。

根套件是 `markion` 應用 crate。源自 Typune 且不依賴 GPUI 的函式庫 crate 位於 `crates/*`：

```powershell
cargo test -p markdown
cargo test -p export
cargo test --workspace
```

一般 `cargo test` 只測試根套件；使用 `cargo test --workspace` 可測試全部成員。在 Windows 上，應用建置為 GUI 子系統執行檔；完成偵錯建置後還可直接執行：

```powershell
.\target\debug\markion.exe
```

## 授權條款

Markion 使用 [MIT 授權條款](LICENSE)。
