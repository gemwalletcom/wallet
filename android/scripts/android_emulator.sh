#!/usr/bin/env bash
set -euo pipefail

SDK_ROOT="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-/opt/homebrew/share/android-commandlinetools}}"
if [[ ! -d "${SDK_ROOT}" ]] && [[ -f local.properties ]]; then
  SDK_ROOT="$(sed -n 's/^sdk\.dir=//p' local.properties | head -n 1)"
fi
[[ -n "${SDK_ROOT}" ]] || { echo "ANDROID_HOME is not set"; exit 1; }

SDKMANAGER="${SDK_ROOT}/cmdline-tools/latest/bin/sdkmanager"
AVDMANAGER="${SDK_ROOT}/cmdline-tools/latest/bin/avdmanager"
EMULATOR="${SDK_ROOT}/emulator/emulator"
ADB="${SDK_ROOT}/platform-tools/adb"
DEVICES_FILE="${ANDROID_EMULATOR_DEVICES_FILE:-}"

BOOT_TIMEOUT_SECONDS="${ANDROID_EMULATOR_BOOT_TIMEOUT:-120}"
ARCH="arm64-v8a"

require_tool() {
  [[ -x "${1}" ]] || {
    echo "${2} is missing under ${SDK_ROOT}" >&2
    exit 1
  }
}

list_devices() {
  [[ -f "${DEVICES_FILE}" ]] || {
    echo "ANDROID_EMULATOR_DEVICES_FILE is not set or does not exist" >&2
    exit 1
  }
  awk -F: '!/^#/ && NF { print $1 }' "${DEVICES_FILE}"
}

default_avd_name() {
  [[ -f "${DEVICES_FILE}" ]] || return 1
  awk -F: '!/^#/ && NF { print $1; exit }' "${DEVICES_FILE}"
}

api_for_avd_name() {
  local avd_name="$1"
  [[ -f "${DEVICES_FILE}" ]] || return 1
  awk -F: -v avd_name="${avd_name}" '$1 == avd_name { print $2; exit }' "${DEVICES_FILE}"
}

resolve_avd_name() {
  if [[ -n "${ANDROID_AVD_NAME:-}" ]]; then
    echo "${ANDROID_AVD_NAME}"
  else
    default_avd_name
  fi
}

resolve_api() {
  local avd_name="$1"
  if [[ -n "${ANDROID_API:-}" ]]; then
    echo "${ANDROID_API}"
  else
    api_for_avd_name "${avd_name}"
  fi
}

install_sdk_packages() {
  local image="$1"
  local compile_sdk="${2:-}"
  local packages=("platform-tools" "emulator" "${image}")

  if [[ -n "${compile_sdk}" ]]; then
    packages+=("platforms;android-${compile_sdk}" "build-tools;${compile_sdk}.0.0")
  fi

  set +o pipefail
  yes | "${SDKMANAGER}" --sdk_root="${SDK_ROOT}" --install "${packages[@]}"
  install_status=$?
  set -o pipefail
  [[ ${install_status} -eq 0 ]] || exit "${install_status}"
}

create_avd_if_missing() {
  local avd_name="$1"
  local image="$2"
  local device="${3:-}"

  if ! "${EMULATOR}" -list-avds | grep -Fxq "${avd_name}"; then
    local create_args=(-n "${avd_name}" -k "${image}" --force)
    if [[ -n "${device}" ]]; then
      create_args+=(--device "${device}")
    fi
    echo "no" | "${AVDMANAGER}" create avd "${create_args[@]}"
  fi
}

provision_one() {
  local avd_name="$1"
  local api="$2"
  local compile_sdk="${ANDROID_COMPILE_SDK:-}"
  local device="${ANDROID_EMULATOR_DEVICE:-}"
  local image="system-images;android-${api};google_apis;${ARCH}"

  require_tool "${SDKMANAGER}" "Android sdkmanager"
  require_tool "${AVDMANAGER}" "Android avdmanager"
  install_sdk_packages "${image}" "${compile_sdk}"
  require_tool "${EMULATOR}" "Android emulator"
  require_tool "${ADB}" "Android adb"
  create_avd_if_missing "${avd_name}" "${image}" "${device}"
}

