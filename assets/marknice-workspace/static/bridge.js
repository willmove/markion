/* Markion bridge: immutable handoff, protected images, and tab-local edits. */
const markdownEl = document.getElementById('markdown');
const previewEl = document.getElementById('preview');
const statusEl = document.getElementById('status');
const themeSelect = document.getElementById('themeSelect');
const fontSizeLabel = document.getElementById('fontSizeLabel');
const fontSizeDown = document.getElementById('fontSizeDown');
const fontSizeUp = document.getElementById('fontSizeUp');
let fontSizeOffset = 0;
let paraSpacingOffset = 0;
const __mnHooks = { beforeRender: [], afterRender: [], ready: [] };

const messages = {
  en: {
    session: 'Edits in this browser tab are session-local and are not saved to Markion.',
    privacy: 'App assets stay local. Remote images in the document may contact their authored hosts without a referrer.',
    opened: 'Publishing workspace opened', expired: 'This publishing session expired. Return to Markion and launch it again.',
    setup: 'The publishing workspace could not load. Return to Markion and launch it again.',
    unresolved: n => `${n} local image(s) could not be resolved.`, resolved: n => `${n} protected local preview image(s)`,
    omitPrompt: n => `${n} local image(s) cannot be pasted directly into WeChat. Choose OK to copy without them, or Cancel to keep editing.`,
    copied: 'Copied rich HTML and plain text.', partial: n => `Copied without ${n} local image(s).`,
    denied: 'Clipboard access was denied. Allow clipboard permission and try again.', empty: 'Enter content before copying.',
    copyMarkdown: 'Copy MD', downloadHtml: 'Save as HTML', downloadDocx: 'Save as Word',
    markdownCopied: 'Copied Markdown source.', markdownDenied: 'Markdown could not be copied. Allow clipboard permission and try again.',
    htmlPreparing: 'Preparing themed HTML…', htmlDownloaded: 'Themed HTML download started.',
    docxPreparing: 'Preparing browser-generated DOCX…', docxDownloaded: 'Browser-generated DOCX download started.',
    exportFailed: 'Export could not be completed. Keep this tab open and try again.',
    exportFallback: n => `${n} local image(s) could not be embedded and were replaced with text.`,
    exportRemote: n => `${n} remote image(s) remain linked to their authored host.`,
    docxNote: 'Browser-generated DOCX targets Microsoft Word; it is distinct from Markion native DOCX export.',
    desktop: 'Desktop', phone: 'Phone', copy: 'Copy for WeChat', theme: 'Template', font: 'Font size', spacing: 'Spacing',
    fontDown: 'Decrease font size', fontUp: 'Increase font size', spacingDown: 'Decrease paragraph spacing', spacingUp: 'Increase paragraph spacing',
    importWord: 'Import Word', importWordTitle: 'Import a Word document (.docx)', importWordPreparing: 'Importing Word document…',
    importWordSuccess: 'Imported into this browser tab only. Copy Markdown or save the session Markdown to keep it in Markion.',
    importWordInvalid: 'Import failed. Choose a .docx file.', importWordTooLarge: 'The Word file is too large for this workspace (20 MB limit).',
    savePdf: 'Save as PDF', pdfPreparing: 'Preparing themed preview for print…',
    pdfPrintOpened: 'Choose “Save as PDF” in the print dialog. This prints the MarkNice preview, not Markion’s native PDF export.',
    pdfNote: 'Browser print-to-PDF of the themed MarkNice preview; distinct from Markion Export → PDF.',
    editorTitle: 'Markdown input', previewTitle: 'WeChat preview', toggleMode: 'Toggle dark mode'
  },
  'zh-hans': {
    session: '浏览器中的修改仅保留在当前标签页，不会保存回 Markion。',
    privacy: '应用资源完全本地；文档中的远程图片可能在无来源信息的情况下连接原始站点。',
    opened: '本地发布工作区已打开', expired: '发布会话已过期，请返回 Markion 重新打开。',
    setup: '发布工作区加载失败，请返回 Markion 重新打开。',
    unresolved: n => `${n} 张本地图片无法解析。`, resolved: n => `已保护加载 ${n} 张本地预览图片`,
    omitPrompt: n => `${n} 张本地图片无法直接粘贴到公众号。确定“复制但不含本地图片”，取消则继续编辑。`,
    copied: '已复制富文本和纯文本。', partial: n => `复制成功，已省略 ${n} 张本地图片。`,
    denied: '剪贴板权限被拒绝，请允许权限后重试。', empty: '请先输入内容再复制。',
    copyMarkdown: '复制 MD', downloadHtml: '存为 HTML', downloadDocx: '存为 Word',
    markdownCopied: 'Markdown 源码已复制。', markdownDenied: 'Markdown 复制失败，请允许剪贴板权限后重试。',
    htmlPreparing: '正在准备主题 HTML…', htmlDownloaded: '主题 HTML 已开始下载。',
    docxPreparing: '正在准备浏览器生成的 DOCX…', docxDownloaded: '浏览器生成的 DOCX 已开始下载。',
    exportFailed: '导出未完成，请保持此标签页打开后重试。',
    exportFallback: n => `${n} 张本地图片无法嵌入，已替换为文字。`,
    exportRemote: n => `${n} 张远程图片仍链接到其原始主机。`,
    docxNote: '浏览器生成的 DOCX 以 Microsoft Word 为目标，与 Markion 原生 DOCX 导出不同。',
    desktop: '桌面', phone: '手机', copy: '复制到公众号', theme: '模板', font: '字号', spacing: '段距',
    fontDown: '减小字号', fontUp: '增大字号', spacingDown: '减小段距', spacingUp: '增大段距',
    importWord: '导入 Word', importWordTitle: '导入 Word 文档（.docx）', importWordPreparing: '正在导入 Word 文档…',
    importWordSuccess: '已导入当前浏览器标签页，不会写回 Markion。请复制 Markdown 或自行另存 MD 后再带回编辑器。',
    importWordInvalid: '导入失败，请选择 .docx 文件。', importWordTooLarge: 'Word 文件过大（上限 20 MB）。',
    savePdf: '存为 PDF', pdfPreparing: '正在准备主题化预览以便打印…',
    pdfPrintOpened: '请在打印对话框中选择“另存为 PDF”。这是打印 MarkNice 预览，不是 Markion 的原生 PDF 导出。',
    pdfNote: '浏览器将主题化 MarkNice 预览打印为 PDF，与 Markion「导出 → PDF」不同。',
    editorTitle: 'Markdown 输入', previewTitle: '公众号预览', toggleMode: '切换深色/浅色模式'
  },
  'zh-hant': {
    session: '瀏覽器中的修改只保留在目前分頁，不會儲存回 Markion。', privacy: '應用資源完全在本機；文件中的遠端圖片可能連線至原始站點。',
    opened: '本機發佈工作區已開啟', expired: '發佈工作階段已過期，請回到 Markion 重新開啟。', setup: '發佈工作區載入失敗，請回到 Markion 重新開啟。',
    unresolved: n => `${n} 張本機圖片無法解析。`, resolved: n => `已保護載入 ${n} 張本機預覽圖片`, omitPrompt: n => `${n} 張本機圖片無法直接貼到公眾號。確定會略過圖片複製，取消則繼續編輯。`,
    copied: '已複製富文字和純文字。', partial: n => `複製成功，已略過 ${n} 張本機圖片。`, denied: '剪貼簿權限遭拒，請允許後重試。', empty: '請先輸入內容。',
    copyMarkdown: '複製 MD', downloadHtml: '存為 HTML', downloadDocx: '存為 Word', markdownCopied: 'Markdown 原始碼已複製。', markdownDenied: 'Markdown 複製失敗，請允許剪貼簿權限後重試。', htmlPreparing: '正在準備主題 HTML…', htmlDownloaded: '主題 HTML 已開始下載。', docxPreparing: '正在準備瀏覽器產生的 DOCX…', docxDownloaded: '瀏覽器產生的 DOCX 已開始下載。', exportFailed: '匯出未完成，請保持此分頁開啟後重試。', exportFallback: n => `${n} 張本機圖片無法嵌入，已替換為文字。`, exportRemote: n => `${n} 張遠端圖片仍連結到原始主機。`, docxNote: '瀏覽器產生的 DOCX 以 Microsoft Word 為目標，與 Markion 原生 DOCX 匯出不同。',
    desktop: '桌面', phone: '手機', copy: '複製到公眾號', theme: '範本', font: '字號', spacing: '段距',
    fontDown: '縮小字號', fontUp: '放大字號', spacingDown: '縮小段距', spacingUp: '放大段距',
    importWord: '匯入 Word', importWordTitle: '匯入 Word 文件（.docx）', importWordPreparing: '正在匯入 Word 文件…',
    importWordSuccess: '已匯入目前瀏覽器分頁，不會寫回 Markion。請複製 Markdown 或自行另存 MD。',
    importWordInvalid: '匯入失敗，請選擇 .docx 檔案。', importWordTooLarge: 'Word 檔案過大（上限 20 MB）。',
    savePdf: '存為 PDF', pdfPreparing: '正在準備主題化預覽以便列印…',
    pdfPrintOpened: '請在列印對話框選擇「另存為 PDF」。這是列印 MarkNice 預覽，不是 Markion 原生 PDF。',
    pdfNote: '瀏覽器將主題化 MarkNice 預覽列印為 PDF，與 Markion「匯出 → PDF」不同。',
    editorTitle: 'Markdown 輸入', previewTitle: '公眾號預覽', toggleMode: '切換深色/淺色模式'
  },
  ja: { session: 'ブラウザーの編集はこのタブだけに保持され、Markion には保存されません。', privacy: 'アプリ資産はローカルです。文書のリモート画像は配信元へ接続する場合があります。', opened: '公開ワークスペースを開きました', expired: 'セッションの期限が切れました。Markion から開き直してください。', setup: 'ワークスペースを読み込めません。Markion から開き直してください。', unresolved: n => `${n} 件のローカル画像を解決できません。`, resolved: n => `${n} 件の保護された画像`, omitPrompt: n => `${n} 件のローカル画像は貼り付けできません。画像なしでコピーするには OK、編集を続けるにはキャンセルを選択してください。`, copied: 'リッチ HTML とテキストをコピーしました。', partial: n => `${n} 件の画像を除いてコピーしました。`, denied: 'クリップボード権限が拒否されました。', empty: '先に内容を入力してください。', copyMarkdown: 'MD をコピー', downloadHtml: 'HTML として保存', downloadDocx: 'Word として保存', markdownCopied: 'Markdown ソースをコピーしました。', markdownDenied: 'Markdown をコピーできませんでした。クリップボード権限を確認してください。', htmlPreparing: 'テーマ付き HTML を準備しています…', htmlDownloaded: 'テーマ付き HTML のダウンロードを開始しました。', docxPreparing: 'ブラウザー生成 DOCX を準備しています…', docxDownloaded: 'ブラウザー生成 DOCX のダウンロードを開始しました。', exportFailed: 'エクスポートを完了できませんでした。このタブを開いたまま再試行してください。', exportFallback: n => `${n} 件のローカル画像を埋め込めず、テキストに置き換えました。`, exportRemote: n => `${n} 件のリモート画像は作成元ホストへのリンクのままです。`, docxNote: 'ブラウザー生成 DOCX は Microsoft Word を対象とし、Markion ネイティブ DOCX とは異なります。', desktop: 'デスクトップ', phone: 'スマートフォン', copy: 'WeChat 用にコピー', theme: 'テンプレート', font: '文字サイズ', spacing: '間隔', fontDown: '文字を小さく', fontUp: '文字を大きく', spacingDown: '段落間隔を縮小', spacingUp: '段落間隔を拡大', importWord: 'Word を取り込み', importWordTitle: 'Word 文書（.docx）を取り込む', importWordPreparing: 'Word 文書を取り込んでいます…', importWordSuccess: 'このブラウザタブにだけ取り込みました。Markion には保存されません。Markdown をコピーするか、セッションの Markdown を別途保存してください。', importWordInvalid: '取り込みに失敗しました。.docx ファイルを選んでください。', importWordTooLarge: 'Word ファイルが大きすぎます（上限 20 MB）。', savePdf: 'PDF として保存', pdfPreparing: 'テーマ付きプレビューを印刷用に準備しています…', pdfPrintOpened: '印刷ダイアログで「PDF に保存」を選んでください。これは MarkNice プレビューの印刷であり、Markion のネイティブ PDF 書き出しではありません。', pdfNote: 'テーマ付き MarkNice プレビューをブラウザ印刷で PDF 化します。Markion の「エクスポート → PDF」とは異なります。', editorTitle: 'Markdown 入力', previewTitle: 'WeChat プレビュー', toggleMode: 'ダークモード切替' },
  fr: { session: 'Les modifications restent dans cet onglet et ne sont pas enregistrées dans Markion.', privacy: 'Les ressources de l’application restent locales. Les images distantes peuvent contacter leur hôte.', opened: 'Espace de publication ouvert', expired: 'La session a expiré. Relancez-la depuis Markion.', setup: 'Impossible de charger l’espace. Relancez-le depuis Markion.', unresolved: n => `${n} image(s) locale(s) non résolue(s).`, resolved: n => `${n} image(s) locale(s) protégée(s)`, omitPrompt: n => `${n} image(s) locale(s) ne peut pas être collée. OK pour copier sans elles, Annuler pour continuer.`, copied: 'HTML enrichi et texte copiés.', partial: n => `Copié sans ${n} image(s) locale(s).`, denied: 'Accès au presse-papiers refusé.', empty: 'Saisissez du contenu avant de copier.', copyMarkdown: 'Copier MD', downloadHtml: 'Enregistrer en HTML', downloadDocx: 'Enregistrer en Word', markdownCopied: 'Source Markdown copiée.', markdownDenied: 'Impossible de copier le Markdown. Autorisez le presse-papiers et réessayez.', htmlPreparing: 'Préparation du HTML thématisé…', htmlDownloaded: 'Téléchargement du HTML thématisé lancé.', docxPreparing: 'Préparation du DOCX généré par le navigateur…', docxDownloaded: 'Téléchargement du DOCX généré par le navigateur lancé.', exportFailed: 'L’exportation n’a pas pu être terminée. Gardez cet onglet ouvert et réessayez.', exportFallback: n => `${n} image(s) locale(s) n’ont pas pu être incorporée(s) et ont été remplacées par du texte.`, exportRemote: n => `${n} image(s) distante(s) reste(nt) liée(s) à leur hôte d’origine.`, docxNote: 'Le DOCX généré par le navigateur cible Microsoft Word et diffère de l’export DOCX natif de Markion.', desktop: 'Bureau', phone: 'Téléphone', copy: 'Copier pour WeChat', theme: 'Modèle', font: 'Taille', spacing: 'Espacement', fontDown: 'Réduire la taille', fontUp: 'Augmenter la taille', spacingDown: 'Réduire l’espacement', spacingUp: 'Augmenter l’espacement', importWord: 'Importer Word', importWordTitle: 'Importer un document Word (.docx)', importWordPreparing: 'Importation du document Word…', importWordSuccess: 'Importé uniquement dans cet onglet, pas dans Markion. Copiez le Markdown ou enregistrez-le pour le conserver.', importWordInvalid: 'Échec de l’importation. Choisissez un fichier .docx.', importWordTooLarge: 'Le fichier Word dépasse la limite de 20 Mo.', savePdf: 'Enregistrer en PDF', pdfPreparing: 'Préparation de l’aperçu thématisé pour l’impression…', pdfPrintOpened: 'Choisissez « Enregistrer au format PDF » dans la boîte de dialogue d’impression. Ceci imprime l’aperçu MarkNice, pas l’export PDF natif de Markion.', pdfNote: 'Impression navigateur de l’aperçu MarkNice thématisé, distincte de Exportation → PDF dans Markion.', editorTitle: 'Saisie Markdown', previewTitle: 'Aperçu WeChat', toggleMode: 'Basculer le mode sombre' },
  de: { session: 'Browser-Änderungen bleiben in diesem Tab und werden nicht in Markion gespeichert.', privacy: 'App-Ressourcen bleiben lokal. Entfernte Dokumentbilder können ihren Host kontaktieren.', opened: 'Veröffentlichungsbereich geöffnet', expired: 'Die Sitzung ist abgelaufen. Bitte aus Markion neu öffnen.', setup: 'Der Bereich konnte nicht geladen werden. Bitte aus Markion neu öffnen.', unresolved: n => `${n} lokale Bilder nicht aufgelöst.`, resolved: n => `${n} geschützte lokale Bilder`, omitPrompt: n => `${n} lokale Bilder können nicht eingefügt werden. OK kopiert ohne Bilder, Abbrechen setzt die Bearbeitung fort.`, copied: 'Rich-HTML und Text kopiert.', partial: n => `Ohne ${n} lokale Bilder kopiert.`, denied: 'Zwischenablagezugriff verweigert.', empty: 'Bitte zuerst Inhalt eingeben.', copyMarkdown: 'MD kopieren', downloadHtml: 'Als HTML speichern', downloadDocx: 'Als Word speichern', markdownCopied: 'Markdown-Quelltext kopiert.', markdownDenied: 'Markdown konnte nicht kopiert werden. Bitte Zwischenablagezugriff erlauben.', htmlPreparing: 'Formatiertes HTML wird vorbereitet…', htmlDownloaded: 'Download des formatierten HTML gestartet.', docxPreparing: 'Browser-generiertes DOCX wird vorbereitet…', docxDownloaded: 'Download des browser-generierten DOCX gestartet.', exportFailed: 'Export konnte nicht abgeschlossen werden. Lassen Sie diesen Tab geöffnet und versuchen Sie es erneut.', exportFallback: n => `${n} lokale Bilder konnten nicht eingebettet werden und wurden durch Text ersetzt.`, exportRemote: n => `${n} entfernte Bilder bleiben mit ihrem ursprünglichen Host verknüpft.`, docxNote: 'Browser-generiertes DOCX richtet sich an Microsoft Word und unterscheidet sich vom nativen Markion-DOCX-Export.', desktop: 'Desktop', phone: 'Telefon', copy: 'Für WeChat kopieren', theme: 'Vorlage', font: 'Schrift', spacing: 'Abstand', fontDown: 'Schrift verkleinern', fontUp: 'Schrift vergrößern', spacingDown: 'Absatzabstand verkleinern', spacingUp: 'Absatzabstand vergrößern', importWord: 'Word importieren', importWordTitle: 'Word-Dokument (.docx) importieren', importWordPreparing: 'Word-Dokument wird importiert…', importWordSuccess: 'Nur in diesem Browser-Tab importiert, nicht nach Markion. Markdown kopieren oder die Sitzung separat speichern.', importWordInvalid: 'Import fehlgeschlagen. Bitte eine .docx-Datei wählen.', importWordTooLarge: 'Die Word-Datei ist zu groß (Grenze 20 MB).', savePdf: 'Als PDF speichern', pdfPreparing: 'Themenvorschau wird für den Druck vorbereitet…', pdfPrintOpened: 'Wählen Sie im Druckdialog „Als PDF speichern“. Das druckt die MarkNice-Vorschau, nicht Markions natives PDF.', pdfNote: 'Browser-Druck der thematisierten MarkNice-Vorschau, unabhängig von Markion „Exportieren → PDF“.', editorTitle: 'Markdown-Eingabe', previewTitle: 'WeChat-Vorschau', toggleMode: 'Dunkelmodus umschalten' },
  es: { session: 'Los cambios del navegador permanecen en esta pestaña y no se guardan en Markion.', privacy: 'Los recursos de la aplicación son locales. Las imágenes remotas pueden contactar su servidor.', opened: 'Espacio de publicación abierto', expired: 'La sesión caducó. Vuelve a abrirla desde Markion.', setup: 'No se pudo cargar el espacio. Vuelve a abrirlo desde Markion.', unresolved: n => `${n} imagen(es) local(es) sin resolver.`, resolved: n => `${n} imagen(es) local(es) protegida(s)`, omitPrompt: n => `${n} imagen(es) local(es) no se pueden pegar. Acepta para copiar sin ellas o cancela para seguir editando.`, copied: 'HTML enriquecido y texto copiados.', partial: n => `Copiado sin ${n} imagen(es) local(es).`, denied: 'Se denegó el acceso al portapapeles.', empty: 'Escribe contenido antes de copiar.', copyMarkdown: 'Copiar MD', downloadHtml: 'Guardar como HTML', downloadDocx: 'Guardar como Word', markdownCopied: 'Código Markdown copiado.', markdownDenied: 'No se pudo copiar Markdown. Permite el acceso al portapapeles e inténtalo de nuevo.', htmlPreparing: 'Preparando HTML con tema…', htmlDownloaded: 'Se inició la descarga del HTML con tema.', docxPreparing: 'Preparando DOCX generado por el navegador…', docxDownloaded: 'Se inició la descarga del DOCX generado por el navegador.', exportFailed: 'No se pudo completar la exportación. Mantén esta pestaña abierta e inténtalo de nuevo.', exportFallback: n => `${n} imagen(es) local(es) no se pudieron incrustar y se reemplazaron por texto.`, exportRemote: n => `${n} imagen(es) remota(s) permanecen vinculadas a su host original.`, docxNote: 'El DOCX generado por el navegador está destinado a Microsoft Word y es distinto de la exportación DOCX nativa de Markion.', desktop: 'Escritorio', phone: 'Teléfono', copy: 'Copiar para WeChat', theme: 'Plantilla', font: 'Tamaño', spacing: 'Espacio', fontDown: 'Reducir tamaño', fontUp: 'Aumentar tamaño', spacingDown: 'Reducir espacio', spacingUp: 'Aumentar espacio', importWord: 'Importar Word', importWordTitle: 'Importar un documento Word (.docx)', importWordPreparing: 'Importando el documento Word…', importWordSuccess: 'Importado solo en esta pestaña, no en Markion. Copia el Markdown o guárdalo para conservarlo.', importWordInvalid: 'Error al importar. Elige un archivo .docx.', importWordTooLarge: 'El archivo Word supera el límite de 20 MB.', savePdf: 'Guardar como PDF', pdfPreparing: 'Preparando la vista previa con tema para imprimir…', pdfPrintOpened: 'Elige «Guardar como PDF» en el cuadro de impresión. Esto imprime la vista previa de MarkNice, no el PDF nativo de Markion.', pdfNote: 'Impresión del navegador de la vista previa MarkNice con tema, distinta de Exportar → PDF en Markion.', editorTitle: 'Entrada Markdown', previewTitle: 'Vista previa de WeChat', toggleMode: 'Alternar modo oscuro' }
};

