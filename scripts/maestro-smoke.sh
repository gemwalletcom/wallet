#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PLATFORM="${1:?usage: maestro-smoke.sh ios|android [device id]}"
DEVICE="${2:-}"
OUTPUT_DIR="${MAESTRO_OUTPUT:-$ROOT_DIR/build/maestro/$PLATFORM}"
SIMULATOR="Gem Maestro"
AVD="Gem_Maestro"
ANDROID_SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
EMULATOR="$ANDROID_SDK/emulator/emulator"
PROXYMAN_CA="$HOME/Library/Application Support/com.proxyman.NSProxy/app-data/proxyman-ca.pem"

constant() {
    sed -nE "s/.*static let $1 = \"(.*)\"/\1/p" "$ROOT_DIR/ios/GemUITestsAppTests/Types/UITestKitConstants.swift"
}

newest() {
    sort -V | tail -1
}

ios_device() {
    local runtime udid
    runtime="$(xcrun simctl list runtimes | grep -E '^iOS [0-9.]+ ' | grep -v unavailable | newest)"
    udid="$(xcrun simctl list devices | sed -n "/^-- ${runtime%% (*} --/,/^-- /p" | grep -F "$SIMULATOR (" | grep -oE '[0-9A-F-]{36}' | head -1 || true)"
    [ -n "$udid" ] || udid="$(xcrun simctl create "$SIMULATOR" "$(xcrun simctl list devicetypes | grep -E '^iPhone [0-9]+ Pro \(' | newest | grep -oE 'com\.apple[^)]+')" "${runtime##* - }")"
    xcrun simctl boot "$udid" 2>/dev/null || true
    xcrun simctl bootstatus "$udid" -b >/dev/null
    xcrun simctl install "$udid" "$BUILD"
    echo "$udid"
}

android_serial() {
    for serial in $(adb devices | awk '/^emulator-/ { print $1 }'); do
        [ "$(adb -s "$serial" emu avd name | head -1 | tr -d '\r')" != "$AVD" ] || echo "$serial"
    done
}

android_device() {
    local serial
    if [ -z "$(android_serial)" ]; then
        "$EMULATOR" -list-avds | grep -qx "$AVD" || echo no | "$ANDROID_SDK/cmdline-tools/latest/bin/avdmanager" create avd -n "$AVD" -d pixel_9 \
            -k "$(cd "$ANDROID_SDK" && ls -d system-images/android-*/google_apis/arm64-v8a | newest | tr / ';')" >/dev/null
        nohup "$EMULATOR" -avd "$AVD" -no-window -no-audio -no-boot-anim -no-snapshot-save >/dev/null 2>&1 &
    fi
    until serial="$(android_serial)"; [ -n "$serial" ]; do sleep 2; done
    adb -s "$serial" wait-for-device shell 'while [ "$(getprop sys.boot_completed)" != 1 ]; do sleep 1; done'
    adb -s "$serial" install -r "$BUILD" >/dev/null
    echo "$serial"
}

case "$PLATFORM" in
    ios) APP_ID="com.gemwallet.ios"; BUILD="$ROOT_DIR/ios/build/DerivedData/Build/Products/Debug-iphonesimulator/Gem.app" ;;
    android) APP_ID="com.gemwallet.android"; BUILD="$ROOT_DIR/android/app/build/outputs/apk/google/debug/app-google-debug.apk" ;;
    *) echo "platform must be ios or android" >&2; exit 1 ;;
esac

LOCK="/tmp/gem-maestro-smoke.lock"
mkdir "$LOCK" 2>/dev/null || { echo "another maestro-smoke run is in progress (remove $LOCK if it is stale)" >&2; exit 1; }
trap 'rmdir "$LOCK"' EXIT

if [ -z "$DEVICE" ]; then
    [ -e "$BUILD" ] || { echo "no build at $BUILD, run: just $PLATFORM build" >&2; exit 1; }
    DEVICE="$("${PLATFORM}_device")"
    AUTOMATION_DEVICE=1
fi

mkdir -p "$OUTPUT_DIR"
failed=""
for flow in "$ROOT_DIR"/maestro/smoke/*.yaml; do
    name="$(basename "$flow" .yaml)"
    echo "==> $name"
    if [ "$PLATFORM" = ios ] && [ -n "${AUTOMATION_DEVICE:-}" ]; then
        xcrun simctl keychain "$DEVICE" reset
        [ ! -f "$PROXYMAN_CA" ] || xcrun simctl keychain "$DEVICE" add-root-cert "$PROXYMAN_CA"
    fi
    maestro --device "$DEVICE" test \
        -e APP_ID="$APP_ID" \
        -e TEST_PHRASE="$(constant words)" \
        -e BITCOIN_ADDRESS="$(constant bitcoinAddress)" \
        --no-reinstall-driver \
        --test-output-dir "$OUTPUT_DIR" \
        "$flow" < /dev/null || failed="$failed $name"
done
[ -z "$failed" ] || { echo "Failed:$failed" >&2; exit 1; }
