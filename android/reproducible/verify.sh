#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  android/reproducible/verify.sh --source <checkout> --variant <fdroid|universal> [--official <signed.apk>] [--work-dir <dir>] [--keep] [--allow-non-linux]

Builds the selected unsigned release APK twice from two independent source
copies on Linux, verifies that no R8 r8-map-id SourceFile value is present in
the DEX files, and compares the unsigned APK hashes. If --official is provided,
the official APK signature is copied onto the rebuilt APK with apksigcopier and
the signed bytes are compared too.
USAGE
}

source_dir=""
variant="universal"
official_apk=""
work_dir=""
keep=0
allow_non_linux=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --source)
      source_dir="${2:?Missing value for --source}"
      shift 2
      ;;
    --variant)
      variant="${2:?Missing value for --variant}"
      shift 2
      ;;
    --official)
      official_apk="${2:?Missing value for --official}"
      shift 2
      ;;
    --work-dir)
      work_dir="${2:?Missing value for --work-dir}"
      shift 2
      ;;
    --keep)
      keep=1
      shift
      ;;
    --allow-non-linux)
      allow_non_linux=1
      shift
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

if [ -z "$source_dir" ]; then
  source_dir="$(pwd)"
fi

if [ "$(uname -s)" != "Linux" ] && [ "${ALLOW_NON_LINUX_REPRO:-}" != "true" ] && [ "$allow_non_linux" -ne 1 ]; then
  echo "Canonical reproducible verification must run on Linux. Set ALLOW_NON_LINUX_REPRO=true only for local smoke tests." >&2
  exit 1
fi

if [ "$(uname -s)" != "Linux" ]; then
  echo "Non-Linux verification is a smoke/debug check only; Linux x86_64 CI is authoritative." >&2
fi

source_dir="$(cd "$source_dir" && pwd)"
if [ ! -f "$source_dir/shell.nix" ] || [ ! -f "$source_dir/android/reproducible/build-apk.sh" ]; then
  echo "Source directory does not look like the Gem Wallet repo: $source_dir" >&2
  exit 1
fi

command -v rsync >/dev/null || { echo "rsync is required" >&2; exit 1; }
command -v unzip >/dev/null || { echo "unzip is required" >&2; exit 1; }
command -v strings >/dev/null || { echo "strings is required" >&2; exit 1; }

if [ -n "$official_apk" ]; then
  official_apk="$(cd "$(dirname "$official_apk")" && pwd)/$(basename "$official_apk")"
  command -v apksigcopier >/dev/null || { echo "apksigcopier is required for --official" >&2; exit 1; }
fi