const markdownFormatMessages = {
  en: {
    toolbar: 'Markdown formatting', applied: 'Markdown formatting applied.',
    heading1: 'Heading 1', heading2: 'Heading 2', heading3: 'Heading 3', bold: 'Bold', italic: 'Italic', underline: 'Underline',
    orderedList: 'Ordered list', unorderedList: 'Unordered list', code: 'Inline code', link: 'Link', quote: 'Quote', codeBlock: 'Code block', image: 'Image syntax', table: 'Table',
    headingPlaceholder: 'Heading', listPlaceholder: 'List item', codePlaceholder: 'code', codeBlockPlaceholder: 'code content', linkPlaceholder: 'link text',
    imagePlaceholder: 'image description', imageUrlPlaceholder: 'image URL', boldPlaceholder: 'bold text', italicPlaceholder: 'italic text', underlinePlaceholder: 'underlined text', quotePlaceholder: 'quote',
    tableTemplate: '| Header 1 | Header 2 | Header 3 |\n| --- | --- | --- |\n| Content | Content | Content |'
  },
  'zh-hans': {
    toolbar: 'Markdown 格式工具栏', applied: '已应用 Markdown 格式。',
    heading1: '一级标题', heading2: '二级标题', heading3: '三级标题', bold: '加粗', italic: '斜体', underline: '下划线',
    orderedList: '有序列表', unorderedList: '无序列表', code: '行内代码', link: '链接', quote: '引用', codeBlock: '代码块', image: '图片语法', table: '表格',
    headingPlaceholder: '标题', listPlaceholder: '列表项', codePlaceholder: '代码', codeBlockPlaceholder: '代码内容', linkPlaceholder: '链接文字',
    imagePlaceholder: '图片描述', imageUrlPlaceholder: '图片地址', boldPlaceholder: '加粗文字', italicPlaceholder: '斜体文字', underlinePlaceholder: '下划线文字', quotePlaceholder: '引用内容',
    tableTemplate: '| 标题一 | 标题二 | 标题三 |\n| --- | --- | --- |\n| 内容 | 内容 | 内容 |'
  },
  'zh-hant': {
    toolbar: 'Markdown 格式工具列', applied: '已套用 Markdown 格式。',
    heading1: '一級標題', heading2: '二級標題', heading3: '三級標題', bold: '粗體', italic: '斜體', underline: '底線',
    orderedList: '編號清單', unorderedList: '項目清單', code: '行內程式碼', link: '連結', quote: '引用', codeBlock: '程式碼區塊', image: '圖片語法', table: '表格',
    headingPlaceholder: '標題', listPlaceholder: '清單項目', codePlaceholder: '程式碼', codeBlockPlaceholder: '程式碼內容', linkPlaceholder: '連結文字',
    imagePlaceholder: '圖片描述', imageUrlPlaceholder: '圖片網址', boldPlaceholder: '粗體文字', italicPlaceholder: '斜體文字', underlinePlaceholder: '底線文字', quotePlaceholder: '引用內容',
    tableTemplate: '| 標題一 | 標題二 | 標題三 |\n| --- | --- | --- |\n| 內容 | 內容 | 內容 |'
  },
  ja: {
    toolbar: 'Markdown 書式設定', applied: 'Markdown の書式を適用しました。',
    heading1: '見出し 1', heading2: '見出し 2', heading3: '見出し 3', bold: '太字', italic: '斜体', underline: '下線',
    orderedList: '番号付きリスト', unorderedList: '箇条書き', code: 'インラインコード', link: 'リンク', quote: '引用', codeBlock: 'コードブロック', image: '画像構文', table: '表',
    headingPlaceholder: '見出し', listPlaceholder: 'リスト項目', codePlaceholder: 'コード', codeBlockPlaceholder: 'コード内容', linkPlaceholder: 'リンク文字',
    imagePlaceholder: '画像の説明', imageUrlPlaceholder: '画像 URL', boldPlaceholder: '太字テキスト', italicPlaceholder: '斜体テキスト', underlinePlaceholder: '下線テキスト', quotePlaceholder: '引用内容',
    tableTemplate: '| 見出し 1 | 見出し 2 | 見出し 3 |\n| --- | --- | --- |\n| 内容 | 内容 | 内容 |'
  },
  fr: {
    toolbar: 'Mise en forme Markdown', applied: 'Mise en forme Markdown appliquée.',
    heading1: 'Titre 1', heading2: 'Titre 2', heading3: 'Titre 3', bold: 'Gras', italic: 'Italique', underline: 'Souligné',
    orderedList: 'Liste numérotée', unorderedList: 'Liste à puces', code: 'Code en ligne', link: 'Lien', quote: 'Citation', codeBlock: 'Bloc de code', image: 'Syntaxe d’image', table: 'Tableau',
    headingPlaceholder: 'Titre', listPlaceholder: 'Élément de liste', codePlaceholder: 'code', codeBlockPlaceholder: 'contenu du code', linkPlaceholder: 'texte du lien',
    imagePlaceholder: 'description de l’image', imageUrlPlaceholder: 'URL de l’image', boldPlaceholder: 'texte en gras', italicPlaceholder: 'texte en italique', underlinePlaceholder: 'texte souligné', quotePlaceholder: 'citation',
    tableTemplate: '| Titre 1 | Titre 2 | Titre 3 |\n| --- | --- | --- |\n| Contenu | Contenu | Contenu |'
  },
  de: {
    toolbar: 'Markdown-Formatierung', applied: 'Markdown-Formatierung angewendet.',
    heading1: 'Überschrift 1', heading2: 'Überschrift 2', heading3: 'Überschrift 3', bold: 'Fett', italic: 'Kursiv', underline: 'Unterstrichen',
    orderedList: 'Nummerierte Liste', unorderedList: 'Aufzählung', code: 'Inline-Code', link: 'Link', quote: 'Zitat', codeBlock: 'Codeblock', image: 'Bildsyntax', table: 'Tabelle',
    headingPlaceholder: 'Überschrift', listPlaceholder: 'Listeneintrag', codePlaceholder: 'Code', codeBlockPlaceholder: 'Codeinhalt', linkPlaceholder: 'Linktext',
    imagePlaceholder: 'Bildbeschreibung', imageUrlPlaceholder: 'Bild-URL', boldPlaceholder: 'fetter Text', italicPlaceholder: 'kursiver Text', underlinePlaceholder: 'unterstrichener Text', quotePlaceholder: 'Zitat',
    tableTemplate: '| Überschrift 1 | Überschrift 2 | Überschrift 3 |\n| --- | --- | --- |\n| Inhalt | Inhalt | Inhalt |'
  },
  es: {
    toolbar: 'Formato Markdown', applied: 'Formato Markdown aplicado.',
    heading1: 'Encabezado 1', heading2: 'Encabezado 2', heading3: 'Encabezado 3', bold: 'Negrita', italic: 'Cursiva', underline: 'Subrayado',
    orderedList: 'Lista numerada', unorderedList: 'Lista con viñetas', code: 'Código en línea', link: 'Enlace', quote: 'Cita', codeBlock: 'Bloque de código', image: 'Sintaxis de imagen', table: 'Tabla',
    headingPlaceholder: 'Encabezado', listPlaceholder: 'Elemento de lista', codePlaceholder: 'código', codeBlockPlaceholder: 'contenido del código', linkPlaceholder: 'texto del enlace',
    imagePlaceholder: 'descripción de imagen', imageUrlPlaceholder: 'URL de imagen', boldPlaceholder: 'texto en negrita', italicPlaceholder: 'texto en cursiva', underlinePlaceholder: 'texto subrayado', quotePlaceholder: 'cita',
    tableTemplate: '| Encabezado 1 | Encabezado 2 | Encabezado 3 |\n| --- | --- | --- |\n| Contenido | Contenido | Contenido |'
  }
};

