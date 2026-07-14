## 1. Application Module Foundation

- [x] 1.1 Create `src/app/`, keep the Windows crate attribute in `src/main.rs`, and delegate startup through `app::run()`.
- [x] 1.2 Move application bootstrap, key bindings, native menus, and window-close wiring into the bootstrap module.

## 2. State and Command Boundaries

- [x] 2.1 Extract editor-tab state, undo history, preview-list synchronization, and path helpers without changing cache/version behavior.
- [x] 2.2 Split core, document, workspace/view, preferences/appearance, search, and editing command handlers into cohesive application modules.

## 3. Rendering Boundaries

- [x] 3.1 Extract GPUI text-input handling and the custom editor element into its own module.
- [x] 3.2 Split the root/panel UI and preview/visual rendering into dedicated modules while preserving virtualization and memoization paths.

## 4. Tests and Validation

- [x] 4.1 Relocate application tests so focused and cross-module regressions remain discoverable and unchanged in intent.
- [x] 4.2 Run formatting, root-package build/tests, and OpenSpec validation; fix only modularization regressions.
