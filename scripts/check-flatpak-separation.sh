#!/usr/bin/env bash

set -euo pipefail
unset LD_PRELOAD

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cli_path="${1:-${repo_root}/Calix.Cli.Zig/zig-out/bin/calix}"
backend_path="${2:-${repo_root}/Calix.Flatpak.Backend/zig-out/lib/libcalix-flatpak-backend.so.1}"

for tool in readelf nm rg bash; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
        echo "error: ${tool} is required for the Flatpak separation check" >&2
        exit 1
    fi
done

if [[ ! -x "${cli_path}" ]]; then
    echo "error: CLI artifact not found: ${cli_path}" >&2
    exit 1
fi
if [[ ! -e "${backend_path}" ]]; then
    echo "error: Flatpak backend artifact not found: ${backend_path}" >&2
    exit 1
fi

cli_needed="$(readelf -d "${cli_path}" | sed -n 's/.*Shared library: \[\(.*\)\]/\1/p')"
if grep -Eq '^(libflatpak|libostree|libgio|libgobject|libglib)' <<<"${cli_needed}"; then
    echo "error: the base CLI has a forbidden native Flatpak dependency" >&2
    grep -E '^(libflatpak|libostree|libgio|libgobject|libglib)' <<<"${cli_needed}" >&2
    exit 1
fi

backend_dynamic="$(readelf -d "${backend_path}")"
if ! grep -Fq 'Shared library: [libflatpak.so.0]' <<<"${backend_dynamic}"; then
    echo "error: the Flatpak backend does not depend on libflatpak.so.0" >&2
    exit 1
fi
if ! grep -Fq 'Library soname: [libcalix-flatpak-backend.so.1]' <<<"${backend_dynamic}"; then
    echo "error: the Flatpak backend SONAME is not ABI version 1" >&2
    exit 1
fi
if ! nm -D --defined-only "${backend_path}" |
    grep -Eq '[[:space:]]calix_flatpak_backend_get_api$'; then
    echo "error: the Flatpak backend entry point is not exported" >&2
    exit 1
fi

if rg -n --glob '*.zig' \
    'flatpak\.bindings|bindings\.libflatpak|@import\("flatpak"\)|g_object_unref|refToString' \
    "${repo_root}/Calix.PackageManager" \
    "${repo_root}/Calix.Cli.Zig"; then
    echo "error: native Flatpak declarations escaped the backend project" >&2
    exit 1
fi

fake_backend="${repo_root}/Calix.PackageManager/zig-out/test-flatpak-backend/libcalix-flatpak-backend-package-manager-test.so"
if [[ -e "${fake_backend}" ]] &&
    readelf -d "${fake_backend}" | grep -Fq 'Shared library: [libflatpak.so.0]'; then
    echo "error: the test-only fake backend unexpectedly links libflatpak" >&2
    exit 1
fi

bash -n "${repo_root}/PKGBUILD-calix"

echo "Flatpak backend separation checks passed."
