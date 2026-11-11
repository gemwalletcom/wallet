#!/usr/bin/env bash
set -euo pipefail

name="$1"
os="${2:-}"

if [ -z "$os" ]; then
    echo "$name"
    exit 0
fi

udid="$(xcrun simctl list devices available | awk -v runtime="-- iOS $os --" -v device="    $name (" '
    /^-- / { inside = ($0 == runtime) }
    inside && index($0, device) == 1 { print; exit }
' | grep -oE '[0-9A-F-]{36}' || true)"

if [ -z "$udid" ]; then
    echo "no $name simulator on iOS $os" >&2
    exit 1
fi
echo "$udid"
