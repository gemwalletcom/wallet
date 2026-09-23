#!/usr/bin/env bash
#
# Generate Gemstone UniFFI sources and iOS Rust static libraries.
# Usage:
#   generate-stone.sh [release]
#

set -euo pipefail

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
unset MACOSX_DEPLOYMENT_TARGET TVOS_DEPLOYMENT_TARGET WATCHOS_DEPLOYMENT_TARGET XROS_DEPLOYMENT_TARGET
unset SWIFT_DEBUG_INFORMATION_FORMAT SWIFT_DEBUG_INFORMATION_VERSION

IOS_DIR=$(dirname "$(dirname "$(realpath "$0")")")
PROJ_PATH="$IOS_DIR/Gem.xcodeproj/project.pbxproj"
CORE_DIR=$(dirname "$IOS_DIR")/core
STONE_DIR="$CORE_DIR/gemstone"
PACKAGES_DIR="$IOS_DIR/Packages/Gemstone"
SWIFT_BINDINGS="$PACKAGES_DIR/Sources/Gemstone/Gemstone.swift"
FFI_HEADER="$PACKAGES_DIR/Sources/GemstoneFFI/include/GemstoneFFI.h"

BUILD_MODE="${1:-${BUILD_MODE:-}}"
if [ -z "$BUILD_MODE" ] && [ "${CONFIGURATION:-Debug}" = "Release" ]; then
    BUILD_MODE="release"
fi

PROFILE="debug"
BUILD_FLAG=""
if [ "$BUILD_MODE" = "release" ]; then
    PROFILE="release"
    BUILD_FLAG="--release"
fi

case "${PLATFORM_NAME:-}" in
    iphoneos)
        DEFAULT_TARGETS="aarch64-apple-ios"
        ;;
    iphonesimulator)
        DEFAULT_TARGETS="aarch64-apple-ios-sim"
        ;;
    *)
        if [ "$BUILD_MODE" = "release" ]; then
            DEFAULT_TARGETS="aarch64-apple-ios"
        else
            DEFAULT_TARGETS="aarch64-apple-ios-sim"
        fi
        ;;
esac
TARGETS="${GEMSTONE_IOS_TARGETS:-$DEFAULT_TARGETS}"

read_deployment_target() {
    /usr/libexec/PlistBuddy -c "Print" "$PROJ_PATH" | awk -F ' = ' '/IPHONEOS_DEPLOYMENT_TARGET/ { print $2; exit }'
}

build_ios_static_libraries() {
    if [ "$(uname -m)" != "arm64" ]; then
        echo "error: Intel Macs are not supported for iOS Gemstone builds." >&2
        exit 1
    fi

    echo "note: Building Gemstone iOS static libraries ($PROFILE: $TARGETS)"
    for rust_target in $TARGETS; do
        CARGO_PROFILE_RELEASE_LTO=fat \
            CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
            IPHONEOS_DEPLOYMENT_TARGET="$(read_deployment_target)" \
            cargo rustc --manifest-path "$STONE_DIR/Cargo.toml" --target "$rust_target" --lib ${BUILD_FLAG} --crate-type staticlib
    done
}

copy_if_changed() {
    cmp -s "$1" "$2" || cp "$1" "$2"
}

generate_bindings() {
    local library="$CORE_DIR/target/${TARGETS%% *}/$PROFILE/libgemstone.a"
    local generated_dir="$STONE_DIR/generated/swift"

    echo "note: Generating Gemstone UniFFI sources from $library"
    (cd "$STONE_DIR" && cargo run --quiet -p uniffi-bindgen -- generate --language swift --crate gemstone --no-format -o "$generated_dir" "$library")
    mkdir -p "$(dirname "$SWIFT_BINDINGS")" "$(dirname "$FFI_HEADER")"
    copy_if_changed "$generated_dir/gemstone.swift" "$SWIFT_BINDINGS"
    copy_if_changed "$generated_dir/GemstoneFFI.h" "$FFI_HEADER"
}

build_ios_static_libraries
generate_bindings