provision() {
  if [[ -f "${DEVICES_FILE}" && -z "${ANDROID_AVD_NAME:-}" && -z "${ANDROID_API:-}" ]]; then
    while IFS=: read -r avd_name api; do
      [[ "${avd_name}" =~ ^#.*$ || -z "${avd_name}" ]] && continue
      provision_one "${avd_name}" "${api}"
    done < "${DEVICES_FILE}"
    return
  fi

  local avd_name
  local api

  avd_name="$(resolve_avd_name)"
  [[ -n "${avd_name}" ]] || { echo "ANDROID_AVD_NAME is not set" >&2; exit 1; }

  api="$(resolve_api "${avd_name}")"
  [[ -n "${api}" ]] || { echo "ANDROID_API is not set for ${avd_name}" >&2; exit 1; }

  provision_one "${avd_name}" "${api}"
}

setup() {
  local avd_name
  local api
  local image
  local image_dir
  local boot_checks=$((BOOT_TIMEOUT_SECONDS / 2))

  avd_name="$(resolve_avd_name)"
  [[ -n "${avd_name}" ]] || { echo "ANDROID_AVD_NAME is not set" >&2; exit 1; }

  api="$(resolve_api "${avd_name}")"
  [[ -n "${api}" ]] || { echo "ANDROID_API is not set for ${avd_name}" >&2; exit 1; }

  image="system-images;android-${api};google_apis;${ARCH}"
  image_dir="${SDK_ROOT}/system-images/android-${api}/google_apis/${ARCH}"

  require_tool "${EMULATOR}" "Android emulator"
  require_tool "${ADB}" "Android adb"
  [[ -d "${image_dir}" ]] || {
    echo "System image ${image} is missing under ${SDK_ROOT}. Run '$0 provision' first." >&2
    exit 1
  }
  "${EMULATOR}" -list-avds | grep -Fxq "${avd_name}" || {
    echo "AVD ${avd_name} is missing under ${SDK_ROOT}. Run '$0 provision' first." >&2
    exit 1
  }

  "${ADB}" emu kill 2>/dev/null || true
  sleep 2

  "${EMULATOR}" -avd "${avd_name}" -no-window -no-audio &
  EMU_PID=$!
  sleep 5
  if ! kill -0 "${EMU_PID}" 2>/dev/null; then
    echo "Emulator failed to start" >&2
    exit 1
  fi

  "${ADB}" wait-for-device
  for _ in $(seq 1 "${boot_checks}"); do
    if [[ "$("${ADB}" shell getprop sys.boot_completed 2>/dev/null)" == "1" ]]; then
      break
    fi
    if ! kill -0 "${EMU_PID}" 2>/dev/null; then
      echo "Emulator failed during boot" >&2
      exit 1
    fi
    sleep 2
  done
  if [[ "$("${ADB}" shell getprop sys.boot_completed 2>/dev/null)" != "1" ]]; then
    echo "Emulator failed to boot within ${BOOT_TIMEOUT_SECONDS}s" >&2
    exit 1
  fi
  if ! "${ADB}" devices | grep -q '^emulator-'; then
    echo "Emulator is not visible to adb" >&2
    exit 1
  fi
  echo "Emulator ${avd_name} ready"
}

install() {
  local version="$1"
  local apk="/tmp/gem_wallet_${version}.apk"

  require_tool "${ADB}" "Android adb"
  curl -L -o "${apk}" "https://apk.gemwallet.com/gem_wallet_universal_${version}.apk"
  "${ADB}" uninstall com.gemwallet.android 2>/dev/null || true
  "${ADB}" install "${apk}"
}

shutdown() {
  [[ -x "${ADB}" ]] || exit 0
  "${ADB}" emu kill >/dev/null 2>&1 || true
}

case "${1:-}" in
  list) list_devices ;;
  provision) provision ;;
  setup) setup ;;
  shutdown) shutdown ;;
  install) install "$2" ;;
  *) echo "Usage: $0 {list|provision|setup|shutdown|install VERSION}" >&2; exit 1 ;;
esac
