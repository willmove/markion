## ADDED Requirements

### Requirement: Diagram rendering has a GUI-free workspace boundary
The workspace SHALL contain a `crates/diagram` member whose Cargo package name is `markion-diagram`. The crate SHALL own the diagram backend trait, registry, pure render request/result/error types, output safety checks, and optional built-in backend adapters without depending on `gpui` or any other GUI toolkit. GPUI image conversion, application scheduling, and presentation state SHALL remain in the root `markion` crate. Because diagram rendering executes on the live preview path, the root manifest SHALL include an explicit `[profile.dev.package.markion-diagram]` optimization override.

#### Scenario: Diagram crate builds without GUI dependencies
- **WHEN** the dependency tree and tests for `markion-diagram` are inspected
- **THEN** the crate contains no GPUI or GUI toolkit dependency and can render/sanitize enabled backend output in a headless environment

#### Scenario: GPUI adaptation stays in the root crate
- **WHEN** sanitized diagram SVG is converted into a GPUI image or presented in a preview element
- **THEN** that implementation resides in the root application crate rather than `markion-diagram`

#### Scenario: Diagram member receives a dev optimization override
- **WHEN** `markion-diagram` is added to the workspace and used by live preview
- **THEN** the root `Cargo.toml` includes an explicit development-profile package override for `markion-diagram`

### Requirement: New diagram backends use explicit compile-time registration
A new in-tree or external diagram backend SHALL be able to depend on the GUI-free `markion-diagram` contract, implement the backend trait for its own backend type, and join Markion through explicit compile-time registry construction. The contract SHALL NOT require backend code to depend on Markdown parser internals, GPUI application state, or HTML export internals, and SHALL NOT claim runtime dynamic-plugin ABI compatibility.

#### Scenario: Contributor backend implements one core contract
- **WHEN** a contributor adds a new statically linked diagram format
- **THEN** format-specific parsing/rendering implements the diagram backend trait and is connected by explicit registry registration without adding a format-specific preview-block variant

#### Scenario: Runtime binary plugins remain out of scope
- **WHEN** the diagram registry is initialized
- **THEN** it uses statically linked backend instances and performs no dynamic-library discovery or loading
