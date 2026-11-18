## ADDED Requirements

### Requirement: Explicit link destinations SHALL take precedence over labels
Preview and built-in exports SHALL preserve the destination of an authored link even when its visible label resembles an automatically detected URL.

#### Scenario: An explicit link has a URL-shaped label
- **WHEN** the link label is a URL different from its authored destination
- **THEN** the label remains visible and the link leads to the authored destination
- **AND** generated HTML contains a single anchor for the link
