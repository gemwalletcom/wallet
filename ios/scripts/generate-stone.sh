#!/usr/bin/env bash
#
# Build the Gemstone XCFramework, skipping when Rust sources are unchanged.
# Usage: generate-stone.sh [--force] [release]

set -euo pipefail

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

CURRENT_DIR=$(dirname "$(realpath "$0")")
IOS_DIR="$CURRENT_DIR/.."
PROJ_PATH="$IOS_DIR/Gem.xcodeproj/project.pbxproj"
CORE_DIR="$IOS_DIR/../core"
STONE_DIR="$CORE_DIR/gemstone"
PACKAGES_DIR="$IOS_DIR/Packages/Gemstone"
XCFRAMEWORK="$PACKAGES_DIR/Sources/GemstoneFFI.xcframework"

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

compute_hash() {
    {
        git -C "$CORE_DIR" rev-parse HEAD
        git -C "$CORE_DIR" diff
        git -C "$CORE_DIR" ls-files --others --exclude-standard -z | \
            (cd "$CORE_DIR" && xargs -0 stat -f '%m %z %N' 2>/dev/null)
    } | shasum -a 256 | cut -d' ' -f1
}

CACHE_DIR="$IOS_DIR/build/.gemstone-cache"
mkdir -p "$CACHE_DIR"
HASH_FILE="$CACHE_DIR/sources-${BUILD_MODE:-debug}.hash"

current_hash=$(compute_hash)

if [ "$FORCE" -eq 0 ] && [ -f "$HASH_FILE" ] && [ -d "$XCFRAMEWORK" ]; then
    cached_hash=$(cat "$HASH_FILE")
    if [ "$current_hash" = "$cached_hash" ]; then
        echo "note: Gemstone sources unchanged (${BUILD_MODE:-debug}) — skipping rebuild"
        exit 0
    fi
fi

read_deployment_target() {
    echo -n $(/usr/libexec/PlistBuddy -c "Print" "$PROJ_PATH" | grep IPHONEOS_DEPLOYMENT_TARGET | awk -F ' = ' '{print $2}' | uniq)
}

if [ "$FORCE" -eq 1 ]; then
    echo "note: Forcing Gemstone rebuild (${BUILD_MODE:-debug})..."
else
    echo "note: Gemstone sources changed — rebuilding XCFramework (${BUILD_MODE:-debug})..."
fi

pushd "$STONE_DIR" > /dev/null
BUILD_MODE=$BUILD_MODE IPHONEOS_DEPLOYMENT_TARGET=$(read_deployment_target) just build-ios
mkdir -p "$PACKAGES_DIR"
ditto target/spm "$PACKAGES_DIR"
popd > /dev/null

echo "$current_hash" > "$HASH_FILE"

echo "note: Gemstone XCFramework rebuilt successfully (${BUILD_MODE:-debug})"
