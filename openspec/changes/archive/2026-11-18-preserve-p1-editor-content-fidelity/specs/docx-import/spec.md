## ADDED Requirements

### Requirement: Word imports SHALL preserve the order of text and embedded resources
Supported embedded drawings SHALL appear in authored order among text and other drawings, including multiple drawings in one run. Rectangular table cells SHALL preserve supported links, images, styling and separation between paragraphs. The importer SHALL report content loss when an unsupported cell would otherwise omit recoverable resources or link targets.

#### Scenario: Several drawings share a text run
- **WHEN** a run interleaves text with two embedded drawings
- **THEN** the converted content includes both drawings among the text in source order

#### Scenario: A rectangular cell contains rich content
- **WHEN** a table cell contains paragraphs, styled text, a hyperlink and an embedded image
- **THEN** the rendered table preserves those contents and the paragraph separation
