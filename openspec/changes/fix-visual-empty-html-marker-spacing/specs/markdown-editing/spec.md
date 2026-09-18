## ADDED Requirements

### Requirement: Visual Edit renders content-free HTML markers as compact rows
A content-free HTML marker — an HTML-only paragraph or block whose tags carry no visible text (for example a bookmark anchor `<a id="english"></a>` or an empty container) — SHALL be owned by the HTML block path rather than dropped into an unsupported source island, and Visual Edit SHALL present it as a single compact source line styled as muted monospace text WITHOUT the source-island chrome (border, background, padding) or block margins that content-bearing HTML blocks use. Authored `<br>` line breaks count as visible content — a run of only breaks keeps the ordinary paragraph path that paints their empty lines. The compact row SHALL remain fully editable: pointer placement, caret painting, IME composition, and text input SHALL work through the same payload-field source projection, and adding visible content SHALL restore the ordinary presentation (styled paragraph for inline HTML with text, bordered HTML presentation for blocks) on the next parse. Authored blank lines around the marker SHALL keep their ordinary whitespace-row presentation.

#### Scenario: Bookmark anchor above a heading
- **WHEN** the document contains `<a id="english"></a>` followed by a blank line and a heading, and the caret is placed on the marker row or elsewhere
- **THEN** the marker occupies one slim source line with no extra vertical whitespace beyond that line and the authored blank lines
- **AND** placing the caret in the row, typing, and undo operate on the anchor's canonical source bytes

#### Scenario: Content-bearing HTML keeps its presentation
- **WHEN** the same position holds HTML with visible content — text inside style tags (`<b>bold</b>`), a labeled link anchor, or a block that flattens to text, an image, or a table
- **THEN** it keeps its existing presentation: inline-styled prose for HTML with text, and the bordered HTML block layout otherwise
