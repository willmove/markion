## ADDED Requirements

### Requirement: Nested container derivation preserves source ownership without crashing
The Markdown parser SHALL preserve the destination container selected when each list item begins, even when a blockquote or nested list begins before that item is flushed. A list item outside a blockquote MUST remain a document-level block, and a list item inside a blockquote MUST remain a child of that quote. Preview and Visual Edit derivation SHALL expose the resulting blocks in authored order through non-reversed, in-bounds, UTF-8-safe source ranges. A blockquote emitted separately from its containing list item SHALL begin at or after the containing item's derived range end. If any malformed derived leaf nevertheless reaches Visual Edit, the editor SHALL preserve its canonical source through a conservative source-backed fallback and MUST NOT panic or terminate the application.

#### Scenario: List contains a blockquote that contains a list
- **WHEN** a document-level list item contains a blockquote whose body contains another list item
- **THEN** the outer item remains a document-level preview block and the inner item remains a child of the blockquote
- **AND** the outer item's derived range ends no later than the blockquote range begins
- **AND** Visual Edit projects the document without reversed, overlapping ownership or process termination

#### Scenario: Nested topology preserves UTF-8 and CRLF boundaries
- **WHEN** the same list-blockquote-list topology contains CJK text or emoji and uses LF or CRLF line endings
- **THEN** every preview and visual source-range endpoint is a valid UTF-8 boundary within the canonical source
- **AND** the complete source remains covered in authored order

#### Scenario: Ordinary list and quote nesting remains unchanged
- **WHEN** a document contains only nested lists, only a blockquote containing a list, or a list containing a paragraph-only blockquote
- **THEN** its items retain their correct document or quote ownership and existing rendered semantics
- **AND** no new unsupported source fallback is introduced solely by those supported topologies

#### Scenario: Invalid derived range falls back safely
- **WHEN** an internal malformed preview leaf presents a reversed, out-of-bounds, or non-UTF-8-boundary source range to Visual Edit derivation
- **THEN** Visual Edit does not index the canonical text with that range and does not panic
- **AND** the affected authored bytes remain available through conservative source-backed coverage
