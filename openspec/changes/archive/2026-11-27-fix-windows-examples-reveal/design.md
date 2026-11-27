## Context

Both Help menus dispatch the same action. Bundled resource lookup finds the index even when its path mixes Windows and Unix separators, but Windows shell path parsing rejects that path with `0x80070057`. The GPUI reveal call logs the failure internally. Markion already launches the system file manager for tab and file tree context actions.

## Goals / Non-Goals

Reveal `README.md` in Explorer and surface launch failures in Markion. The action only updates menu and status state. Document loading, editing, and resource packaging are outside the scope of this change.

## Decisions

- Use the existing Windows file manager launcher for the Help action. Pass a native path and quote it as the value of Explorer's `/select,` argument, so spaces and commas stay within the path. Keep the path as an operating system string to preserve Unicode.
- Reuse the localized file manager success and error messages on Windows and the localized missing-collection message on all platforms. macOS and Linux continue through their existing GPUI reveal integration.
- Exercise the actual Windows Help menu and inspect the selection in Explorer. A successful process launch alone cannot verify that Explorer selected the index.
