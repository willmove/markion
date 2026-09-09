## ADDED Requirements

### Requirement: HTML import conversion has a GUI-free workspace boundary
The workspace SHALL contain a `crates/html-import` member whose Cargo package name is `markion-html-import`. The crate SHALL own HTML-to-Markdown conversion of clipboard rich text without depending on `gpui` or any other GUI toolkit, and SHALL build and pass its tests headless. GPUI clipboard access and paste orchestration SHALL remain in the root `markion` crate (including the vendored GPUI patch that reads the HTML flavor). The root manifest SHALL include an explicit `[profile.dev.package.markion-html-import]` optimization override.

#### Scenario: HTML import crate builds without GUI dependencies
- **WHEN** the dependency tree and tests for `markion-html-import` are inspected
- **THEN** the crate contains no GPUI or GUI toolkit dependency and converts clipboard HTML fixtures in a headless environment

#### Scenario: GPUI adaptation stays in the root crate
- **WHEN** clipboard HTML is acquired from the OS or converted Markdown is inserted into a document
- **THEN** that implementation resides in the root application crate rather than `markion-html-import`

#### Scenario: HTML import member receives a dev optimization override
- **WHEN** `markion-html-import` is added to the workspace
- **THEN** the root `Cargo.toml` includes an explicit development-profile package override for `markion-html-import`
