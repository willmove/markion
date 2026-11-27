## Context

The pacman package currently installs the DEB executable as `/usr/bin/markion` and lists Lavapipe as an optional dependency. The observed Vulkan crash terminated both the application and `vulkaninfo`, while an isolated Lavapipe launch succeeded.

## Decisions

- Keep `/usr/bin/markion` as the package's public entry point and place the executable next to its resources under `/usr/lib/markion`. The existing desktop entry continues to invoke the public command.
- Try the normal renderer in a child process. Use a private temporary state file to distinguish renderer initialization, an initial renderer failure, and the first successful window creation. Retry a reported renderer failure or a segmentation fault during renderer initialization once. Preserve the arguments and the exit status for successful launches, unrelated failures, and failures after a window opened.
- Run the retry with the installed Lavapipe manifest. Scope driver selection and layer filtering to the process. Support the existing software Vulkan force and disable settings at the pacman launcher.
- Exercise a launcher fixture that terminates with SIGSEGV, plus an installed X11 launch whose initial driver manifest cannot load. Record native Omarchy testing separately from the Arch container result.
