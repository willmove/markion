## 1. Package and startup recovery

- [x] 1.1 Wire the pacman launcher, dependency and renderer state reporting, then check shell syntax and compile the application.
- [x] 1.2 Exercise the launcher with signal, normal error, argument preservation, force and disable fixtures and add an installed-window check to Arch packaging.

## 2. Verification and documentation

- [x] 2.1 Update Linux startup guidance and run the available regression tests and strict OpenSpec validation. Record which Linux checks ran and which require a native environment.

## Verification record

- All four changed shell scripts passed Bash syntax checks. The launcher fixture passed all nine cases in Git Bash, including a startup segmentation fault, a segmentation fault after readiness, ordinary failures, caller arguments, and forced rendering modes.
- The Windows app test build passed all three renderer fallback tests, including the readiness and retry state transition. Rust formatting, strict OpenSpec validation and the diff check passed.
- The Arch release gate now verifies installed files and dependencies, runs the launcher fixtures, and checks for the main Markion window after recovery under Xvfb. Running the gate with a rebuilt Linux application and verifying a real Omarchy Wayland session require a Linux environment. Neither was available on the local Windows host.
