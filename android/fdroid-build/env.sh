#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"

# shellcheck source=/dev/null
source "$HOME/.cargo/env"

export FDROID_BUILD=true
export ANDROID_REPRODUCIBLE_BUILD=true
export SKIP_SIGN=true
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export NDK_TOOLCHAIN_DIR="$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin"

export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"

export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android28-clang"
export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi28-clang"

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$CC_armv7_linux_androideabi"
export GEMSTONE_ANDROID_ABIS="arm64-v8a,armeabi-v7a"

export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$repo_root=. --remap-path-prefix=$HOME=FDROID_HOME"
