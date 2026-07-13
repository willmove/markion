//! Syntax highlighter for code blocks.
//!
//! Two layers behind one interface: languages covered by the bundled syntect
//! grammar registry (absorbed from Typune, `crates/markdown`) are highlighted
//! by scope parsing, with scopes classified back into Markion's `HighlightKind`
//! token classes so every application theme keeps controlling the colors.
//! Languages the registry does not cover — and any syntect parse failure —
//! fall back to the hand-written token-class lexer below, so no advertised
//! language regresses to plain text.

use std::sync::OnceLock;

use syntect::parsing::{ParseState, Scope, ScopeStack};
use typune_markdown::highlight::LanguageRegistry;

use crate::model::{HighlightKind, HighlightedSpan};

/// Shared grammar registry. Loading the bundled grammars costs ~100ms, so it
/// is created once, lazily; `warm_highlighter` lets startup pay that cost on a
/// background thread instead of the first highlighted code block.
static REGISTRY: OnceLock<LanguageRegistry> = OnceLock::new();

fn registry() -> &'static LanguageRegistry {
    // two-face's extended set (bat's curated grammars) instead of syntect's
    // bundled defaults: adds TypeScript, TOML, Kotlin, Swift, Dockerfile,
    // PowerShell and other modern languages the defaults are missing.
    REGISTRY.get_or_init(|| LanguageRegistry::with_syntax_set(two_face::syntax::extra_newlines()))
}

/// Eagerly initializes the grammar registry. Called from a background thread
/// at app startup so the first code block never blocks the typing path.
pub fn warm_highlighter() {
    let start = std::time::Instant::now();
    let _ = registry();
    tracing::debug!(
        elapsed_ms = start.elapsed().as_millis() as u64,
        "highlight grammar registry ready"
    );
}

/// Classifies a syntect scope stack into Markion's token classes. The stack is
/// scanned innermost-first and the first scope with a mapping wins;
/// punctuation (e.g. string quotes) is transparent so a quoted literal stays
/// one `String` span, and `keyword.operator` deliberately stays `Plain` to
/// match the legacy lexer's look.
fn classify_scopes(scopes: &[Scope]) -> HighlightKind {
    for scope in scopes.iter().rev() {
        let name = scope.build_string();
        if name.starts_with("comment") {
            return HighlightKind::Comment;
        }
        if name.starts_with("string") {
            return HighlightKind::String;
        }
        if name.starts_with("constant.numeric") {
            return HighlightKind::Number;
        }
        if name.starts_with("constant.language") {
            return HighlightKind::Keyword;
        }
        if name.starts_with("keyword") && !name.starts_with("keyword.operator") {
            return HighlightKind::Keyword;
        }
        if name.starts_with("storage") {
            return HighlightKind::Keyword;
        }
        if name.starts_with("entity.name.type")
            || name.starts_with("entity.name.class")
            || name.starts_with("entity.other.inherited-class")
            || name.starts_with("support.type")
            || name.starts_with("support.class")
        {
            return HighlightKind::Type;
        }
    }
    HighlightKind::Plain
}

/// Highlights `code` via syntect scope parsing. Returns `None` when the
/// language is not covered by the grammar registry or parsing fails, so the
/// caller falls back to the legacy lexer. `ParseState` and `ScopeStack`
/// persist across lines, so multi-line strings and block comments highlight
/// correctly (the legacy lexer is line-local).
fn syntect_highlight(code: &str, language: &str) -> Option<Vec<Vec<HighlightedSpan>>> {
    let registry = registry();
    let syntax = registry.find(language)?;
    let syntax_set = registry.syntax_set();

    let mut parse_state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut lines = Vec::new();

    for line in code.lines() {
        // The registry's grammars are the newlines variant, which requires
        // the terminator to be present while parsing.
        let line_with_newline = format!("{line}\n");
        let ops = parse_state
            .parse_line(&line_with_newline, syntax_set)
            .ok()?;

        let mut spans: Vec<HighlightedSpan> = Vec::new();
        let mut cursor = 0usize;
        for (offset, op) in ops {
            let end = offset.min(line.len());
            if end > cursor {
                push_highlight_span(
                    &mut spans,
                    &line[cursor..end],
                    classify_scopes(stack.as_slice()),
                );
                cursor = end;
            }
            stack.apply(&op).ok()?;
        }
        if cursor < line.len() {
            push_highlight_span(
                &mut spans,
                &line[cursor..],
                classify_scopes(stack.as_slice()),
            );
        }

        if spans.is_empty() {
            spans.push(HighlightedSpan {
                text: String::new(),
                kind: HighlightKind::Plain,
            });
        }
        lines.push(spans);
    }

    if lines.is_empty() {
        lines.push(vec![HighlightedSpan {
            text: String::new(),
            kind: HighlightKind::Plain,
        }]);
    }
    Some(lines)
}

