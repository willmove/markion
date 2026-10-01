## ADDED Requirements

### Requirement: Underlined inline HTML in Visual Edit

Visual Edit SHALL render well-paired inline `<u>` and `<ins>` elements with underline and hidden tag syntax, including elements inside Markdown link labels, headings, lists, blockquotes, and table cells. Tag names SHALL be case-insensitive and existing supported ignorable attributes SHALL remain accepted. Underline SHALL compose with surrounding Markdown and HTML styles and SHALL end at the matching closing element. Link labels SHALL retain their exact destination and navigation metadata.

#### Scenario: Imported table of contents uses underlined links
- **WHEN** an unfocused paragraph contains `[<u>第一章 项目概述</u>    4](#_toc232450996)`
- **THEN** the label displays `第一章 项目概述    4` with its title underlined and its destination preserved
- **AND** neither the HTML tags nor Markdown link delimiters appear as ordinary label text

#### Scenario: Underline composes with other styles
- **WHEN** supported inline `u` or `ins` elements contain nested Markdown or HTML formatting
- **THEN** their content retains both underline and the nested styles
- **AND** neighboring text outside the element does not inherit its underline

#### Scenario: Focusing an underlined link reveals exact source
- **WHEN** a caret or selection endpoint enters an underlined link label
- **THEN** the complete safe containing link group is revealed exactly once
- **AND** all caret mappings remain UTF-8 safe and document text, version, and derived caches remain unchanged

### Requirement: Literal markup and malformed HTML remain source faithful

Visual Edit SHALL distinguish Markdown-escaped punctuation and entity-decoded text from actual inline HTML events. Escaped markup SHALL retain literal meaning, including at the start of a Markdown link or formatting group. HTML pair validation SHALL use matching tag names, not merely equivalent style effects; invalid pairs and unsupported attributes SHALL retain source-backed fallback without dropping source bytes.

#### Scenario: Escaped TOC title and tag text stay literal
- **WHEN** source contains `\*\*目  录\*\*` or `[\<u>第一章 项目概述\</u>    4](#_toc232450996)`
- **THEN** the visible stars and angle-bracket tag text remain literal and only escape backslashes and link delimiters are hidden
- **AND** the literal tag text does not acquire semantic underline formatting

#### Scenario: Style aliases are not matching HTML tags
- **WHEN** source contains a mismatched pair such as `<u>text</ins>` or `<em>text</i>`
- **THEN** Visual Edit preserves the authored malformed element through conservative source rendering
- **AND** it does not silently hide the mismatched tags as a valid pair
