#!/usr/bin/env bash
#
# Xcode Run Script Build Phase that rebuilds the Gemstone XCFramework
# only when Rust sources have changed.
#

set -euo pipefail

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IOS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CORE_DIR="$(cd "$IOS_DIR/../core" && pwd)"
PACKAGES_DIR="$IOS_DIR/Packages/Gemstone"
XCFRAMEWORK="$PACKAGES_DIR/Sources/GemstoneFFI.xcframework"

if [ "${CONFIGURATION:-Debug}" = "Release" ]; then
    BUILD_MODE="release"
else
    BUILD_MODE=""
fi

CACHE_DIR="$IOS_DIR/build/.gemstone-cache"
HASH_FILE="$CACHE_DIR/sources-${BUILD_MODE:-debug}.hash"

current_hash=$("$SCRIPT_DIR/gemstone-hash.sh" "$CORE_DIR")

# Skip rebuild if sources unchanged and xcframework exists.
if [ -f "$HASH_FILE" ] && [ -d "$XCFRAMEWORK" ]; then
    cached_hash=$(cat "$HASH_FILE")
    if [ "$current_hash" = "$cached_hash" ]; then
        echo "note: Gemstone sources unchanged (${BUILD_MODE:-debug}) — skipping rebuild"
        exit 0
    fi
fi

echo "note: Gemstone sources changed — rebuilding XCFramework (${BUILD_MODE:-debug})..."

"$SCRIPT_DIR/generate-stone.sh" "$BUILD_MODE"

echo "$current_hash" > "$HASH_FILE"

echo "note: Gemstone XCFramework rebuilt successfully (${BUILD_MODE:-debug})"