const SUPPORTED_HIGHLIGHT_LANGUAGES: &[&str] = &[
    "bash",
    "c",
    "clojure",
    "cpp",
    "csharp",
    "css",
    "dart",
    "diff",
    "dockerfile",
    "elixir",
    "elm",
    "erlang",
    "go",
    "graphql",
    "haskell",
    "html",
    "java",
    "javascript",
    "json",
    "jsx",
    "julia",
    "kotlin",
    "lua",
    "makefile",
    "markdown",
    "nix",
    "objective-c",
    "perl",
    "php",
    "powershell",
    "protobuf",
    "python",
    "r",
    "ruby",
    "rust",
    "scala",
    "scss",
    "shell",
    "sql",
    "swift",
    "toml",
    "tsx",
    "typescript",
    "vim",
    "vue",
    "xml",
    "yaml",
    "zig",
    "solidity",
    "wasm",
    "terraform",
    "ocaml",
    "fsharp",
];

pub fn highlight_code(code: &str, language: Option<&str>) -> Vec<Vec<HighlightedSpan>> {
    // Same first-token extraction as the legacy path (fence info strings can
    // carry extras, e.g. ```rust,ignore); the registry handles aliasing.
    let raw_language = language
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches('.');

    // Grammar-based path first. An unspecified language stays plain (the
    // legacy path below renders it as unstyled text), so only named languages
    // consult the registry.
    if !raw_language.is_empty() {
        if let Some(lines) = syntect_highlight(code, raw_language) {
            return lines;
        }
    }

    let normalized_language = normalized_highlight_language(language.unwrap_or_default());
    let lines = code.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return vec![vec![HighlightedSpan {
            text: String::new(),
            kind: HighlightKind::Plain,
        }]];
    }

    lines
        .into_iter()
        .map(|line| highlight_code_line(line, &normalized_language))
        .collect()
}

/// The advertised language list: the union of the legacy lexer identifiers
/// and every grammar name in the active registry (lowercased), sorted and
/// deduplicated — so the advertisement reflects real coverage. Built lazily
/// once; the strings are leaked deliberately (bounded, one-time).
pub fn supported_highlight_languages() -> &'static [&'static str] {
    static UNION: OnceLock<Vec<&'static str>> = OnceLock::new();
    UNION
        .get_or_init(|| {
            let mut names: Vec<String> = SUPPORTED_HIGHLIGHT_LANGUAGES
                .iter()
                .map(|name| name.to_string())
                .collect();
            names.extend(
                registry()
                    .language_names()
                    .into_iter()
                    .map(|name| name.to_lowercase()),
            );
            names.sort();
            names.dedup();
            names
                .into_iter()
                .map(|name| &*Box::leak(name.into_boxed_str()))
                .collect()
        })
        .as_slice()
}

fn normalized_highlight_language(language: &str) -> String {
    let language = language
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    match language.as_str() {
        "sh" | "zsh" | "fish" => "shell".to_string(),
        "ps1" | "pwsh" => "powershell".to_string(),
        "js" | "mjs" | "cjs" => "javascript".to_string(),
        "ts" => "typescript".to_string(),
        "py" | "py3" => "python".to_string(),
        "rb" => "ruby".to_string(),
        "rs" => "rust".to_string(),
        "cs" => "csharp".to_string(),
        "c++" | "cc" | "cxx" | "hpp" => "cpp".to_string(),
        "objc" | "objectivec" => "objective-c".to_string(),
        "kt" | "kts" => "kotlin".to_string(),
        "ex" | "exs" => "elixir".to_string(),
        "erl" => "erlang".to_string(),
        "hs" => "haskell".to_string(),
        "md" | "mdx" => "markdown".to_string(),
        "yml" => "yaml".to_string(),
        "tf" | "hcl" => "terraform".to_string(),
        "docker" => "dockerfile".to_string(),
        other => other.to_string(),
    }
}

