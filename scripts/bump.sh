#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
IOS_FILE="$ROOT_DIR/ios/Gem.xcodeproj/project.pbxproj"
ANDROID_FILE="$ROOT_DIR/android/app/build.gradle.kts"
TARGET="${1:-patch}"

cd "$ROOT_DIR"

current_ios_version=$(grep -oE "MARKETING_VERSION = [0-9]+\.[0-9]+;" "$IOS_FILE" | head -n1 | grep -oE "[0-9]+\.[0-9]+")
current_android_version=$(grep 'versionName = "' "$ANDROID_FILE" | sed 's/.*versionName = "//' | sed 's/".*//')

if [[ -z "$current_ios_version" || -z "$current_android_version" ]]; then
  echo "❌ Unable to read current versions from iOS or Android." >&2
  exit 1
fi

if [[ "$current_ios_version" != "$current_android_version" ]]; then
  echo "❌ iOS version ($current_ios_version) and Android version ($current_android_version) differ." >&2
  exit 1
fi

current_version="$current_ios_version"

resolve_version() {
  local input="$1"
  if [[ "$input" =~ ^[0-9]+\.[0-9]+$ ]]; then
    echo "$input"
    return
  fi

  local major release
  IFS="." read -r major release <<< "$current_version"

  case "$input" in
    major)
      echo "$((major + 1)).0"
      ;;
    patch)
      echo "${major}.$((release + 1))"
      ;;
    *)
      echo "❌ Invalid bump target: $input. Use patch, major, or an explicit X.Y version." >&2
      exit 1
      ;;
  esac
}

new_version="$(resolve_version "$TARGET")"

if git rev-parse -q --verify "refs/tags/$new_version" >/dev/null; then
  echo "❌ Tag $new_version already exists." >&2
  exit 1
fi

current_ios_build=$(grep -oE "CURRENT_PROJECT_VERSION = [0-9]+;" "$IOS_FILE" | head -n1 | grep -oE "[0-9]+")
current_android_build=$(grep "versionCode = " "$ANDROID_FILE" | sed 's/.*versionCode = //' | sed 's/[^0-9].*//')

new_ios_build=$((current_ios_build + 1))
new_android_build=$((current_android_build + 1))

sed -i '' "s/MARKETING_VERSION = $current_version;/MARKETING_VERSION = $new_version;/g" "$IOS_FILE"
sed -i '' "s/CURRENT_PROJECT_VERSION = $current_ios_build;/CURRENT_PROJECT_VERSION = $new_ios_build;/g" "$IOS_FILE"
sed -i '' "s/versionName = \"$current_version\"/versionName = \"$new_version\"/" "$ANDROID_FILE"
sed -i '' "s/versionCode = $current_android_build/versionCode = $new_android_build/" "$ANDROID_FILE"

git add "$IOS_FILE" "$ANDROID_FILE"
git commit -S -m "Bump to $new_version (iOS $new_ios_build, Android $new_android_build)"
git tag -s "$new_version" -m "$new_version"
git push
git push origin "$new_version"

echo "✅ Bumped to $new_version (iOS $new_ios_build, Android $new_android_build)"
