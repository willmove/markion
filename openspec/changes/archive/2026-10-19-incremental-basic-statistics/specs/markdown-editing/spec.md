## ADDED Requirements

### Requirement: Incremental basic statistics for range edits
The document SHALL update populated character, word, text-line and logical-line counters from the replaced span and bounded adjacent context after a validated range edit. Results SHALL equal a full recomputation, including Unicode whitespace and trailing newline semantics. Cold caches and whole-document replacements SHALL retain the existing lazy invalidation behavior.

#### Scenario: Japanese composition and local edits
- **WHEN** a validated range edit inserts, replaces or removes Japanese text, Unicode whitespace, emoji, CR or LF
- **THEN** cached counters equal full recomputation without changing the canonical text
- **AND** Visual Edit, IME and mutation validation continue through their existing code paths
