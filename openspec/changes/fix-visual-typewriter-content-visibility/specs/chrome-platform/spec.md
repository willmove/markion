## ADDED Requirements

### Requirement: Visual Edit typewriter viewport content coverage

When typewriter mode positions the Visual Edit caret, the editor SHALL paint the source-backed document content whose rendered portions intersect the document viewport under the resulting layout, including content preceding the active block. Caret centering SHALL NOT leave an artificial blank region by omitting visible preceding blocks. Coverage SHALL hold on the first painted frame using current geometry after each input action and throughout subsequent unchanged refinement frames. These presentation updates SHALL preserve existing centering, manual-scroll, and ordinary-reveal behavior without modifying document state or per-version caches.

#### Scenario: Typing and single Enter in an existing document
- **WHEN** Visual Edit and typewriter mode are enabled in a small window containing an existing document with multiple paragraphs separated by blank lines
- **AND** the user repeatedly types a line of text in a later paragraph and presses Enter once
- **THEN** after each text input and each Enter, the caret remains centered using current geometry within the existing centering tolerance and valid scroll range
- **AND** every preceding rendered text portion that still intersects the viewport is painted
- **AND** successive actions do not alternate that visible preceding content between painted text and an artificial blank region

#### Scenario: Upward centering includes visible predecessors
- **WHEN** an edit or caret movement requires upward typewriter centering from a later visual block
- **AND** the resulting viewport extends into one or more preceding blocks
- **THEN** those preceding blocks participate in painting wherever their rendered content intersects the viewport
- **AND** resolving the first visible block preserves the attainable requested pixel position of the caret

#### Scenario: First frame and unchanged frames have complete coverage
- **WHEN** current caret and viewport geometry are available after a typewriter input action
- **THEN** the first painted frame has both centered caret geometry and complete viewport content coverage
- **AND** unchanged refinement frames retain that coverage without waiting for another input or scroll event to restore missing content

#### Scenario: Resizing and typography changes preserve content coverage
- **WHEN** an existing multi-block document is being edited in typewriter mode and the window width, viewport height, font size, or paragraph spacing changes
- **THEN** centering uses the resulting current measurements and paints the document portions intersecting the new viewport
- **AND** wrapped or differently sized blocks do not cause preceding visible content to disappear

#### Scenario: Document boundaries retain intentional space
- **WHEN** the caret is on the first or last editable row in Visual Edit typewriter mode
- **THEN** presentation-only boundary space continues to permit centering within the valid scroll range
- **AND** blank regions represent document whitespace or boundary space, rather than omitted viewport-intersecting text
- **AND** content that legitimately moves outside the viewport need not remain painted

#### Scenario: Manual scrolling and disabled typewriter behavior are preserved
- **WHEN** the user scrolls without caret activity while typewriter mode is enabled, or edits after disabling typewriter mode
- **THEN** the editor retains its existing manual-scroll and minimum-distance reveal behavior respectively
- **AND** it paints the document content intersecting the resulting viewport without introducing continuous recentering

#### Scenario: Coverage correction preserves document and cache state
- **WHEN** the editor corrects typewriter positioning or performs unchanged refinement frames without an additional document edit
- **THEN** document text, dirty state, document version, and undo history remain unchanged
- **AND** shared per-version Markdown and visual caches, memoized highlighting, and cached text handles are reused without scroll-induced recomputation
