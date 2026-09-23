#!/usr/bin/env bash
# Build a Linux x86_64 RPM from the cargo-packager DEB payload so the RPM keeps
# the exact install layout Markion's resource discovery already understands.
# cargo-packager 0.11.x cannot emit RPM directly (no PackageFormat::Rpm).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' packager.toml | head -n 1)"
fi
if [[ -z "${version}" ]]; then
  echo "Unable to determine package version" >&2
  exit 1
fi

deb="dist/markion_${version}_amd64.deb"
if [[ ! -f "${deb}" ]]; then
  echo "DEB payload not found: ${deb}" >&2
  exit 1
fi

workdir="$(mktemp -d)"
cleanup() { rm -rf "${workdir}"; }
trap cleanup EXIT

mkdir -p "${workdir}/payload" "${workdir}/rpmbuild"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
dpkg-deb -x "${deb}" "${workdir}/payload"

# Preserve the DEB file tree as the RPM install root. The spec only packages
# whatever dpkg-deb extracted, including usr/bin, usr/lib, and usr/share.
spec="${workdir}/rpmbuild/SPECS/markion.spec"
cat >"${spec}" <<EOF
Name:           markion
Version:        ${version}
Release:        1%{?dist}
Summary:        A Rust + GPUI Markdown editor with live preview, themes, and i18n
License:        MIT
URL:            https://github.com/willmove/markion
BuildArch:      x86_64
AutoReqProv:    no

# Fedora/RHEL-style runtime libraries matching packager.toml [deb] depends.
Requires:       wayland
Requires:       libxkbcommon
Requires:       libxcb
Requires:       vulkan-loader
Requires:       fontconfig
Requires:       alsa-lib
Requires:       glib2

%description
Markion is a Rust + GPUI Markdown editor with live preview, themes, and i18n.

%install
mkdir -p %{buildroot}
cp -a "${workdir}/payload/." %{buildroot}/

%files
%defattr(-,root,root,-)
/
EOF

rpmbuild \
  --define "_topdir ${workdir}/rpmbuild" \
  --define "_build_id_links none" \
  --buildroot "${workdir}/rpmbuild/BUILDROOT" \
  -bb "${spec}"

rpm_path="$(find "${workdir}/rpmbuild/RPMS" -type f -name '*.rpm' | head -n 1)"
if [[ -z "${rpm_path}" ]]; then
  echo "rpmbuild produced no RPM" >&2
  exit 1
fi

mkdir -p dist
# Stable release-asset name aligned with markion_<version>_x64-setup.exe.
out="dist/markion_${version}_x86_64.rpm"
cp -f "${rpm_path}" "${out}"
echo "Wrote ${out}"
