## ADDED Requirements

### Requirement: Setup route selection SHALL be directly visible with generic actions
The Backup-and-Sync setup dialog SHALL present all setup routes as one directly visible selectable set: each route is a selectable control, the contextually recommended route is preselected when the dialog opens, and the current selection is marked. Route switching SHALL retain the existing per-route field defaults. The dialog SHALL NOT wrap route alternatives in a disclosure control or repeat the selected route's name as its action.

The dialog's bottom actions SHALL be a generic confirm and a generic cancel, independent of the selected route, error state, or advanced-required state.

#### Scenario: Three routes are visible and one is preselected
- **WHEN** the setup dialog opens
- **THEN** every setup route is visible and selectable without expanding anything
- **AND** the contextually recommended route is already selected and marked

#### Scenario: Switching routes keeps field behavior
- **WHEN** the user selects a different route
- **THEN** the per-route field defaults adjust as before and the selection mark moves

#### Scenario: Bottom actions stay generic
- **WHEN** the dialog is shown in any state — fresh, after an error, or with advanced review required
- **THEN** the primary action is a plain confirm and the secondary action is cancel, neither repeating a route name
