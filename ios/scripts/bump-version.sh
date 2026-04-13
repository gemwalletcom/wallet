#!/bin/bash
set -euo pipefail

file="Gem.xcodeproj/project.pbxproj"
bump=${1:-patch} # default to patch

version=$(grep -oE "MARKETING_VERSION = [0-9]+\.[0-9]+;" "$file" | head -n1 | grep -oE "[0-9]+\.[0-9]+")
if [[ -z "$version" ]]; then
  echo "❌ No MARKETING_VERSION found in $file" >&2
  exit 1
fi

IFS="." read -r major release <<< "$version"

case "$bump" in
  major) new_version="$((major + 1)).0" ;;
  patch) new_version="${major}.$((release + 1))" ;;
  *) echo "❌ Invalid bump type: $bump" >&2; exit 1 ;;
esac

sed -i '' "s/MARKETING_VERSION = $version;/MARKETING_VERSION = $new_version;/g" "$file"

echo "$new_version"
