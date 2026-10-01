## ADDED Requirements

### Requirement: GFM alerts SHALL render as styled callouts
Split Preview, Read mode, and Visual Edit SHALL present a GFM alert (a blockquote opening with `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, or `[!CAUTION]`) as a callout distinct from a plain blockquote. The callout SHALL use the alert kind's accent color for its left border, SHALL tint its background with that accent color, and SHALL show a title row with the kind's icon and its label (Note, Tip, Important, Warning, Caution). Each kind SHALL have a distinct accent color and icon, readable on light and dark themes. Plain blockquotes and blockquotes with unrecognized markers SHALL keep their existing presentation. In Split Preview and Read mode, activating the callout title row SHALL fold the alert body so only the title row remains, and activating it again SHALL unfold it. Fold state SHALL be view-only and SHALL NOT persist across launches. In Visual Edit, alerts SHALL NOT fold, and the whole quote group (title row and body rows) SHALL carry the kind's accent border and tint while the existing marker-line reveal behavior is unchanged. Styling and folding SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches.

#### Scenario: Preview renders an alert as a callout card
- **WHEN** a document containing `> [!WARNING]` followed by quoted body text is shown in Split Preview or Read mode
- **THEN** the alert renders with the Warning accent left border, a Warning-tinted background, and a title row with the Warning icon and the label "Warning"
- **AND** the body text renders inside the card below the title row

#### Scenario: Each alert kind has its own color and icon
- **WHEN** a document contains one alert of each of the five kinds
- **THEN** each callout shows a different accent color and a different icon

#### Scenario: Plain blockquotes are unchanged
- **WHEN** a document contains a plain blockquote, or a blockquote whose first line is an unrecognized marker such as `[!CUSTOM]` or `[!NOTE]-`
- **THEN** it renders with the existing blockquote presentation and no callout title row

#### Scenario: Folding an alert in Read mode
- **WHEN** the user activates the title row of an alert in Split Preview or Read mode
- **THEN** the alert body is hidden and only the title row remains
- **AND** activating the title row again shows the body
- **AND** the document text, version, dirty state, and undo history are unchanged

#### Scenario: Visual Edit alert groups carry the kind accent
- **WHEN** a GFM alert is shown in Visual Edit
- **THEN** its title row shows the kind icon next to the label
- **AND** the title row and every body row of that quote group use the kind's accent border and background tint
- **AND** focusing the title row still reveals the authored marker line for editing, and the alert does not fold
