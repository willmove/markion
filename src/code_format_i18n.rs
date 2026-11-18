//! Localized strings for code-block formatting across all supported languages.

use crate::Language;

macro_rules! catalog {
    ($($name:ident => [$($text:literal),+]),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(usize)]
        pub enum CodeFormatMsg {
            $($name),+
        }

        impl CodeFormatMsg {
            pub const ALL: &'static [Self] = &[$(Self::$name),+];
        }

        pub fn code_format_t(language: Language, message: CodeFormatMsg) -> &'static str {
            const LABELS: &[[&str; 8]] = &[$([$($text),+]),+];
            let index = match language {
                Language::En => 0,
                Language::ZhHans => 1,
                Language::ZhHant => 2,
                Language::Ja => 3,
                Language::Fr => 4,
                Language::De => 5,
                Language::Es => 6,
                Language::Ko => 7,
            };
            LABELS[message as usize][index]
        }

        pub fn code_format_tf(language: Language, message: CodeFormatMsg, args: &[&str]) -> String {
            let mut out = code_format_t(language, message).to_string();
            for (index, value) in args.iter().enumerate() {
                let placeholder = format!("{{{index}}}");
                out = out.replace(&placeholder, value);
            }
            out
        }
    }
}

catalog! {
    FormatCodeBlock => [
        "Format Code Block",
        "格式化代码块",
        "格式化程式碼區塊",
        "コードブロックを整形",
        "Formater le bloc de code",
        "Codeblock formatieren",
        "Formatear bloque de código",
        "코드 블록 서식 지정"
    ],
    FormatButton => [
        "Format",
        "格式化",
        "格式化",
        "整形",
        "Formater",
        "Formatieren",
        "Formatear",
        "서식"
    ],
    StatusFormatting => [
        "Formatting {0} code block…",
        "正在格式化 {0} 代码块…",
        "正在格式化 {0} 程式碼區塊…",
        "{0} コードブロックを整形中…",
        "Formatage du bloc {0}…",
        "{0}-Codeblock wird formatiert…",
        "Formateando bloque de código {0}…",
        "{0} 코드 블록 서식 지정 중…"
    ],
    StatusFormatted => [
        "Formatted {0} code block",
        "已格式化 {0} 代码块",
        "已格式化 {0} 程式碼區塊",
        "{0} コードブロックを整形しました",
        "Bloc de code {0} formaté",
        "{0}-Codeblock formatiert",
        "Bloque de código {0} formateado",
        "{0} 코드 블록 서식을 지정했습니다"
    ],
    StatusAlreadyFormatted => [
        "Code block is already formatted",
        "代码块已处于规范格式",
        "程式碼區塊已符合格式",
        "コードブロックは既に整形されています",
        "Le bloc de code est déjà formaté",
        "Codeblock ist bereits formatiert",
        "El bloque de código ya está formateado",
        "코드 블록이 이미 정렬되어 있습니다"
    ],
    StatusNoCodeBlock => [
        "Place the cursor inside a fenced code block to format",
        "请将光标置于围栏代码块内再执行格式化",
        "請將游標置於圍欄程式碼區塊內再執行格式化",
        "カーソルをコードブロック内に置いて整形してください",
        "Placez le curseur dans un bloc de code pour le formater",
        "Setzen Sie den Cursor zum Formatieren in einen Codeblock",
        "Coloque el cursor dentro de un bloque de código para formatear",
        "서식을 지정할 코드 블록 안에 커서를 두세요"
    ],
    StatusMissingLanguage => [
        "Specify a language on the code fence before formatting",
        "请先为代码块指定语言标识后再格式化",
        "請先為程式碼區塊指定語言識別再格式化",
        "整形する前にコードブロックの言語を指定してください",
        "Indiquez un langage sur le bloc de code avant de le formater",
        "Geben Sie vor dem Formatieren eine Sprache im Codeblock an",
        "Especifique un lenguaje en el bloque de código antes de formatear",
        "서식을 지정하기 전에 코드 블록 언어를 지정하세요"
    ],
    StatusUnsupportedLanguage => [
        "Code formatting is not supported for '{0}'",
        "暂不支持格式化 '{0}' 代码块",
        "暫不支援格式化 '{0}' 程式碼區塊",
        "'{0}' のコード整形には対応していません",
        "Le formatage n’est pas pris en charge pour « {0} »",
        "Code-Formatierung wird für „{0}“ nicht unterstützt",
        "El formato de código no es compatible con «{0}»",
        "'{0}' 언어의 코드 서식 지정은 지원되지 않습니다"
    ],
    StatusFormatterUnavailable => [
        "{0} formatter is unavailable ({1})",
        "{0} 格式化器不可用（{1}）",
        "{0} 格式化器無法使用（{1}）",
        "{0} フォーマッターは利用できません ({1})",
        "Le formateur {0} est indisponible ({1})",
        "Formatierer für {0} ist nicht verfügbar ({1})",
        "El formateador de {0} no está disponible ({1})",
        "{0} 포매터를 사용할 수 없습니다 ({1})"
    ],
    StatusFormatFailed => [
        "Could not format {0} code block: {1}",
        "无法格式化 {0} 代码块：{1}",
        "無法格式化 {0} 程式碼區塊：{1}",
        "{0} コードブロックを整形できませんでした: {1}",
        "Impossible de formater le bloc {0} : {1}",
        "{0}-Codeblock konnte nicht formatiert werden: {1}",
        "No se pudo formatear el bloque {0}: {1}",
        "{0} 코드 블록 서식을 지정하지 못했습니다: {1}"
    ],
    StatusFormatTimedOut => [
        "Formatting {0} code block timed out",
        "格式化 {0} 代码块超时",
        "格式化 {0} 程式碼區塊逾時",
        "{0} コードブロックの整形がタイムアウトしました",
        "Dépassement du délai lors du formatage {0}",
        "Zeitüberschreitung beim Formatieren von {0}",
        "Se agotó el tiempo al formatear el bloque {0}",
        "{0} 코드 블록 서식 지정 시간이 초과되었습니다"
    ],
    StatusFormatStale => [
        "Skipped formatting result because the document changed",
        "文档已发生修改，已放弃过时的格式化结果",
        "文件已變更，已略過過時的格式化結果",
        "文書が変更されたため整形結果を破棄しました",
        "Résultat ignoré car le document a été modifié",
        "Formatierung verworfen, da das Dokument geändert wurde",
        "Se descartó el resultado porque el documento cambió",
        "문서가 변경되어 서식 결과를 적용하지 않았습니다"
    ],
    StatusReadOnly => [
        "Code blocks cannot be formatted in Read mode",
        "纯阅读模式下不能格式化代码块",
        "閱讀模式下無法格式化程式碼區塊",
        "閲覧モードではコードブロックを整形できません",
        "Impossible de formater les blocs de code en mode Lecture",
        "Im Lesemodus können Codeblöcke nicht formatiert werden",
        "No se pueden formatear bloques en modo Lectura",
        "읽기 모드에서는 코드 블록 서식을 지정할 수 없습니다"
    ],
    StatusFormatOnPasteOn => [
        "Auto-format pasted code blocks enabled",
        "已开启粘贴代码后自动格式化",
        "已開啟貼上程式碼後自動格式化",
        "貼り付け時のコードブロック自動整形を有効にしました",
        "Formatage automatique au collage activé",
        "Automatische Codeblock-Formatierung beim Einfügen aktiviert",
        "Formato automático al pegar activado",
        "붙여넣기 시 코드 블록 자동 서식을 켰습니다"
    ],
    StatusFormatOnPasteOff => [
        "Auto-format pasted code blocks disabled",
        "已关闭粘贴代码后自动格式化",
        "已關閉貼上程式碼後自動格式化",
        "貼り付け時のコードブロック自動整形を無効にしました",
        "Formatage automatique au collage désactivé",
        "Automatische Codeblock-Formatierung beim Einfügen deaktiviert",
        "Formato automático al pegar desactivado",
        "붙여넣기 시 코드 블록 자동 서식을 껐습니다"
    ],
    StatusFormatterPathSet => [
        "Configured formatter path: {0}",
        "已设置格式化器路径：{0}",
        "已設定格式化器路徑：{0}",
        "フォーマッターのパスを設定しました: {0}",
        "Chemin du formateur défini : {0}",
        "Formatierer-Pfad festgelegt: {0}",
        "Ruta del formateador configurada: {0}",
        "포매터 경로를 설정했습니다: {0}"
    ],
    StatusFormatterPathReset => [
        "Reset formatter path to auto-detect",
        "已重置格式化器路径为自动检测",
        "已重設格式化器路徑為自動偵測",
        "フォーマッターのパスを自動検出に戻しました",
        "Chemin réinitialisé sur la détection automatique",
        "Formatierer-Pfad auf automatische Erkennung zurückgesetzt",
        "Ruta restablecida a detección automática",
        "포매터 경로를 자동 감지로 재설정했습니다"
    ],
    PrefTabEditor => [
        "Editor",
        "编辑器",
        "編輯器",
        "エディター",
        "Éditeur",
        "Editor",
        "Editor",
        "편집기"
    ],
    PrefCodeBlocksSection => [
        "Code Blocks",
        "代码块",
        "程式碼區塊",
        "コードブロック",
        "Blocs de code",
        "Codeblöcke",
        "Bloques de código",
        "코드 블록"
    ],
    PrefFormatOnPaste => [
        "Format on Paste",
        "粘贴时自动格式化",
        "貼上時自動格式化",
        "貼り付け時に自動整形",
        "Formater au collage",
        "Beim Einfügen formatieren",
        "Formatear al pegar",
        "붙여넣기 시 자동 서식"
    ],
    PrefFormatOnPasteHint => [
        "Automatically format fenced code blocks with a recognized language after pasting code (off by default).",
        "在已指定受支持语言的围栏代码块内粘贴代码后自动排版（默认关闭）。",
        "在已指定支援語言的圍欄程式碼區塊內貼上程式碼後自動排版（預設關閉）。",
        "対応言語が指定されたコードブロックへの貼り付け時に自動で整形します（既定ではオフ）。",
        "Formate automatiquement les blocs de code avec un langage pris en charge après un collage (désactivé par défaut).",
        "Formatiert unterstützte Codeblöcke nach dem Einfügen automatisch (standardmäßig deaktiviert).",
        "Formatea automáticamente bloques con lenguaje compatible al pegar código (desactivado por defecto).",
        "지원되는 언어가 지정된 코드 블록에 붙여넣을 때 자동으로 서식을 지정합니다(기본값: 꺼짐)."
    ],
    PrefJsTsBackend => [
        "JS / TS formatter",
        "JS / TS 格式化器",
        "JS / TS 格式化器",
        "JS / TS フォーマッター",
        "Formateur JS / TS",
        "JS / TS-Formatierer",
        "Formateador JS / TS",
        "JS / TS 포매터"
    ],
    PrefBackendAuto => [
        "Auto (Biome → Prettier)",
        "自动（Biome → Prettier）",
        "自動（Biome → Prettier）",
        "自動（Biome → Prettier）",
        "Auto (Biome → Prettier)",
        "Auto (Biome → Prettier)",
        "Automático (Biome → Prettier)",
        "자동 (Biome → Prettier)"
    ],
    PrefBackendBiome => [
        "Biome",
        "Biome",
        "Biome",
        "Biome",
        "Biome",
        "Biome",
        "Biome",
        "Biome"
    ],
    PrefBackendPrettier => [
        "Prettier",
        "Prettier",
        "Prettier",
        "Prettier",
        "Prettier",
        "Prettier",
        "Prettier",
        "Prettier"
    ],
    PrefFormatterConfiguration => [
        "Formatter Configuration",
        "格式化器配置",
        "格式化器設定",
        "フォーマッター設定",
        "Configuration des formateurs",
        "Formatierer-Konfiguration",
        "Configuración de formateadores",
        "포매터 구성"
    ],
    PrefFormatterAvailability => [
        "Formatter Availability",
        "格式化器可用性",
        "格式化器可用狀態",
        "フォーマッターの利用可否",
        "Disponibilité des formateurs",
        "Verfügbarkeit der Formatierer",
        "Disponibilidad de formateadores",
        "포매터 사용 가능 여부"
    ],
    PrefBuiltinJson => [
        "Built-in (serde_json)",
        "内置（serde_json）",
        "內建（serde_json）",
        "組み込み (serde_json)",
        "Intégré (serde_json)",
        "Integriert (serde_json)",
        "Integrado (serde_json)",
        "내장 (serde_json)"
    ],
    PrefDefaultPath => [
        "Auto-detect from PATH",
        "从 PATH 自动检测",
        "從 PATH 自動偵測",
        "PATH から自動検出",
        "Détection automatique (PATH)",
        "Automatisch über PATH erkennen",
        "Detectar automáticamente en PATH",
        "PATH에서 자동 감지"
    ],
    PrefBrowse => [
        "Browse…",
        "浏览…",
        "瀏覽…",
        "参照…",
        "Parcourir…",
        "Durchsuchen…",
        "Examinar…",
        "찾아보기…"
    ],
    PrefReset => [
        "Reset",
        "重置",
        "重設",
        "リセット",
        "Réinitialiser",
        "Zurücksetzen",
        "Restablecer",
        "재설정"
    ],
    PrefRefreshAvailability => [
        "Refresh",
        "刷新检查",
        "重新整理",
        "再確認",
        "Actualiser",
        "Aktualisieren",
        "Actualizar",
        "새로 고침"
    ],
    PrefAvailable => [
        "Available",
        "可用",
        "可用",
        "利用可能",
        "Disponible",
        "Verfügbar",
        "Disponible",
        "사용 가능"
    ],
    PrefUnavailable => [
        "Unavailable",
        "不可用",
        "無法使用",
        "利用不可",
        "Indisponible",
        "Nicht verfügbar",
        "No disponible",
        "사용 불가"
    ],
    PrefChecking => [
        "Checking…",
        "检查中…",
        "檢查中…",
        "確認中…",
        "Vérification…",
        "Wird geprüft…",
        "Comprobando…",
        "확인 중…"
    ],
    PrefPickTitle => [
        "Choose formatter executable",
        "选择格式化器可执行文件",
        "選擇格式化器執行檔",
        "フォーマッター実行ファイルを選択",
        "Choisir l’exécutable du formateur",
        "Formatierer-Programm wählen",
        "Elegir ejecutable del formateador",
        "포매터 실행 파일 선택"
    ],
    PrefPickJavaTitle => [
        "Choose google-java-format executable or JAR",
        "选择 google-java-format 可执行文件或 JAR",
        "選擇 google-java-format 執行檔或 JAR",
        "google-java-format 実行ファイルまたは JAR を選択",
        "Choisir l’exécutable ou le JAR google-java-format",
        "google-java-format-Programm oder JAR wählen",
        "Elegir ejecutable o JAR de google-java-format",
        "google-java-format 실행 파일 또는 JAR 선택"
    ],
    PrefNoDownloadHint => [
        "Markion does not download external formatters automatically. Configure local binary or JAR paths below.",
        "Markion 不会自动下载外部格式化工具，请在下方配置本地可执行文件或 JAR 路径。",
        "Markion 不會自動下載外部格式化工具，請在下方設定本機執行檔或 JAR 路徑。",
        "Markion は外部フォーマッターを自動ダウンロードしません。ローカルの実行ファイルや JAR パスを設定してください。",
        "Markion ne télécharge pas de formateurs automatiquement. Configurez des binaires locaux ou un fichier JAR ci-dessous.",
        "Markion lädt keine externen Formatierer automatisch herunter. Konfigurieren Sie lokale Programme oder JAR-Pfade.",
        "Markion no descarga formateadores automáticamente. Configure rutas locales de binarios o JAR abajo.",
        "Markion은 외부 포매터를 자동으로 다운로드하지 않습니다. 아래에서 로컬 실행 파일 또는 JAR 경로를 설정하세요."
    ],
    PathJavaRuntime => [
        "Java runtime (java)",
        "Java 运行时（java）",
        "Java 執行環境（java）",
        "Java ランタイム (java)",
        "Environnement Java (java)",
        "Java-Laufzeit (java)",
        "Entorno Java (java)",
        "Java 런타임 (java)"
    ],
    LangJson => [
        "JSON",
        "JSON",
        "JSON",
        "JSON",
        "JSON",
        "JSON",
        "JSON",
        "JSON"
    ],
    LangJavaScript => [
        "JavaScript",
        "JavaScript",
        "JavaScript",
        "JavaScript",
        "JavaScript",
        "JavaScript",
        "JavaScript",
        "JavaScript"
    ],
    LangTypeScript => [
        "TypeScript",
        "TypeScript",
        "TypeScript",
        "TypeScript",
        "TypeScript",
        "TypeScript",
        "TypeScript",
        "TypeScript"
    ],
    LangJava => [
        "Java",
        "Java",
        "Java",
        "Java",
        "Java",
        "Java",
        "Java",
        "Java"
    ],
    LangRust => [
        "Rust",
        "Rust",
        "Rust",
        "Rust",
        "Rust",
        "Rust",
        "Rust",
        "Rust"
    ],
    LangPython => [
        "Python",
        "Python",
        "Python",
        "Python",
        "Python",
        "Python",
        "Python",
        "Python"
    ],
}
