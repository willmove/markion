## ADDED Requirements

### Requirement: Source editor IME candidate geometry
The source editor SHALL report a non-empty platform text-input rectangle at the requested composition anchor. During one IME composition, successive preedit replacements SHALL keep the candidate anchor at the composition start unless the edited row itself moves, including when typewriter mode recenters the source viewport. This behavior SHALL apply on Linux Wayland without changing document text, selection semantics, or other platforms' native input behavior.

#### Scenario: Typewriter composition stays at the visible source caret
- **WHEN** an IME composition starts in the source editor while typewriter mode has centered the active row
- **THEN** the platform candidate rectangle has positive caret width and current source line height
- **AND** its origin is the visible composition anchor rather than the window origin

#### Scenario: Preedit growth keeps a stable anchor
- **WHEN** one source-editor composition replaces its marked text with successively longer preedit strings
- **THEN** each platform candidate rectangle remains anchored at the same source insertion position
- **AND** the input method does not derive horizontal placement from the changing preedit length

#### Scenario: Ordinary source editing retains accurate candidate placement
- **WHEN** typewriter mode is disabled and an IME composition starts or updates in the source editor
- **THEN** the candidate rectangle remains non-empty and aligned with the requested source composition anchor

