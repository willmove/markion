## ADDED Requirements

### Requirement: GFM alerts SHALL accept a custom title after the marker
A blockquote whose first line is a GFM alert marker (`[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, or `[!CAUTION]`, case-insensitive) followed by whitespace and non-empty title text SHALL be treated as an alert of that kind with that title. The title row SHALL show the title text, as rendered plain text without inline Markdown markers, in place of the default kind label. Every other aspect of the alert's presentation SHALL be the same as for an untitled alert of that kind. The marker line SHALL NOT be rendered as part of the alert body, and the body SHALL start with the quote's next line. A titled alert with no further lines SHALL render only its title row. In Visual Edit, the marker line, including the title, SHALL be owned by the callout title row, SHALL render the title when unfocused, and SHALL reveal the exact authored line when the caret enters it. Editing that line SHALL be one source mutation through the existing history path. Lines whose marker type is not one of the five kinds, or whose marker is not followed by whitespace, SHALL keep rendering as plain blockquote text.

#### Scenario: Titled alert renders its title
- **WHEN** a document contains `> [!NOTE] 注意` followed by `> 经过了前 6 章的准备。` and is shown in Split Preview or Read mode
- **THEN** it renders as a Note callout whose title row reads "注意"
- **AND** the body shows only "经过了前 6 章的准备。", without the `[!NOTE] 注意` line

#### Scenario: Marker case does not matter
- **WHEN** the first line is `> [!warning] Mind the gap`
- **THEN** it renders as a Warning callout titled "Mind the gap"

#### Scenario: Title-only alert
- **WHEN** a blockquote consists only of `> [!TIP] Remember this`
- **THEN** it renders as a Tip callout with the title row "Remember this" and no body

#### Scenario: Visual Edit reveals the titled marker line
- **WHEN** a titled alert is shown in Visual Edit and the caret is outside its first line
- **THEN** the title row shows the custom title with the kind's icon and accent
- **AND** moving the caret into the title row reveals the exact authored line `> [!NOTE] 注意`
- **AND** the body rows still render as quoted content below it

#### Scenario: Non-alert markers stay literal
- **WHEN** the first line is `> [!CUSTOM] Title` or `> [!NOTE]Title`
- **THEN** the quote renders as a plain blockquote with that line as text
