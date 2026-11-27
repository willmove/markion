## ADDED Requirements

### Requirement: Pacman installations recover from Vulkan startup failures
The pacman package SHALL install the Arch Lavapipe software Vulkan dependency and route both the desktop entry and terminal command through the same launcher. By default, the launcher SHALL try the system Vulkan stack first. If the renderer fails to initialize or the process segfaults during renderer startup before opening its first window, the launcher SHALL retry once with Lavapipe and implicit Vulkan layers disabled for that process. The launcher SHALL preserve arguments and respect a user request to force or disable software Vulkan. The Arch packaging gate SHALL verify retry behavior and the installed application's ability to show a window after an initial renderer failure.

#### Scenario: Working hardware opens a window on the first launch
- **WHEN** the system Vulkan stack opens the first Markion window
- **THEN** the original process continues with its system Vulkan configuration

#### Scenario: Renderer startup segfaults on a cloud desktop
- **WHEN** the system Vulkan process segfaults during renderer initialization before Markion opens its first window
- **THEN** the launcher starts exactly one retry on Lavapipe with implicit Vulkan layers disabled
- **AND** the renderer retry receives the original document and command-line arguments

#### Scenario: Renderer startup reports a recoverable error
- **WHEN** Markion reports that its initial renderer cannot start
- **THEN** the installed launcher opens Markion using the software Vulkan driver
- **AND** a failure of the software launch exits without a further retry

#### Scenario: A running application later crashes
- **WHEN** the process crashes after its first window has opened
- **THEN** the launcher preserves the failure status without starting another application process
