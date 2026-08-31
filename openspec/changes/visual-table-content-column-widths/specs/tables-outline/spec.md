## ADDED Requirements

### Requirement: GFM table columns size from cell content

The editor SHALL allocate GFM pipe-table column widths from recommended content widths of the cells in each column, not from an equal split, in Visual Edit, Split Preview, and Read mode. The table SHALL remain stretched to the document content column. A column whose cells are short SHALL receive a smaller share than a column whose cells are long, so a typical two-column name/description table presents a narrow first column and a wider second column. Long cell text SHALL wrap inside its allocated column rather than overflowing the pane. Visual Edit and the read-only preview grid SHALL use the same width recommendation for the same table rows.

Column recommendations SHALL be derived from each cell's rendered plain text (not from focused Visual Edit source markup, and not from GFM source pipe padding). Computing or applying those widths SHALL NOT mutate document text, dirty state, undo/redo history, or per-document-version derived Markdown caches. Typography-driven reflow MAY change the pixel sizes without bumping document version.

One-column tables SHALL continue to occupy the full content column. Empty or missing cells SHALL NOT collapse their column below a readable minimum share.

No column in a table of `n` columns (`n` ≥ 2) SHALL receive more than three equal-shares (`3 / n` of the recommended width sum), except when that column’s header minimum itself exceeds the cap. Header recommendations SHALL keep a parenthesis pair whose inner non-whitespace text is 1–3 characters on one line when that unit fits, and SHALL be wide enough that the header’s recommended wrap is at most three lines.

#### Scenario: Unequal content yields unequal columns

- **WHEN** a GFM table has a short-text column (for example header `名称` and cells such as `操作系统`) beside a long-text column (for example header `说明` and a multi-word technical description)
- **THEN** Visual Edit, Split Preview, and Read mode render the short column narrower than the long column
- **AND** the two columns are not an equal-width split of the table

#### Scenario: Visual Edit matches Read mode

- **WHEN** the same GFM table is shown in Visual Edit and in Read mode
- **THEN** both surfaces use the same per-column width recommendation for those rows

#### Scenario: Focused source markup does not reflow columns

- **WHEN** the user focuses a Visual Edit table cell whose authored source is longer than its rendered plain text (for example `**bold**`)
- **THEN** column width shares stay based on the rendered cell text
- **AND** focusing the cell does not widen that column solely because source markup is visible

#### Scenario: Table stays within the content column

- **WHEN** a GFM table is rendered in Visual Edit, Split Preview, or Read mode
- **THEN** the table still spans the document content column
- **AND** cell text that exceeds its allocated column wraps inside the cell instead of overflowing the pane

#### Scenario: Layout is presentation-only

- **WHEN** column widths are computed or applied for a GFM table
- **THEN** the document text, dirty flag, undo/redo history, and per-document-version derived Markdown caches remain unchanged

#### Scenario: One-column and empty cells remain usable

- **WHEN** a GFM table has a single column, or a column whose cells are empty
- **THEN** that column still occupies a usable share of the table
- **AND** a one-column table still spans the document content column

#### Scenario: Long body columns cannot exceed three equal-shares

- **WHEN** a GFM table has six columns and two of them contain paragraph-length body text
- **THEN** neither column’s recommended share exceeds half of the table (`3 / 6`)
- **AND** a short unit header such as `实际功率（W）` still receives a larger share than it would under an uncapped linear split of the same preferred widths

#### Scenario: Short header parenthesis units stay together

- **WHEN** a table header cell contains a parenthesis pair whose inner non-whitespace text is 1–3 characters (for example `实际功率（W）` or `Power (W)`)
- **THEN** that column’s recommended minimum is at least as wide as the parenthesis unit plus cell padding
- **AND** Visual Edit and Read mode use that same minimum

#### Scenario: Header wrap stays within three recommended lines

- **WHEN** a table header cell’s unwrapped content is wider than three times a single-line glyph run
- **THEN** that column’s recommended minimum is at least one third of the unwrapped header content width plus cell padding
