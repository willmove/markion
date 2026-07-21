## MODIFIED Requirements

### Requirement: Configurable rendered-document font size
The editor SHALL use a global rendered-document font-size preference, expressed in logical pixels, as the body-text basis for Visual Edit, the preview pane of Split Preview, and Read mode. The default SHALL be 14px and the supported range SHALL be 10–32px inclusive. Headings, lists, block quotes, tables, code, Visual Edit surfaces (including rendered editors, progressive-reveal runs, and transitional source views for WYSIWYG coverage gaps), and inline/display math SHALL derive their text and line metrics from the resolved body size while preserving the current default visual proportions.

#### Scenario: Reading font size applies across rendered modes
- **WHEN** the user changes the rendered-document font size
- **THEN** Visual Edit, Split Preview's rendered pane, and Read mode use the selected body size on their next render
- **AND** dependent heading, list, quote, table, code, and math typography scales consistently
