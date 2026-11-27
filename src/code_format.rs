//! Markdown fenced code block language detection, formatter registry,
//! bounded shell-free execution, and source-preserving replacement helpers.

use std::{
    env,
    ffi::OsString,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{
    CodeBlockFormatPreferences, DocumentInstanceId, JsTsFormatterBackend, MarkdownDocument,
    VisualBlockEditor, VisualBlockId, VisualBlockKind, VisualEditorFieldKind,
    code_format_i18n::CodeFormatMsg,
    external_command::{
        CommandCancellation, ExternalCommandError, ExternalCommandLimits, ExternalCommandRequest,
        ExternalCommandTermination, run_external_command_with_stdin,
    },
};

pub const DEFAULT_FORMAT_TIMEOUT: Duration = Duration::from_secs(5);
pub const DEFAULT_MAX_FORMAT_INPUT_BYTES: usize = 512 * 1024;
pub const DEFAULT_MAX_FORMAT_OUTPUT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeLanguage {
    Json,
    JavaScript,
    TypeScript,
    Java,
    Rust,
    Python,
}

impl CodeLanguage {
    pub const ALL: [Self; 6] = [
        Self::Json,
        Self::JavaScript,
        Self::TypeScript,
        Self::Java,
        Self::Rust,
        Self::Python,
    ];

    pub fn from_token(token: &str) -> Option<Self> {
        let cleaned = token
            .trim()
            .trim_start_matches('.')
            .trim_end_matches([',', ';'])
            .to_ascii_lowercase();
        match cleaned.as_str() {
            "json" => Some(Self::Json),
            "js" | "javascript" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "typescript" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
            "java" => Some(Self::Java),
            "rs" | "rust" => Some(Self::Rust),
            "py" | "python" | "python3" => Some(Self::Python),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Java => "Java",
            Self::Rust => "Rust",
            Self::Python => "Python",
        }
    }

    pub fn i18n_msg(self) -> CodeFormatMsg {
        match self {
            Self::Json => CodeFormatMsg::LangJson,
            Self::JavaScript => CodeFormatMsg::LangJavaScript,
            Self::TypeScript => CodeFormatMsg::LangTypeScript,
            Self::Java => CodeFormatMsg::LangJava,
            Self::Rust => CodeFormatMsg::LangRust,
            Self::Python => CodeFormatMsg::LangPython,
        }
    }

    pub fn stdin_filename_for_token(self, token: Option<&str>) -> &'static str {
        let lower = token
            .map(|t| t.trim().trim_start_matches('.').to_ascii_lowercase())
            .unwrap_or_default();
        match self {
            Self::Json => "snippet.json",
            Self::JavaScript => match lower.as_str() {
                "jsx" => "snippet.jsx",
                "mjs" => "snippet.mjs",
                "cjs" => "snippet.cjs",
                _ => "snippet.js",
            },
            Self::TypeScript => match lower.as_str() {
                "tsx" => "snippet.tsx",
                "mts" => "snippet.mts",
                "cts" => "snippet.cts",
                _ => "snippet.ts",
            },
            Self::Java => "Snippet.java",
            Self::Rust => "snippet.rs",
            Self::Python => "snippet.py",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeFormatOptions {
    pub timeout: Duration,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
}

impl Default for CodeFormatOptions {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_FORMAT_TIMEOUT,
            max_input_bytes: DEFAULT_MAX_FORMAT_INPUT_BYTES,
            max_output_bytes: DEFAULT_MAX_FORMAT_OUTPUT_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CodeFormatError {
    #[error("code block has no language identifier")]
    EmptyLanguage,
    #[error("unsupported code block language: {0}")]
    UnsupportedLanguage(String),
    #[error("code block exceeds maximum size ({bytes} > {limit} bytes)")]
    InputTooLarge { bytes: usize, limit: usize },
    #[error("no formatter registered for {0:?}")]
    NoFormatterRegistered(CodeLanguage),
    #[error("{language:?} formatter is unavailable: {detail}")]
    FormatterUnavailable {
        language: CodeLanguage,
        detail: String,
    },
    #[error("{language:?} formatter timed out")]
    TimedOut { language: CodeLanguage },
    #[error("{language:?} formatter cancelled")]
    Cancelled { language: CodeLanguage },
    #[error("{language:?} formatter failed: {detail}")]
    ExecutionFailed {
        language: CodeLanguage,
        detail: String,
    },
    #[error("{language:?} formatter produced invalid output: {detail}")]
    InvalidOutput {
        language: CodeLanguage,
        detail: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatterProbeStatus {
    pub language: CodeLanguage,
    pub available: bool,
    pub tool_name: &'static str,
    pub resolved_path: Option<PathBuf>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormatterAvailability {
    pub statuses: Vec<FormatterProbeStatus>,
}

impl FormatterAvailability {
    pub fn status_for(&self, language: CodeLanguage) -> Option<&FormatterProbeStatus> {
        self.statuses
            .iter()
            .find(|status| status.language == language)
    }
}

pub trait CodeFormatter: Send + Sync {
    fn id(&self) -> &'static str;
    fn supports(&self, language: CodeLanguage) -> bool;
    fn format(
        &self,
        language: CodeLanguage,
        language_token: Option<&str>,
        source: &str,
        prefs: &CodeBlockFormatPreferences,
        options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError>;
    fn probe_availability(
        &self,
        language: CodeLanguage,
        prefs: &CodeBlockFormatPreferences,
    ) -> FormatterProbeStatus;
}

#[derive(Clone, Default)]
pub struct CodeFormatterRegistry {
    formatters: Vec<Arc<dyn CodeFormatter>>,
}

impl std::fmt::Debug for CodeFormatterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<_> = self.formatters.iter().map(|fmt| fmt.id()).collect();
        f.debug_struct("CodeFormatterRegistry")
            .field("formatters", &ids)
            .finish()
    }
}

impl CodeFormatterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(JsonFormatter));
        registry.register(Arc::new(JsTsFormatter));
        registry.register(Arc::new(JavaFormatter));
        registry.register(Arc::new(RustFormatter));
        registry.register(Arc::new(PythonFormatter));
        registry
    }

    /// Registers a formatter with highest priority so extensions or test stubs
    /// can override or add language support without altering core editor code.
    pub fn register(&mut self, formatter: Arc<dyn CodeFormatter>) {
        self.formatters.insert(0, formatter);
    }

    pub fn formatter_for(&self, language: CodeLanguage) -> Option<&dyn CodeFormatter> {
        self.formatters
            .iter()
            .find(|fmt| fmt.supports(language))
            .map(Arc::as_ref)
    }

    pub fn format_snippet(
        &self,
        language: CodeLanguage,
        language_token: Option<&str>,
        source: &str,
        prefs: &CodeBlockFormatPreferences,
        options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError> {
        if source.len() > options.max_input_bytes {
            return Err(CodeFormatError::InputTooLarge {
                bytes: source.len(),
                limit: options.max_input_bytes,
            });
        }
        let Some(formatter) = self.formatter_for(language) else {
            return Err(CodeFormatError::NoFormatterRegistered(language));
        };
        formatter.format(
            language,
            language_token,
            source,
            prefs,
            options,
            cancellation,
        )
    }

    pub fn probe_all(&self, prefs: &CodeBlockFormatPreferences) -> FormatterAvailability {
        let statuses = CodeLanguage::ALL
            .into_iter()
            .map(|language| match self.formatter_for(language) {
                Some(fmt) => fmt.probe_availability(language, prefs),
                None => FormatterProbeStatus {
                    language,
                    available: false,
                    tool_name: "none",
                    resolved_path: None,
                    detail: Some("no formatter registered".to_string()),
                },
            })
            .collect();
        FormatterAvailability { statuses }
    }
}

pub struct JsonFormatter;

impl CodeFormatter for JsonFormatter {
    fn id(&self) -> &'static str {
        "json-serde"
    }

    fn supports(&self, language: CodeLanguage) -> bool {
        language == CodeLanguage::Json
    }

    fn format(
        &self,
        language: CodeLanguage,
        _language_token: Option<&str>,
        source: &str,
        _prefs: &CodeBlockFormatPreferences,
        _options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError> {
        if cancellation.is_cancelled() {
            return Err(CodeFormatError::Cancelled { language });
        }
        if source.trim().is_empty() {
            return Ok(String::new());
        }
        let value: serde_json::Value =
            serde_json::from_str(source).map_err(|err| CodeFormatError::InvalidOutput {
                language,
                detail: err.to_string(),
            })?;
        serde_json::to_string_pretty(&value).map_err(|err| CodeFormatError::InvalidOutput {
            language,
            detail: err.to_string(),
        })
    }

    fn probe_availability(
        &self,
        language: CodeLanguage,
        _prefs: &CodeBlockFormatPreferences,
    ) -> FormatterProbeStatus {
        FormatterProbeStatus {
            language,
            available: true,
            tool_name: "serde_json",
            resolved_path: None,
            detail: None,
        }
    }
}

pub struct JsTsFormatter;

impl JsTsFormatter {
    fn resolve_backend(
        prefs: &CodeBlockFormatPreferences,
    ) -> Result<(JsTsFormatterBackend, PathBuf), &'static str> {
        match prefs.js_ts_backend {
            JsTsFormatterBackend::Biome => resolve_executable(prefs.biome_path.as_deref(), "biome")
                .map(|path| (JsTsFormatterBackend::Biome, path))
                .ok_or("biome"),
            JsTsFormatterBackend::Prettier => {
                resolve_executable(prefs.prettier_path.as_deref(), "prettier")
                    .map(|path| (JsTsFormatterBackend::Prettier, path))
                    .ok_or("prettier")
            }
            JsTsFormatterBackend::Auto => {
                if let Some(path) = resolve_executable(prefs.biome_path.as_deref(), "biome") {
                    Ok((JsTsFormatterBackend::Biome, path))
                } else if let Some(path) =
                    resolve_executable(prefs.prettier_path.as_deref(), "prettier")
                {
                    Ok((JsTsFormatterBackend::Prettier, path))
                } else {
                    Err("biome / prettier")
                }
            }
        }
    }
}

impl CodeFormatter for JsTsFormatter {
    fn id(&self) -> &'static str {
        "js-ts-external"
    }

    fn supports(&self, language: CodeLanguage) -> bool {
        matches!(
            language,
            CodeLanguage::JavaScript | CodeLanguage::TypeScript
        )
    }

    fn format(
        &self,
        language: CodeLanguage,
        language_token: Option<&str>,
        source: &str,
        prefs: &CodeBlockFormatPreferences,
        options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError> {
        if source.trim().is_empty() {
            return Ok(String::new());
        }
        let (backend, program) = Self::resolve_backend(prefs).map_err(|missing| {
            CodeFormatError::FormatterUnavailable {
                language,
                detail: missing.to_string(),
            }
        })?;
        let filename = language.stdin_filename_for_token(language_token);
        let args: Vec<OsString> = match backend {
            JsTsFormatterBackend::Biome => vec![
                OsString::from("format"),
                OsString::from("--stdin-file-path"),
                OsString::from(filename),
            ],
            JsTsFormatterBackend::Prettier | JsTsFormatterBackend::Auto => {
                vec![OsString::from("--stdin-filepath"), OsString::from(filename)]
            }
        };
        run_stdin_formatter(language, program, args, source, options, cancellation)
    }

    fn probe_availability(
        &self,
        language: CodeLanguage,
        prefs: &CodeBlockFormatPreferences,
    ) -> FormatterProbeStatus {
        match Self::resolve_backend(prefs) {
            Ok((backend, path)) => FormatterProbeStatus {
                language,
                available: true,
                tool_name: match backend {
                    JsTsFormatterBackend::Biome => "biome",
                    JsTsFormatterBackend::Prettier | JsTsFormatterBackend::Auto => "prettier",
                },
                resolved_path: Some(path),
                detail: None,
            },
            Err(missing) => FormatterProbeStatus {
                language,
                available: false,
                tool_name: missing,
                resolved_path: None,
                detail: Some(format!("{missing} not found")),
            },
        }
    }
}

pub struct JavaFormatter;

enum JavaInvocation {
    Binary(PathBuf),
    Jar { java: PathBuf, jar: PathBuf },
}

impl JavaFormatter {
    fn resolve_invocation(prefs: &CodeBlockFormatPreferences) -> Result<JavaInvocation, String> {
        if let Some(configured) = normalized_optional_path(prefs.google_java_format_path.as_deref())
        {
            let path = PathBuf::from(&configured);
            let is_jar = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"));
            if is_jar {
                if !path.is_file() {
                    return Err(format!("google-java-format JAR not found: {configured}"));
                }
                let java = resolve_executable(prefs.java_path.as_deref(), "java")
                    .ok_or_else(|| "java runtime".to_string())?;
                return Ok(JavaInvocation::Jar { java, jar: path });
            }
            return resolve_executable(Some(&configured), "google-java-format")
                .map(JavaInvocation::Binary)
                .ok_or_else(|| "google-java-format".to_string());
        }
        resolve_executable(None, "google-java-format")
            .map(JavaInvocation::Binary)
            .ok_or_else(|| "google-java-format".to_string())
    }
}

impl CodeFormatter for JavaFormatter {
    fn id(&self) -> &'static str {
        "google-java-format"
    }

    fn supports(&self, language: CodeLanguage) -> bool {
        language == CodeLanguage::Java
    }

    fn format(
        &self,
        language: CodeLanguage,
        _language_token: Option<&str>,
        source: &str,
        prefs: &CodeBlockFormatPreferences,
        options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError> {
        if source.trim().is_empty() {
            return Ok(String::new());
        }
        let invocation = Self::resolve_invocation(prefs)
            .map_err(|detail| CodeFormatError::FormatterUnavailable { language, detail })?;
        let (program, args) = match invocation {
            JavaInvocation::Binary(bin) => (bin, vec![OsString::from("-")]),
            JavaInvocation::Jar { java, jar } => (
                java,
                vec![
                    OsString::from("-jar"),
                    jar.into_os_string(),
                    OsString::from("-"),
                ],
            ),
        };
        run_stdin_formatter(language, program, args, source, options, cancellation)
    }

    fn probe_availability(
        &self,
        language: CodeLanguage,
        prefs: &CodeBlockFormatPreferences,
    ) -> FormatterProbeStatus {
        match Self::resolve_invocation(prefs) {
            Ok(JavaInvocation::Binary(path)) => FormatterProbeStatus {
                language,
                available: true,
                tool_name: "google-java-format",
                resolved_path: Some(path),
                detail: None,
            },
            Ok(JavaInvocation::Jar { jar, .. }) => FormatterProbeStatus {
                language,
                available: true,
                tool_name: "google-java-format (JAR)",
                resolved_path: Some(jar),
                detail: None,
            },
            Err(detail) => FormatterProbeStatus {
                language,
                available: false,
                tool_name: "google-java-format",
                resolved_path: None,
                detail: Some(detail),
            },
        }
    }
}

pub struct RustFormatter;

impl CodeFormatter for RustFormatter {
    fn id(&self) -> &'static str {
        "rustfmt"
    }

    fn supports(&self, language: CodeLanguage) -> bool {
        language == CodeLanguage::Rust
    }

    fn format(
        &self,
        language: CodeLanguage,
        _language_token: Option<&str>,
        source: &str,
        prefs: &CodeBlockFormatPreferences,
        options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError> {
        if source.trim().is_empty() {
            return Ok(String::new());
        }
        let program =
            resolve_executable(prefs.rustfmt_path.as_deref(), "rustfmt").ok_or_else(|| {
                CodeFormatError::FormatterUnavailable {
                    language,
                    detail: "rustfmt".to_string(),
                }
            })?;
        let args = vec![OsString::from("--edition"), OsString::from("2021")];
        run_stdin_formatter(language, program, args, source, options, cancellation)
    }

    fn probe_availability(
        &self,
        language: CodeLanguage,
        prefs: &CodeBlockFormatPreferences,
    ) -> FormatterProbeStatus {
        match resolve_executable(prefs.rustfmt_path.as_deref(), "rustfmt") {
            Some(path) => FormatterProbeStatus {
                language,
                available: true,
                tool_name: "rustfmt",
                resolved_path: Some(path),
                detail: None,
            },
            None => FormatterProbeStatus {
                language,
                available: false,
                tool_name: "rustfmt",
                resolved_path: None,
                detail: Some("rustfmt not found".to_string()),
            },
        }
    }
}

pub struct PythonFormatter;

impl CodeFormatter for PythonFormatter {
    fn id(&self) -> &'static str {
        "ruff-format"
    }

    fn supports(&self, language: CodeLanguage) -> bool {
        language == CodeLanguage::Python
    }

    fn format(
        &self,
        language: CodeLanguage,
        _language_token: Option<&str>,
        source: &str,
        prefs: &CodeBlockFormatPreferences,
        options: &CodeFormatOptions,
        cancellation: &CommandCancellation,
    ) -> Result<String, CodeFormatError> {
        if source.trim().is_empty() {
            return Ok(String::new());
        }
        let program = resolve_executable(prefs.ruff_path.as_deref(), "ruff").ok_or_else(|| {
            CodeFormatError::FormatterUnavailable {
                language,
                detail: "ruff".to_string(),
            }
        })?;
        let args = vec![
            OsString::from("format"),
            OsString::from("--stdin-filename"),
            OsString::from("snippet.py"),
            OsString::from("-"),
        ];
        run_stdin_formatter(language, program, args, source, options, cancellation)
    }

    fn probe_availability(
        &self,
        language: CodeLanguage,
        prefs: &CodeBlockFormatPreferences,
    ) -> FormatterProbeStatus {
        match resolve_executable(prefs.ruff_path.as_deref(), "ruff") {
            Some(path) => FormatterProbeStatus {
                language,
                available: true,
                tool_name: "ruff format",
                resolved_path: Some(path),
                detail: None,
            },
            None => FormatterProbeStatus {
                language,
                available: false,
                tool_name: "ruff",
                resolved_path: None,
                detail: Some("ruff not found".to_string()),
            },
        }
    }
}

fn normalized_optional_path(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub fn resolve_executable(configured: Option<&str>, default_name: &str) -> Option<PathBuf> {
    if let Some(configured) = normalized_optional_path(configured) {
        let candidate = PathBuf::from(&configured);
        if let Some(found) = resolve_explicit_path(&candidate) {
            return Some(found);
        }
        if candidate.components().count() == 1 {
            return find_in_path(&configured);
        }
        return None;
    }
    find_in_path(default_name)
}

fn resolve_explicit_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    #[cfg(windows)]
    if path.extension().is_none() {
        for ext in windows_pathext() {
            let ext_trimmed = ext.trim_start_matches('.');
            let with_ext = path.with_extension(ext_trimmed);
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }
    }
    None
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    let name_path = Path::new(name);
    let has_ext = name_path.extension().is_some();
    for dir in env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let direct = dir.join(name_path);
        if direct.is_file() {
            return Some(direct);
        }
        #[cfg(windows)]
        if !has_ext {
            for ext in windows_pathext() {
                let mut file_name = OsString::from(name);
                file_name.push(ext);
                let candidate = dir.join(file_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        #[cfg(not(windows))]
        let _ = has_ext;
    }
    None
}

#[cfg(windows)]
fn windows_pathext() -> Vec<String> {
    let raw = env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD;.PS1".to_string());
    let mut exts: Vec<String> = raw
        .split(';')
        .map(str::trim)
        .filter(|ext| !ext.is_empty())
        .map(|ext| {
            if ext.starts_with('.') {
                ext.to_string()
            } else {
                format!(".{ext}")
            }
        })
        .collect();
    if !exts.iter().any(|ext| ext.eq_ignore_ascii_case(".ps1")) {
        exts.push(".PS1".to_string());
    }
    exts
}

fn prepare_command_request(program: PathBuf, arguments: Vec<OsString>) -> ExternalCommandRequest {
    let working_directory = env::temp_dir();
    #[cfg(windows)]
    {
        let is_ps1 = program
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"));
        if is_ps1 {
            let shell = find_in_path("pwsh.exe")
                .or_else(|| find_in_path("powershell.exe"))
                .unwrap_or_else(|| PathBuf::from("powershell.exe"));
            let mut ps_args = vec![
                OsString::from("-NoLogo"),
                OsString::from("-NoProfile"),
                OsString::from("-NonInteractive"),
                OsString::from("-ExecutionPolicy"),
                OsString::from("Bypass"),
                OsString::from("-File"),
                program.into_os_string(),
            ];
            ps_args.extend(arguments);
            return ExternalCommandRequest {
                program: shell,
                arguments: ps_args,
                working_directory,
            };
        }
    }
    ExternalCommandRequest {
        program,
        arguments,
        working_directory,
    }
}

pub fn run_stdin_formatter(
    language: CodeLanguage,
    program: PathBuf,
    arguments: Vec<OsString>,
    source: &str,
    options: &CodeFormatOptions,
    cancellation: &CommandCancellation,
) -> Result<String, CodeFormatError> {
    let request = prepare_command_request(program, arguments);
    let limits = ExternalCommandLimits {
        timeout: options.timeout,
        max_stdout_bytes: options.max_output_bytes,
        max_stderr_bytes: options.max_output_bytes,
    };
    let output = run_external_command_with_stdin(&request, source.as_bytes(), limits, cancellation)
        .map_err(|err| match err {
            ExternalCommandError::Spawn(io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                CodeFormatError::FormatterUnavailable {
                    language,
                    detail: io_err.to_string(),
                }
            }
            other => CodeFormatError::ExecutionFailed {
                language,
                detail: other.to_string(),
            },
        })?;

    match output.termination {
        ExternalCommandTermination::TimedOut => return Err(CodeFormatError::TimedOut { language }),
        ExternalCommandTermination::Cancelled => {
            return Err(CodeFormatError::Cancelled { language });
        }
        ExternalCommandTermination::Exited => {}
    }

    if !output.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            format!("exit code {:?}", output.status_code)
        } else {
            stderr.lines().next().unwrap_or(&stderr).to_string()
        };
        return Err(CodeFormatError::ExecutionFailed { language, detail });
    }

    if output.stdout_truncated {
        return Err(CodeFormatError::InvalidOutput {
            language,
            detail: "formatted output exceeded size limit".to_string(),
        });
    }

    String::from_utf8(output.stdout).map_err(|err| CodeFormatError::InvalidOutput {
        language,
        detail: format!("formatter emitted non-UTF-8 output: {err}"),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FencedCodeBlockTarget {
    pub document_id: DocumentInstanceId,
    pub document_version: u64,
    pub block_id: VisualBlockId,
    pub block_range: Range<usize>,
    pub payload_range: Range<usize>,
    pub info_range: Option<Range<usize>>,
    pub raw_info: Option<String>,
    pub language_token: Option<String>,
    pub language: Option<CodeLanguage>,
    pub is_closed: bool,
    pub line_ending: &'static str,
    pub line_indent: String,
    pub expected_payload: String,
}

pub fn fenced_code_block_at_offset(
    document: &MarkdownDocument,
    offset: usize,
    payload_only: bool,
) -> Option<FencedCodeBlockTarget> {
    fenced_code_block_for_range(document, offset..offset, payload_only)
}

pub fn fenced_code_block_for_range(
    document: &MarkdownDocument,
    range: Range<usize>,
    payload_only: bool,
) -> Option<FencedCodeBlockTarget> {
    let text = document.text();
    let doc_crlf = text.contains("\r\n");
    let blocks = document.visual_blocks_shared();

    let mut candidate: Option<FencedCodeBlockTarget> = None;
    for block in blocks.iter() {
        if block.quote_context.is_some() {
            continue;
        }
        let VisualBlockKind::CodeBlock {
            language: parsed_lang,
            ..
        } = &block.kind
        else {
            continue;
        };
        let Some(VisualBlockEditor::Code {
            payload,
            info_range,
            closing_fence,
            ..
        }) = &block.editor
        else {
            continue;
        };
        if payload.kind != VisualEditorFieldKind::CodePayload {
            continue;
        }

        let in_payload =
            payload.source_range.start <= range.start && range.end <= payload.source_range.end;
        let in_block =
            block.source_range.start <= range.start && range.end <= block.source_range.end;
        if payload_only {
            if !in_payload {
                continue;
            }
        } else if !in_block {
            continue;
        }

        let block_slice = text.get(block.source_range.clone())?;
        let expected_payload = text.get(payload.source_range.clone())?.to_string();
        let line_ending = if block_slice.contains("\r\n") || doc_crlf {
            "\r\n"
        } else {
            "\n"
        };
        let line_start = text[..block.source_range.start]
            .rfind('\n')
            .map_or(0, |idx| idx + 1);
        let prefix = &text[line_start..block.source_range.start];
        let line_indent = if prefix.bytes().all(|b| b == b' ' || b == b'\t') {
            prefix.to_string()
        } else {
            String::new()
        };

        let raw_info = info_range
            .as_ref()
            .and_then(|r| text.get(r.clone()))
            .map(ToString::to_string);
        let language_token = raw_info
            .as_deref()
            .and_then(|info| info.split_whitespace().next())
            .map(ToString::to_string)
            .or_else(|| parsed_lang.clone());
        let language = language_token.as_deref().and_then(CodeLanguage::from_token);

        let target = FencedCodeBlockTarget {
            document_id: document.instance_id(),
            document_version: document.version(),
            block_id: block.id,
            block_range: block.source_range.clone(),
            payload_range: payload.source_range.clone(),
            info_range: info_range.clone(),
            raw_info,
            language_token,
            language,
            is_closed: !closing_fence.is_empty(),
            line_ending,
            line_indent,
            expected_payload,
        };
        if in_payload {
            return Some(target);
        }
        candidate = Some(target);
    }
    candidate
}

pub fn fenced_code_block_by_id(
    document: &MarkdownDocument,
    block_id: VisualBlockId,
) -> Option<FencedCodeBlockTarget> {
    let blocks = document.visual_blocks_shared();
    let block = blocks.iter().find(|block| block.id == block_id)?;
    let VisualBlockEditor::Code { payload, .. } = block.editor.as_ref()? else {
        return None;
    };
    if payload.kind != VisualEditorFieldKind::CodePayload {
        return None;
    }
    fenced_code_block_at_offset(document, payload.source_range.start, false)
}

pub fn prepare_code_input(target: &FencedCodeBlockTarget) -> String {
    let mut normalized = target
        .expected_payload
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    if !target.line_indent.is_empty() {
        let mut out = String::with_capacity(normalized.len());
        for chunk in normalized.split_inclusive('\n') {
            if let Some(stripped) = chunk.strip_prefix(&target.line_indent) {
                out.push_str(stripped);
            } else {
                out.push_str(chunk);
            }
        }
        normalized = out;
    }
    if target.is_closed && !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

pub fn build_formatted_payload_replacement(
    target: &FencedCodeBlockTarget,
    formatted: &str,
) -> String {
    let normalized = formatted.replace("\r\n", "\n").replace('\r', "\n");
    let body = normalized.trim_end_matches('\n');
    if body.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(body.len() + 32);
    let mut lines = body.split('\n').peekable();
    while let Some(line) = lines.next() {
        if !line.is_empty() && !target.line_indent.is_empty() {
            out.push_str(&target.line_indent);
        }
        out.push_str(line);
        if lines.peek().is_some() {
            out.push_str(target.line_ending);
        }
    }

    if !target.is_closed && target.expected_payload.ends_with('\n') {
        out.push_str(target.line_ending);
    }
    out
}

pub fn clamped_cursor_after_format(
    selection: Range<usize>,
    target: &FencedCodeBlockTarget,
    replacement: &str,
) -> Range<usize> {
    let start = target.payload_range.start;
    let old_end = target.payload_range.end;
    let content_end_rel = if target.is_closed && replacement.ends_with(target.line_ending) {
        replacement.len().saturating_sub(target.line_ending.len())
    } else {
        replacement.len()
    };
    let content_end = start + content_end_rel;

    if selection.end <= start && !target.block_range.contains(&selection.start) {
        return selection;
    }
    if selection.start >= old_end && selection.start > target.block_range.end {
        let delta = replacement.len() as isize - target.expected_payload.len() as isize;
        let shifted_start = (selection.start as isize + delta).max(0) as usize;
        let shifted_end = (selection.end as isize + delta).max(shifted_start as isize) as usize;
        return shifted_start..shifted_end;
    }

    let rel = selection.start.saturating_sub(start).min(content_end_rel);
    let mut clamped_rel = rel;
    while clamped_rel > 0 && !replacement.is_char_boundary(clamped_rel) {
        clamped_rel -= 1;
    }
    let caret = (start + clamped_rel).min(content_end);
    caret..caret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_detection_maps_aliases_and_ignores_extra_info_tokens() {
        assert_eq!(CodeLanguage::from_token("json"), Some(CodeLanguage::Json));
        assert_eq!(CodeLanguage::from_token("JSON"), Some(CodeLanguage::Json));
        assert_eq!(
            CodeLanguage::from_token("js"),
            Some(CodeLanguage::JavaScript)
        );
        assert_eq!(
            CodeLanguage::from_token("javascript"),
            Some(CodeLanguage::JavaScript)
        );
        assert_eq!(
            CodeLanguage::from_token("jsx"),
            Some(CodeLanguage::JavaScript)
        );
        assert_eq!(
            CodeLanguage::from_token("ts"),
            Some(CodeLanguage::TypeScript)
        );
        assert_eq!(
            CodeLanguage::from_token("tsx"),
            Some(CodeLanguage::TypeScript)
        );
        assert_eq!(CodeLanguage::from_token("java"), Some(CodeLanguage::Java));
        assert_eq!(CodeLanguage::from_token("rust"), Some(CodeLanguage::Rust));
        assert_eq!(CodeLanguage::from_token("rs"), Some(CodeLanguage::Rust));
        assert_eq!(
            CodeLanguage::from_token("python"),
            Some(CodeLanguage::Python)
        );
        assert_eq!(CodeLanguage::from_token("py"), Some(CodeLanguage::Python));
        assert_eq!(CodeLanguage::from_token("mermaid"), None);
        assert_eq!(CodeLanguage::from_token(""), None);
    }

    #[test]
    fn json_formatter_formats_valid_json_and_rejects_invalid_json() {
        let registry = CodeFormatterRegistry::with_defaults();
        let prefs = CodeBlockFormatPreferences::default();
        let opts = CodeFormatOptions::default();
        let cancel = CommandCancellation::default();

        let formatted = registry
            .format_snippet(
                CodeLanguage::Json,
                Some("json"),
                "{\"b\":2,\"a\":[1,2]}",
                &prefs,
                &opts,
                &cancel,
            )
            .unwrap();
        assert_eq!(
            formatted,
            "{\n  \"b\": 2,\n  \"a\": [\n    1,\n    2\n  ]\n}"
        );

        let err = registry
            .format_snippet(
                CodeLanguage::Json,
                Some("json"),
                "{invalid json",
                &prefs,
                &opts,
                &cancel,
            )
            .unwrap_err();
        assert!(matches!(err, CodeFormatError::InvalidOutput { .. }));
    }

    #[test]
    fn custom_formatter_can_extend_registry_without_editing_core() {
        struct MockJava;
        impl CodeFormatter for MockJava {
            fn id(&self) -> &'static str {
                "mock-java"
            }
            fn supports(&self, language: CodeLanguage) -> bool {
                language == CodeLanguage::Java
            }
            fn format(
                &self,
                _language: CodeLanguage,
                _language_token: Option<&str>,
                source: &str,
                _prefs: &CodeBlockFormatPreferences,
                _options: &CodeFormatOptions,
                _cancellation: &CommandCancellation,
            ) -> Result<String, CodeFormatError> {
                Ok(format!("// formatted\n{}", source.trim()))
            }
            fn probe_availability(
                &self,
                language: CodeLanguage,
                _prefs: &CodeBlockFormatPreferences,
            ) -> FormatterProbeStatus {
                FormatterProbeStatus {
                    language,
                    available: true,
                    tool_name: "mock-java",
                    resolved_path: None,
                    detail: None,
                }
            }
        }

        let mut registry = CodeFormatterRegistry::with_defaults();
        registry.register(Arc::new(MockJava));
        let out = registry
            .format_snippet(
                CodeLanguage::Java,
                Some("java"),
                "class  Foo{}",
                &CodeBlockFormatPreferences::default(),
                &CodeFormatOptions::default(),
                &CommandCancellation::default(),
            )
            .unwrap();
        assert_eq!(out, "// formatted\nclass  Foo{}");
    }

    #[test]
    fn enforces_input_size_limit_without_spawning() {
        let registry = CodeFormatterRegistry::with_defaults();
        let err = registry
            .format_snippet(
                CodeLanguage::Json,
                Some("json"),
                "{\"a\":1}",
                &CodeBlockFormatPreferences::default(),
                &CodeFormatOptions {
                    max_input_bytes: 3,
                    ..CodeFormatOptions::default()
                },
                &CommandCancellation::default(),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            CodeFormatError::InputTooLarge { bytes: 7, limit: 3 }
        ));
    }

    #[test]
    fn fenced_extraction_excludes_paragraphs_inline_code_and_indented_blocks() {
        let md = "Paragraph `{\"a\":1}` text.\n\n    {\"indented\":1}\n\n```json title=\"cfg\"\n{\"a\":1}\n```\n";
        let doc = MarkdownDocument::from_text(md);
        // Paragraph
        assert!(fenced_code_block_at_offset(&doc, 2, false).is_none());
        // Inline code inside paragraph
        let inline_pos = md.find("{\"a\":1}").unwrap();
        assert!(fenced_code_block_at_offset(&doc, inline_pos, false).is_none());
        // Indented code block
        let indented_pos = md.find("{\"indented\":1}").unwrap();
        assert!(fenced_code_block_at_offset(&doc, indented_pos, false).is_none());
        // Fenced code block payload
        let fence_pos = md.rfind("{\"a\":1}").unwrap();
        let target = fenced_code_block_at_offset(&doc, fence_pos, true).unwrap();
        assert_eq!(target.language, Some(CodeLanguage::Json));
        assert_eq!(target.raw_info.as_deref(), Some("json title=\"cfg\""));
        assert_eq!(target.line_ending, "\n");
        assert!(target.is_closed);
        assert_eq!(target.expected_payload, "{\"a\":1}");

        // Opening fence line is excluded for payload_only=true, included for payload_only=false
        let fence_start = md.find("```json").unwrap();
        assert!(fenced_code_block_at_offset(&doc, fence_start + 2, true).is_none());
        assert!(fenced_code_block_at_offset(&doc, fence_start + 2, false).is_some());
    }

    #[test]
    fn preserves_crlf_fences_info_string_and_list_indentation() {
        let md = "- item\r\n\r\n  ~~~json keep=1\r\n  {\"b\":2,\"a\":1}\r\n  ~~~\r\n";
        let doc = MarkdownDocument::from_text(md);
        let pos = md.find("{\"b\":2").unwrap();
        let target = fenced_code_block_at_offset(&doc, pos, true).unwrap();
        assert_eq!(target.line_ending, "\r\n");
        assert_eq!(target.line_indent, "  ");
        assert_eq!(target.raw_info.as_deref(), Some("json keep=1"));

        let input = prepare_code_input(&target);
        assert_eq!(input, "{\"b\":2,\"a\":1}\n");
        let replacement =
            build_formatted_payload_replacement(&target, "{\n  \"a\": 1,\n  \"b\": 2\n}");
        assert_eq!(replacement, "  {\r\n    \"a\": 1,\r\n    \"b\": 2\r\n  }");
        let mut updated = md.to_string();
        updated.replace_range(target.payload_range.clone(), &replacement);
        assert_eq!(
            updated,
            "- item\r\n\r\n  ~~~json keep=1\r\n  {\r\n    \"a\": 1,\r\n    \"b\": 2\r\n  }\r\n  ~~~\r\n"
        );
    }

    #[test]
    fn formats_rust_with_real_rustfmt_when_available() {
        let registry = CodeFormatterRegistry::with_defaults();
        let prefs = CodeBlockFormatPreferences::default();
        if !registry
            .probe_all(&prefs)
            .status_for(CodeLanguage::Rust)
            .is_some_and(|s| s.available)
        {
            return;
        }
        let out = registry
            .format_snippet(
                CodeLanguage::Rust,
                Some("rust"),
                "fn   main( ){let x=1+2;println!(\"{}\",x);}",
                &prefs,
                &CodeFormatOptions::default(),
                &CommandCancellation::default(),
            )
            .unwrap();
        let normalized = out.replace("\r\n", "\n");
        assert!(normalized.contains("fn main() {\n    let x = 1 + 2;"));
    }

    #[test]
    fn handles_spaces_in_executable_path_errors_and_timeouts() {
        let dir = tempfile::tempdir().unwrap();
        let spaced = dir.path().join("my formatters");
        std::fs::create_dir_all(&spaced).unwrap();

        #[cfg(windows)]
        let (ok_script, err_script, hang_script, java_jar_stub) = {
            let ok = spaced.join("fmt ok.cmd");
            std::fs::write(&ok, "@echo off\r\nmore > nul\r\necho const x = 1;\r\n").unwrap();
            let err = spaced.join("fmt fail.cmd");
            std::fs::write(&err, "@echo off\r\necho syntax error 1>&2\r\nexit /b 2\r\n").unwrap();
            let hang = spaced.join("fmt hang.ps1");
            std::fs::write(&hang, "Start-Sleep -Seconds 30\n").unwrap();
            let fake_jar = spaced.join("google-java-format.jar");
            std::fs::write(&fake_jar, b"not-a-real-jar").unwrap();
            (ok, err, hang, fake_jar)
        };

        #[cfg(not(windows))]
        let (ok_script, err_script, hang_script, java_jar_stub) = {
            use std::os::unix::fs::PermissionsExt;
            let make_exec = |name: &str, body: &str| {
                let p = spaced.join(name);
                std::fs::write(&p, body).unwrap();
                let mut perms = std::fs::metadata(&p).unwrap().permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&p, perms).unwrap();
                p
            };
            let ok = make_exec(
                "fmt ok.sh",
                "#!/bin/sh\ncat >/dev/null\nprintf 'const x = 1;\\n'\n",
            );
            let err = make_exec(
                "fmt fail.sh",
                "#!/bin/sh\necho 'syntax error' >&2\nexit 2\n",
            );
            let hang = make_exec("fmt hang.sh", "#!/bin/sh\nsleep 30\n");
            let fake_jar = spaced.join("google-java-format.jar");
            std::fs::write(&fake_jar, b"not-a-real-jar").unwrap();
            (ok, err, hang, fake_jar)
        };

        let registry = CodeFormatterRegistry::with_defaults();
        let mut prefs = CodeBlockFormatPreferences {
            js_ts_backend: JsTsFormatterBackend::Prettier,
            prettier_path: Some(ok_script.display().to_string()),
            ..CodeBlockFormatPreferences::default()
        };

        let ok = registry
            .format_snippet(
                CodeLanguage::JavaScript,
                Some("js"),
                "const x=1",
                &prefs,
                &CodeFormatOptions::default(),
                &CommandCancellation::default(),
            )
            .unwrap();
        assert_eq!(ok.trim(), "const x = 1;");

        prefs.prettier_path = Some(err_script.display().to_string());
        let fail = registry
            .format_snippet(
                CodeLanguage::JavaScript,
                Some("js"),
                "const x=1",
                &prefs,
                &CodeFormatOptions::default(),
                &CommandCancellation::default(),
            )
            .unwrap_err();
        assert!(matches!(fail, CodeFormatError::ExecutionFailed { .. }));

        prefs.prettier_path = Some(hang_script.display().to_string());
        let timeout = registry
            .format_snippet(
                CodeLanguage::TypeScript,
                Some("ts"),
                "const x:number=1",
                &prefs,
                &CodeFormatOptions {
                    timeout: Duration::from_millis(120),
                    ..CodeFormatOptions::default()
                },
                &CommandCancellation::default(),
            )
            .unwrap_err();
        assert!(matches!(timeout, CodeFormatError::TimedOut { .. }));

        // Verify .jar detection selects JavaInvocation::Jar
        prefs.google_java_format_path = Some(java_jar_stub.display().to_string());
        prefs.java_path = Some(ok_script.display().to_string());
        let status = registry.probe_all(&prefs);
        let java_status = status.status_for(CodeLanguage::Java).unwrap();
        assert!(java_status.available);
        assert_eq!(java_status.tool_name, "google-java-format (JAR)");

        // Missing configured binary reports unavailable gracefully
        prefs.ruff_path = Some(spaced.join("missing-ruff").display().to_string());
        let py_err = registry
            .format_snippet(
                CodeLanguage::Python,
                Some("python"),
                "x=1",
                &prefs,
                &CodeFormatOptions::default(),
                &CommandCancellation::default(),
            )
            .unwrap_err();
        assert!(matches!(
            py_err,
            CodeFormatError::FormatterUnavailable { .. }
        ));
    }
}