fn highlight_code_line(line: &str, language: &str) -> Vec<HighlightedSpan> {
    let mut spans = Vec::new();
    let mut index = 0usize;

    while index < line.len() {
        let rest = &line[index..];
        if is_line_comment_start(rest, language) {
            push_highlight_span(&mut spans, rest, HighlightKind::Comment);
            break;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };

        if ch.is_whitespace() {
            let end = consume_while(line, index, char::is_whitespace);
            push_highlight_span(&mut spans, &line[index..end], HighlightKind::Plain);
            index = end;
        } else if is_string_delimiter(ch, language) {
            let end = consume_string(line, index, ch);
            push_highlight_span(&mut spans, &line[index..end], HighlightKind::String);
            index = end;
        } else if ch.is_ascii_digit() {
            let end = consume_while(line, index, |ch| {
                ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.')
            });
            push_highlight_span(&mut spans, &line[index..end], HighlightKind::Number);
            index = end;
        } else if is_identifier_start(ch) {
            let end = consume_while(line, index, is_identifier_continue);
            let word = &line[index..end];
            let kind = if is_keyword(word, language) {
                HighlightKind::Keyword
            } else if is_type_word(word, language) {
                HighlightKind::Type
            } else {
                HighlightKind::Plain
            };
            push_highlight_span(&mut spans, word, kind);
            index = end;
        } else {
            let end = index + ch.len_utf8();
            push_highlight_span(&mut spans, &line[index..end], HighlightKind::Plain);
            index = end;
        }
    }

    if spans.is_empty() {
        spans.push(HighlightedSpan {
            text: String::new(),
            kind: HighlightKind::Plain,
        });
    }

    spans
}

fn push_highlight_span(spans: &mut Vec<HighlightedSpan>, text: &str, kind: HighlightKind) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = spans.last_mut() {
        if last.kind == kind {
            last.text.push_str(text);
            return;
        }
    }

    spans.push(HighlightedSpan {
        text: text.to_string(),
        kind,
    });
}

fn is_line_comment_start(rest: &str, language: &str) -> bool {
    match language {
        "python" | "shell" | "bash" | "powershell" | "toml" | "yaml" | "ruby" | "perl" | "r"
        | "makefile" | "dockerfile" | "nix" => rest.starts_with('#'),
        "html" | "xml" | "markdown" | "vue" => rest.starts_with("<!--"),
        "sql" => rest.starts_with("--"),
        "lua" => rest.starts_with("--"),
        "vim" => rest.starts_with('"'),
        _ => rest.starts_with("//"),
    }
}

fn is_string_delimiter(ch: char, language: &str) -> bool {
    matches!(ch, '"' | '\'')
        || (matches!(language, "javascript" | "typescript" | "jsx" | "tsx") && ch == '`')
}

