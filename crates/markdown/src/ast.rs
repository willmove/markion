//! Abstract Syntax Tree (AST) types for Markdown documents.
//!
//! Supports CommonMark block-level elements, GFM extensions, and inline elements.
//!
//! ## Performance: Arc-based sharing
//!
//! The AST uses `Arc` to enable cheap cloning of immutable sub-trees. When the
//! incremental parser reuses unchanged regions, their blocks are shared via
//! `Arc` rather than deep-copied, significantly reducing allocation pressure
//! for large documents with localised edits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// NodeId
// ---------------------------------------------------------------------------

/// Unique identifier for an AST node. Used for incremental updates and navigation.
pub type NodeId = usize;

/// A shared, reference-counted list of blocks. Used by the incremental parser
/// to share unchanged regions between document versions without deep copying.
pub type SharedBlocks = Arc<Vec<Block>>;

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

/// The top-level document, containing a list of block-level elements and
/// optional YAML front matter metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// The ordered list of top-level block elements in the document.
    pub blocks: Vec<Block>,
    /// Parsed YAML front matter, if present.
    pub metadata: Option<YamlFrontMatter>,
    /// Document version counter (incremented on each edit).
    pub version: u64,
    /// Map of footnote labels to their definition block IDs.
    pub footnote_map: HashMap<String, NodeId>,
    /// Arc-shared block regions from the incremental parser. These enable
    /// cheap sharing of unchanged sub-trees between document versions.
    /// Each entry corresponds to one parsed region; concatenating all inner
    /// vecs produces `blocks` (after renumbering).
    pub shared_regions: Vec<SharedBlocks>,
}

impl Document {
    /// Creates a new document from a list of blocks.
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            metadata: None,
            version: 0,
            footnote_map: HashMap::new(),
            shared_regions: Vec::new(),
        }
    }

    /// Creates a document with metadata.
    pub fn with_metadata(blocks: Vec<Block>, metadata: YamlFrontMatter) -> Self {
        Self {
            blocks,
            metadata: Some(metadata),
            version: 0,
            footnote_map: HashMap::new(),
            shared_regions: Vec::new(),
        }
    }

    /// Creates a document with pre-shared region blocks (used by incremental parser).
    pub fn with_shared_regions(blocks: Vec<Block>, shared_regions: Vec<SharedBlocks>) -> Self {
        Self {
            blocks,
            metadata: None,
            version: 0,
            footnote_map: HashMap::new(),
            shared_regions,
        }
    }
}

// ---------------------------------------------------------------------------
// Block
// ---------------------------------------------------------------------------

/// A block-level Markdown element.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// A heading (`#`, `##`, …, `######`). `level` is 1–6.
    Heading {
        level: u8,
        content: Vec<Inline>,
        id: NodeId,
    },

    /// A paragraph of inline content.
    Paragraph { content: Vec<Inline>, id: NodeId },

    /// A fenced or indented code block, with optional language identifier.
    CodeBlock {
        lang: Option<String>,
        code: String,
        id: NodeId,
    },

    /// A LaTeX math block (`$$…$$`).
    MathBlock { latex: String, id: NodeId },

    /// A GFM table.
    Table {
        headers: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
        alignment: Vec<Alignment>,
        id: NodeId,
    },

    /// An ordered or unordered list.
    List {
        items: Vec<ListItem>,
        ordered: bool,
        /// Starting number for ordered lists (e.g. `1`, `2`, …).
        start: Option<u32>,
        id: NodeId,
    },

    /// A block quote (`> …`).
    BlockQuote { content: Vec<Block>, id: NodeId },

    /// A horizontal rule (`---`, `***`, `___`).
    HorizontalRule { id: NodeId },

    /// A footnote definition (`[^id]: text`).
    FootnoteDefinition {
        label: String,
        content: Vec<Block>,
        id: NodeId,
    },

    /// An HTML block (block-level HTML content).
    HtmlBlock { content: String, id: NodeId },
}

impl Block {
    /// Returns the `NodeId` of this block.
    pub fn node_id(&self) -> NodeId {
        match self {
            Block::Heading { id, .. } => *id,
            Block::Paragraph { id, .. } => *id,
            Block::CodeBlock { id, .. } => *id,
            Block::MathBlock { id, .. } => *id,
            Block::Table { id, .. } => *id,
            Block::List { id, .. } => *id,
            Block::BlockQuote { id, .. } => *id,
            Block::HorizontalRule { id } => *id,
            Block::FootnoteDefinition { id, .. } => *id,
            Block::HtmlBlock { id, .. } => *id,
        }
    }
}

// ---------------------------------------------------------------------------
// Inline
// ---------------------------------------------------------------------------

