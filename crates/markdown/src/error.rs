use thiserror::Error;

/// Unified error type for the markdown module
#[derive(Error, Debug)]
pub enum MarkdownError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Syntax highlighting error: {0}")]
    SyntaxHighlightError(String),

    #[error("Math rendering error: {0}")]
    MathRenderError(String),

    #[error("Invalid markdown structure: {0}")]
    InvalidStructure(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unknown markdown error: {0}")]
    Other(String),
}

/// Result type alias for markdown operations
pub type MarkdownResult<T> = Result<T, MarkdownError>;
