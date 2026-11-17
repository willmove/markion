## 1. Preferences

- [x] 1.1 Add the default-on automatic-download preference, TOML compatibility and reset behavior; verify missing/invalid/false values and round trips with storage tests.
- [x] 1.2 Add localized General update controls and fallback guidance for all supported languages; verify localization coverage and preference changes preserve document caches.

## 2. Update lifecycle

- [x] 2.1 Add a five-hour scheduler, timestamp persistence and overlap guards while preserving manual checks; verify interval boundaries, startup override, invalid/future dates and busy/ready states with deterministic tests.
- [x] 2.2 Split authenticated download/staging from installation, enforce matching manifest versions, retain ready packages and guard staged integrity; verify version mismatch and tampering rejection with offline tests.
- [x] 2.3 Add ready/restart prompts and dirty-document gates, safe Windows handoff and existing AppImage replacement/relaunch; verify Later/retry and dirty-state behavior with app tests and platform compilation.

## 3. Integration validation

- [x] 3.1 Run formatting, focused update/preference tests, workspace tests and OpenSpec validation; record results and explicitly document any native-platform validation unavailable on this host.
