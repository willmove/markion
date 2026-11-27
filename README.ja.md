<p align="center"><img src="assets/markion-logo.svg" width="80" height="80" alt="Markion"></p>
<h1 align="center">Markion</h1>
<p align="center"><strong>自然に書いて、Markdownの自由をそのままに。</strong><br>メモから技術文書、公開する記事まで。Rust + GPUI製のネイティブエディター。</p>

<p align="center"><a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.zh-TW.md">繁體中文</a> · <strong>日本語</strong> · <a href="README.ko.md">한국어</a> · <a href="README.fr.md">Français</a> · <a href="README.de.md">Deutsch</a> · <a href="README.es.md">Español</a></p>

<p align="center">
  <a href="https://github.com/willmove/markion/releases/latest"><img src="https://img.shields.io/github/v/release/willmove/markion?color=168a86" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-536878" alt="Windows, macOS, Linux">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/willmove/markion?color=168a86" alt="MIT"></a>
</p>

<p align="center"><a href="#install"><strong>ダウンロード</strong></a> · <a href="#new-in-v064">新機能</a> · <a href="#performance">実測性能</a> · <a href="docs/user-guide.md">使い方</a></p>

![Windows版Markionの実機画面：ファイルツリーと独立した右側アウトラインの間にMarkdown、タスク、表を表示](docs/images/markion-workspace-windows.png)

<p align="center"><sub>Windows実機画面、v0.6.0開発ビルド · <a href="docs/images/README.md">スクリーンショットとサンプル</a></sub></p>

## Markionを選ぶ理由

| ✍️ 書くことに集中 | 📁 ファイルは自分の手元に | 📤 下書きから公開まで |
| --- | --- | --- |
| ビジュアル編集では必要に応じて構文を隠し、いつでもソースや分割プレビューへ切り替えられます。 | 通常のローカルMarkdownファイル。アカウント不要。Electronや内蔵WebViewを使わないRust + GPUI製エディターです。 | Wordを取り込み、コード・数式・図を書いて、PDF、Word、HTML、画像に書き出せます。 |

<a id="new-in-v060"></a><a id="new-in-v061"></a><a id="new-in-v063"></a><a id="new-in-v064"></a>

## v0.6.4の新機能

| 注目機能 | 書きやすくなるポイント |
| --- | --- |
| **コードブロックは展開状態で開く** | コードの折りたたみを任意にしました。**設定 → 外観**の「コードブロック」で**ビジュアル編集でコードの折りたたみを許可**をオンにすると、「コードを折りたたむ / コードを展開」操作が戻ります。 |
| **表が扱いやすく** | 表のハンドルメニューに表全体への操作を集約し、**列幅を均等にする**を追加。「下に挿入」は段落と同じ項目から選べます。高い表で文字を選んでもページが飛びません。 |
| **書いたとおりに表示** | 引用内の見出しや画像、表のヘッダー強調と列の配置、入れ子の太字・斜体、前の文字に付く句読点が、どのモードでも正しく表示されます。 |
| **取り込みと書き出しの再現度** | Word取り込みでリストの番号、手動の改行、明示的な書式を保持。LaTeX書き出しは入れ子リストと開始番号を、Word書き出しは書式付きリンクを1つのリンクとして保持します。 |
| **初期状態でより安全** | 閲覧モードでは文書を変更しなくなりました。未保存の編集があるタブがある間はファイルやフォルダーの名前変更を拒否し、セッション復元で「最近使ったファイル」を並べ替えません。 |

<p align="center"><img src="docs/images/markion-block-menu-windows.png" width="760" alt="Markionの実機ブロックメニュー：見出し、リスト、ブロック操作、インライン書式"></p>
<p align="center"><sub>v0.6.0</sub></p>

[リリースノート](docs/release-notes/release-notes-v0.6.4.md) · [開発状況と検証記録](docs/project-status.md)

<a id="install"></a>

## ダウンロードして書き始める