fn consume_string(line: &str, start: usize, delimiter: char) -> usize {
    let mut escaped = false;
    let mut seen_open = false;
    for (relative, ch) in line[start..].char_indices() {
        if !seen_open {
            seen_open = true;
            continue;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == delimiter {
            return start + relative + ch.len_utf8();
        }
    }
    line.len()
}

fn consume_while(line: &str, start: usize, predicate: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    for ch in line[start..].chars() {
        if !predicate(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_keyword(word: &str, language: &str) -> bool {
    let common = matches!(
        word,
        "if" | "else"
            | "for"
            | "while"
            | "loop"
            | "match"
            | "switch"
            | "case"
            | "break"
            | "continue"
            | "return"
            | "true"
            | "false"
            | "null"
            | "None"
            | "nil"
            | "class"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "interface"
            | "function"
            | "fn"
            | "def"
            | "let"
            | "const"
            | "var"
            | "mut"
            | "pub"
            | "use"
            | "import"
            | "from"
            | "async"
            | "await"
            | "try"
            | "catch"
            | "throw"
            | "new"
    );
    if common {
        return true;
    }

    match language {
        "rust" => matches!(
            word,
            "crate" | "mod" | "where" | "Self" | "self" | "super" | "unsafe" | "move"
        ),
        "javascript" | "typescript" | "jsx" | "tsx" => matches!(
            word,
            "export"
                | "default"
                | "extends"
                | "typeof"
                | "instanceof"
                | "yield"
                | "this"
                | "super"
                | "implements"
                | "readonly"
                | "namespace"
                | "declare"
        ),
        "python" => matches!(
            word,
            "elif"
                | "lambda"
                | "with"
                | "as"
                | "pass"
                | "raise"
                | "yield"
                | "in"
                | "is"
                | "and"
                | "or"
                | "not"
                | "global"
                | "nonlocal"
        ),
        "go" => matches!(
            word,
            "package" | "defer" | "go" | "chan" | "select" | "range"
        ),
        "java" | "kotlin" | "scala" => matches!(
            word,
            "package"
                | "private"
                | "protected"
                | "public"
                | "static"
                | "final"
                | "override"
                | "extends"
                | "implements"
                | "throws"
        ),
        "c" | "cpp" | "csharp" | "objective-c" => matches!(
            word,
            "include"
                | "define"
                | "namespace"
                | "using"
                | "private"
                | "protected"
                | "public"
                | "virtual"
                | "override"
                | "static"
        ),
        "php" => matches!(
            word,
            "namespace" | "use" | "echo" | "require" | "include" | "extends" | "implements"
        ),
        "ruby" => matches!(
            word,
            "module" | "begin" | "rescue" | "ensure" | "elsif" | "unless" | "do" | "end"
        ),
        "swift" => matches!(
            word,
            "func" | "actor" | "extension" | "protocol" | "guard" | "defer" | "where"
        ),
        "sql" => matches!(
            word.to_ascii_uppercase().as_str(),
            "SELECT"
                | "FROM"
                | "WHERE"
                | "JOIN"
                | "LEFT"
                | "RIGHT"
                | "INNER"
                | "OUTER"
                | "GROUP"
                | "ORDER"
                | "INSERT"
                | "UPDATE"
                | "DELETE"
                | "CREATE"
                | "ALTER"
                | "DROP"
        ),
        "html" | "xml" | "css" | "scss" | "vue" => matches!(
            word,
            "style" | "script" | "template" | "media" | "import" | "keyframes" | "font-face"
        ),
        "elixir" | "erlang" | "haskell" | "ocaml" | "fsharp" => matches!(
            word,
            "module" | "where" | "type" | "data" | "case" | "of" | "with" | "do" | "end"
        ),
        "lua" => matches!(
            word,
            "local" | "then" | "elseif" | "repeat" | "until" | "end"
        ),
        "dart" => matches!(
            word,
            "library" | "part" | "mixin" | "extends" | "implements" | "with" | "factory"
        ),
        _ => false,
    }
}

fn is_type_word(word: &str, language: &str) -> bool {
    match language {
        "rust" => matches!(
            word,
            "String"
                | "str"
                | "usize"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "bool"
                | "Option"
                | "Result"
                | "Vec"
        ),
        "typescript" | "tsx" => {
            matches!(
                word,
                "string" | "number" | "boolean" | "unknown" | "void" | "never"
            )
        }
        "python" => matches!(
            word,
            "str" | "int" | "float" | "bool" | "list" | "dict" | "tuple"
        ),
        "go" => matches!(
            word,
            "string" | "int" | "int64" | "float64" | "bool" | "error" | "byte" | "rune"
        ),
        "java" | "kotlin" | "scala" | "c" | "cpp" | "csharp" | "swift" | "dart" => matches!(
            word,
            "String"
                | "Int"
                | "Integer"
                | "Float"
                | "Double"
                | "Boolean"
                | "Bool"
                | "void"
                | "Void"
                | "char"
                | "long"
                | "short"
                | "byte"
        ),
        _ => word.chars().next().is_some_and(char::is_uppercase),
    }
}
