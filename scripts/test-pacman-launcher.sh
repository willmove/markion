#!/usr/bin/env bash
set -euo pipefail
ulimit -c 0

launcher="${1:-$(dirname -- "${BASH_SOURCE[0]}")/markion-pacman-launcher.sh}"
fixture="$(mktemp -d -t markion-launcher-test.XXXXXXXXXX)"
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/lib/markion" "$fixture/share/vulkan/icd.d"
install -m 0755 "$launcher" "$fixture/bin/markion"

export MARKION_TEST_MANIFEST="$fixture/share/vulkan/icd.d/lvp_icd.json"
export MARKION_TEST_TRACE="$fixture/trace"
touch "$MARKION_TEST_MANIFEST"
cat >"$fixture/lib/markion/markion" <<'EOF'
#!/bin/bash
set -eu
[[ $# -eq 2 && $1 == '--' && $2 == 'sample document 中文.md' ]] || exit 20
if [[ "${VK_DRIVER_FILES:-}" == "$MARKION_TEST_MANIFEST" ]]; then
  printf 'software\n' >>"$MARKION_TEST_TRACE"
  [[ "${VK_ICD_FILENAMES:-}" == "$MARKION_TEST_MANIFEST" ]] || exit 21
  [[ "${VK_LOADER_LAYERS_DISABLE:-}" == '~implicit~' && "${NODEVICE_SELECT:-}" == 1 ]] || exit 22
  [[ ! -v VK_INSTANCE_LAYERS && ! -v VK_LOADER_LAYERS_ENABLE && ! -v VK_LOADER_LAYERS_ALLOW ]] || exit 23
  [[ ! -v MARKION_PACMAN_RENDERER_STATE ]] || exit 24
  exit "$MARKION_TEST_SOFTWARE_EXIT"
fi
printf 'system\n' >>"$MARKION_TEST_TRACE"
[[ "${VK_DRIVER_FILES:-}" == /host-driver.json ]] || exit 25
if [[ -n "${MARKION_PACMAN_RENDERER_STATE:-}" ]]; then
  printf '%s' "$MARKION_TEST_PHASE" >"$MARKION_PACMAN_RENDERER_STATE"
fi
if [[ "$MARKION_TEST_SYSTEM_EXIT" == 139 ]]; then
  kill -SEGV "$BASHPID"
fi
exit "$MARKION_TEST_SYSTEM_EXIT"
EOF
chmod +x "$fixture/lib/markion/markion"

run_case() {
  local mode=$1 phase=$2 system_exit=$3 software_exit=$4 expected_status=$5 expected_trace=$6
  local status=0
  : >"$MARKION_TEST_TRACE"
  env MARKION_SOFTWARE_VULKAN="$mode" MARKION_TEST_PHASE="$phase" \
    MARKION_TEST_SYSTEM_EXIT="$system_exit" MARKION_TEST_SOFTWARE_EXIT="$software_exit" \
    VK_DRIVER_FILES=/host-driver.json VK_INSTANCE_LAYERS=host-layer \
    VK_LOADER_LAYERS_ENABLE=host-layer VK_LOADER_LAYERS_ALLOW=host-layer \
    "$fixture/bin/markion" -- 'sample document 中文.md' >"$fixture/output" 2>&1 || status=$?
  if [[ "$status" != "$expected_status" || "$(<"$MARKION_TEST_TRACE")" != "$expected_trace" ]]; then
    cat "$fixture/output" "$MARKION_TEST_TRACE" >&2
    printf 'Launcher case %s/%s exited %s, expected %s\n' "$mode" "$phase" "$status" "$expected_status" >&2
    exit 1
  fi
}

run_case auto ready 0 0 0 system
run_case auto initializing 139 0 0 $'system\nsoftware'
run_case auto failed 1 0 0 $'system\nsoftware'
run_case auto failed 1 11 11 $'system\nsoftware'
run_case auto ready 139 0 139 system
run_case auto '' 139 0 139 system
run_case auto initializing 17 0 17 system
run_case 1 initializing 139 0 0 software
run_case 0 initializing 139 0 139 system
printf '%s\n' 'Pacman launcher retry, arguments, and exit behavior verified'
