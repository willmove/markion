## ADDED Requirements

### Requirement: Setup disclosure controls SHALL reflect their expanded state
Every show/hide disclosure control in the Backup-and-Sync setup and sync surfaces — including the advanced setup section of the onboarding dialog — SHALL label itself according to its current state: while its section is expanded the control offers to hide it, and while collapsed the control offers to show it. The label SHALL be localized chrome.

#### Scenario: Advanced section toggle reflects state
- **WHEN** the advanced setup section of the onboarding dialog is expanded
- **THEN** its disclosure control is labeled as hiding the advanced section
- **WHEN** the section is collapsed
- **THEN** the control is labeled as showing the advanced section
