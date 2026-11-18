#!/usr/bin/env bash
set -euo pipefail

[ "$#" -gt 0 ] || { echo "usage: docker.sh <channel>..." >&2; exit 1; }

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
outputs="$root/android/app/build/outputs"
ndk_version="$(git -C "$root" show HEAD:android/gradle/libs.versions.toml | sed -n 's/^androidNdk = "\(.*\)"$/\1/p')"
: "${ndk_version:?androidNdk is missing from android/gradle/libs.versions.toml}"

mkdir -p "$outputs"

build() {
    local platform="$1"
    local channel="$2"
    local image="gem-wallet-android-build-${platform#linux/}"

    git -C "$root" archive --format=tar HEAD android/reproducible android/gradlew android/gradle/wrapper rust-toolchain.toml \
        | docker build --platform "$platform" --build-arg NDK_VERSION="$ndk_version" --file android/reproducible/Dockerfile --tag "$image" -

    echo "Building $channel unsigned on $platform from $(git -C "$root" rev-parse HEAD)"
    git -C "$root" -c tar.umask=0022 archive --format=tar HEAD \
        | docker run --rm --init --interactive --platform "$platform" --volume "$outputs:/outputs" --env UPDATE_URL "$image" "$channel"
}

for channel in "$@"; do
    case "$channel" in
        fdroid) build linux/amd64 "$channel" ;;
        *) build linux/arm64 "$channel" ;;
    esac
done
