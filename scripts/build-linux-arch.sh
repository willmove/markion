#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' packager.toml | head -n 1)"
fi
package_release="${2:-${MARKION_PACMAN_PKGREL:-1}}"
if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Expected a stable semantic version, got: ${version}" >&2
  exit 1
fi
if [[ ! "${package_release}" =~ ^[0-9]+$ ]]; then
  echo "Expected a numeric pacman package release, got: ${package_release}" >&2
  exit 1
fi
if [[ "$(uname -m)" != "x86_64" || ! -e /etc/arch-release ]]; then
  echo "The pacman package must be built in an Arch Linux x86_64 userspace" >&2
  exit 1
fi
if (( EUID != 0 )); then
  echo "Run this script as root inside a disposable Arch container; makepkg itself runs unprivileged" >&2
  exit 1
fi

deb="${repo_root}/dist/markion_${version}_amd64.deb"
if [[ ! -f "${deb}" ]]; then
  echo "DEB payload not found: ${deb}" >&2
  exit 1
fi

pacman -Syu --noconfirm --needed dpkg xorg-server-xvfb xdotool ttf-dejavu
for tool in dpkg-deb makepkg pacman bsdtar runuser; do
  command -v "${tool}" >/dev/null || {
    echo "Required Arch packaging tool is unavailable: ${tool}" >&2
    exit 1
  }
done

workdir="$(mktemp -d)"
cleanup() { rm -rf "${workdir}"; }
trap cleanup EXIT
mkdir -p "${workdir}/payload" "${workdir}/out"
dpkg-deb -x "${deb}" "${workdir}/payload"
test -x "${workdir}/payload/usr/bin/markion"
install -d "${workdir}/payload/usr/lib/markion"
mv "${workdir}/payload/usr/bin/markion" "${workdir}/payload/usr/lib/markion/markion"
install -m 0755 scripts/markion-pacman-launcher.sh "${workdir}/payload/usr/bin/markion"

cat >"${workdir}/PKGBUILD" <<EOF
pkgname=markion
pkgver=${version}
pkgrel=${package_release}
pkgdesc='A Rust + GPUI Markdown editor with live preview, themes, and i18n'
arch=('x86_64')
url='https://github.com/willmove/markion'
license=('MIT')
depends=(
  'bash'
  'coreutils'
  'wayland'
  'libxkbcommon'
  'libxkbcommon-x11'
  'libxcb'
  'vulkan-icd-loader'
  'vulkan-swrast'
  'fontconfig'
  'alsa-lib'
  'glib2'
)
options=('!strip' '!debug')

package() {
  cp -a "\${startdir}/payload/." "\${pkgdir}/"
}
EOF

builder=markion-builder
if ! id -u "${builder}" >/dev/null 2>&1; then
  useradd --create-home --shell /bin/bash "${builder}"
fi
chown -R "${builder}:${builder}" "${workdir}"
(
  cd "${workdir}"
  runuser -u "${builder}" -- env \
    PKGDEST="${workdir}/out" \
    PKGEXT='.pkg.tar.zst' \
    makepkg --nodeps --noconfirm --force
)

package="${workdir}/out/markion-${version}-${package_release}-x86_64.pkg.tar.zst"
if [[ ! -f "${package}" ]]; then
  echo "makepkg did not produce the expected package: ${package}" >&2
  exit 1
fi

pkginfo="$(bsdtar -xOf "${package}" .PKGINFO)"
for expected in \
  "pkgname = markion" \
  "pkgver = ${version}-${package_release}" \
  "arch = x86_64"
do
  grep -Fqx "${expected}" <<<"${pkginfo}" || {
    echo "Pacman metadata is missing: ${expected}" >&2
    exit 1
  }
done
for dependency in \
  bash \
  coreutils \
  wayland \
  libxkbcommon \
  libxkbcommon-x11 \
  libxcb \
  vulkan-icd-loader \
  vulkan-swrast \
  fontconfig \
  alsa-lib \
  glib2
do
  grep -Fqx "depend = ${dependency}" <<<"${pkginfo}" || {
    echo "Pacman dependency is missing: ${dependency}" >&2
    exit 1
  }
done

pacman -U --noconfirm "${package}"
pacman -Q markion
pacman -Q vulkan-swrast
pacman -Qk markion
installed_paths="$(pacman -Ql markion | awk '{print $2}')"
grep -Fqx /usr/bin/markion <<<"${installed_paths}" || {
  echo "Installed package is missing: /usr/bin/markion" >&2
  exit 1
}
mapfile -t desktop_files < <(grep -E '/usr/share/applications/[^/]+\.desktop$' <<<"${installed_paths}")
if (( ${#desktop_files[@]} != 1 )); then
  echo "Expected exactly one installed desktop launcher, found ${#desktop_files[@]}" >&2
  exit 1
fi
desktop_file="${desktop_files[0]}"
grep -Eq '^Exec=.*markion([[:space:]]|$)' "${desktop_file}" || {
  echo "Desktop launcher does not execute markion: ${desktop_file}" >&2
  exit 1
}
echo "Verified installed desktop launcher: ${desktop_file}"
for pattern in \
  '^/usr/lib/markion/markion$' \
  '/usr/share/icons/.*\.png$' \
  '/assets/examples/README\.md$' \
  '/assets/marknice-workspace/bundle-manifest\.json$'
do
  grep -Eq "${pattern}" <<<"${installed_paths}" || {
    echo "Installed package has no path matching: ${pattern}" >&2
    exit 1
  }
done

manifest="/$(pacman -Ql markion | awk '{print $2}' | sed -n 's#^/\(.*assets/marknice-workspace/bundle-manifest\.json\)$#\1#p' | head -n 1)"
test -f "${manifest}"
cmp scripts/markion-pacman-launcher.sh /usr/bin/markion
cmp "${workdir}/payload/usr/lib/markion/markion" /usr/lib/markion/markion
cmp "${workdir}/payload${manifest}" "${manifest}"

bash scripts/test-pacman-launcher.sh /usr/bin/markion
runuser -u "${builder}" -- xvfb-run -a -s '-screen 0 1024x768x24' bash scripts/smoke-test-pacman.sh

mkdir -p dist
cp -f "${package}" "dist/$(basename "${package}")"
echo "Wrote dist/$(basename "${package}") and verified installation in Arch Linux"
