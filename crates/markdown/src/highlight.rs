//! Language registry for syntax highlighting.
//!
//! [`LanguageRegistry`] resolves a fenced-code-block language identifier (e.g.
//! `rust`, `ts`, `py`) to a `syntect` `SyntaxReference`. It is the sole part of
//! this module that Markion consumes (`src/highlight.rs` drives syntect's
//! scope-level parsing itself, so colours stay under Markion's theme system).
//!
//! The earlier Typune `SyntaxHighlighter` / `AsyncHighlighter` / `HighlightCache`
//! facade was removed in `prune-unused-typune-inventory`: it returned
//! hard-coded theme colours that conflicted with Markion's 14 themes, and its
//! async path solved a bottleneck that measurements (see
//! `unify-pulldown-cmark/design.md`) showed does not exist — the real
//! large-document cost is in the parsing layer, not highlighting.

use std::collections::HashMap;

use syntect::parsing::SyntaxSet;

// ---------------------------------------------------------------------------
// Language registry
// ---------------------------------------------------------------------------

/// A registry mapping language identifiers and aliases to `syntect` syntaxes.
pub struct LanguageRegistry {
    syntax_set: SyntaxSet,
    /// alias → canonical syntax name stored in the SyntaxSet.
    aliases: HashMap<String, String>,
}

impl LanguageRegistry {
    /// Build a registry from syntect's default (bundled) syntax definitions.
    pub fn new() -> Self {
        Self::with_syntax_set(SyntaxSet::load_defaults_newlines())
    }

    /// Build a registry from a caller-supplied [`SyntaxSet`] (e.g. an
    /// extended set such as two-face's). The alias table is conditional on
    /// which canonical syntaxes actually exist, so richer sets automatically
    /// activate more aliases.
    pub fn with_syntax_set(syntax_set: SyntaxSet) -> Self {
        let mut aliases: HashMap<String, String> = HashMap::new();

        // Register every bundled syntax under its own name and its defined
        // file extensions so that common identifiers like "rs", "js", "py"
        // all resolve to the right syntax.
        for syntax in syntax_set.syntaxes() {
            let name_lower = syntax.name.to_lowercase();
            aliases.insert(name_lower.clone(), syntax.name.clone());

            for ext in &syntax.file_extensions {
                let ext_lower = ext.to_lowercase();
                aliases
                    .entry(ext_lower)
                    .or_insert_with(|| syntax.name.clone());
            }
        }

        // Additional hand-crafted aliases that users are likely to type in
        // fenced code blocks.
        let extra: &[(&str, &str)] = &[
            // Rust
            ("rust", "Rust"),
            // JavaScript / TypeScript
            ("javascript", "JavaScript"),
            ("js", "JavaScript"),
            ("typescript", "TypeScript"),
            ("ts", "TypeScript"),
            ("tsx", "TypeScript"),
            ("jsx", "JavaScript"),
            // Python
            ("python", "Python"),
            ("py", "Python"),
            ("python3", "Python"),
            // Ruby
            ("ruby", "Ruby"),
            ("rb", "Ruby"),
            // Go
            ("go", "Go"),
            ("golang", "Go"),
            // C / C++
            ("c", "C"),
            ("cpp", "C++"),
            ("c++", "C++"),
            ("cxx", "C++"),
            ("cc", "C++"),
            // C#
            ("csharp", "C#"),
            ("cs", "C#"),
            ("c#", "C#"),
            // Java
            ("java", "Java"),
            // Kotlin
            ("kotlin", "Kotlin"),
            ("kt", "Kotlin"),
            // Swift
            ("swift", "Swift"),
            // Scala
            ("scala", "Scala"),
            // PHP
            ("php", "PHP"),
            // HTML / CSS / SCSS
            ("html", "HTML"),
            ("htm", "HTML"),
            ("css", "CSS"),
            ("scss", "SCSS"),
            ("sass", "Sass"),
            // Shell
            ("bash", "Bash"),
            ("sh", "Bash"),
            ("zsh", "Bash"),
            ("shell", "Shell Script (Bash)"),
            // SQL
            ("sql", "SQL"),
            // YAML / TOML / JSON / XML
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("toml", "TOML"),
            ("json", "JSON"),
            ("xml", "XML"),
            // Markdown
            ("markdown", "Markdown"),
            ("md", "Markdown"),
            // Makefile
            ("makefile", "Makefile"),
            ("make", "Makefile"),
            // Lua
            ("lua", "Lua"),
            // Perl
            ("perl", "Perl"),
            ("pl", "Perl"),
            // R
            ("r", "R"),
            // Haskell
            ("haskell", "Haskell"),
            ("hs", "Haskell"),
            // Elixir
            ("elixir", "Elixir"),
            ("ex", "Elixir"),
            ("exs", "Elixir"),
            // Erlang
            ("erlang", "Erlang"),
            // Clojure
            ("clojure", "Clojure"),
            ("clj", "Clojure"),
            // Dart
            ("dart", "Dart"),
            // OCaml
            ("ocaml", "OCaml"),
            ("ml", "OCaml"),
            // F#
            ("fsharp", "F#"),
            ("fs", "F#"),
            ("f#", "F#"),
            // Groovy
            ("groovy", "Groovy"),
            // PowerShell
            ("powershell", "PowerShell"),
            ("ps1", "PowerShell"),
            ("posh", "PowerShell"),
            // Dockerfile
            ("dockerfile", "Dockerfile"),
            ("docker", "Dockerfile"),
            // Diff
            ("diff", "Diff"),
            ("patch", "Diff"),
            // INI / Properties
            ("ini", "INI"),
            ("properties", "Java Properties"),
            // Batch (Windows)
            ("bat", "Batch File"),
            ("batch", "Batch File"),
            ("cmd", "Batch File"),
            // Assembly
            ("asm", "ASM (x86_64)"),
            ("assembly", "ASM (x86_64)"),
            ("nasm", "ASM (x86_64)"),
            // TeX / LaTeX
            ("tex", "LaTeX"),
            ("latex", "LaTeX"),
            // Plain text fallback
            ("text", "Plain Text"),
            ("txt", "Plain Text"),
            ("plain", "Plain Text"),
        ];

        for (alias, canonical) in extra {
            // Only insert if the canonical name actually exists in the set.
            if syntax_set.find_syntax_by_name(canonical).is_some() {
                aliases
                    .entry(alias.to_lowercase())
                    .or_insert_with(|| canonical.to_string());
            }
        }

        Self {
            syntax_set,
            aliases,
        }
    }