const markdownFormatShortcuts = { bold: 'Ctrl/Cmd+B', italic: 'Ctrl/Cmd+I', underline: 'Ctrl/Cmd+U', link: 'Ctrl/Cmd+K' };
const markdownFormatKeys = Object.keys(markdownFormatMessages.en).sort().join('\u0000');
for (const [language, localized] of Object.entries(markdownFormatMessages)) {
  if (Object.keys(localized).sort().join('\u0000') !== markdownFormatKeys || Object.values(localized).some(value => typeof value !== 'string' || !value.trim())) {
    throw new Error(`Markion Markdown-format locale is incomplete: ${language}`);
  }
}

const localeMessageKeys = Object.keys(messages.en).sort().join('\u0000');
const requiredExportMessageKeys = [
  'copyMarkdown', 'downloadHtml', 'downloadDocx', 'markdownCopied',
  'markdownDenied', 'htmlPreparing', 'htmlDownloaded', 'docxPreparing',
  'docxDownloaded', 'exportFailed', 'exportFallback', 'exportRemote', 'docxNote',
  'importWord', 'importWordTitle', 'importWordPreparing', 'importWordSuccess',
  'importWordInvalid', 'importWordTooLarge', 'savePdf', 'pdfPreparing',
  'pdfPrintOpened', 'pdfNote', 'editorTitle', 'previewTitle', 'toggleMode'
];
for (const [language, localized] of Object.entries(messages)) {
  if (Object.keys(localized).sort().join('\u0000') !== localeMessageKeys) {
    throw new Error(`Markion workspace locale keys are out of sync: ${language}`);
  }
  for (const key of requiredExportMessageKeys) {
    const value = localized[key];
    const resolved = typeof value === 'function' ? value(1) : value;
    if (typeof resolved !== 'string' || !resolved.trim()) {
      throw new Error(`Markion workspace export locale is incomplete: ${language}.${key}`);
    }
  }
}

