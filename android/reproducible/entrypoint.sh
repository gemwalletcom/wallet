#!/usr/bin/env bash
set -euo pipefail

mkdir /src
tar -xf - -C /src
cd /src/android

for channel in "$@"; do
    reproducible/build.sh "$channel"
done

cd app/build/outputs
find . \( -name "*.apk" -o -name "*.aab" \) -exec cp --parents {} /outputs \;
