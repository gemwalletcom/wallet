#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  android/reproducible/sign-apk.sh --unsigned <apk> --out <apk> --ks <keystore> --ks-key-alias <alias>

Signs an already-built unsigned APK. Run inside shell.nix so zipalign and
apksigner come from the pinned Android build-tools.
USAGE
}

unsigned_apk=""
output_apk=""
keystore=""
alias=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --unsigned)
      unsigned_apk="${2:?Missing value for --unsigned}"
      shift 2
      ;;
    --out)
      output_apk="${2:?Missing value for --out}"
      shift 2
      ;;
    --ks)
      keystore="${2:?Missing value for --ks}"
      shift 2
      ;;
    --ks-key-alias)
      alias="${2:?Missing value for --ks-key-alias}"
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

[ -n "$unsigned_apk" ] || { usage >&2; exit 2; }
[ -n "$output_apk" ] || { usage >&2; exit 2; }
[ -n "$keystore" ] || { usage >&2; exit 2; }
[ -n "$alias" ] || { usage >&2; exit 2; }
[ -n "${ANDROID_KEYSTORE_PASSWORD:-}" ] || { echo "ANDROID_KEYSTORE_PASSWORD is required" >&2; exit 1; }

build_tools="${GEM_ANDROID_SIGNING_BUILD_TOOLS:-${ANDROID_HOME:-}/build-tools/34.0.0}"
zipalign="$build_tools/zipalign"
apksigner="$build_tools/apksigner"

[ -x "$zipalign" ] || { echo "zipalign not found: $zipalign" >&2; exit 1; }
[ -x "$apksigner" ] || { echo "apksigner not found: $apksigner" >&2; exit 1; }

aligned_apk="$(mktemp "${TMPDIR:-/tmp}/gem-aligned.XXXXXX.apk")"
trap 'rm -f "$aligned_apk"' EXIT

"$zipalign" -p -f 4 "$unsigned_apk" "$aligned_apk"
"$apksigner" sign \
  --ks "$keystore" \
  --ks-key-alias "$alias" \
  --ks-pass env:ANDROID_KEYSTORE_PASSWORD \
  --key-pass env:ANDROID_KEYSTORE_PASSWORD \
  --v1-signing-enabled true \
  --v2-signing-enabled true \
  --v3-signing-enabled false \
  --v4-signing-enabled false \
  --out "$output_apk" \
  "$aligned_apk"
"$apksigner" verify --verbose "$output_apk"
