## MODIFIED Requirements

### Requirement: Block grips SHALL provide optional compact quick settings

When automatic grip-click popups are enabled, completing an ordinary non-drag click on an eligible Visual Edit block grip SHALL select that single complete source unit and open a nearby compact quick-settings popup. Ordinary grip clicks SHALL tolerate sub-threshold pointer jitter without entering drag, SHALL NOT be lost to intermediate-frame layout shifts caused by moving focus or selection between press and release, SHALL NOT be suppressed by a non-semantic macOS `function` modifier flag (`NSFunctionKeyMask`), and SHALL NOT be dismissed by zero-delta or sub-threshold trackpad/mouse scroll-wheel phase noise. Selecting a block via its grip SHALL NOT expand a following collapsible source block solely because the exclusive selection end equals that next block's start. When a platform input method synchronously commits an active composition immediately before a grip press, the grip gesture SHALL resolve the unambiguously corresponding block in the committed document version while genuinely stale targets continue to be rejected. Display math blocks with a Visual Edit math editor SHALL expose the same block-leading grip chrome as other reorderable blocks. The popup SHALL offer direct Text, H1-H6, bulleted/numbered/task list, quote, code, divider and table controls, the existing Duplicate, Move Up, Move Down and separated Delete operations, plus Copy, Cut and Add Below. It SHALL identify the current type, use localized labels or tooltips and active-theme colors, and keep disabled operations visibly unavailable. Every enabled command SHALL be reachable by pointer and keyboard. Shift-click SHALL extend a block range without opening the popup. Dragging past the grip drag threshold SHALL retain complete selected-unit reordering and SHALL NOT open a click popup. Disabled automatic popups SHALL retain selection-only clicks. Opening/dismissing the popup after selection SHALL leave source, version, history, dirty state, the established selection and derived caches unchanged.

#### Scenario: Completed click opens nearby quick settings
- **WHEN** automatic popups are enabled and the user clicks and releases an eligible block grip without dragging or extending selection
- **THEN** only that complete block unit is selected and a compact popup opens near the grip within the viewport
- **AND** its enabled type controls and editing actions are reachable without being obscured by later document content

#### Scenario: Small pointer jitter during a grip click still opens quick settings
- **WHEN** automatic popups are enabled and the pointer drifts slightly within the grip click-slop radius (for example 3–4 px on a macOS trackpad or Magic Mouse) between mouse-down and mouse-up
- **THEN** no block drag starts
- **AND** releasing the button selects that block and opens its compact quick-settings popup

#### Scenario: Mid-click intermediate-frame layout shift does not drop the grip popup
- **WHEN** the caret is inside an earlier block whose height collapses on defocus (such as an expanded math/diagram/HTML source pane or a wrapped block with revealed Markdown markers) and one or more frames render between grip mouse-down and mouse-up
- **THEN** the ordinary grip press does not collapse the earlier block before release
- **AND** releasing the click selects the clicked block and opens its quick-settings popup

#### Scenario: Zero-delta and micro scroll events do not prematurely dismiss the popup
- **WHEN** a Visual Edit block menu is open and the surface receives zero-delta scroll-phase events (such as trackpad `Started` or `Ended`) or sub-threshold micro scroll deltas
- **THEN** the block menu remains open
- **AND** a deliberate scroll exceeding the dismissal threshold closes the menu without mutating source, selection, version, history or derived caches

#### Scenario: Synchronous IME commit before grip press resolves the current block
- **WHEN** an active IME composition commits immediately before a grip mouse-down, advancing the document version from `V` to `V + 1` before the next frame renders
- **THEN** the grip gesture unambiguously relocates the pressed block across that IME change set and opens its quick-settings popup on release
- **AND** genuinely stale targets from unrelated or external changes remain rejected without mutating source or selection

#### Scenario: Display math block exposes the block-leading grip
- **WHEN** a document in Visual Edit contains a display math block (`$$...$$`)
- **THEN** the math block renders its block-leading grip with the math format symbol in the leading gutter
- **AND** clicking the grip selects the math block and opens its quick-settings popup

#### Scenario: macOS function flag does not suppress the grip popup and exclusive ends keep following collapsible blocks closed
- **WHEN** a grip click carries macOS `NSFunctionKeyMask` (`modifiers.function == true`) with no Shift/Ctrl/Alt/Cmd modifier on a block immediately before a collapsible HTML, math, diagram, or front-matter block
- **THEN** the click opens the block quick-settings popup for the clicked block
- **AND** the following collapsible block remains collapsed instead of expanding its source pane

#### Scenario: Shift and drag remain selection gestures
- **WHEN** the user Shift-clicks another block grip or drags an already selected block range past the grip drag threshold
- **THEN** the range extends or moves through the existing structured-selection/reordering behavior
- **AND** no automatic click popup opens during or after the gesture

#### Scenario: Copy and Cut target the clicked block
- **WHEN** the user invokes Copy or Cut from a valid grip popup
- **THEN** portable plain-text and HTML clipboard representations describe that complete source unit without content or markers from the following block
- **AND** Copy leaves the document unchanged, while Cut removes that exact unit in one checked edit and one undo step

#### Scenario: Add Below preserves the next heading
- **WHEN** the user invokes Add Below on a block immediately before a heading
- **THEN** one editable blank paragraph is inserted below the complete target unit and the caret moves there
- **AND** the following heading's prefix, unrelated source and LF/CRLF style are preserved
- **AND** one Undo restores the exact source and prior selection

#### Scenario: Dismissal and stale targets are safe
- **WHEN** the user dismisses a grip popup or its captured document version or block identity becomes stale
- **THEN** it closes without guessing a source mutation or changing the established selection merely to dismiss it
- **AND** explicit right-click and keyboard context-menu entry points remain usable
