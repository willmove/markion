## ADDED Requirements

### Requirement: HTML cell inline formatting parity

HTML table cells SHALL preserve the same supported inline HTML styles, colors, links, scripts and explicit breaks as ordinary HTML content. Cell boundaries SHALL prevent unclosed styling or link state from leaking into the next cell, while rowspan/colspan and source-preserving HTML editing remain intact.

#### Scenario: Rich HTML cell retains its styles
- **WHEN** a cell contains colored, underlined, highlighted, scripted or linked text and repeated br elements
- **THEN** all rendered modes preserve each style and break inside that cell

#### Scenario: Adjacent cells do not inherit malformed inline state
- **WHEN** one cell ends with an unclosed inline style or link
- **THEN** the following cell starts with its own default inline state
