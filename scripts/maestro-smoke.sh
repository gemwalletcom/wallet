#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PLATFORM="${1:?usage: maestro-smoke.sh ios|android}"
OUTPUT_DIR="${MAESTRO_OUTPUT:-$ROOT_DIR/build/maestro/$PLATFORM}"
EMULATOR_LOG="$OUTPUT_DIR/emulator.log"
AVD_LOG="$OUTPUT_DIR/avd.log"
SIMULATOR="Gem Maestro"
AVD="Gem_Maestro"
ANDROID_SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
EMULATOR="$ANDROID_SDK/emulator/emulator"
# a CI runner ships platform-tools inside the SDK and off PATH
ADB="${ADB:-$(command -v adb || echo "$ANDROID_SDK/platform-tools/adb")}"
PROXYMAN_CA="$HOME/Library/Application Support/com.proxyman.NSProxy/app-data/proxyman-ca.pem"

constant() {
    sed -nE "s/.*static let $1 = \"(.*)\"/\1/p" "$ROOT_DIR/ios/GemUITestsAppTests/Types/UITestKitConstants.swift"
}

newest() {
    sort -V | tail -1
}

ios_device() {
    local device_type runtime udid
    udid="$(xcrun simctl list devices available | grep -F "$SIMULATOR (" | grep -oE '[0-9A-F-]{36}' | tail -1 || true)"
    if [ -z "$udid" ]; then
        device_type="$(xcrun simctl list devicetypes | grep -E '^iPhone [0-9]+ Pro \(' | newest | grep -oE 'com\.apple[^)]+')"
        for runtime in $(xcrun simctl list runtimes | grep -E '^iOS [0-9.]+ ' | grep -v unavailable | sort -rV | sed 's/.* - //'); do
            udid="$(xcrun simctl create "$SIMULATOR" "$device_type" "$runtime" 2>/dev/null)" && break
        done
        [ -n "$udid" ] || { echo "no iOS runtime can create $device_type" >&2; exit 1; }
    fi
    xcrun simctl boot "$udid" 2>/dev/null || true
    xcrun simctl bootstatus "$udid" -b >/dev/null
    xcrun simctl spawn "$udid" defaults write -g AppleLocale en_US
    xcrun simctl install "$udid" "$BUILD"
    echo "$udid"
}

android_serial() {
    for serial in $("$ADB" devices | awk '/^emulator-/ { print $1 }'); do
        [ "$("$ADB" -s "$serial" emu avd name | head -1 | tr -d '\r')" != "$AVD" ] || echo "$serial"
    done
}

# Says what the emulator said, and whether this host can accelerate it at all.
die_emulator() {
    echo "$1" >&2
    [ ! -s "$AVD_LOG" ] || { echo "--- avd.log" >&2; cat "$AVD_LOG" >&2; }
    [ ! -s "$EMULATOR_LOG" ] || { echo "--- emulator.log" >&2; cat "$EMULATOR_LOG" >&2; }
    echo "--- /dev/kvm $([ -e /dev/kvm ] && echo present || echo absent), avd home ${ANDROID_AVD_HOME:-$HOME/.android/avd} (ANDROID_AVD_HOME=${ANDROID_AVD_HOME:-unset} ANDROID_SDK_HOME=${ANDROID_SDK_HOME:-unset})" >&2
    ls -l "${ANDROID_AVD_HOME:-$HOME/.android/avd}" >&2 || true
    exit 1
}

android_device() {
    local serial deadline image emulator_pid=""
    if [ -z "$(android_serial)" ]; then
        [ -x "$EMULATOR" ] || { echo "no emulator package at $EMULATOR: sdkmanager \"emulator\" \"platform-tools\"" >&2; exit 1; }
        [ -x "$ADB" ] || { echo "no adb at $ADB: sdkmanager \"platform-tools\"" >&2; exit 1; }
        if ! "$EMULATOR" -list-avds | grep -qx "$AVD"; then
            image="$(cd "$ANDROID_SDK" && ls -d system-images/android-*/google_apis/* 2>/dev/null | newest || true)"
            [ -n "$image" ] || { echo "no google_apis system image under $ANDROID_SDK/system-images" >&2; exit 1; }
            # avdmanager exits 0 having created nothing when the AVD home is missing, so make it first
            mkdir -p "${ANDROID_AVD_HOME:-$HOME/.android/avd}"
            echo no | "$ANDROID_SDK/cmdline-tools/latest/bin/avdmanager" create avd -n "$AVD" -d pixel_9 -k "$(tr / ';' <<<"$image")" >"$AVD_LOG" 2>&1
            # avdmanager can exit 0 without creating anything, which would leave nothing to wait for
            "$EMULATOR" -list-avds | grep -qx "$AVD" || die_emulator "avdmanager produced no $AVD from $image"
        fi
        nohup "$EMULATOR" -avd "$AVD" -no-window -no-audio -no-boot-anim -no-snapshot-save >"$EMULATOR_LOG" 2>&1 &
        emulator_pid=$!
    fi
    # every wait is bounded, and a dead emulator ends it at once instead of spinning for hours
    deadline=$((SECONDS + 180))
    until serial="$(android_serial)" && [ -n "$serial" ]; do
        { [ -n "$emulator_pid" ] && kill -0 "$emulator_pid" 2>/dev/null; } || die_emulator "the emulator process exited"
        [ "$SECONDS" -lt "$deadline" ] || die_emulator "no $AVD device in adb within 180s"
        sleep 2
    done
    deadline=$((SECONDS + 420))
    until [ "$("$ADB" -s "$serial" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ]; do
        { [ -n "$emulator_pid" ] && kill -0 "$emulator_pid" 2>/dev/null; } || die_emulator "the emulator process exited while booting"
        [ "$SECONDS" -lt "$deadline" ] || die_emulator "$AVD never finished booting"
        sleep 5
    done
    "$ADB" -s "$serial" install -r "$BUILD" >/dev/null
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
