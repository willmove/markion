## ADDED Requirements

### Requirement: AI assistance SHALL have a GUI-free workspace boundary
The workspace SHALL contain `crates/ai` with package name `markion-ai`, owning provider protocols, bounded streaming/agent state, context budgets, and pure proposal/scope models without GPUI or GUI toolkit dependencies. The root application SHALL own GPUI views/tasks, live buffer/selection/undo coordination, credential-store integration, and filesystem execution/recovery. AI processing SHALL run outside typing/rendering paths and SHALL not invalidate document-derived state until a reviewed edit is accepted. Headless tests SHALL exercise protocols and agent limits without user credentials or paid service requests.

#### Scenario: AI member is tested headlessly
- **WHEN** the AI member's tests and dependency graph are inspected
- **THEN** they require neither GUI libraries nor a cloud API key

#### Scenario: Stream events repaint the panel
- **WHEN** AI text or tool progress arrives without applying a document edit
- **THEN** the root app updates only AI presentation state and preserves document version-derived caches

#### Scenario: GPUI integration stays in the app
- **WHEN** an AI proposal is applied to a live document or presented in a panel
- **THEN** live GPUI integration remains in the root application crate
