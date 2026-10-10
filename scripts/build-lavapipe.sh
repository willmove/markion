#!/usr/bin/env bash
# Build the software Vulkan stack bundled in the Linux AppImage: Mesa's
# lavapipe driver (statically linked against LLVM so it carries no libLLVM
# dependency) plus the Khronos Vulkan loader.
#
# Markion's AppImage only activates this stack when the host Vulkan stack
# cannot initialize GPUI's renderer (no GPU, no Vulkan driver installed, VMs);
# see src/app/renderer_fallback.rs. Hosts with working Vulkan never load it.
#
# Usage: build-lavapipe.sh <output-dir>
#
# Output layout (copied verbatim into <AppDir>/usr/lib/markion-vulkan/):
#   libvulkan.so.1          Vulkan loader
#   libvulkan_lvp.so        lavapipe ICD
#   lvp_icd.x86_64.json     ICD manifest with a manifest-relative library path
#   lib*.so.*               small X11/terminfo libraries the ICD needs that the
#                           AppImage excludelist does not guarantee on the host
#   licenses/               license texts of every bundled component
#   VERSIONS                component versions, for diagnostics
#
# Build host requirements (Ubuntu 22.04, matching the AppImage glibc baseline):
#   apt: build-essential pkg-config python3-pip python3-mako python3-yaml bison
#        flex llvm-15-dev libx11-dev libx11-xcb-dev libxcb1-dev libxcb-dri3-dev
#        libxcb-present-dev libxcb-shm0-dev libxcb-randr0-dev libxcb-sync-dev
#        libxcb-xfixes0-dev libxshmfence-dev libxrandr-dev libwayland-dev
#        libdrm-dev zlib1g-dev cmake ninja-build
#   pip: meson (installed by this script into a private virtualenv)

set -euo pipefail

MESA_VERSION="24.3.4"
MESA_SHA256="e641ae27191d387599219694560d221b7feaa91c900bcec46bf444218ed66025"
VULKAN_LOADER_VERSION="1.3.296"
MESON_VERSION="1.5.2"
LLVM_CONFIG="${LLVM_CONFIG:-llvm-config-15}"

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <output-dir>" >&2
    exit 1
fi

out_dir="$(realpath -m "$1")"
work_dir="$(mktemp -d)"
cleanup() {
    rm -rf "$work_dir"
}
trap cleanup EXIT

jobs="$(nproc)"
llvm_config_path="$(command -v "$LLVM_CONFIG")"
llvm_version="$("$llvm_config_path" --version)"

echo "Building lavapipe: Mesa $MESA_VERSION, LLVM $llvm_version, Vulkan loader $VULKAN_LOADER_VERSION"

python3 -m venv "$work_dir/venv"
"$work_dir/venv/bin/pip" install --quiet "meson==$MESON_VERSION" mako pyyaml
meson="$work_dir/venv/bin/meson"

cd "$work_dir"
curl -fsSL --retry 4 -o mesa.tar.xz "https://archive.mesa3d.org/mesa-$MESA_VERSION.tar.xz"
echo "$MESA_SHA256  mesa.tar.xz" | sha256sum -c -
tar -xJf mesa.tar.xz

cat > llvm.ini <<EOF
[binaries]
llvm-config = '$llvm_config_path'
EOF

# Ubuntu's LLVM is built with Polly linked into its tools, so
# `llvm-config --link-static --libs lto` names -lPolly/-lPollyISL, yet the
# distribution ships Polly only as a loadable plugin. Nothing lavapipe links
# references Polly; empty archives satisfy the linker.
mkdir -p polly-stubs
ar rcs polly-stubs/libPolly.a
ar rcs polly-stubs/libPollyISL.a
link_args="-L$work_dir/polly-stubs"

# Only lavapipe is built. LLVM is linked statically, and every optional host
# dependency that is not needed to rasterize into an X11/Wayland window is
# disabled, so the resulting ICD only needs libraries every desktop has.
"$meson" setup mesa-build "mesa-$MESA_VERSION" \
    --native-file llvm.ini \
    --wrap-mode=default \
    --force-fallback-for=wayland-protocols \
    -Dbuildtype=release \
    -Db_ndebug=true \
    -Dc_link_args="$link_args" \
    -Dcpp_link_args="$link_args" \
    -Dprefix=/usr \
    -Dplatforms=x11,wayland \
    -Dgallium-drivers= \
    -Dvulkan-drivers=swrast \
    -Dvulkan-layers= \
    -Dllvm=enabled \
    -Dshared-llvm=disabled \
    -Dopengl=false \
    -Dglx=disabled \
    -Degl=disabled \
    -Dgbm=disabled \
    -Dgles1=disabled \
    -Dgles2=disabled \
    -Dtools= \
    -Dvideo-codecs= \
    -Dbuild-tests=false \
    -Dvalgrind=disabled \
    -Dlibunwind=disabled \
    -Dlmsensors=disabled \
    -Dxmlconfig=disabled \
    -Dzstd=disabled
