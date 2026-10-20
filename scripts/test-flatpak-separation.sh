#!/usr/bin/env bash

set -euo pipefail
unset LD_PRELOAD

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
keep_build_root="${CALIX_FLATPAK_KEEP_BUILD_ROOT:-0}"
owns_build_root=0
if [[ -n "${CALIX_FLATPAK_BUILD_ROOT:-}" ]]; then
    build_root="${CALIX_FLATPAK_BUILD_ROOT}"
else
    build_root="$(mktemp -d /tmp/calix-flatpak-separation.XXXXXX)"
    owns_build_root=1
fi

cleanup() {
    if [[ "${owns_build_root}" != "1" || "${keep_build_root}" == "1" ]]; then
        return
    fi
    case "${build_root}" in
        /tmp/calix-flatpak-separation.*)
            rm -rf -- "${build_root}"
            ;;
        *)
            echo "warning: refusing to remove unexpected build root: ${build_root}" >&2
            ;;
    esac
}
trap cleanup EXIT

mkdir -p \
    "${build_root}/backend" \
    "${build_root}/cli" \
    "${build_root}/cache" \
    "${build_root}/global-cache"

zig_common=(
    --cache-dir "${build_root}/cache"
    --global-cache-dir "${build_root}/global-cache"
)

(
    cd "${repo_root}/Calix.Flatpak.Backend"
    zig build "${zig_common[@]}" \
        --prefix "${build_root}/backend" \
        -Dcpu=baseline \
        -Doptimize=ReleaseSafe
    zig build test "${zig_common[@]}"
    zig build abi-test "${zig_common[@]}"
    zig build integration-test "${zig_common[@]}"
)

(
    cd "${repo_root}/Calix.PackageManager"
    zig build flatpak-test "${zig_common[@]}"
)

(
    cd "${repo_root}/Calix.Cli.Zig"
    zig build "${zig_common[@]}" \
        --prefix "${build_root}/cli" \
        -Dcpu=baseline \
        -Doptimize=ReleaseSmall
    zig build test "${zig_common[@]}"
)

"${repo_root}/scripts/check-flatpak-separation.sh" \
    "${build_root}/cli/bin/calix" \
    "${build_root}/backend/lib/libcalix-flatpak-backend.so.1"

"${build_root}/cli/bin/calix" --help >/dev/null
"${build_root}/cli/bin/calix" --version >/dev/null
"${build_root}/cli/bin/calix" utility --completions bash >/dev/null

echo "Flatpak separation build and core-only smoke tests passed."
