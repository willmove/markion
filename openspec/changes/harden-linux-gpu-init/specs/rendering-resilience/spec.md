## Purpose

Keeps Markion launchable and communicative on machines whose rendering support is limited or broken: graceful, localized, actionable failure when no renderer can be initialized, and usable operation when only software rendering (software Vulkan or OpenGL) is available.

## ADDED Requirements

### Requirement: Actionable failure when the renderer cannot initialize

When GPU/renderer context initialization fails at startup on any platform, Markion SHALL present the failure through an operating-system-native message that does not depend on Markion's own renderer, stating that the display renderer could not start and including platform-appropriate remediation steps. Markion SHALL also record the failure and the underlying error detail in the diagnostic log, and SHALL exit with a non-zero status instead of proceeding to an unusable state. A bare panic message SHALL NOT be the only user-visible indication of the failure.

On Linux the remediation steps SHALL include: installing a software Vulkan driver package, forcing a specific Vulkan driver through the Vulkan loader's driver-selection environment variables, disabling a stale or crashing Vulkan driver manifest or implicit layer, and consulting the repository's Linux GPU troubleshooting document for the full procedure.

#### Scenario: Linux GPU-init failure shows remediation guidance

- **WHEN** Markion starts on Linux and the GPU context cannot be initialized (for example, every Vulkan driver fails)
- **THEN** an operating-system-native message appears explaining that the renderer failed to start
- **AND** the message lists remediation steps including installing a software Vulkan driver, selecting or forcing a Vulkan driver via environment variables, and disabling broken Vulkan manifests or implicit layers
- **AND** the process exits with a non-zero status after the message is dismissed

#### Scenario: Failure detail is preserved in the log

- **WHEN** renderer initialization fails
- **THEN** the diagnostic log records the failure together with the underlying renderer error string
- **AND** the record is written before the exit path runs

#### Scenario: Guidance text follows the active interface language

- **WHEN** the failure message is composed
- **THEN** its text is taken from the localization catalog in the active interface language

#### Scenario: Native message unavailable degrades to standard error

- **WHEN** the operating-system-native message mechanism itself cannot be displayed
- **THEN** the same guidance text is written to standard error and the non-zero exit status is preserved

#### Scenario: Documented guidance stays consistent

- **WHEN** the failure guidance or the repository's Linux GPU troubleshooting document changes
- **THEN** the remediation steps described in each remain consistent with the other

### Requirement: Startup on software Vulkan adapters

Markion SHALL start and remain usable for document editing when the only available Vulkan adapter is a software rasterizer (such as llvmpipe or lavapipe) with no hardware GPU present, without requiring an environment-variable opt-in. Reduced rendering performance under software rasterization is acceptable; loss of editing functionality is not.

#### Scenario: Starts with only a software Vulkan adapter

- **WHEN** Markion starts on a Linux machine whose only Vulkan adapter is a software rasterizer
- **THEN** the main window opens and the editor becomes usable without any special environment variables

#### Scenario: Editing remains functional under software rendering

- **WHEN** Markion is running on a software Vulkan adapter
- **THEN** typing, caret navigation, saving, and view-mode switching behave correctly, at possibly reduced smoothness

### Requirement: OpenGL fallback on Linux

On Linux, when no usable Vulkan implementation can be initialized but OpenGL (including Mesa software OpenGL) is available, Markion SHALL start and render through an OpenGL-based backend rather than failing, preferring any usable Vulkan path first. When this fallback activates, Markion SHALL record it in the diagnostic log.

#### Scenario: Broken Vulkan with working OpenGL still starts

- **WHEN** the system's Vulkan stack cannot be initialized at all (for example, no loadable Vulkan driver) while OpenGL rendering works
- **THEN** Markion starts and remains usable for editing

#### Scenario: Vulkan remains preferred

- **WHEN** both a usable Vulkan implementation and OpenGL are available
- **THEN** Markion initializes the Vulkan path and does not activate the OpenGL fallback
