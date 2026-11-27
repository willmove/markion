#!/bin/bash
set -u

launcher="$(realpath -- "${BASH_SOURCE[0]}")" || exit 1
prefix="$(dirname -- "$(dirname -- "$launcher")")"
binary="$prefix/lib/markion/markion"
manifest="$prefix/share/vulkan/icd.d/lvp_icd.json"
if [[ ! -r "$manifest" ]]; then
  manifest="$prefix/share/vulkan/icd.d/lvp_icd.x86_64.json"
fi

# This file is only for communication with the first child of this launcher.
unset MARKION_PACMAN_RENDERER_STATE
run_software() {
  printf '%s\n' 'Markion: starting on system Lavapipe (software Vulkan)' >&2
  exec env -u VK_INSTANCE_LAYERS -u VK_LOADER_LAYERS_ENABLE -u VK_LOADER_LAYERS_ALLOW \
    MARKION_SOFTWARE_VULKAN=0 VK_DRIVER_FILES="$manifest" VK_ICD_FILENAMES="$manifest" \
    VK_LOADER_LAYERS_DISABLE='~implicit~' NODEVICE_SELECT=1 "$binary" "$@"
}

mode=${MARKION_SOFTWARE_VULKAN:-auto}
case "${mode,,}" in
  0|off|false|no|never) exec "$binary" "$@" ;;
  1|force|on|true|yes)
    if [[ -r "$manifest" ]]; then
      run_software "$@"
    fi
    printf '%s\n' 'Markion: Lavapipe is unavailable; checking the system Vulkan configuration' >&2
    ;;
esac

if [[ ! -r "$manifest" ]] || ! state="$(mktemp -t markion-vulkan.XXXXXXXXXX)"; then
  exec "$binary" "$@"
fi
trap 'rm -f -- "$state"' EXIT

MARKION_PACMAN_RENDERER_STATE="$state" "$binary" "$@" <&0 &
child=$!

stop_child() {
  kill -TERM "$child" 2>/dev/null || true
  wait "$child" 2>/dev/null || true
  exit "$1"
}
trap 'stop_child 129' HUP
trap 'stop_child 130' INT
trap 'stop_child 143' TERM

wait "$child"
status=$?
trap - HUP INT TERM

phase="$(<"$state")"
if [[ "$status:$phase" == '1:failed' || "$status:$phase" == '139:initializing' ]]; then
  rm -f -- "$state"
  trap - EXIT
  run_software "$@"
fi
exit "$status"