/// An inline Markdown element.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    /// Plain text content.
    Text(String),

    /// Bold / strong emphasis (`**text**` or `__text__`).
    Strong(Vec<Inline>),

    /// Italic / emphasis (`*text*` or `_text_`).
    Emphasis(Vec<Inline>),

    /// GFM strikethrough (`~~text~~`).
    Strikethrough(Vec<Inline>),

    /// Inline code (`` `code` ``).
    Code(String),

    /// Inline math (`$formula$`).
    InlineMath(String),

    /// Superscript (`^text^`).
    Superscript(Vec<Inline>),

    /// Subscript (`~text~`).
    Subscript(Vec<Inline>),

    /// Highlight / mark (`==text==`).
    Highlight(Vec<Inline>),

    /// Emoji from shortcode (`:smile:` → 😊).
    Emoji { shortcode: String, unicode: String },

    /// A hyperlink.
    Link {
        text: Vec<Inline>,
        url: String,
        title: Option<String>,
    },

    /// An embedded image.
    Image {
        alt: String,
        url: String,
        title: Option<String>,
    },

    /// A hard line break (`  \n` or `\\\n`).
    LineBreak,

    /// A footnote reference (`[^id]`).
    FootnoteReference(String),

    /// Inline HTML content.
    HtmlInline(String),
}

// ---------------------------------------------------------------------------
// Table helpers
// ---------------------------------------------------------------------------

/// A single table cell, containing inline content.
#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub content: Vec<Inline>,
}

impl TableCell {
    /// Creates a new cell with the given inline content.
    pub fn new(content: Vec<Inline>) -> Self {
        Self { content }
    }

    /// Creates a cell containing a single plain-text string.
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content: vec![Inline::Text(s.into())],
        }
    }
}

/// Column alignment in a GFM table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    /// No alignment specified (default / left).
    None,
    Left,
    Center,
    Right,
}

// ---------------------------------------------------------------------------
// ListItem
// ---------------------------------------------------------------------------

/// A single item in a list, which may contain inline content, block content,
/// a task-list checkbox, and/or nested sub-items.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    /// Inline content for simple list items.
    pub content: Vec<Inline>,
    /// Block content for complex list items (paragraphs, code blocks, etc.).
    pub blocks: Vec<Block>,
    /// `Some(true)` = checked task item, `Some(false)` = unchecked, `None` = not a task item.
    pub checked: Option<bool>,
    /// Nested list items (sub-lists).
    pub sub_items: Vec<ListItem>,
}

impl ListItem {
    /// Creates a simple list item with inline content.
    pub fn simple(content: Vec<Inline>) -> Self {
        Self {
            content,
            blocks: Vec::new(),
            checked: None,
            sub_items: Vec::new(),
        }
    }

    /// Creates a task list item.
    pub fn task(content: Vec<Inline>, checked: bool) -> Self {
        Self {
            content,
            blocks: Vec::new(),
            checked: Some(checked),
            sub_items: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// YamlFrontMatter
// ---------------------------------------------------------------------------

/// Parsed YAML front matter from the top of a Markdown document.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct YamlFrontMatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    /// Any additional key–value pairs not captured by the named fields.
    #[serde(flatten)]
    pub custom: HashMap<String, serde_yaml::Value>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_creation() {
        let doc = Document::new(vec![Block::Paragraph {
            content: vec![Inline::Text("Hello".into())],
            id: 0,
        }]);
        assert_eq!(doc.blocks.len(), 1);
        assert!(doc.metadata.is_none());
        assert_eq!(doc.version, 0);
    }

    #[test]
    fn block_node_id() {
        let heading = Block::Heading {
            level: 1,
            content: vec![],
            id: 42,
        };
        assert_eq!(heading.node_id(), 42);

        let hr = Block::HorizontalRule { id: 7 };
        assert_eq!(hr.node_id(), 7);
    }

    #[test]
    fn table_cell_text_helper() {
        let cell = TableCell::text("hello");
        assert_eq!(cell.content, vec![Inline::Text("hello".into())]);
    }

    #[test]
    fn list_item_task() {
        let item = ListItem::task(vec![Inline::Text("Buy milk".into())], true);
        assert_eq!(item.checked, Some(true));
    }

    #[test]
    fn yaml_front_matter_serde_roundtrip() {
        let fm = YamlFrontMatter {
            title: Some("My Doc".into()),
            author: Some("Alice".into()),
            date: Some("2024-01-01".into()),
            tags: vec!["rust".into(), "markdown".into()],
            custom: HashMap::new(),
        };

        let yaml = serde_yaml::to_string(&fm).unwrap();
        let parsed: YamlFrontMatter = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.title, fm.title);
        assert_eq!(parsed.tags, fm.tags);
    }
}
