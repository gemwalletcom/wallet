#!/usr/bin/env bash
set -euxo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$script_dir/versions.sh"
rust_version="$(sed -n 's/^channel = "\(.*\)"$/\1/p' "$script_dir/../../rust-toolchain.toml")"

if command -v rustup >/dev/null 2>&1; then
    rustup toolchain install "$rust_version" --profile minimal
    rustup default "$rust_version"
else
    curl -sfL https://sh.rustup.rs -o /tmp/rustup.sh
    chmod +x /tmp/rustup.sh
    /tmp/rustup.sh -y --profile minimal --default-toolchain "$rust_version"
fi

source "$HOME/.cargo/env"
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi

cargo install --locked "cargo-ndk@$CARGO_NDK_VERSION"
