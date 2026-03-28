#!/usr/bin/env bash
set -euo pipefail

SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
if [[ -z "${SDK_ROOT}" ]] && [[ -f local.properties ]]; then
  SDK_ROOT="$(sed -n 's/^sdk\.dir=//p' local.properties | head -n 1)"
fi
[[ -n "${SDK_ROOT}" ]] || { echo "ANDROID_HOME is not set"; exit 1; }

SDKMANAGER="${SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager"
AVDMANAGER="${SDK_ROOT}/cmdline-tools/latest/bin/avdmanager"
EMULATOR="${SDK_ROOT}/emulator/emulator"
ADB="${SDK_ROOT}/platform-tools/adb"

AVD_NAME="${ANDROID_DEVICE:-ci-emulator}"
API="${ANDROID_API:-35}"
DEVICE="${ANDROID_EMULATOR_DEVICE:-pixel_6}"
BOOT_TIMEOUT_SECONDS="${ANDROID_EMULATOR_BOOT_TIMEOUT:-120}"

if [[ "$(uname -m)" == "arm64" ]]; then
  ARCH="arm64-v8a"
else
  ARCH="x86_64"
fi

IMAGE="system-images;android-${API};google_apis;${ARCH}"
IMAGE_DIR="${SDK_ROOT}/system-images/android-${API}/google_apis/${ARCH}"

setup() {
  [[ -x "${SDKMANAGER}" && -x "${AVDMANAGER}" ]] || {
    echo "Android command-line tools are missing under ${SDK_ROOT}" >&2
    exit 1
  }

  if [[ ! -x "${EMULATOR}" || ! -x "${ADB}" || ! -d "${IMAGE_DIR}" ]]; then
    yes | "${SDKMANAGER}" --sdk_root="${SDK_ROOT}" --install \
      "platform-tools" \
      "emulator" \
      "${IMAGE}"
  fi

  "${ADB}" emu kill 2>/dev/null || true
  sleep 2

  if ! "${EMULATOR}" -list-avds | grep -Fxq "${AVD_NAME}"; then
    echo "no" | "${AVDMANAGER}" create avd \
      -n "${AVD_NAME}" \
      -k "${IMAGE}" \
      --device "${DEVICE}" \
      --force
  fi

  EMULATOR_LOG="${TMPDIR:-/tmp}/${AVD_NAME}.log"
  nohup "${EMULATOR}" -avd "${AVD_NAME}" -no-window -no-audio -no-boot-anim -no-snapshot-save -gpu swiftshader_indirect > "${EMULATOR_LOG}" 2>&1 &
  EMU_PID=$!
  sleep 5
  if ! kill -0 "${EMU_PID}" 2>/dev/null; then
    echo "Emulator failed to start" >&2
    cat "${EMULATOR_LOG}" >&2 || true
    exit 1
  fi

  "${ADB}" wait-for-device
  boot_waited=0
  for boot_waited in $(seq 1 "${BOOT_TIMEOUT_SECONDS}"); do
    if [[ "$("${ADB}" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; then
      break
    fi
    if ! kill -0 "${EMU_PID}" 2>/dev/null; then
      echo "Emulator failed during boot after ${boot_waited}s" >&2
      cat "${EMULATOR_LOG}" >&2 || true
      exit 1
    fi
    sleep 1
  done
  if [[ "$("${ADB}" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" != "1" ]]; then
    echo "Emulator failed to boot within ${BOOT_TIMEOUT_SECONDS}s" >&2
    cat "${EMULATOR_LOG}" >&2 || true
    exit 1
  fi
  if ! "${ADB}" devices | grep -q '^emulator-'; then
    echo "Emulator is not visible to adb" >&2
    cat "${EMULATOR_LOG}" >&2 || true
    exit 1
  fi
  "${ADB}" shell settings put global window_animation_scale 0
  "${ADB}" shell settings put global transition_animation_scale 0
  "${ADB}" shell settings put global animator_duration_scale 0
  echo "Emulator ${AVD_NAME} ready after ${boot_waited}s"
}

shutdown() {
  [[ -x "${ADB}" ]] || exit 0
  "${ADB}" emu kill >/dev/null 2>&1 || true
}

case "${1:-}" in
  setup) setup ;;
  shutdown) shutdown ;;
  *) echo "Usage: $0 {setup|shutdown}" >&2; exit 1 ;;
esac
