#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: android/reproducible/build-apk.sh --variant <fdroid|universal> [--output <apk>]

Builds an unsigned release APK in the current checkout. Run from the repository
root inside shell.nix.
USAGE
}

variant="universal"
output=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --variant)
      variant="${2:?Missing value for --variant}"
      shift 2
      ;;
    --output)
      output="${2:?Missing value for --output}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

case "$variant" in
  fdroid)
    task=":app:assembleFdroidRelease"
    apk_dir="android/app/build/outputs/apk/fdroid/release"
    export FDROID_BUILD=true
    ;;
  universal)
    task=":app:assembleUniversalRelease"
    apk_dir="android/app/build/outputs/apk/universal/release"
    ;;
  *)
    echo "Unsupported variant: $variant" >&2
    exit 2
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
repo_root_physical="$(cd "$repo_root" && pwd -P)"
cd "$repo_root"

export SKIP_SIGN=true
export ANDROID_REPRODUCIBLE_BUILD=true
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --pretty=%ct 2>/dev/null || echo 0)}"
export GRADLE_USER_HOME="${GRADLE_USER_HOME:-$repo_root/.gradle-reproducible}"
export CARGO_HOME="${GEM_REPRO_CARGO_HOME:-$repo_root/.cargo-reproducible}"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export GEMSTONE_ANDROID_ABIS="${GEMSTONE_ANDROID_ABIS:-arm64-v8a,armeabi-v7a}"
mkdir -p "$GRADLE_USER_HOME" "$CARGO_HOME"
unset RUSTC_WRAPPER
unset RUSTC_WORKSPACE_WRAPPER
unset CARGO_BUILD_RUSTC_WRAPPER

append_rustflag() {
  case " ${RUSTFLAGS:-} " in
    *" $1 "*) ;;
    *) export RUSTFLAGS="${RUSTFLAGS:-} $1" ;;
  esac
}

append_path_remap() {
  local from="$1"
  local to="$2"
  [ -n "$from" ] || return 0

  append_rustflag "--remap-path-prefix=$from=$to"
  if [ -e "$from" ]; then
    local physical
    physical="$(cd "$from" && pwd -P)"
    if [ "$physical" != "$from" ]; then
      append_rustflag "--remap-path-prefix=$physical=$to"
    fi
  fi
}

append_path_remap "$repo_root" "."
append_path_remap "$repo_root_physical" "."
append_path_remap "$CARGO_HOME" "CARGO_HOME"
append_path_remap "${ANDROID_HOME:-}" "ANDROID_HOME"
append_path_remap "${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}" "ANDROID_NDK_HOME"

touch android/local.properties

(
  cd android
  gradle_arguments=(
    clean "$task"
    --no-configuration-cache
    --no-daemon
    --console plain
  )
  if [ -n "${JAVA_HOME:-}" ]; then
    gradle_arguments=(
      "-Dorg.gradle.java.home=$JAVA_HOME"
      "-Dorg.gradle.java.installations.paths=$JAVA_HOME"
      "-Dorg.gradle.java.installations.auto-detect=false"
      "-Dorg.gradle.java.installations.auto-download=false"
      "${gradle_arguments[@]}"
    )
  fi
  ./gradlew "${gradle_arguments[@]}"
)

apk_path="$(find "$apk_dir" -maxdepth 1 -type f -name '*.apk' | sort | head -n 1)"
if [ -z "$apk_path" ]; then
  echo "No APK produced in $apk_dir" >&2
  exit 1
fi

if unzip -p "$apk_path" 'classes*.dex' 2>/dev/null | strings | grep -Fq 'r8-map-id-'; then
  echo "R8 map id leaked into DEX SourceFile attributes: $apk_path" >&2
  exit 1
fi

if [ -n "$output" ]; then
  mkdir -p "$(dirname "$output")"
  cp "$apk_path" "$output"
  apk_path="$output"
fi

echo "$apk_path"