let locale = messages.en;
let markdownFormatLocale = markdownFormatMessages.en;
let sessionToken = '';
const protectedImages = new Map();
const objectUrls = new Set();
let unresolvedCount = 0;
let resolvedCount = 0;

marked.setOptions({ breaks: true, gfm: true });
function escapeMathHtml(value) {
  return String(value || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}
marked.use({ extensions: [
  { name: 'mathBlock', level: 'block', start: src => (src.match(/\$\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$\$([\s\S]+?)\$\$/); if (m) return { type: 'mathBlock', raw: m[0], tex: m[1].trim() }; }, renderer: token => { const tex = escapeMathHtml(token.tex); return `<div class="math-block" data-tex="${tex}">\\[${tex}\\]</div>`; } },
  { name: 'mathInline', level: 'inline', start: src => (src.match(/\$/) || { index: -1 }).index, tokenizer(src) { const m = src.match(/^\$([^$\n]+?)\$/); if (m) return { type: 'mathInline', raw: m[0], tex: m[1].trim() }; }, renderer: token => { const tex = escapeMathHtml(token.tex); return `<span class="math-inline" data-tex="${tex}">\\(${tex}\\)</span>`; } }
] });

function normalizeImageReference(value) {
  try { value = decodeURI(String(value || '')); } catch (_) { value = String(value || ''); }
  return value.replace(/[?#].*$/, '').replace(/\\/g, '/').replace(/^\.\//, '').toLowerCase();
}
function imageSrcIsLocalCandidate(value) {
  return !!value && !/^(https?:|data:|blob:|about:|javascript:|\/\/)/i.test(value);
}
function applyLocalImages(root) {
  resolvedCount = 0;
  root.querySelectorAll('img').forEach(img => {
    const authored = img.getAttribute('src') || '';
    const resource = protectedImages.get(normalizeImageReference(authored));
    if (!resource) return;
    img.setAttribute('src', resource.objectUrl);
    img.setAttribute('data-mn-local-image-id', resource.id);
    resolvedCount++;
  });
}
function localImageStatusSuffix() {
  if (unresolvedCount) return ` · ${locale.unresolved(unresolvedCount)}`;
  if (resolvedCount) return ` · ${locale.resolved(resolvedCount)}`;
  return '';
}

async function api(path, options = {}) {
  const headers = new Headers(options.headers || {});
  if (sessionToken) headers.set('Authorization', `Bearer ${sessionToken}`);
  const response = await fetch(path, { ...options, headers, cache: 'no-store', referrerPolicy: 'no-referrer' });
  if (response.status === 401) expire();
  if (!response.ok) throw new Error(`workspace request failed (${response.status})`);
  return response;
}
function expire() {
  sessionToken = '';
  sessionStorage.removeItem('markion.wechat.session');
  document.querySelector('.workspace').innerHTML = `<div class="expired"><p>${locale.expired}</p></div>`;
  statusEl.textContent = locale.expired;
}
async function exchangeClaim(claim) {
  const response = await fetch('/api/claim', { method: 'POST', headers: { Authorization: `Bearer ${claim}` }, cache: 'no-store', referrerPolicy: 'no-referrer' });
  if (!response.ok) throw new Error('claim denied');
  return (await response.json()).session_token;
}
async function loadProtectedImages(resources) {
  await Promise.all(resources.map(async resource => {
    const response = await api(`/api/resource/${encodeURIComponent(resource.id)}`);
    const blob = await response.blob();
    const objectUrl = URL.createObjectURL(blob);
    objectUrls.add(objectUrl);
    protectedImages.set(normalizeImageReference(resource.authored_url), { ...resource, blob, objectUrl });
  }));
}

function localImagesRemovedFromCopy(html) {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  let omitted = 0;
  doc.querySelectorAll('img').forEach(img => {
    const src = img.getAttribute('src') || '';
    if (img.hasAttribute('data-mn-local-image-id') || imageSrcIsLocalCandidate(src) || /^(blob:|file:)/i.test(src)) {
      img.remove(); omitted++;
    }
  });
  const output = doc.body.innerHTML;
  if (/(?:blob:|file:|https?:\/\/(?:127\.0\.0\.1|\[::1\]|localhost)(?::\d+)?)/i.test(output)) {
    throw new Error('unsafe local URL remained in copy output');
  }
  return { html: output, omitted };
}

async function copyCurrentPreview() {
  const html = previewEl.dataset.html || '';
  if (!html.trim()) { statusEl.textContent = locale.empty; return; }
  const localCount = previewEl.querySelectorAll('img[data-mn-local-image-id]').length + unresolvedCount;
  try {
    let payload = { html, omitted: 0 };
    if (localCount) {
      if (!window.confirm(locale.omitPrompt(localCount))) return;
      payload = localImagesRemovedFromCopy(html);
    }
    await copyRichHtml(payload.html);
    statusEl.textContent = payload.omitted ? locale.partial(payload.omitted) : locale.copied;
  } catch (error) {
    console.error(error);
    statusEl.textContent = locale.denied;
  }
}

function setLocalizedText(id, value) {
  const node = document.getElementById(id);
  if (node) node.textContent = value;
}

function setLocalizedAttr(id, name, value) {
  const node = document.getElementById(id);
  if (node) node.setAttribute(name, value);
}

function applyColorMode(dark) {
  if (dark) document.body.setAttribute('data-mode', 'dark');
  else document.body.removeAttribute('data-mode');
}

function initColorMode() {
  const media = window.matchMedia ? window.matchMedia('(prefers-color-scheme: dark)') : null;
  applyColorMode(!!media?.matches);
  document.getElementById('modeToggleBtn')?.addEventListener('click', () => {
    applyColorMode(document.body.getAttribute('data-mode') !== 'dark');
  });
}

function setPreviewMode(phone) {
  document.getElementById('previewContainer')?.classList.toggle('phone-mode', phone);
  document.querySelectorAll('.mode-btn').forEach(button => button.classList.remove('active'));
  document.getElementById(phone ? 'phoneModeBtn' : 'desktopModeBtn')?.classList.add('active');
}

function localize(language) {
  locale = messages[language] || messages.en;
  markdownFormatLocale = markdownFormatMessages[language] || markdownFormatMessages.en;
  document.documentElement.lang = language;
  setLocalizedText('sessionDisclosure', locale.session);
  setLocalizedText('privacyDisclosure', locale.privacy);
  setLocalizedText('themeLabel', locale.theme);
  setLocalizedText('fontSizeText', locale.font);
  setLocalizedText('spacingText', locale.spacing);
  setLocalizedText('copyBtnLabel', locale.copy);
  setLocalizedText('copyMarkdownLabel', locale.copyMarkdown);
  setLocalizedText('downloadHtmlLabel', locale.downloadHtml);
  setLocalizedText('savePdfLabel', locale.savePdf);
  setLocalizedText('downloadDocxLabel', locale.downloadDocx);
  setLocalizedText('importWordText', locale.importWord);
  setLocalizedText('editorHeading', locale.editorTitle);
  setLocalizedText('previewHeading', locale.previewTitle);
  fontSizeDown.setAttribute('aria-label', locale.fontDown);
  fontSizeUp.setAttribute('aria-label', locale.fontUp);
  setLocalizedAttr('paraSpacingDown', 'aria-label', locale.spacingDown);
  setLocalizedAttr('paraSpacingUp', 'aria-label', locale.spacingUp);
  setLocalizedAttr('copyBtn', 'aria-label', locale.copy);
  setLocalizedAttr('copyMarkdownBtn', 'aria-label', locale.copyMarkdown);
  setLocalizedAttr('downloadHtmlBtn', 'aria-label', locale.downloadHtml);
  setLocalizedAttr('savePdfBtn', 'aria-label', locale.savePdf);
  setLocalizedAttr('downloadDocxBtn', 'aria-label', locale.downloadDocx);
  setLocalizedAttr('desktopModeBtn', 'aria-label', locale.desktop);
  setLocalizedAttr('desktopModeBtn', 'title', locale.desktop);
  setLocalizedAttr('phoneModeBtn', 'aria-label', locale.phone);
  setLocalizedAttr('phoneModeBtn', 'title', locale.phone);
  setLocalizedAttr('modeToggleBtn', 'aria-label', locale.toggleMode);
  setLocalizedAttr('importWordLabel', 'title', locale.importWordTitle);
  const formatToolbar = document.getElementById('markdownFormatToolbar');
  formatToolbar.setAttribute('aria-label', markdownFormatLocale.toolbar);
  formatToolbar.querySelectorAll('[data-md-action]').forEach(button => {
    const action = button.dataset.mdAction;
    const label = markdownFormatLocale[action];
    const title = markdownFormatShortcuts[action] ? `${label} (${markdownFormatShortcuts[action]})` : label;
    button.setAttribute('title', title);
    button.setAttribute('aria-label', label);
  });
}

function markdownFormatText(key) {
  return markdownFormatLocale[key] || markdownFormatMessages.en[key] || '';
}

async function bootstrapWorkspace() {
  const claimParams = new URLSearchParams(location.hash.slice(1));
  const claim = claimParams.get('claim');
  history.replaceState(null, '', `${location.pathname}${location.search}`);
  try {
    sessionToken = claim ? await exchangeClaim(claim) : (sessionStorage.getItem('markion.wechat.session') || '');
    if (!sessionToken) throw new Error('missing session');
    sessionStorage.setItem('markion.wechat.session', sessionToken);
    const payload = await (await api('/api/document')).json();
    localize(payload.language);
    unresolvedCount = payload.unresolved_local_images.length;
    await loadProtectedImages(payload.resources);
    document.getElementById('documentName').textContent = payload.display_name;
    markdownEl.value = payload.markdown;
    render();
    statusEl.textContent = `${locale.opened}${localImageStatusSuffix()}`;
    if (unresolvedCount) {
      const warning = document.getElementById('warning');
      warning.hidden = false;
      warning.textContent = locale.unresolved(unresolvedCount);
    }
    setInterval(() => api('/api/heartbeat', { method: 'POST' }).catch(() => {}), 60_000);
  } catch (error) {
    console.error(error);
    statusEl.textContent = locale.setup;
    document.querySelector('.workspace').innerHTML = `<div class="expired"><p>${locale.setup}</p></div>`;
  }
}

document.addEventListener('DOMContentLoaded', () => {
  initColorMode();
  themeSelect.addEventListener('change', render);
  fontSizeDown.addEventListener('click', () => { fontSizeOffset = Math.max(-6, fontSizeOffset - 1); fontSizeLabel.textContent = fontSizeOffset; render(); });
  fontSizeUp.addEventListener('click', () => { fontSizeOffset = Math.min(6, fontSizeOffset + 1); fontSizeLabel.textContent = fontSizeOffset; render(); });
  const spacingLabel = document.getElementById('paraSpacingLabel');
  document.getElementById('paraSpacingDown').addEventListener('click', () => { paraSpacingOffset = Math.max(-16, paraSpacingOffset - 2); spacingLabel.textContent = paraSpacingOffset; render(); });
  document.getElementById('paraSpacingUp').addEventListener('click', () => { paraSpacingOffset = Math.min(24, paraSpacingOffset + 2); spacingLabel.textContent = paraSpacingOffset; render(); });
  markdownEl.addEventListener('input', () => { clearTimeout(window.__markionRenderTimer); window.__markionRenderTimer = setTimeout(render, 180); });
  document.getElementById('copyBtn').addEventListener('click', copyCurrentPreview);
  document.getElementById('desktopModeBtn').addEventListener('click', () => setPreviewMode(false));
  document.getElementById('phoneModeBtn').addEventListener('click', () => setPreviewMode(true));
  bootstrapWorkspace();
});
window.addEventListener('pagehide', () => objectUrls.forEach(url => URL.revokeObjectURL(url)));
