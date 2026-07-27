## ADDED Requirements

### Requirement: Visual Edit preserves ordered blockquote child flow
The derived Markdown model SHALL preserve supported paragraphs and list items nested in a blockquote as one child flow in authored order. Split Preview and Read SHALL render that flow in the same order. Visual Edit SHALL project each supported quoted leaf exactly once through a disjoint, contiguous, UTF-8-safe canonical source range carrying its blockquote context, and SHALL render ordered, unordered, nested, and task-list items inside the quote presentation rather than as top-level rows or `Unsupported` source islands. Quote and inner list/task prefixes SHALL retain exact source mappings for progressive reveal and structural editing. Authored straight quotes or other smart-punctuation candidates in otherwise supported quoted prose SHALL remain rendered editable text and SHALL NOT alone force the quoted flow into a complete source island. All derived quote flow, prefix, and presentation metadata SHALL follow the existing per-document-version cache and incremental/full-equivalence rules.

#### Scenario: Mixed quoted prose and ordered list retains authored order
- **WHEN** a blockquote contains an introductory paragraph, an ordered list, and a trailing paragraph
- **THEN** Split Preview, Read, and Visual Edit present the introduction, list items, and trailing paragraph in authored order
- **AND** every list item appears exactly once inside the blockquote presentation

#### Scenario: Quoted list variants remain inside the quote
- **WHEN** a blockquote contains unordered, task-list, non-1-start ordered, or nested list items
- **THEN** Visual Edit renders their bullet, checked state, ordered number, and relative indentation inside the quote styling
- **AND** none of the supported rows is marked `Unsupported` solely because it is nested in the blockquote

#### Scenario: Quoted leaf ranges are exact and non-overlapping
- **WHEN** the source-mapped visual model is derived for a blockquote containing multiple paragraphs, blank quoted separators, and list items
- **THEN** each canonical source byte in the supported quote belongs to exactly one ordered quoted leaf or quote-context whitespace row
- **AND** all owned ranges and prefix ranges are UTF-8 boundaries, monotonically ordered, and non-overlapping

#### Scenario: Composite quote and list prefixes remain editable
- **WHEN** the caret enters or structurally edits a quoted list prefix such as `> 1. ` or `> - [x] `
- **THEN** Visual Edit reveals only the exact required quote or inner-list marker layer
- **AND** Enter continues the combined quote/list prefix while Backspace demotes the innermost list structure before removing the quote structure
- **AND** each action is one canonical source mutation with exact selection and undo/redo restoration

#### Scenario: Smart-punctuation candidates do not create a source island
- **WHEN** supported quoted prose or a quoted list item contains authored ASCII double quotes, single quotes, or dash sequences recognized by smart punctuation
- **THEN** Visual Edit keeps the authored punctuation as source-exact rendered editable text
- **AND** the enclosing quoted flow does not become a complete source island solely because Preview would substitute different punctuation glyphs

#### Scenario: Incremental quoted edit equals fresh full derivation
- **WHEN** the user inserts, deletes, or replaces UTF-8 text or a structural marker inside one quoted paragraph or list item
- **THEN** incremental derivation produces the same ordered child variants, quote context, prefixes, content, and byte ranges as a fresh full-document derivation
- **AND** unaffected quoted siblings retain stable visual identities when their source lineage is proven unchanged

#### Scenario: Quoted flow stays version-cached
- **WHEN** the user moves the caret, changes selection, scrolls, or repaints a mixed blockquote/list flow without mutating document text
- **THEN** the document version and cached quoted visual model remain unchanged
- **AND** GPUI rendering does not reparse the blockquote or rebuild derived Markdown state
