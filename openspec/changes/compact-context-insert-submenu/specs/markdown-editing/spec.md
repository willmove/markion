## MODIFIED Requirements

### Requirement: Context-labeled hover flyouts
Visual Edit editing menus SHALL offer separate Row and Column hover flyouts for tables, insertion for ordinary blocks and image scaling as applicable. Paragraph/heading/block formatting and whole-block actions SHALL appear directly at the top of the root menu, before selection editing, without a format flyout. Insert SHALL be the final ordinary-block root item and SHALL offer new paragraphs, H1–H5, bulleted/numbered/task lists, quotes, code blocks, dividers, tables, all five supported GFM callout kinds, and Link/Formula/Image settings. Block insertion on non-empty targets SHALL add a new source unit after the clicked complete block without converting it or replacing unrelated selected text, preserve neighbors and line endings, place the caret inside the new payload, and create one undo entry. Inline settings SHALL retain exact selection/caret targeting. Flyouts SHALL open without clicking, remain pointer-accessible beside the root, fit the viewport by flipping/scrolling, and support keyboard Right/Enter/Left/Escape. Clipboard and inline editing SHALL preserve the captured selection scope; whole-unit commands SHALL identify their actual context in localized labels. Changing hover or selection presentation SHALL not mutate canonical source, history or per-version derivations. Stale target/page callbacks SHALL not select or execute a different action.

The Insert flyout SHALL use the same compact button dimensions, spacing, theme styling, localized tooltips, and hover/keyboard-selected states as the root menu. Paragraph and H1–H5 SHALL occupy one six-button row, with distinct H1–H5 captions; bulleted/numbered/task lists, quote, code block, and divider SHALL occupy the following six-button row. The five supported callout kinds SHALL appear as a separate compact icon row with localized tooltips identifying each kind. Table and Link/Formula/Image SHALL remain labeled insertion actions separated from compact groups. All existing insertion actions SHALL retain their order, availability, captured targets, and pointer/keyboard dispatch behavior. Keyboard scrolling SHALL keep the selected compact button or labeled action visible without scrolling the document.

#### Scenario: Block controls precede editing controls
- **WHEN** an ordinary paragraph, heading or supported block context menu opens
- **THEN** its format and whole-block actions appear directly before clipboard/inline controls, with Insert last and no paragraph/block submenu
- **AND** current/disabled states and keyboard reachability are retained

#### Scenario: Context menus omit H6
- **WHEN** the user opens the block context menu or its Insert flyout
- **THEN** heading choices are limited to H1–H5 and no H6 command is shown
- **AND** existing H6 document content and other heading-formatting entry points retain their support

#### Scenario: Insert into an empty paragraph
- **WHEN** a context menu opens on an exactly mapped empty document, blank paragraph or document-end insertion row
- **THEN** the root heading is localized as Format Block and the Insert flyout's supported block, callout and inline settings actions are enabled independently of whole-block clipboard/deletion
- **AND** block insertion reuses the exact clicked/caret-owned blank line, preserves unrelated selection and neighboring content with correct LF/CRLF separators, rejects stale captures and is one undoable edit
- **AND** opening/canceling inline settings leaves source, selection, version, history and derived caches unchanged

#### Scenario: Insert a new block without converting the target
- **WHEN** a new heading, list, task, quote, code, divider, table or callout is selected from Insert
- **THEN** it is inserted after the complete clicked source unit and before the following unit with valid separators
- **AND** unrelated selection/source bytes remain intact, stale captures are rejected and one Undo restores source and selection

#### Scenario: Hover into a flyout and back
- **WHEN** the pointer enters an enabled contextual parent and then crosses to its flyout
- **THEN** the flyout opens without a click and stays actionable without dismissing or moving the root
- **AND** hovering another root control changes or closes the flyout, while an outside click or document transition dismisses the complete menu

#### Scenario: Insert uses the root menu's compact presentation
- **WHEN** an ordinary block's Insert flyout opens in a viewport with sufficient space
- **THEN** paragraph and H1–H5 appear in one compact row and list/quote/code/divider controls appear in the next compact row
- **AND** five callout buttons form a separate compact group, while Table and Link/Formula/Image retain labeled entries
- **AND** button sizing, spacing, tooltips and theme states match the root menu, with no H6 action or missing insertion command

#### Scenario: Compact insertion actions preserve exact dispatch
- **WHEN** a compact heading, list or callout insertion button is confirmed by pointer or keyboard
- **THEN** the corresponding existing insertion command targets the captured block and remains one undoable canonical edit
- **AND** a heading insertion button labeled H2 inserts Heading 2 rather than converting the clicked block or selecting a neighboring button's command

#### Scenario: Compact insertion remains reachable in a small viewport
- **WHEN** the Insert flyout opens near a viewport edge and the user navigates through every insertion action
- **THEN** the flyout stays within the usable viewport and menu-local scrolling keeps the selected button or labeled action visible
- **AND** opening, navigating, scrolling and dismissing preserve source, selection, version, history, dirty state and shared per-version derived caches
