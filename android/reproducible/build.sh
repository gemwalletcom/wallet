#!/usr/bin/env bash
set -euo pipefail

channel="${1:?usage: build.sh <channel>}"
variant="${channel^}Release"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

case "$channel" in
    google) gradle_arguments=(":app:bundle$variant") ;;
    *) gradle_arguments=(-PuseLegacyPackaging=true ":app:assemble$variant") ;;
esac

source "$HOME/.cargo/env"

export BUILD_MODE=release
export ANDROID_NDK_HOME="${NDK_PATH:?NDK_PATH must point to the Android NDK}"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=FDROID_HOME --remap-path-prefix=$repo_root=."

touch local.properties

rm -rf "$HOME/.gradle/caches/transforms-"*
"${GRADLE_COMMAND:-gradle}" --no-daemon --console plain -Pchannel="$channel" "${gradle_arguments[@]}"
