#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
build_root="/private/tmp/gem-wallet-ios"
output="$root/ios/build/reproducible"

echo "Building Gem.app unsigned from $(git -C "$root" rev-parse HEAD) with $(xcodebuild -version | tr '\n' ' ')"

rm -rf "$build_root"
mkdir -m 700 "$build_root"
git -C "$root" archive --format=tar HEAD | tar -xf - -C "$build_root"
git -C "$root" ls-tree -r HEAD | awk '$2 == "commit" { print $3, $4 }' | while read -r commit path; do
    git -C "$root/$path" archive --format=tar "$commit" | tar -xf - -C "$build_root/$path"
done

cd "$build_root/ios"
PATH="$HOME/.cargo/bin:$PATH"
rustup target add aarch64-apple-ios
rustc="$(rustup which rustc)"
rust_sources="$("$rustc" --print sysroot)/lib/rustlib/src/rust"
rust_commit="$("$rustc" -vV | sed -n 's/^commit-hash: //p')"
rust_flags="--remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=$rust_sources=/rustc/$rust_commit"

build() {
    env -i HOME="$HOME" PATH="$PATH" USER="$USER" TMPDIR="${TMPDIR:-/tmp}" LANG=en_US.UTF-8 \
        ${DEVELOPER_DIR:+DEVELOPER_DIR="$DEVELOPER_DIR"} \
        CARGO_TARGET_AARCH64_APPLE_IOS_RUSTFLAGS="$rust_flags" \
        "$@"
}

build just generate-stone release
build just archive-release

rm -rf "$output"
mkdir -p "$output"
cp -R build/DerivedData/Gem.xcarchive/Products/Applications/Gem.app "$output/"
echo "$output/Gem.app"
