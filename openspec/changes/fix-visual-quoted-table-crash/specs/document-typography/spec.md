## ADDED Requirements

### Requirement: Windows font resolution survives missing preferred families

On Windows, when a requested font family cannot be loaded, Markion SHALL try the available families in its configured fallback chain in order, followed by the system UI family. The same fallback behavior SHALL apply when test support is enabled.

#### Scenario: Missing code font falls back to an installed monospace family

- **WHEN** a Windows source editor needs an unavailable code font and a configured monospace fallback is installed
- **THEN** it uses the first loadable fallback family and lays out the text successfully

#### Scenario: No explicit fallback family is available

- **WHEN** the requested family and all explicitly configured fallbacks are unavailable on Windows
- **THEN** text uses the system UI family if it can be loaded
