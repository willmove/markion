## 1. Regression Signal

- [x] 1.1 Add a focused geometry regression test proving collapsed source IME ranges currently produce an invalid empty rectangle.
- [x] 1.2 Add a GPUI source-editor composition regression covering stable candidate origins across preedit growth with typewriter mode both enabled and disabled.

## 2. Source IME Geometry

- [x] 2.1 Normalize source-editor range bounds to a non-empty caret rectangle at the requested anchor without changing document or cache state.
- [x] 2.2 Re-run the focused regression loop and confirm both ordinary and typewriter source composition geometry pass.

## 3. Verification

- [x] 3.1 Run formatting and the relevant root-package test suite.
- [x] 3.2 Validate the OpenSpec change and document the Linux Wayland limitation of local verification.
