#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
IOS_FILE="$ROOT_DIR/ios/Gem.xcodeproj/project.pbxproj"
ANDROID_FILE="$ROOT_DIR/android/app/build.gradle.kts"
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

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "$1 is required."
}

github_repo_from_url() {
  local url="$1"
  local path=""

  case "$url" in
    https://github.com/*) path="${url#https://github.com/}" ;;
    git@github.com:*) path="${url#git@github.com:}" ;;
    ssh://git@github.com/*) path="${url#ssh://git@github.com/}" ;;
    *) return 1 ;;
  esac

  path="${path%.git}"
  [[ "$path" =~ ^[^/]+/[^/]+$ ]] || return 1
  echo "$path"
}

fetch_latest() {
  git fetch --tags "$REMOTE_NAME" "$BRANCH_NAME"
}

remote_head() {
  git rev-parse "$REMOTE_BRANCH"
}

remote_tag_exists() {
  local status=0

  git ls-remote --exit-code --tags "$REMOTE_NAME" "refs/tags/$1" >/dev/null 2>&1 || status=$?
  case "$status" in
    0) return 0 ;;
    2) return 1 ;;
    *) fail "Unable to check whether tag $1 exists on $REMOTE_NAME." ;;
  esac
}

create_signed_tag() {
  git tag -s "$1" -m "$1" >/dev/null || fail "Unable to create signed tag $1."
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

  fetch_latest
  [[ "$(git rev-parse HEAD)" == "$(remote_head)" ]] || fail "$branch must match $REMOTE_BRANCH before bumping."
}

verify_github_access() {
  local repo can_push

  require_command gh

  repo="$(github_repo_from_url "$(git remote get-url "$REMOTE_NAME")")" || fail "$REMOTE_NAME must point to a GitHub repository."
  gh auth status -h github.com >/dev/null || fail "gh must be authenticated with github.com."

  can_push="$(gh api "repos/$repo" --jq '.permissions | (.admin or .maintain or .push)')" || fail "Unable to read GitHub permissions for $repo."
  [[ "$can_push" == "true" ]] || fail "The active GitHub account cannot push to $repo."
}

verify_tag_available() {
  local tag="$1"

  git rev-parse -q --verify "refs/tags/$tag" >/dev/null && fail "Tag $tag already exists locally."
  remote_tag_exists "$tag" && fail "Tag $tag already exists on $REMOTE_NAME."
  return 0
}

resolve_version() {
  local input="$1"
  local major release

  if [[ "$input" =~ ^[0-9]+\.[0-9]+$ ]]; then
    echo "$input"
    return
  fi

  IFS="." read -r major release <<< "$current_version"
  case "$input" in
    major) echo "$((major + 1)).0" ;;
    patch) echo "${major}.$((release + 1))" ;;
    *) fail "Invalid bump target: $input. Use patch, major, or an explicit X.Y version." ;;
  esac
}

verify_clean_latest_branch

current_ios_version=$(grep -oE "MARKETING_VERSION = [0-9]+\.[0-9]+;" "$IOS_FILE" | head -n1 | grep -oE "[0-9]+\.[0-9]+")
current_android_version=$(grep 'versionName = "' "$ANDROID_FILE" | sed 's/.*versionName = "//' | sed 's/".*//')

[[ -n "$current_ios_version" && -n "$current_android_version" ]] || fail "Unable to read current versions from iOS or Android."
[[ "$current_ios_version" == "$current_android_version" ]] || fail "iOS version ($current_ios_version) and Android version ($current_android_version) differ."

current_version="$current_ios_version"
new_version="$(resolve_version "$TARGET")"

current_ios_build=$(grep -oE "CURRENT_PROJECT_VERSION = [0-9]+;" "$IOS_FILE" | head -n1 | grep -oE "[0-9]+")
current_android_build=$(grep "versionCode = " "$ANDROID_FILE" | sed 's/.*versionCode = //' | sed 's/[^0-9].*//')

new_ios_build=$((current_ios_build + 1))
new_android_build=$((current_android_build + 1))

verify_tag_available "$new_version"
verify_github_access

sed -i '' "s/MARKETING_VERSION = $current_version;/MARKETING_VERSION = $new_version;/g" "$IOS_FILE"
sed -i '' "s/CURRENT_PROJECT_VERSION = $current_ios_build;/CURRENT_PROJECT_VERSION = $new_ios_build;/g" "$IOS_FILE"
sed -i '' "s/versionName = \"$current_version\"/versionName = \"$new_version\"/" "$ANDROID_FILE"
sed -i '' "s/versionCode = $current_android_build/versionCode = $new_android_build/" "$ANDROID_FILE"

git add "$IOS_FILE" "$ANDROID_FILE"
git commit -S -m "Bump to $new_version (iOS $new_ios_build, Android $new_android_build)"
create_signed_tag "$new_version"

fetch_latest
[[ "$(remote_head)" == "$(git rev-parse HEAD^)" ]] || fail "$REMOTE_BRANCH changed while bumping. Rebase and run again."
remote_tag_exists "$new_version" && fail "Tag $new_version was created on $REMOTE_NAME while bumping."

push_refs=("HEAD:$BRANCH_REF" "refs/tags/$new_version:refs/tags/$new_version")
git push --dry-run --atomic "$REMOTE_NAME" "${push_refs[@]}" >/dev/null || fail "Final dry-run push failed."
git push --atomic "$REMOTE_NAME" "${push_refs[@]}"

echo "✅ Bumped to $new_version (iOS $new_ios_build, Android $new_android_build)"
