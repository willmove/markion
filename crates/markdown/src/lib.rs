pub mod ast;
pub mod emoji;
pub mod error;
pub mod extended_inline;
pub mod highlight;
pub mod incremental;
pub mod math;
pub mod parser;
pub mod renderer;

pub use ast::{
    Alignment, Block, Document, Inline, ListItem, NodeId, SharedBlocks, TableCell, YamlFrontMatter,
};
pub use emoji::{is_valid_shortcode, shortcode_to_unicode};
pub use error::{MarkdownError, MarkdownResult};
pub use extended_inline::parse_extended_inlines;
pub use highlight::LanguageRegistry;
pub use incremental::{AsyncParseHandle, AsyncParseResult, IncrementalParser, TextChange};
pub use math::{
    MathError, MathErrorKind, MathRenderOptions, MathRenderer, MathStyle, RENDERER_VERSION,
    RenderedMath, Size, is_self_contained_svg, serialize_math_html,
};
pub use parser::{Parser, ParserOptions};
pub use renderer::render_to_markdown;
