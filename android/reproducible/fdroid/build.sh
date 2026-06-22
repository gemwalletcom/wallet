#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export NDK_PATH="${NDK_PATH:?NDK_PATH must point to the Android NDK}"
source "$script_dir/env.sh"

touch local.properties

rm -rf "$HOME/.gradle/caches/transforms-"*
"${GRADLE_COMMAND:-gradle}" --no-daemon --console plain --no-configuration-cache -Pchannel=fdroid :app:assembleFdroidRelease
