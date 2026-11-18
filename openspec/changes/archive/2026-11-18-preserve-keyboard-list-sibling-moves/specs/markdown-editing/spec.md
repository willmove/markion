## ADDED Requirements

### Requirement: Keyboard moves of one list subtree SHALL use adjacent sibling units

When the keyboard move command resolves a selection to a single complete list subtree, it SHALL use the same adjacent source-unit target as the block menu and keep the target's descendants together.

#### Scenario: A list parent moves down and back up with the keyboard
- **WHEN** the user moves a list parent below its sibling with the keyboard and moves it back up
- **THEN** both parents retain their respective descendants and the original order is restored
- **AND** following independent prose remains outside the list after each move
