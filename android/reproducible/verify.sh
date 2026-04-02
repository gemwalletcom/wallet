#!/usr/bin/env bash
# Rebuild a tagged release in nix-shell and compare against the published APK.
# Usage: ./verify.sh <tag> <official.apk>
set -euo pipefail

TAG="${1:?Usage: verify.sh <tag> <official.apk>}"
OFFICIAL_APK="${2:?Usage: verify.sh <tag> <official.apk>}"
OUT_DIR="artifacts/reproducible/${TAG}"

command -v nix-shell >/dev/null || { echo "Nix is required. Install: curl -L https://nixos.org/nix/install | sh -s -- --daemon"; exit 1; }
command -v apksigcopier >/dev/null || { echo "apksigcopier is required. Install: pip install apksigcopier"; exit 1; }

mkdir -p "${OUT_DIR}"
cp "${OFFICIAL_APK}" "${OUT_DIR}/official.apk"

echo "==> Cloning tag ${TAG}..."
WORK_DIR=$(mktemp -d)
trap 'rm -rf "${WORK_DIR}"' EXIT
git clone --recurse-submodules --branch "${TAG}" --depth 1 https://github.com/gemwalletcom/wallet.git "${WORK_DIR}/wallet"

echo "==> Building in nix-shell..."
touch "${WORK_DIR}/wallet/android/local.properties"
nix-shell "${WORK_DIR}/wallet/shell.nix" --run "
  rustup default stable
  rustup target add x86_64-linux-android aarch64-linux-android armv7-linux-androideabi
  cd ${WORK_DIR}/wallet/android && ./gradlew clean assembleUniversalRelease --no-configuration-cache
"
cp "${WORK_DIR}/wallet/android/app/build/outputs/apk/universal/release/app-universal-release.apk" "${OUT_DIR}/rebuilt.apk"

echo "==> Stripping signatures..."
apksigcopier copy "${OUT_DIR}/official.apk" "${OUT_DIR}/rebuilt.apk" "${OUT_DIR}/rebuilt-signed.apk"

echo "==> Comparing..."
OFFICIAL_HASH=$(sha256sum "${OUT_DIR}/official.apk" | cut -d' ' -f1)
REBUILT_HASH=$(sha256sum "${OUT_DIR}/rebuilt-signed.apk" | cut -d' ' -f1)

echo "Official:  ${OFFICIAL_HASH}"
echo "Rebuilt:   ${REBUILT_HASH}"

if [ "${OFFICIAL_HASH}" = "${REBUILT_HASH}" ]; then
  echo "MATCH — reproducible build verified."
else
  echo "MISMATCH — APKs differ."
  if command -v diffoscope >/dev/null; then
    echo "==> Running diffoscope..."
    diffoscope "${OUT_DIR}/official.apk" "${OUT_DIR}/rebuilt-signed.apk" --html "${OUT_DIR}/diff.html" || true
    echo "Diff saved to ${OUT_DIR}/diff.html"
  fi
  exit 1
fi
