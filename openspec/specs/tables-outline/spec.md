# tables-outline

## Purpose

Covers GFM table rendering, the row/column editing toolbars, and the document outline panel. Direct cell-level visual table editing and outline folding/collapsing are **not** part of this capability — they are future candidates.

## Requirements

### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces and SHALL provide toolbars (in the preview, in Visual Edit, and via source commands) to add, delete, and move rows and columns of the corresponding source table. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters; the preview and Visual Edit grids render cells as plain text (inline styles inside table cells are not required to render, though the HTML export keeps full fidelity). Direct cell-level visual table editing is not required for Visual Edit v1; table cell content SHALL remain editable through the source editing workflow.

#### Scenario: GFM table renders as a visual table
- **WHEN** the document contains a GFM-style table
- **THEN** the preview renders it as a visual grid

#### Scenario: GFM table renders in Visual Edit
- **WHEN** the document contains a GFM-style table and the active view mode is Visual Edit
- **THEN** the editor renders the table as a visual grid in the single editing surface
- **AND** direct cell-level visual editing is not required

#### Scenario: Row and column operations via the preview toolbar
- **WHEN** the user clicks the add/delete/move row or column buttons on a preview table's toolbar
- **THEN** the corresponding source table is updated and the preview re-renders

#### Scenario: Row and column operations via the Visual Edit toolbar
- **WHEN** the user clicks the add/delete/move row or column buttons on a Visual Edit table's toolbar
- **THEN** the corresponding source table is updated through the same source-table edit path as existing table commands
- **AND** the visual editing surface re-renders from the updated Markdown source

#### Scenario: Row and column operations via source commands
- **WHEN** the user invokes a source table command (format, add/delete/move row or column)
- **THEN** the source Markdown table is reformatted or edited accordingly

#### Scenario: Alignment is parsed and honored by exporters
- **WHEN** a table's separator row declares column alignments
- **THEN** the LaTeX and HTML exports emit the declared alignment, even though the preview and Visual Edit grids use a fixed flex layout

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports click-to-jump navigation, highlights the heading for the section containing the cursor, and updates as headings change. The outline is a flat indented list; collapse/expand of subsections is **not** supported.

#### Scenario: Outline lists headings and tracks the document
- **WHEN** the outline panel is visible
- **THEN** it lists all headings with their nesting indentation and updates when headings are added, removed, or changed

#### Scenario: Click to jump
- **WHEN** the user clicks a heading in the outline
- **THEN** the editor scrolls to that heading's source position

#### Scenario: Active section highlight
- **WHEN** the cursor is within a given section
- **THEN** the outline highlights the heading corresponding to that section
