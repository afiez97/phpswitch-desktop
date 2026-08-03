#!/bin/bash
# build-deb.sh — Build a phpswitch .deb package
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION="${1:-1.0.1}"
PKG_NAME="phpswitch_${VERSION}_all"
BUILD_ROOT="${ROOT}/build/${PKG_NAME}"

if ! command -v dpkg-deb &>/dev/null; then
    echo "✗ dpkg-deb not found. Install it with: sudo apt install dpkg-dev" >&2
    exit 1
fi

rm -rf "${ROOT}/build"
mkdir -p \
    "${BUILD_ROOT}/DEBIAN" \
    "${BUILD_ROOT}/usr/bin" \
    "${BUILD_ROOT}/usr/share/doc/phpswitch" \
    "${BUILD_ROOT}/etc/sudoers.d"

install -m 755 "${ROOT}/phpswitch" "${BUILD_ROOT}/usr/bin/phpswitch"
install -m 440 "${ROOT}/debian/phpswitch.sudoers" "${BUILD_ROOT}/etc/sudoers.d/phpswitch"
install -m 644 "${ROOT}/README.md" "${BUILD_ROOT}/usr/share/doc/phpswitch/README.md"

sed "s/^Version:.*/Version: ${VERSION}/" "${ROOT}/debian/control" > "${BUILD_ROOT}/DEBIAN/control"
install -m 755 "${ROOT}/debian/postinst" "${BUILD_ROOT}/DEBIAN/postinst"
install -m 755 "${ROOT}/debian/postrm" "${BUILD_ROOT}/DEBIAN/postrm"
install -m 644 "${ROOT}/debian/conffiles" "${BUILD_ROOT}/DEBIAN/conffiles"

dpkg-deb --build --root-owner-group "${BUILD_ROOT}" "${ROOT}/build/${PKG_NAME}.deb"

echo ""
echo "✓ Built ${ROOT}/build/${PKG_NAME}.deb"
echo ""
echo "Install with:"
echo "  sudo dpkg -i ${ROOT}/build/${PKG_NAME}.deb"
echo "  sudo apt-get install -f   # if dependencies are missing"
