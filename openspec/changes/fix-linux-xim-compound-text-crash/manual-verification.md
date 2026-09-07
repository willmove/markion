# Linux X11 manual verification

Date: 2026-09-07

## Reported baseline (pre-fix)

The supplied Markion 0.3.3 log came from Linux X11 with IBus connected, an X11
compositor present, and the `llvmpipe` Vulkan adapter. Two non-ASCII preedit
updates were accepted before `zed-xim 0.4.0-zed` panicked while decoding
COMPOUND_TEXT with `UnsupportedEncoding`. This is evidence for the original
failure, not post-fix verification.

## Automated Linux evidence

- The vendored `zed-xim` tests were linked for `x86_64-unknown-linux-gnu` with
  the `x11rb-client` feature and executed under Ubuntu 26.04 WSL: all three
  request-boundary tests passed.
- A minimal Linux-target harness compiled GPUI's X11-only path against the
  repository-local `vendor/zed-xim`; `cargo tree` resolved `xim-ctext 0.4.1`.
- The Markion IME regression verifies that failure cleanup retains the last
  accepted Chinese preedit and selection, clears the marked range and IME undo
  capture without advancing the document version, preserves the same derived
  block cache, and accepts later direct input through the normal mutation path.

## Desktop input-method matrix

No desktop X11 input method is installed in the available Windows/WSL
environment (`ibus`, `fcitx`, and `fcitx5` are absent), so the post-fix desktop
checks below remain pending on a Linux X11 workstation.

| Input method | Locale/input source | Preedit + commit | Cancel | Undo/redo | Direct typing after failure | Result |
| --- | --- | --- | --- | --- | --- | --- |
| IBus | Chinese (GB2312-producing engine) | Pending | Pending | Pending | Pending | Not run |
| IBus | Korean (KS C 5601-producing engine) | Pending | Pending | Pending | Pending | Not run |
| Fcitx/Fcitx5 | Chinese | Pending | Pending | Pending | Pending | Not run |
| Fcitx/Fcitx5 | Korean | Pending | Pending | Pending | Pending | Not run |

For the follow-up, run the patched Markion build in an X11 session, exercise
multiple preedit updates followed by commit and cancellation, verify one-step
composition undo/redo, and confirm the process remains alive. If malformed
COMPOUND_TEXT can be produced by the test server, verify the mark clears and a
plain keyboard character can still be entered after the logged XIM client
error. Do not include composition payload bytes or user text in the evidence.
