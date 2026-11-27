#!/usr/bin/env bash
# Run as an unprivileged user under Xvfb after installing the pacman package.
set -euo pipefail

test_dir="$(mktemp -d -t markion-pacman-smoke.XXXXXXXXXX)"
app_pid=
cleanup() {
  if [[ -n "$app_pid" ]]; then
    kill -TERM "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
  fi
  rm -rf -- "$test_dir"
}
trap cleanup EXIT

export XDG_CONFIG_HOME="$test_dir/config"
export XDG_CACHE_HOME="$test_dir/cache"
export XDG_DATA_HOME="$test_dir/data"
export XDG_STATE_HOME="$test_dir/state"
export XDG_RUNTIME_DIR="$test_dir/runtime"
mkdir -m 0700 "$XDG_RUNTIME_DIR"
unset WAYLAND_DISPLAY WAYLAND_SOCKET MARKION_SOFTWARE_VULKAN
ulimit -c 0

# An empty host driver selection forces an early renderer failure.
export VK_DRIVER_FILES="$test_dir/unavailable-icd.json"
export VK_ICD_FILENAMES="$VK_DRIVER_FILES"
/usr/bin/markion >"$test_dir/app.log" 2>&1 &
app_pid=$!
for ((attempt = 0; attempt < 30; attempt++)); do
  if ! kill -0 "$app_pid" 2>/dev/null; then
    break
  fi
  if grep -Fq 'starting on system Lavapipe' "$test_dir/app.log" \
      && timeout 2 xdotool search --onlyvisible --name '^Markion($| - )' >/dev/null 2>&1; then
    sleep 1
    if kill -0 "$app_pid" 2>/dev/null; then
      cat "$test_dir/app.log"
      printf '%s\n' 'Installed pacman package opened a Markion window after the software retry'
      exit 0
    fi
    break
  fi
  sleep 1
done
cat "$test_dir/app.log" >&2
printf '%s\n' 'Installed pacman package failed to open a Markion window on Lavapipe' >&2
exit 1
