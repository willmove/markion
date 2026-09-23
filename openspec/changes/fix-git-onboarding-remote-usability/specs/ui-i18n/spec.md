## ADDED Requirements

### Requirement: Sync address correction and disclosure chrome SHALL be localized
The copy for the setup disclosure label pair (show/hide advanced), the sync-address replacement confirmation including the current and new address, the unreachability warning and its continue-anyway confirmation, and the inline address rejection message SHALL be translated through the i18n layer in every supported language. Engine-generated error text SHALL NOT be surfaced untranslated in the setup dialog for these flows.

#### Scenario: Replacement confirmation follows the UI language
- **WHEN** the user is asked to confirm resetting the sync address
- **THEN** the comparison of the current and new address and the confirm/cancel labels appear in the active interface language

#### Scenario: Unreachability warning follows the UI language
- **WHEN** the reachability probe fails and the user is warned
- **THEN** the warning and the continue/cancel labels appear in the active interface language

#### Scenario: Disclosure labels follow the UI language
- **WHEN** the advanced setup section is expanded or collapsed
- **THEN** its disclosure control's label appears in the active interface language in both states
