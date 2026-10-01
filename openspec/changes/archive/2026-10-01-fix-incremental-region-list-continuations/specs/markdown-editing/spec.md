## ADDED Requirements

### Requirement: Incremental derivation preserves indented continuations
Incremental preview/visual block derivation (`SourceMappedCache::update` and region reuse) SHALL produce block structures identical to a full-document derivation for list items and block quotes whose continuation content is separated by blank lines and indented by the container's marker width (including 2–3 space indents and indented code fences). This equivalence SHALL hold in release builds, not only under the debug-assertions oracle.

#### Scenario: Ordered list item with indented continuation paragraph
- **WHEN** a document contains `1. item`, a blank line, a 3-space-indented continuation paragraph, a blank line, and `2. two`, and any single-character edit is applied
- **THEN** the derived blocks keep the continuation paragraph inside item 1 and item numbering intact, identical to a fresh full parse of the same text

#### Scenario: Quote nested in a list item with following continuation
- **WHEN** a list item contains an indented block quote and a further indented continuation paragraph separated by blank lines, and the document is edited
- **THEN** the quote and continuation remain attached to their list item in the derived blocks, identical to a fresh full parse

#### Scenario: Region boundaries never split a container
- **WHEN** `split_regions` processes text where a blank line is followed by a line starting with whitespace (container continuation or indented fence)
- **THEN** no region boundary is inserted at that line

#### Scenario: Debug builds detect incremental mismatches via fallback counter
- **WHEN** the incremental result would differ from full derivation in a debug build
- **THEN** the full-fallback counter increments, and regression tests assert it stays constant for the covered fixtures
