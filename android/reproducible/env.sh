#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"

export NDK_PATH="${NDK_PATH:?NDK_PATH must point to the Android NDK}"

if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

case "$(uname -s)" in
    Linux)
        ndk_host_tag="linux-x86_64"
        ;;
    Darwin)
        ndk_host_tag="darwin-x86_64"
        ;;
    *)
        echo "Unsupported host for Android NDK: $(uname -s)" >&2
        exit 1
        ;;
esac

export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export ANDROID_NDK_HOME="$NDK_PATH"
export ANDROID_NDK_ROOT="$NDK_PATH"
export NDK_HOME="$NDK_PATH"
export NDK_TOOLCHAIN_DIR="$NDK_PATH/toolchains/llvm/prebuilt/$ndk_host_tag/bin"

export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"

export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android28-clang"
export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7a-linux-androideabi28-clang"

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$CC_armv7_linux_androideabi"
export GEMSTONE_ANDROID_ABIS="${GEMSTONE_ANDROID_ABIS:-arm64-v8a,armeabi-v7a}"

home_prefix="${REPRO_BUILD_HOME_PREFIX:-BUILD_HOME}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=$home_prefix --remap-path-prefix=$repo_root=."
