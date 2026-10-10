#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PLATFORM="${1:?usage: maestro-smoke.sh ios|android <device id> [output dir]}"
DEVICE="${2:?usage: maestro-smoke.sh ios|android <device id> [output dir]}"
OUTPUT_DIR="${3:-$ROOT_DIR/build/maestro/$PLATFORM}"
CONSTANTS_FILE="$ROOT_DIR/ios/GemUITestsAppTests/Types/UITestKitConstants.swift"

case "$PLATFORM" in
    ios) APP_ID="com.gemwallet.ios" ;;
    android) APP_ID="com.gemwallet.android" ;;
    *) echo "platform must be ios or android" >&2; exit 1 ;;
esac

TEST_PHRASE="$(sed -nE 's/.*static let words = "(.*)"/\1/p' "$CONSTANTS_FILE")"
BITCOIN_ADDRESS="$(sed -nE 's/.*static let bitcoinAddress = "(.*)"/\1/p' "$CONSTANTS_FILE")"

mkdir -p "$OUTPUT_DIR"
maestro --device "$DEVICE" test \
    -e APP_ID="$APP_ID" \
    -e TEST_PHRASE="$TEST_PHRASE" \
    -e BITCOIN_ADDRESS="$BITCOIN_ADDRESS" \
    --test-output-dir "$OUTPUT_DIR" \
    --format junit \
    --output "$OUTPUT_DIR/report.xml" \
    "$ROOT_DIR/maestro/smoke" < /dev/null