ninja -C mesa-build -j "$jobs" \
    src/gallium/targets/lavapipe/libvulkan_lvp.so \
    src/gallium/targets/lavapipe/lvp_icd.x86_64.json

lvp_api_version="$(python3 - "mesa-build/src/gallium/targets/lavapipe/lvp_icd.x86_64.json" <<'PY'
import json, sys
print(json.load(open(sys.argv[1]))["ICD"]["api_version"])
PY
)"

git clone --quiet --depth 1 --branch "v$VULKAN_LOADER_VERSION" \
    https://github.com/KhronosGroup/Vulkan-Headers.git vulkan-headers
git clone --quiet --depth 1 --branch "v$VULKAN_LOADER_VERSION" \
    https://github.com/KhronosGroup/Vulkan-Loader.git vulkan-loader
cmake -S vulkan-headers -B vulkan-headers/build -G Ninja \
    -DCMAKE_INSTALL_PREFIX="$work_dir/vulkan-headers-install" >/dev/null
cmake --build vulkan-headers/build --target install >/dev/null
cmake -S vulkan-loader -B vulkan-loader/build -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DVULKAN_HEADERS_INSTALL_DIR="$work_dir/vulkan-headers-install" \
    -DBUILD_TESTS=OFF \
    -DBUILD_WSI_XCB_SUPPORT=ON \
    -DBUILD_WSI_XLIB_SUPPORT=ON \
    -DBUILD_WSI_WAYLAND_SUPPORT=ON \
    -DUPDATE_DEPS=OFF >/dev/null
cmake --build vulkan-loader/build -j "$jobs" >/dev/null

rm -rf "$out_dir"
mkdir -p "$out_dir/licenses"
install -m 0644 mesa-build/src/gallium/targets/lavapipe/libvulkan_lvp.so "$out_dir/libvulkan_lvp.so"
install -m 0644 "$(readlink -f vulkan-loader/build/loader/libvulkan.so.1)" "$out_dir/libvulkan.so.1"
strip --strip-unneeded "$out_dir/libvulkan_lvp.so" "$out_dir/libvulkan.so.1"

# Every shared library the stack needs is either provided by the host (it is on
# the AppImage excludelist, https://github.com/AppImage/pkg2appimage/blob/master/excludelist,
# so bundling it would be wrong) or bundled next to the ICD. An unclassified
# dependency fails the build instead of silently relying on the host.
host_libs=" libc.so.6 libm.so.6 ld-linux-x86-64.so.2 libgcc_s.so.1 libstdc++.so.6 \
libz.so.1 libdrm.so.2 libxcb.so.1 libX11-xcb.so.1 libxcb-dri3.so.0 libwayland-client.so.0 "
bundled_libs=" libtinfo.so.6 libxcb-randr.so.0 libxcb-present.so.0 libxcb-xfixes.so.0 \
libxcb-sync.so.1 libxcb-shm.so.0 libxshmfence.so.1 "
for lib in "$out_dir/libvulkan_lvp.so" "$out_dir/libvulkan.so.1"; do
    for needed in $(readelf -d "$lib" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p'); do
        if [[ "$host_libs" == *" $needed "* ]]; then
            continue
        elif [[ "$bundled_libs" == *" $needed "* ]]; then
            [[ -f "$out_dir/$needed" ]] && continue
            source_path="$(ldconfig -p | awk -v name="$needed" '$1 == name && /x86-64/ { print $NF; exit }')"
            real_path="$(readlink -f "$source_path")"
            install -m 0644 "$real_path" "$out_dir/$needed"
            package="$(dpkg -S "*/$(basename "$real_path")" | head -n 1 | cut -d: -f1)"
            install -m 0644 "/usr/share/doc/$package/copyright" "$out_dir/licenses/$package-copyright.txt"
        else
            echo "ERROR: $(basename "$lib") needs $needed, which is neither host-provided nor bundled" >&2
            exit 1
        fi
    done
done

# A library_path containing a directory separator is resolved relative to the
# manifest, which keeps the ICD valid at whatever path the AppImage mounts.
cat > "$out_dir/lvp_icd.x86_64.json" <<EOF
{
    "file_format_version": "1.0.1",
    "ICD": {
        "library_path": "./libvulkan_lvp.so",
        "api_version": "$lvp_api_version"
    }
}
EOF

install -m 0644 "mesa-$MESA_VERSION/docs/license.rst" "$out_dir/licenses/mesa-license.rst"
install -m 0644 vulkan-loader/LICENSE.txt "$out_dir/licenses/vulkan-loader-LICENSE.txt"
llvm_license="/usr/share/doc/llvm-${llvm_version%%.*}/copyright"
if [[ -f "$llvm_license" ]]; then
    install -m 0644 "$llvm_license" "$out_dir/licenses/llvm-copyright.txt"
fi

cat > "$out_dir/VERSIONS" <<EOF
mesa=$MESA_VERSION
llvm=$llvm_version
vulkan-loader=$VULKAN_LOADER_VERSION
lavapipe-api-version=$lvp_api_version
EOF

echo "lavapipe stack written to $out_dir:"
ls -l "$out_dir"
