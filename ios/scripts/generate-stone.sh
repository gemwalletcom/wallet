#!/usr/bin/env bash
#
# Generate Gemstone UniFFI sources and iOS Rust static libraries.
# Usage:
#   generate-stone.sh [--force] [release]
#

set -euo pipefail

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
unset MACOSX_DEPLOYMENT_TARGET TVOS_DEPLOYMENT_TARGET WATCHOS_DEPLOYMENT_TARGET XROS_DEPLOYMENT_TARGET
unset SWIFT_DEBUG_INFORMATION_FORMAT SWIFT_DEBUG_INFORMATION_VERSION

CURRENT_DIR=$(dirname "$(realpath "$0")")
IOS_DIR="$CURRENT_DIR/.."
PROJ_PATH="$IOS_DIR/Gem.xcodeproj/project.pbxproj"
CORE_DIR="$IOS_DIR/../core"
STONE_DIR="$CORE_DIR/gemstone"
PACKAGES_DIR="$IOS_DIR/Packages/Gemstone"
SWIFT_BINDINGS="$PACKAGES_DIR/Sources/Gemstone/Gemstone.swift"
FFI_HEADER="$PACKAGES_DIR/Sources/GemstoneFFI/include/GemstoneFFI.h"

FORCE=0
POSITIONAL=()
for arg in "$@"; do
    case "$arg" in
        --force) FORCE=1 ;;
        *) POSITIONAL+=("$arg") ;;
    esac
done

BUILD_MODE="${POSITIONAL[0]:-${BUILD_MODE:-}}"
if [ -z "$BUILD_MODE" ] && [ "${CONFIGURATION:-Debug}" = "Release" ]; then
    BUILD_MODE="release"
fi

core_source_fingerprint() {
    {
        shasum -a 256 "$0"
        git -C "$CORE_DIR" rev-parse HEAD
        git -C "$CORE_DIR" diff
        git -C "$CORE_DIR" diff --cached
        git -C "$CORE_DIR" ls-files --others --exclude-standard -z | \
            (cd "$CORE_DIR" && xargs -0 stat -f '%m %z %N' 2>/dev/null || true)
    }
}

gemstone_hash() {
    {
        core_source_fingerprint
        printf '%s\n' "$@"
    } | shasum -a 256 | cut -d' ' -f1
}

read_deployment_target() {
    /usr/libexec/PlistBuddy -c "Print" "$PROJ_PATH" | awk -F ' = ' '/IPHONEOS_DEPLOYMENT_TARGET/ { print $2; exit }'
}

generate_bindings() {
    local cache_dir="$IOS_DIR/build/.gemstone-cache"
    local hash_file="$cache_dir/sources-${BUILD_MODE:-debug}.hash"
    local current_hash
    mkdir -p "$cache_dir"
    current_hash=$(gemstone_hash "${BUILD_MODE:-debug}")

    if [ "$FORCE" -eq 0 ] && [ -f "$hash_file" ] && [ -f "$SWIFT_BINDINGS" ] && [ -f "$FFI_HEADER" ]; then
        if [ "$current_hash" = "$(cat "$hash_file")" ]; then
            echo "note: Gemstone UniFFI sources unchanged (${BUILD_MODE:-debug}) - skipping regeneration"
            return 0
        fi
    fi

    if [ "$FORCE" -eq 1 ]; then
        echo "note: Forcing Gemstone UniFFI source regeneration (${BUILD_MODE:-debug})..."
    else
        echo "note: Gemstone sources changed - regenerating UniFFI sources (${BUILD_MODE:-debug})..."
    fi

    pushd "$STONE_DIR" > /dev/null
    env -u MACOSX_DEPLOYMENT_TARGET -u TVOS_DEPLOYMENT_TARGET -u WATCHOS_DEPLOYMENT_TARGET -u XROS_DEPLOYMENT_TARGET \
        -u SWIFT_DEBUG_INFORMATION_FORMAT -u SWIFT_DEBUG_INFORMATION_VERSION \
        BUILD_MODE=$BUILD_MODE IPHONEOS_DEPLOYMENT_TARGET=$(read_deployment_target) just bindgen-swift
    mkdir -p "$PACKAGES_DIR/Sources/Gemstone" "$PACKAGES_DIR/Sources/GemstoneFFI/include"
    cp generated/swift/gemstone.swift "$SWIFT_BINDINGS"
    cp generated/swift/GemstoneFFI.h "$FFI_HEADER"
    popd > /dev/null

    echo "$current_hash" > "$hash_file"
    echo "note: Gemstone UniFFI sources regenerated successfully (${BUILD_MODE:-debug})"
}

build_ios_static_libraries() {
    local profile="debug"
    local clean_profile_flag="--profile dev"
    local build_flag=""
    local host_arch
    host_arch="$(uname -m)"

    if [ "${BUILD_MODE:-}" = "release" ]; then
        profile="release"
        clean_profile_flag="--release"
        build_flag="--release"
    fi

    if [ "$host_arch" != "arm64" ]; then
        echo "error: Intel Macs are not supported for iOS Gemstone builds." >&2
        exit 1
    fi

    echo "note: Cleaning Gemstone iOS target artifacts ($profile)"
    for rust_target in aarch64-apple-ios-sim aarch64-apple-ios; do
        cargo clean --manifest-path "$STONE_DIR/Cargo.toml" --target "$rust_target" --package gemstone ${clean_profile_flag} --quiet
    done

    echo "note: Building Gemstone iOS static libraries ($profile)"
    for rust_target in aarch64-apple-ios-sim aarch64-apple-ios; do
        env -u MACOSX_DEPLOYMENT_TARGET -u TVOS_DEPLOYMENT_TARGET -u WATCHOS_DEPLOYMENT_TARGET -u XROS_DEPLOYMENT_TARGET \
            -u SWIFT_DEBUG_INFORMATION_FORMAT -u SWIFT_DEBUG_INFORMATION_VERSION \
            IPHONEOS_DEPLOYMENT_TARGET="$(read_deployment_target)" \
            cargo rustc --manifest-path "$STONE_DIR/Cargo.toml" --target "$rust_target" --lib ${build_flag} --crate-type staticlib
    done

    echo "note: Gemstone iOS static libraries ready ($profile)"
}

generate_bindings
build_ios_static_libraries
