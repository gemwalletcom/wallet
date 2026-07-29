#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
IOS_FILE="$ROOT_DIR/ios/Gem.xcodeproj/project.pbxproj"
ANDROID_FILE="$ROOT_DIR/android/app/build.gradle.kts"
CORE_FILE="$ROOT_DIR/core/Cargo.toml"
CORE_LOCK_FILE="$ROOT_DIR/core/Cargo.lock"
TARGET="${1:-patch}"
REMOTE_NAME="${BUMP_REMOTE:-origin}"
BRANCH_NAME="${BUMP_BRANCH:-main}"
REMOTE_BRANCH="$REMOTE_NAME/$BRANCH_NAME"
BRANCH_REF="refs/heads/$BRANCH_NAME"

cd "$ROOT_DIR"

fail() {
  echo "❌ $*" >&2
  exit 1
}

verify_clean_latest_branch() {
  local branch upstream_remote upstream_merge upstream_branch

  branch="$(git branch --show-current)"
  [[ "$branch" == "$BRANCH_NAME" ]] || fail "Run this from $BRANCH_NAME, not ${branch:-detached HEAD}."
  [[ -z "$(git status --porcelain)" ]] || fail "Working tree must be clean before bumping."

  upstream_remote="$(git config "branch.$branch.remote" || true)"
  upstream_merge="$(git config "branch.$branch.merge" || true)"
  upstream_branch="${upstream_merge#refs/heads/}"
  [[ "$upstream_remote" == "$REMOTE_NAME" && "$upstream_branch" == "$BRANCH_NAME" ]] || fail "$branch must track $REMOTE_BRANCH."

  git fetch --tags "$REMOTE_NAME" "$BRANCH_NAME"
  [[ "$(git rev-parse HEAD)" == "$(git rev-parse "$REMOTE_BRANCH")" ]] || fail "$branch must match $REMOTE_BRANCH before bumping."
}

resolve_version() {
  local input="$1"
  local major minor patch

  if [[ "$input" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "$input"
    return
  fi

  IFS="." read -r major minor patch <<< "$current_version"
  case "$input" in
    major) echo "$((major + 1)).0.0" ;;
    minor) echo "${major}.$((minor + 1)).0" ;;
    patch) echo "${major}.${minor}.$((patch + 1))" ;;
    *) fail "Invalid bump target: $input. Use patch, minor, major, or an explicit X.Y.Z version." ;;
  esac
}

verify_clean_latest_branch

current_ios_version=$(grep -oE "MARKETING_VERSION = [0-9]+\.[0-9]+\.[0-9]+;" "$IOS_FILE" | head -n1 | grep -oE "[0-9]+\.[0-9]+\.[0-9]+")
current_android_version=$(grep 'versionName = "' "$ANDROID_FILE" | sed 's/.*versionName = "//' | sed 's/".*//')
current_core_version=$(grep -oE '^version = "[0-9]+\.[0-9]+\.[0-9]+"' "$CORE_FILE" | head -n1 | sed 's/version = "//; s/"//')

[[ -n "$current_ios_version" && -n "$current_android_version" && -n "$current_core_version" ]] || fail "Unable to read current versions from iOS, Android, or Core."
[[ "$current_ios_version" == "$current_android_version" ]] || fail "iOS version ($current_ios_version) and Android version ($current_android_version) differ."
[[ "$current_core_version" == "$current_ios_version" ]] || fail "Core version ($current_core_version) and app version ($current_ios_version) differ."

current_version="$current_ios_version"
new_version="$(resolve_version "$TARGET")"
new_core_version="$new_version"

current_ios_build=$(grep -oE "CURRENT_PROJECT_VERSION = [0-9]+;" "$IOS_FILE" | head -n1 | grep -oE "[0-9]+")
current_android_build=$(grep "versionCode = " "$ANDROID_FILE" | sed 's/.*versionCode = //' | sed 's/[^0-9].*//')

new_ios_build=$((current_ios_build + 1))
new_android_build=$((current_android_build + 1))

git rev-parse -q --verify "refs/tags/$new_version" >/dev/null && fail "Tag $new_version already exists."

sed -i '' "s/MARKETING_VERSION = $current_version;/MARKETING_VERSION = $new_version;/g" "$IOS_FILE"
sed -i '' "s/CURRENT_PROJECT_VERSION = $current_ios_build;/CURRENT_PROJECT_VERSION = $new_ios_build;/g" "$IOS_FILE"
sed -i '' "s/versionName = \"$current_version\"/versionName = \"$new_version\"/" "$ANDROID_FILE"
sed -i '' "s/versionCode = $current_android_build/versionCode = $new_android_build/" "$ANDROID_FILE"
sed -i '' "s/^version = \"$current_core_version\"/version = \"$new_core_version\"/" "$CORE_FILE"
cargo metadata --manifest-path "$CORE_FILE" --format-version 1 >/dev/null
core_versions="$(cargo metadata --manifest-path "$CORE_FILE" --format-version 1 --no-deps | grep -oE '"version":"[^"]+"' | sort -u)"
[[ "$core_versions" == "\"version\":\"$new_core_version\"" ]] || fail "Core workspace packages do not all use version $new_core_version: $core_versions"

git add "$IOS_FILE" "$ANDROID_FILE" "$CORE_FILE" "$CORE_LOCK_FILE"
git commit -S -m "Bump to $new_version (iOS $new_ios_build, Android $new_android_build)"
git tag -s "$new_version" -m "$new_version"
git push --atomic "$REMOTE_NAME" "HEAD:$BRANCH_REF" "refs/tags/$new_version:refs/tags/$new_version"

echo "✅ Bumped to $new_version (iOS $new_ios_build, Android $new_android_build)"