sha256_file() {
  if command -v sha256sum >/dev/null; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

print_r8_markers() {
  local label="$1"
  local apk="$2"
  echo "R8 markers in $label:" >&2
  unzip -p "$apk" 'classes*.dex' 2>/dev/null \
    | strings \
    | grep -F '~~R8{' \
    | sort -u \
    | sed 's/^/    /' >&2 || true
}

print_apk_entry_diff() {
  local first="$1"
  local second="$2"
  python3 - "$first" "$second" <<'PY' >&2 || true
import hashlib
import sys
from zipfile import ZipFile

first, second = sys.argv[1:3]

def entries(path):
    with ZipFile(path) as archive:
        result = {}
        for info in archive.infolist():
            data = archive.read(info.filename)
            result[info.filename] = (
                hashlib.sha256(data).hexdigest(),
                len(data),
                info.compress_size,
                info.CRC,
            )
        return result

left = entries(first)
right = entries(second)
names = sorted(set(left) | set(right))
missing = [name for name in names if name not in left or name not in right]
changed = [name for name in names if name in left and name in right and left[name] != right[name]]

print("Changed APK entries:")
for name in missing[:50]:
    print(f"    missing {name}: first={name in left} second={name in right}")
for name in changed[:100]:
    left_hash, left_size, left_compressed, left_crc = left[name]
    right_hash, right_size, right_compressed, right_crc = right[name]
    print(
        "    "
        f"{name}: "
        f"{left_hash[:16]} size={left_size} compressed={left_compressed} crc={left_crc:08x} -> "
        f"{right_hash[:16]} size={right_size} compressed={right_compressed} crc={right_crc:08x}"
    )
if len(missing) > 50 or len(changed) > 100:
    print(f"    ... truncated: missing={len(missing)} changed={len(changed)}")
PY
}

run_diffoscope() {
  local left="$1"
  local right="$2"
  local output="$3"
  if command -v diffoscope >/dev/null && diffoscope --version >/dev/null 2>&1; then
    diffoscope "$left" "$right" --html "$output" || true
    echo "Diffoscope report: $output" >&2
  fi
}

if [ -z "$work_dir" ]; then
  work_dir="$(mktemp -d)"
else
  mkdir -p "$work_dir"
  work_dir="$(cd "$work_dir" && pwd)"
fi

if [ "$keep" -eq 0 ]; then
  trap 'rm -rf "$work_dir"' EXIT
else
  echo "Keeping work directory: $work_dir"
fi

source_epoch="$(cd "$source_dir" && git log -1 --pretty=%ct 2>/dev/null || echo 0)"
export SOURCE_DATE_EPOCH="$source_epoch"
export GEM_REPRO_CARGO_HOME="$work_dir/cargo-home"
mkdir -p "$GEM_REPRO_CARGO_HOME"

copy_source() {
  local destination="$1"
  mkdir -p "$destination"
  rsync -a --delete --delete-excluded \
    --exclude '.git' \
    --exclude '.jj' \
    --exclude '.gradle' \
    --exclude '.gradle-reproducible' \
    --exclude '.cargo-reproducible' \
    --exclude '.kotlin' \
    --exclude 'result' \
    --exclude 'android/local.properties' \
    --exclude 'android/.gradle' \
    --exclude 'android/.kotlin' \
    --exclude 'android/app/build' \
    --exclude 'android/app/release.keystore' \
    --exclude 'android/build' \
    --exclude 'core/target' \
    --exclude 'ios/build' \
    --exclude 'artifacts' \
    "$source_dir/" "$destination/"
}

build_once() {
  local name="$1"
  local checkout="$work_dir/$name/source"
  local apk="$work_dir/$name/$variant.apk"
  local log="$work_dir/$name/build.log"
  copy_source "$checkout"
  if ! (
    cd "$checkout"
    android/reproducible/build-apk.sh --variant "$variant" --output "$apk"
  ) 2>&1 | tee "$log" >&2; then
    echo "Build failed. Log: $log" >&2
    return 1
  fi
  find "$checkout/android/app/build/outputs/mapping" \
    -maxdepth 3 \
    -type f \
    \( -name 'mapping.txt' -o -name 'configuration.txt' -o -name 'seeds.txt' -o -name 'usage.txt' \) \
    -exec sh -c 'for file do cp "$file" "$0/$(basename "$(dirname "$file")")-$(basename "$file")"; done' "$work_dir/$name" {} + \
    2>/dev/null || true
  printf '%s\n' "$apk"
}

echo "==> Building first $variant APK"
apk_one="$(build_once build-one)" || exit 1
hash_one="$(sha256_file "$apk_one")"
echo "    $hash_one  $apk_one"

echo "==> Building second $variant APK"
apk_two="$(build_once build-two)" || exit 1
hash_two="$(sha256_file "$apk_two")"
echo "    $hash_two  $apk_two"

if [ "$hash_one" != "$hash_two" ]; then
  echo "Unsigned APK mismatch" >&2
  print_apk_entry_diff "$apk_one" "$apk_two"
  print_r8_markers "first APK" "$apk_one"
  print_r8_markers "second APK" "$apk_two"
  run_diffoscope "$apk_one" "$apk_two" "$work_dir/unsigned-diff.html"
  exit 1
fi

echo "Unsigned APKs match."

if [ -n "$official_apk" ]; then
  rebuilt_signed="$work_dir/rebuilt-signed.apk"
  echo "==> Copying official signature"
  apksigcopier copy "$official_apk" "$apk_one" "$rebuilt_signed"

  official_hash="$(sha256_file "$official_apk")"
  rebuilt_hash="$(sha256_file "$rebuilt_signed")"
  echo "    $official_hash  $official_apk"
  echo "    $rebuilt_hash  $rebuilt_signed"

  if [ "$official_hash" != "$rebuilt_hash" ]; then
    echo "Official signed APK mismatch" >&2
    run_diffoscope "$official_apk" "$rebuilt_signed" "$work_dir/signed-diff.html"
    exit 1
  fi

  echo "Official signed APK matches after signature copy."
fi
