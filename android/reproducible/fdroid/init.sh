#!/usr/bin/env bash
set -euxo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$script_dir/../versions.sh"

export NDK_PATH="${NDK_PATH:?NDK_PATH must point to the Android NDK}"

if command -v rustup >/dev/null 2>&1; then
    rustup toolchain install "$RUST_VERSION" --profile minimal
    rustup default "$RUST_VERSION"
else
    curl -sfL https://sh.rustup.rs -o /tmp/rustup.sh
    chmod +x /tmp/rustup.sh
    /tmp/rustup.sh -y --profile minimal --default-toolchain "$RUST_VERSION"
fi

source "$HOME/.cargo/env"
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi

cargo install --locked "cargo-ndk@$CARGO_NDK_VERSION"
mkdir -p "$HOME/.cargo"
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$script_dir/cargo-config.toml.template" > "$HOME/.cargo/config.toml"
