#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PLATFORM="${1:?usage: maestro-smoke.sh ios|android}"
OUTPUT_DIR="${MAESTRO_OUTPUT:-$ROOT_DIR/build/maestro/$PLATFORM}"
EMULATOR_LOG="$OUTPUT_DIR/emulator.log"
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
    udid="$(xcrun simctl list devices available | grep -F "$SIMULATOR (" | grep -oE '[0-9A-F-]{36}' | tail -1 || true)"
    [ -n "$udid" ] || udid="$(xcrun simctl create "$SIMULATOR" "$(xcrun simctl list devicetypes | grep -E '^iPhone [0-9]+ Pro \(' | newest | grep -oE 'com\.apple[^)]+')" "${runtime##* - }")"
    xcrun simctl boot "$udid" 2>/dev/null || true
    xcrun simctl bootstatus "$udid" -b >/dev/null
    xcrun simctl spawn "$udid" defaults write -g AppleLocale en_US
    xcrun simctl install "$udid" "$BUILD"
    echo "$udid"
}

android_serial() {
    for serial in $(adb devices | awk '/^emulator-/ { print $1 }'); do
        [ "$(adb -s "$serial" emu avd name | head -1 | tr -d '\r')" != "$AVD" ] || echo "$serial"
    done
}

android_device() {
    local serial deadline image
    if [ -z "$(android_serial)" ]; then
        [ -x "$EMULATOR" ] || { echo "no emulator at $EMULATOR, ANDROID_HOME=$ANDROID_HOME" >&2; exit 1; }
        if ! "$EMULATOR" -list-avds | grep -qx "$AVD"; then
            image="$(cd "$ANDROID_SDK" && ls -d system-images/android-*/google_apis/* 2>/dev/null | newest || true)"
            [ -n "$image" ] || { echo "no google_apis system image under $ANDROID_SDK/system-images" >&2; exit 1; }
            echo no | "$ANDROID_SDK/cmdline-tools/latest/bin/avdmanager" create avd -n "$AVD" -d pixel_9 -k "$(tr / ';' <<<"$image")" >/dev/null
        fi
        nohup "$EMULATOR" -avd "$AVD" -no-window -no-audio -no-boot-anim -no-snapshot-save >"$EMULATOR_LOG" 2>&1 &
    fi
    # a dead emulator would otherwise spin here forever, so both waits are bounded and print its log
    deadline=$((SECONDS + 180))
    until serial="$(android_serial)" && [ -n "$serial" ]; do
        [ "$SECONDS" -lt "$deadline" ] || { echo "no $AVD device in adb; emulator log:" >&2; cat "$EMULATOR_LOG" >&2; exit 1; }
        sleep 2
    done
    deadline=$((SECONDS + 420))
    until [ "$(adb -s "$serial" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ]; do
        [ "$SECONDS" -lt "$deadline" ] || { echo "$AVD never finished booting; emulator log:" >&2; cat "$EMULATOR_LOG" >&2; exit 1; }
        sleep 5
    done
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

[ -e "$BUILD" ] || { echo "no build at $BUILD, run: just $PLATFORM build" >&2; exit 1; }
mkdir -p "$OUTPUT_DIR"
DEVICE="$("${PLATFORM}_device")"

mkdir -p "$OUTPUT_DIR"
failed=""
for flow in "$ROOT_DIR"/maestro/smoke/*.yaml; do
    name="$(basename "$flow" .yaml)"
    echo "==> $name"
    if [ "$PLATFORM" = ios ]; then
        xcrun simctl keychain "$DEVICE" reset
        [ ! -f "$PROXYMAN_CA" ] || xcrun simctl keychain "$DEVICE" add-root-cert "$PROXYMAN_CA"
    fi
    maestro --device "$DEVICE" test \
        -e APP_ID="$APP_ID" \
        -e TEST_PHRASE="$(constant words)" \
        -e INVALID_PHRASE="$(constant invalidWords)" \
        -e BITCOIN_ADDRESS="$(constant bitcoinAddress)" \
        -e BITCOIN_ADDRESS_2="$(constant bitcoinAddress2)" \
        -e ETHEREUM_ADDRESS="$(constant ethereumAddress)" \
        -e ETHEREUM_NAME="$(constant ethereumName)" \
        -e USDT_CONTRACT="$(constant usdtContract)" \
        -e USDC_CONTRACT="$(constant usdcContract)" \
        --no-reinstall-driver \
        --test-output-dir "$OUTPUT_DIR" \
        "$flow" < /dev/null || failed="$failed $name"
done
[ -z "$failed" ] || { echo "Failed:$failed" >&2; exit 1; }
