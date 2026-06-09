#!/usr/bin/env bash
set -euxo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

curl -sfL https://sh.rustup.rs -o /tmp/rustup.sh
chmod +x /tmp/rustup.sh
/tmp/rustup.sh -y --profile minimal --default-toolchain 1.91.0

# shellcheck source=/dev/null
source "$HOME/.cargo/env"
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi

cargo install --locked cargo-ndk@4.1.2
mkdir -p "$HOME/.cargo"
sed -e "s|{NDK_PATH}|$NDK_PATH|g" "$script_dir/cargo-config.toml.template" > "$HOME/.cargo/config.toml"
