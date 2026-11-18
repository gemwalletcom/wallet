#!/usr/bin/env bash
set -euo pipefail

[ "$#" -gt 0 ] || { echo "usage: docker.sh <channel>..." >&2; exit 1; }

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
outputs="$root/android/app/build/outputs"
image="gem-wallet-android-build"
ndk_version="$(git -C "$root" show HEAD:android/gradle/libs.versions.toml | sed -n 's/^androidNdk = "\(.*\)"$/\1/p')"
: "${ndk_version:?androidNdk is missing from android/gradle/libs.versions.toml}"

mkdir -p "$outputs"

git -C "$root" archive --format=tar HEAD android/reproducible rust-toolchain.toml \
    | docker build --platform linux/amd64 --build-arg NDK_VERSION="$ndk_version" --file android/reproducible/Dockerfile --tag "$image" -

echo "Building $* unsigned from $(git -C "$root" rev-parse HEAD)"
git -C "$root" -c tar.umask=0022 archive --format=tar HEAD \
    | docker run --rm --init --interactive --platform linux/amd64 --volume "$outputs:/outputs" --env UPDATE_URL "$image" "$@"