| 環境 | v0.6.4のダウンロード |
| --- | --- |
| **Windows x64** | [インストーラー](https://github.com/willmove/markion/releases/download/v0.6.4/markion_0.6.4_x64-setup.exe) · [ポータブルZIP](https://github.com/willmove/markion/releases/download/v0.6.4/markion_0.6.4_x64-portable.zip) |
| **macOS 11+** | [Apple Silicon](https://github.com/willmove/markion/releases/download/v0.6.4/Markion_0.6.4_aarch64.dmg) · [Intel](https://github.com/willmove/markion/releases/download/v0.6.4/Markion_0.6.4_x64.dmg) |
| **Linux x86_64** | [AppImage](https://github.com/willmove/markion/releases/download/v0.6.4/markion_0.6.4_x86_64.AppImage) · [DEB](https://github.com/willmove/markion/releases/download/v0.6.4/markion_0.6.4_amd64.deb) · [RPM](https://github.com/willmove/markion/releases/download/v0.6.4/markion_0.6.4_x86_64.rpm) · [pacman（Arch/Omarchy）](https://github.com/willmove/markion/releases/download/v0.6.4/markion-0.6.4-1-x86_64.pkg.tar.zst) |

[全リリースと検証情報](https://github.com/willmove/markion/releases)

- **Windows：**インストーラーを実行、またはZIPを展開します。Authenticode署名は未対応のため、SmartScreenで**詳細情報 → 実行**が必要な場合があります。
- **macOS。**お使いの Mac に合った Apple Silicon または Intel 用の DMG を開き、Markion を「アプリケーション」へドラッグして、ダブルクリックします。どちらの DMG も Developer ID で署名済みで、Apple の公証を受けています。[初回起動](docs/user-guide.md#install)。
- **Linux：**AppImageに実行権限を付けて起動（`chmod +x markion*.AppImage`）、またはDEB/RPM/pacmanをインストールします。Arch LinuxとOmarchyでは`sudo pacman -U markion-0.6.4-1-x86_64.pkg.tar.zst`を実行します。FUSEがない場合は`--appimage-extract-and-run`を使えます。

Markdownファイルかフォルダーを開くだけで始められます。サンプルを編集する前に、**画像を含むexamplesフォルダー全体をコピーしてください**。[入門とテンプレート](assets/examples/README.md)。

## ひとつの文書、4つの使い方

| モード | 用途 |
| --- | --- |
| **ビジュアル編集**・既定 | 表示されたMarkdown上で本文、表、コード、数式を直接編集。 |
| **ソース** | Markdown構文を細部まで操作。 |
| **分割プレビュー** | ソースとプレビューを並べ、任意でスクロールを同期。 |
| **閲覧** | 編集しない、すっきりした表示。 |

切り替えても位置、選択範囲、取り消し履歴を維持します。[ショートカット](docs/keyboard-shortcuts.md)。

| 下書きから完成まで | できること |
| --- | --- |
| **技術文書** | GFM表、コード強調、LaTeX数式、Mermaid、脚注、コールアウト、移動できるアウトライン。 |
| **既存の文書を取り込む** | ローカルDOCXの取り込みと、Word・Google Docs・Webのリッチテキスト貼り付け。[Word取り込み](docs/word-import.md) |
| **書き出しと公開** | Markdown、装飾付き/プレーンHTML、LaTeX、DOCX、PDF、PNG/JPEG、WeChat用のローカルMarkNiceワークスペース。[公開ガイド](docs/marknice-workspace.md) |
| **画像とリソース** | 貼り付け、ドロップ、コピー/ダウンロード、PicGoや独自コマンドによるアップロード。[画像ガイド](docs/image-handling.md) |
| **好みに合わせる** | **設定 → 外観**でテーマ、表示面ごとのフォントや組版を設定。集中・タイプライターモード、独自ショートカットも。 |
| **必要なときだけ接続** | Gitによるバックアップと同期、自分のAPIキーやローカルモデルを使うAI Agent。AIは**初期状態で無効**。[同期](docs/git-sync.md) · [AI](docs/ai-agent-guide.md) |

<a id="performance"></a>

## 根拠を確認できる性能

**Windows releaseビルドの過去の実測 · 2026-11-11。**Intel Core Ultra 5 225U、約31 GiB RAM、各サイズ100回の局所編集。文書サイズによる**モデル処理時間**を比較しており、UIレイアウト・paint・GPU表示は含みません。

| 混合Markdown | 初回モデル構築 | 編集P50 | 編集P95 |
| --- | ---: | ---: | ---: |
| 100 KiB | 10.77 ms | **0.64 ms** | 1.21 ms |
| 300 KiB | 28.67 ms | **2.55 ms** | 3.69 ms |
| 600 KiB | 55.03 ms | **5.81 ms** | 6.86 ms |
| 1 MiB | 96.84 ms | **10.33 ms** | 11.24 ms |

別の1 MiB文書で**1080回のシステムキー入力**を測定した結果、アプリの入力コールバック → キャレットpaintは**P50 26.63 ms / P95 33.14 ms**でした。ハードウェア/OSの入力伝達とGPU表示は除きます。既存の記録であり、v0.6.4の新規測定や競合順位ではありません。[方法・生データ・再現手順](docs/performance.md)。

## 使い始める前に

- Markdownが文書の正本です。未対応またはバイト境界が曖昧な構文は慎重なソース編集を使います。GFMの結合セルと画像のピクセル切り抜きは未対応。[対応範囲と残る課題](docs/visual-editing-quality.md)。
- 内蔵PDF/DOCXに外部変換器は不要。任意のPandocが失敗すると内蔵へ戻り、再現度は形式により異なります。ブラウザーのMarkNiceは固定版の編集画面に沿っています。Word取り込みはセッション内のみでMarkionへ書き戻さず、戻すには**Copy MD**か明示的なMarkdown保存が必要です。ブラウザーのWordとテーマ付き印刷PDFはネイティブ書き出しとは異なります。このワークスペースではMarkdown/PDF/画像の取り込みやサンプル操作は提供しません。
- Windowsインストール版とLinux AppImageはMinisign検証後のアプリ内更新に対応。macOSとDEB/RPM/pacmanはダウンロードを開きます。更新検証とOSのコード署名は別です。
- **README と UI は 8 言語**：英語、簡体字/繁体字中国語、日本語、韓国語、フランス語、ドイツ語、スペイン語。設定で **한국어** を選択すると韓国語 UI を使用できます。

## ビルドと貢献

Rust stableと各環境のネイティブビルドツールを用意し（[CIの設定](.github/workflows/release.yml)）、リポジトリーのルートで実行：

```powershell
cargo run
cargo test --workspace
pwsh ./scripts/check-quality.ps1
```

[使い方 / 使用指南](docs/user-guide.md) · [FAQ](docs/faq.md) · [不具合報告](https://github.com/willmove/markion/issues) · [リリース手順](docs/release-process.md) · [MITライセンス](LICENSE)