    /// Look up a syntax by language identifier.  Returns `None` when the
    /// language is unknown (caller should fall back to plain text).
    pub fn find(&self, language: &str) -> Option<&syntect::parsing::SyntaxReference> {
        let key = language.trim().to_lowercase();

        // 1. Try the alias table.
        if let Some(canonical) = self.aliases.get(&key) {
            if let Some(syn) = self.syntax_set.find_syntax_by_name(canonical) {
                return Some(syn);
            }
        }

        // 2. Try syntect's built-in name lookup.
        if let Some(syn) = self.syntax_set.find_syntax_by_name(language) {
            return Some(syn);
        }

        // 3. Try syntect's extension lookup.
        if let Some(syn) = self.syntax_set.find_syntax_by_extension(&key) {
            return Some(syn);
        }

        None
    }

    /// Returns the underlying [`SyntaxSet`], for callers that drive syntect's
    /// scope-level parsing (`ParseState::parse_line`) directly rather than
    /// going through a themed highlighter.
    pub fn syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }

    /// Returns a sorted list of all registered language identifiers.
    pub fn language_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .syntax_set
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        names.sort();
        names
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_finds_common_languages() {
        let reg = LanguageRegistry::new();
        assert!(reg.find("rust").is_some());
        assert!(reg.find("py").is_some());
        assert!(reg.find("JavaScript").is_some());
    }

    #[test]
    fn registry_returns_none_for_unknown_language() {
        let reg = LanguageRegistry::new();
        assert!(reg.find("totally-not-a-language-xyz").is_none());
    }

    #[test]
    fn registry_exposes_syntax_set() {
        let reg = LanguageRegistry::new();
        let _set: &SyntaxSet = reg.syntax_set();
        // syntax_set() lets callers run scope-level parsing themselves.
        assert!(_set.find_syntax_by_name("Rust").is_some());
    }

    #[test]
    fn registry_language_names_are_sorted() {
        let reg = LanguageRegistry::new();
        let names = reg.language_names();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
