## ADDED Requirements

### Requirement: Merge-blocker regressions require semantic boundary evidence
Correctness fixes for cache identity, generated structured text, or parser event reconstruction SHALL include fixtures that exercise the boundary responsible for the defect. Tests MUST compare semantic outcomes rather than only output shape: cache tests compare identities and decoded results for deliberately adversarial equal-length inputs; YAML tests parse generated text and compare typed values; extended-inline tests exercise the default parser and conflicting escape/GFM syntax. Synthetic invalid-state tests SHALL be described as invariant containment and MUST NOT be cited as proof that the same state is reachable through ordinary user input.

#### Scenario: Cache collision regression is exercised deterministically
- **WHEN** a regression test builds two equal-length valid image sources that differ only outside the former sampled regions
- **THEN** it proves their keys and decoded render results are distinct
- **AND** it does not rely on probabilistic random inputs or elapsed-time thresholds

#### Scenario: Generated YAML is validated semantically
- **WHEN** tests cover front-matter rendering or export title overrides
- **THEN** they parse the complete generated front-matter block with the production YAML type
- **AND** they compare the reparsed typed values with the inputs rather than accepting substring assertions alone

#### Scenario: Parser boundary fix protects competing syntax
- **WHEN** tests prove an extended-inline construct across adjacent text events
- **THEN** the same suite also covers the default parser, escaped delimiters, GFM strikethrough, Unicode text, and boundaries adjacent to non-text inline events

#### Scenario: Synthetic invalid state is labeled accurately
- **WHEN** a test injects a malformed visual block or invalid caret/selection range that ordinary parsing does not produce
- **THEN** the test and change report identify it as defense-in-depth invariant evidence
- **AND** they do not claim an ordinary user-triggered reproduction without a separate end-to-end fixture
